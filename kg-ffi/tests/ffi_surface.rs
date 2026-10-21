//! Uji "kontrak FFI": memanggil fungsi publik `kg-ffi` persis seperti yang
//! dipanggil Kotlin lewat UniFFI, tetapi di dalam proses Rust.
//!
//! Tujuannya menangkap kesalahan sebelum APK dibangun: nama fungsi, tipe
//! kembalian, dan bentuk data yang dilihat layar (label, badge, hasil kuis).

use kg_ffi::*;

fn s(v: &str) -> String {
    v.to_string()
}

#[test]
fn ffi_surface_behaves_like_the_app_expects() {
    // 1. Semua topik terbaca, id unik, dan yang punya latihan tidak kosong.
    let topics = all_topics();
    assert_eq!(topics.len(), 10, "harus ada 10 topik");
    let ids: Vec<String> = topics.iter().map(|t| t.id.clone()).collect();
    let mut uniq = ids.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), ids.len(), "id topik harus unik");
    assert_eq!(topic(s("hangeul")).map(|t| t.no), Some(1));
    assert!(topic(s("tidak-ada")).is_none());
    for t in &topics {
        assert!(!t.title.is_empty(), "judul topik {} kosong", t.id);
        assert!(!t.quiz.is_empty(), "kuis topik {} kosong", t.id);
        // Label tombol menyebut jumlah soal yang benar.
        assert!(
            t.quiz_label.contains(&t.quiz.len().to_string()),
            "label kuis topik {} ({} soal) = {}",
            t.id,
            t.quiz.len(),
            t.quiz_label
        );
    }

    // 2. Versi inti dan kunci penyimpanan stabil.
    assert!(!core_version().is_empty());
    assert_eq!(store_key(), "kg-progress-v1");
    assert_eq!(theme_key(), "kg-theme-v1");

    // 3. Progres awal kosong; penghitung "soal dijawab" bertambah, dan progres
    //    dianggap ada setelah ada jawaban (bukan setelah topik selesai).
    let empty = progress_new();
    assert!(!progress_has_any(empty.clone()));
    let one = progress_add_answered(empty.clone(), 1);
    assert!(progress_has_any(one.clone()));
    // Belum ada topik tuntas, jadi daftar entri masih kosong.
    assert!(progress_entries(one.clone()).is_empty());
    assert_eq!(home(one).answered, "1");
    assert_eq!(progress_reset(), empty);

    // 4. Layar awal: kartu untuk semua topik + label lanjut.
    let topik_pertama = topics[0].id.clone();
    let h = home(empty.clone());
    assert_eq!(h.cards.len(), 10);
    assert_eq!(h.topics_done, "0/10");
    assert!(!h.has_progress);
    assert_eq!(h.resume_topic, topik_pertama);
    assert!(!h.resume_label.is_empty());
    // Kartu pertama belum dikerjakan: badge berisi nama level, bukan skor.
    assert!(!h.cards[0].done);
    assert!(!h.cards[0].badge_is_score);
    assert_eq!(h.cards[0].badge, topics[0].level_name);
    assert!(h.cards[0].name.starts_with("1. "));

    // 5. Kuis berjalan sampai selesai dengan skor 100%.
    assert!(quiz_start(s("hangeul"), s("lesson"), 0));
    assert!(quiz_active());
    let mut answered = 0;
    loop {
        let st = quiz_state().expect("state harus ada selama kuis");
        if !st.answered {
            assert!(
                quiz_pick(st.answer as i32),
                "menjawab kunci yang benar harus diterima"
            );
            answered += 1;
        }
        let after = quiz_state().expect("state sesudah menjawab");
        assert!(after.answered, "sesudah memilih, soal harus terjawab");
        match quiz_advance().as_str() {
            "more" => continue,
            "finished" => break,
            other => panic!("advance tak terduga: {other}"),
        }
    }
    assert_eq!(answered, 10, "semua soal topik 1 harus terlewati");

    let res = quiz_result(61_000).expect("hasil akhir kuis");
    assert_eq!(res.topic_id, "hangeul");
    assert_eq!(res.total, 10);
    assert_eq!(res.correct, 10);
    assert_eq!(res.percent, 100);
    assert_eq!(res.wrong, 0);
    assert_eq!(res.headline, "Sempurna!");
    assert_eq!(res.ring_tone, "ok");
    assert_eq!(res.time, "1:01");
    assert!(res.message.is_empty() || !res.message.is_empty()); // hanya cek tipe
    assert!(res.wrong_answers.is_empty());

    quiz_end();
    assert!(!quiz_active());
    assert!(quiz_result(0).is_none());

    // 6. Progres mencatat skor terbaik tanpa pernah turun.
    let rec = progress_record(empty.clone(), res.clone(), 1_700_000_000_000);
    let entries = progress_entries(rec.clone());
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(e.id, "hangeul");
    assert_eq!(e.best_pct, 100);
    assert_eq!(e.best_correct, 10);
    assert!(e.done > 0);

    // Layar awal sekarang melaporkan progres.
    let h2 = home(rec);
    assert!(h2.has_progress);
    assert_eq!(h2.topics_done, "1/10");
    assert_eq!(h2.quiz_score, "100%");
    assert!(h2.cards[0].done);
    assert!(h2.cards[0].badge_is_score);
    assert_eq!(h2.cards[0].badge, "100%");

    // 7. Jumlah soal dijawab naik satu per pilihan dan tetap tersimpan
    //    bersama hasil kuis.
    let a1 = progress_add_answered(empty.clone(), 1);
    let a2 = progress_add_answered(a1, 1);
    assert_eq!(home(a2.clone()).answered, "2");
    let combined = progress_record(a2, res.clone(), 1_700_000_000_001);
    let h3 = home(combined.clone());
    assert_eq!(h3.answered, "2");
    assert_eq!(h3.quiz_score, "100%");

    // 8. Latihan menulis: spasi & tanda akhir diabaikan, jawaban salah dikenali.
    let t = topic(s("urutan")).expect("topik urutan");
    assert!(!t.lat.is_empty(), "topik urutan harus punya latihan");
    let ex = &t.lat[0];
    assert_eq!(lat_check(s("urutan"), 0, ex.ko.clone()), "ok");
    // Kotak kosong: tidak ada status (spasi saja dianggap "bad", sama seperti web).
    assert_eq!(lat_check(s("urutan"), 0, String::new()), "");
    assert_eq!(lat_check(s("urutan"), 0, s("salah sekali")), "bad");
    // Topik/soal tidak dikenal juga tidak memberi status.
    assert_eq!(lat_check(s("urutan"), 999, ex.ko.clone()), "");
    assert_eq!(lat_check(s("tidak-ada"), 0, ex.ko.clone()), "");
    assert_eq!(lat_inline_result(s("urutan"), 0, ex.ko.clone()), "Tepat!");
    assert_eq!(lat_inline_result(s("urutan"), 0, s("salah")), "");

    // "Tampilkan jawaban" mengisi semua baris dan memberi ringkasan.
    let typed: Vec<String> = (0..t.lat.len()).map(|_| String::new()).collect();
    let reveal = lat_reveal(s("urutan"), typed).expect("reveal");
    assert_eq!(reveal.lines.len(), t.lat.len());
    assert!(reveal.status.contains(" dari "));
    assert!(reveal.status.ends_with("tepat."));
    assert!(reveal.lines[0].text.starts_with("Jawaban: "));

    // 9. Teks pelafalan: romanisasi dalam kurung dibuang, panah dipakai.
    assert_eq!(tts_text(s("안녕하세요 (annyeonghaseyo)")), "안녕하세요");
    assert_eq!(tts_text(s("저 → saya")), "saya");
    assert!(!tts_map().is_empty());
}

