//! Uji alur menyeluruh — padanan `tools/smoke-test.js` milik versi web.
//!
//! Versi web mengemudikan `js/app.js` dengan DOM tiruan. Di sini alur yang
//! sama dikemudikan langsung pada mesin Rust: buka semua materi, kerjakan
//! 10 kuis sampai 100%, jawab salah dan lihat pembahasan, latihan menulis,
//! tombol "Tampilkan jawaban", progres tersimpan, tema, dan hapus progres.

use kg_core::*;

fn now(start: u64, step: u64) -> u64 {
    start + step * 1000
}

#[test]
fn data_10_topics_lenkap() {
    assert_eq!(TOPICS.len(), 10, "10 topik");

    let mut quiz_total = 0;
    let mut lat_total = 0;
    let mut examples = 0;
    let mut ids = std::collections::HashSet::new();
    for (i, t) in TOPICS.iter().enumerate() {
        assert_eq!(t.no, (i + 1) as u32, "nomor topik {} berurutan", t.id);
        assert!(ids.insert(t.id), "id topik unik: {}", t.id);
        assert!(!t.title.is_empty() && !t.title_id.is_empty() && !t.intro.is_empty());
        assert!(!t.icon.is_empty());
        assert!((1..=3).contains(&t.level), "level {} masuk akal", t.id);
        assert!(!t.blocks.is_empty(), "topik {} punya blok", t.id);
        for q in t.quiz {
            assert!(!q.q.is_empty());
            assert!(!q.why.is_empty(), "soal '{}' punya pembahasan", q.q);
            for opt in q.options {
                assert!(!opt.is_empty(), "semua pilihan terisi");
            }
            assert!(q.answer < 4);
        }
        for b in t.blocks {
            if let Block::Examples { items } = b {
                examples += items.len();
                for ex in items.iter() {
                    assert!(!ex.ko.is_empty());
                    assert!(!speak_text(ex.ko).trim().is_empty(), "contoh '{}' bisa dibacakan", ex.ko);
                }
            }
        }
        quiz_total += t.quiz.len();
        lat_total += t.lat.len();
    }
    assert_eq!(quiz_total, 82, "82 soal");
    assert_eq!(lat_total, 32, "32 latihan menulis");
    assert_eq!(examples, 61, "61 contoh kalimat");
}

#[test]
fn quiz_full_run_scores_100() {
    for t in TOPICS {
        let mut s = QuizSession::start(t, "lesson", 0);
        let initial = s.state();
        assert_eq!(initial.options.len(), 4, "topik {} selalu 4 pilihan", t.id);
        assert_eq!(initial.counter, format!("Soal 1 dari {}", t.quiz.len()));
        assert_eq!(initial.topic_label, format!("Topik {} · {}", t.no, t.title_id));
        assert_eq!(initial.progress, 0);
        assert!(!initial.answered);

        let mut guard = 0;
        loop {
            assert!(guard < 200, "alur tidak macet");
            guard += 1;
            let st = s.state();
            assert_eq!(st.answer, t.quiz[st.index as usize].answer as u32);
            assert!(s.pick(st.answer as i32), "bisa memilih saat belum dijawab");

            let st = s.state();
            assert!(st.answered);
            assert!(st.explanation.is_some(), "pembahasan tampil setelah menjawab");
            assert_eq!(st.feedback, Some(Feedback::Correct));
            assert!(!s.pick(0), "klik kedua diabaikan");
            assert_eq!(s.correct(), st.index + 1);

            match s.advance() {
                Advance::More => {
                    let next = s.state();
                    assert_eq!(next.index, st.index + 1);
                    assert!(!next.answered, "status kembali bersih di soal berikutnya");
                    assert_eq!(next.progress, percent(st.index + 1, st.total));
                }
                Advance::Finished => break,
                Advance::Blocked => panic!("tidak boleh terhalang setelah menjawab"),
            }
        }

        let r = s.result(3000);
        assert_eq!(r.percent, 100, "topik {}", t.id);
        assert_eq!(r.correct, t.quiz.len() as u32);
        assert_eq!(r.headline, "Sempurna!");
        assert!(r.wrong.is_empty());
        assert_eq!(r.seconds, 3);
    }
}

#[test]
fn wrong_answer_shows_explanation() {
    let t = find_topic("hangeul").unwrap();
    let mut s = QuizSession::start(t, "lesson", 0);
    let mut wrong_count = 0;
    let mut i = 0;
    loop {
        let st = s.state();
        let correct = st.answer as i32;
        let pick = if i == 0 { (correct + 1) % 4 } else { correct };
        if pick != correct {
            wrong_count += 1;
        }
        assert!(s.pick(pick));
        let after = s.state();
        if pick != correct {
            assert!(matches!(after.feedback, Some(Feedback::Wrong { .. })));
            assert_eq!(
                after.feedback.as_ref().unwrap().text(),
                format!("Belum tepat. Jawaban benar: {}", after.options[after.answer as usize])
            );
        } else {
            assert_eq!(after.feedback, Some(Feedback::Correct));
        }
        i += 1;
        if s.advance() == Advance::Finished {
            break;
        }
    }
    assert_eq!(wrong_count, 1);
    let r = s.result(10_000);
    assert_eq!(r.wrong.len(), 1);
    assert_eq!(r.wrong[0].q, t.quiz[0].q);
    assert_eq!(r.wrong[0].right, t.quiz[0].options[t.quiz[0].answer as usize]);
    assert_eq!(r.wrong[0].why, t.quiz[0].why);
    assert_eq!(r.correct, t.quiz.len() as u32 - 1);
    assert!(r.percent < 100);
    assert_eq!(format_duration(r.seconds), "0:10");
}

#[test]
fn advance_blocked_before_answering() {
    let t = &TOPICS[0];
    let mut s = QuizSession::start(t, "home", 0);
    assert_eq!(s.advance(), Advance::Blocked);
    assert_eq!(s.index(), 0);
    assert!(!s.pick(-1), "indeks negatif ditolak");
    assert!(!s.pick(4), "indeks di luar A–D ditolak");
    assert!(!s.state().answered);
}

#[test]
fn last_question_labels() {
    let t = &TOPICS[2];
    let mut s = QuizSession::start(t, "lesson", 0);
    let n = t.quiz.len();
    for _ in 0..n - 1 {
        let st = s.state();
        assert_eq!(st.next_label, "Lanjut");
        assert!(!st.is_last);
        assert_eq!(st.score_label, format!("Benar {}", s.correct()));
        s.pick(st.answer as i32);
        s.advance();
    }
    let st = s.state();
    assert!(st.is_last);
    assert_eq!(st.next_label, "Lihat hasil");
    assert_eq!(st.progress, percent(n as u32 - 1, n as u32));
    s.pick(st.answer as i32);
    assert_eq!(s.state().score_label, format!("Benar {n} dari {n}"));
}

#[test]
fn progress_round_trip_like_localstorage() {
    let mut store = ProgressStore::new();
    let mut answered = 0i64;
    for (i, t) in TOPICS.iter().enumerate() {
        let mut s = QuizSession::start(t, "lesson", 0);
        loop {
            let st = s.state();
            s.pick(st.answer as i32);
            answered += 1;
            if s.advance() == Advance::Finished {
                break;
            }
        }
        store.record(&s.result(now(0, i as u64)), now(0, i as u64) as i64);
    }
    store.add_answered(answered);

    assert_eq!(store.count_done(), 10);
    assert_eq!(store.average_score(), Some(100));
    assert_eq!(store.answered, 82);
    assert_eq!(store.topics[0].0, "hangeul", "urutan saat di-encode stabil");

    // string JSON yang sama bentuknya dengan yang ditulis versi web
    let raw = store.to_json();
    assert!(raw.starts_with("{\"version\":1,\"topics\":{\"hangeul\":{\"bestPct\":100"));
    assert_eq!(ProgressStore::from_json(&raw), store);

    // "Lanjut topik" pindah ke topik 1 lagi setelah semua tuntas
    assert_eq!(resume_label(&store).0, "Ulangi dari topik 1");

    let mut cleared = store.clone();
    cleared.reset();
    assert_eq!(cleared, ProgressStore::new());
    assert_eq!(cleared.count_done(), 0);
    assert_eq!(cleared.average_score(), None);
    assert_eq!(resume_label(&cleared).0, "Lanjut topik 1: Hangeul: dasar membaca");
}

#[test]
fn best_score_never_drops() {
    let t = &TOPICS[0];
    let mut store = ProgressStore::new();

    let mut good = QuizSession::start(t, "lesson", 0);
    loop {
        let st = good.state();
        good.pick(st.answer as i32);
        if good.advance() == Advance::Finished {
            break;
        }
    }
    store.record(&good.result(0), 0);
    assert_eq!(store.get(t.id).unwrap().best_pct, 100);

    let mut bad = QuizSession::start(t, "lesson", 0);
    loop {
        let st = bad.state();
        bad.pick(((st.answer + 1) % 4) as i32);
        if bad.advance() == Advance::Finished {
            break;
        }
    }
    store.record(&bad.result(0), 5);
    let p = store.get(t.id).unwrap();
    assert_eq!(p.best_pct, 100, "skor terbaik tetap 100%");
    assert_eq!(p.best_correct, t.quiz.len() as i32);
    assert_eq!(p.last_at, 5, "waktu terakhir diperbarui");
}

#[test]
fn latihan_menulis_flow() {
    let t = find_topic("hangeul").unwrap();
    assert!(!t.lat.is_empty());

    let first = t.lat[0];
    let typed = first.ko.trim_end_matches(['.', '!', '?']).to_string();
    assert!(is_correct(t, 0, &typed), "tanpa tanda baca tetap benar");
    assert!(is_correct(t, 0, &format!("  {}  ", first.ko)), "spasi di tepi diabaikan");
    assert!(!is_correct(t, 0, "salah"));
    assert!(!is_correct(t, 0, ""), "kotak kosong tidak pernah benar");
    assert_eq!(check(t, 0, ""), Check::Empty);
    assert_eq!(check(t, 0, first.ko), Check::Ok);
    assert_eq!(check(t, 1, "salah"), Check::Bad);
    assert_eq!(inline_result(Check::Ok), "Tepat!");

    let mut typed: Vec<String> = t.lat.iter().map(|_| String::new()).collect();
    typed[0] = first.ko.to_string();
    let r = reveal(t, &typed);
    assert_eq!(r.status, format!("1 dari {} tepat.", t.lat.len()));
    assert_eq!(r.lines[0].check, Check::Ok);
    assert!(r.lines[1].text.starts_with("Jawaban: "));
    assert!(r.lines[1].text.contains(t.lat[1].ko));
}

#[test]
fn navigation_helpers() {
    let t = find_topic("hangeul").unwrap();
    assert_eq!(topic_meta(t), "Topik 1 · Dasar");
    assert_eq!(quiz_button_label(t), format!("Kuis topik ini ({} soal)", t.quiz.len()));
    assert_eq!(letter(0), "A");
    assert_eq!(letter(3), "D");
    assert_eq!(level_name(2), "Menengah");
    assert_eq!(level_name(3), "Lanjutan");
    assert_eq!(topic_after(t).map(|n| n.id), TOPICS.get(1).map(|n| n.id));
    assert_eq!(topic_after(TOPICS.last().unwrap()), None);
}