#[test]
fn ffi_theme_is_sanitized_like_the_web() {
    assert_eq!(sanitize_theme(s("dark")), "dark");
    assert_eq!(sanitize_theme(s("light")), "light");
    assert_eq!(sanitize_theme(s("")), "dark");
    assert_eq!(sanitize_theme(s("LIGHT")), "dark");
    assert_eq!(sanitize_theme(s("apa-saja")), "dark");
}

#[test]
fn ffi_wrong_answers_are_explained() {
    // Jawab semua soal dengan pilihan yang SALAH (kecuali bila mustahil).
    let topik = topic(s("hangeul")).expect("topik hangeul");
    assert!(quiz_start(s("hangeul"), s("home"), 0));
    loop {
        let st = quiz_state().expect("state");
        if !st.answered {
            let wrong = ((st.answer + 1) % 4) as i32;
            assert!(quiz_pick(wrong), "pilihan salah tetap harus diterima");
        }
        if quiz_advance() == "finished" {
            break;
        }
    }
    let res = quiz_result(0).expect("hasil");
    assert_eq!(res.correct, 0);
    assert_eq!(res.percent, 0);
    assert_eq!(res.wrong, topik.quiz.len() as u32);
    assert_eq!(res.headline, "Ayo coba lagi");
    assert_eq!(res.ring_tone, "bad");
    assert_eq!(res.wrong_answers.len(), topik.quiz.len());
    let w = &res.wrong_answers[0];
    assert!(!w.q.is_empty(), "pertanyaan harus ada di pembahasan");
    assert!(!w.right.is_empty(), "jawaban benar harus ada");
    assert_ne!(w.picked, w.right);
    quiz_end();
}
