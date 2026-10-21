//! Uji alur lengkap aplikasi desktop **tanpa membuka jendela**.
//!
//! Yang diuji adalah lapisan status (`App`) yang dipakai jendela: apakah
//! rangkaian klik pengguna menghasilkan progres, skor, dan navigasi yang
//! benar. Semua angka tetap dihitung Rust (`kg-ffi`), jadi uji ini sekaligus
//! membuktikan aplikasi desktop memakai mesin yang sama dengan Android.
//!
//! **Penting:** `kg-ffi` menyimpan satu sesi kuis di variabel statis (sama
//! seperti versi Android, yang juga hanya punya satu layar kuis aktif). Karena
//! itu uji ini harus berjalan berurutan — lihat `serial()` di bawah.

use std::sync::{Mutex, MutexGuard, OnceLock};

use kg_desktop::app::{App, LatCheck, Route};
use kg_desktop::audio::Speaker;
use kg_desktop::store::Store;

/// Kunci global: cegah dua uji memakai sesi kuis yang sama bersamaan.
fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn temp_dir() -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("kg-desktop-flow-{n}"));
    p
}

/// Aplikasi baru dengan folder data sementara dan suara dimatikan.
fn app() -> (App, std::path::PathBuf) {
    let dir = temp_dir();
    let app = App::new(Store::open(&dir), Speaker::silent());
    (app, dir)
}

/// Jawab seluruh kuis dengan benar dan kembalikan hasilnya.
fn complete_quiz_fully_correct(app: &mut App) {
    let mut guard = 0;
    loop {
        let st = app.quiz.clone().expect("status kuis");
        if !st.answered {
            assert!(app.pick(st.answer as i32), "kunci jawaban dari Rust harus diterima");
        }
        guard += 1;
        assert!(guard < 200, "kuis tidak boleh berputar tanpa akhir");
        assert!(app.advance(), "advance harus diterima setelah menjawab");
        if matches!(app.route, Route::Result { .. }) {
            break;
        }
    }
}

#[test]
fn home_lists_all_topics_from_rust() {
    let _g = serial();
    let (app, dir) = app();
    let home = app.home();
    assert_eq!(home.cards.len(), 10);
    assert_eq!(home.topics_done, "0/10");
    assert!(!home.has_progress);
    assert!(home.resume_label.contains("topik"), "label dari Rust: {}", home.resume_label);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn full_quiz_reaches_result_and_persists() {
    let _g = serial();
    let (mut app, dir) = app();

    app.open_topic("hangeul");
    assert!(matches!(app.route, Route::Lesson { .. }));

    assert!(app.start_quiz("hangeul", "lesson"));
    assert!(matches!(app.route, Route::Quiz { .. }));

    complete_quiz_fully_correct(&mut app);

    let res = app.result.clone().expect("hasil kuis");
    assert_eq!(res.percent, 100);
    assert_eq!(res.correct, res.total);
    assert_eq!(res.ring_tone, "ok");
    assert!(res.wrong_answers.is_empty(), "semua benar, tak ada pembahasan");

    // Progres harus tersimpan di berkas dan terbaca lagi.
    let saved = app.store.load_progress().expect("progres tersimpan");
    let reopened = App::new(Store::open(&dir), Speaker::silent());
    assert_eq!(reopened.progress_json(), saved);
    let home = reopened.home();
    assert!(home.has_progress);
    assert_eq!(home.cards[0].badge, "100%");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn wrong_answer_shows_explanation_and_counts_answered() {
    let _g = serial();
    let (mut app, dir) = app();
    assert!(app.start_quiz("hangeul", "home"));

    let st = app.quiz.clone().unwrap();
    let wrong = (st.answer as i32 + 1) % st.options.len() as i32;
    assert!(app.pick(wrong));

    let after = app.quiz.clone().unwrap();
    assert!(after.answered);
    assert!(after.feedback.starts_with("Belum tepat"), "feedback: {}", after.feedback);
    assert_eq!(after.feedback_kind, "bad");
    assert!(!after.explanation.is_empty());

    // Menjawab satu soal menaikkan penghitung "soal dijawab".
    assert_eq!(app.home().answered, "1");
    app.go_home();
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn best_score_never_drops_across_sessions() {
    let _g = serial();
    let (mut app, dir) = app();

    // Sesi 1: semua benar.
    assert!(app.start_quiz("hangeul", "home"));
    complete_quiz_fully_correct(&mut app);
    assert_eq!(app.home().cards[0].badge, "100%");

    // Sesi 2: semua salah. Skor terbaik harus tetap 100%.
    app.go_home();
    assert!(app.start_quiz("hangeul", "home"));
    let mut guard = 0;
    loop {
        let st = app.quiz.clone().unwrap();
        if !st.answered {
            let w = (st.answer as i32 + 1) % 4;
            app.pick(w);
        }
        guard += 1;
        assert!(guard < 200);
        app.advance();
        if matches!(app.route, Route::Result { .. }) {
            break;
        }
    }
    assert_eq!(app.home().cards[0].badge, "100%", "skor terbaik tidak boleh turun");
    app.go_home();
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn latihan_matches_rust_and_reveals_answers() {
    let _g = serial();
    let (mut app, dir) = app();
    app.open_latihan("hangeul");
    assert!(matches!(app.route, Route::Latihan { .. }));
    assert!(!app.lat_entries.is_empty());

    let ko = app
        .topics
        .iter()
        .find(|t| t.id == "hangeul")
        .unwrap()
        .lat[0]
        .ko
        .clone();

    // Jawaban benar -> Ok dan teks kecil dari Rust.
    app.type_latihan("hangeul", 0, ko);
    assert_eq!(app.lat_entries[0].check, LatCheck::Ok);
    assert!(!app.lat_entries[0].inline.is_empty());

    // Jawaban salah -> Bad.
    app.type_latihan("hangeul", 0, "bukan jawaban".into());
    assert_eq!(app.lat_entries[0].check, LatCheck::Bad);

    // Kosong -> Empty, dan tidak dilaporkan salah.
    app.type_latihan("hangeul", 0, String::new());
    assert_eq!(app.lat_entries[0].check, LatCheck::Empty);

    // Tampilkan jawaban mengisi status.
    app.reveal_latihan("hangeul");
    assert!(app.lat_status.contains("tepat"), "status: {}", app.lat_status);

    app.clear_latihan();
    assert!(app.lat_entries.iter().all(|e| e.typed.is_empty()));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn theme_toggles_and_is_sanitized_by_rust() {
    let _g = serial();
    let (mut app, dir) = app();
    assert_eq!(app.theme, "dark", "bawaan gelap seperti web");
    app.toggle_theme();
    assert_eq!(app.theme, "light");
    assert!(!app.palette().is_dark);
    app.toggle_theme();
    assert_eq!(app.theme, "dark");

    // Pilihan tersimpan ke berkas.
    app.toggle_theme();
    let reopened = App::new(Store::open(&dir), Speaker::silent());
    assert_eq!(reopened.theme, "light");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn reset_progress_clears_file_and_state() {
    let _g = serial();
    let (mut app, dir) = app();
    assert!(app.start_quiz("i-ga", "home"));
    let st = app.quiz.clone().unwrap();
    app.pick(st.answer as i32);
    assert!(app.home().has_progress);

    app.go_home();
    app.reset_progress();
    assert!(!app.home().has_progress);
    assert_eq!(app.home().topics_done, "0/10");
    assert!(app.store.load_progress().is_none(), "berkas progres dihapus");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn navigation_between_topics_works() {
    let _g = serial();
    let (mut app, dir) = app();
    app.open_topic("hangeul");
    let (prev, next) = app.neighbours("hangeul");
    assert!(prev.is_none(), "topik 1 tidak punya sebelumnya");
    assert!(next.is_some());

    app.open_next_topic("hangeul");
    assert_eq!(app.route, Route::Lesson { topic_id: "urutan".into() });

    app.open_prev_topic("urutan");
    assert_eq!(app.route, Route::Lesson { topic_id: "hangeul".into() });
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn quitting_quiz_returns_to_origin() {
    let _g = serial();
    let (mut app, dir) = app();

    // dari beranda -> kembali ke beranda
    assert!(app.start_quiz("hangeul", "home"));
    app.quit_quiz();
    assert_eq!(app.route, Route::Home);

    // dari materi -> kembali ke materi
    app.open_topic("hangeul");
    assert!(app.start_quiz("hangeul", "lesson"));
    app.quit_quiz();
    assert!(matches!(app.route, Route::Lesson { .. }));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn every_topic_opens_with_quiz_and_latihan() {
    let _g = serial();
    let (mut app, dir) = app();
    let ids: Vec<String> = app.topics.iter().map(|t| t.id.clone()).collect();
    assert_eq!(ids.len(), 10);

    for id in ids {
        app.open_topic(&id);
        match &app.route {
            Route::Lesson { topic_id } => assert_eq!(topic_id, &id),
            other => panic!("rute salah untuk {id}: {other:?}"),
        }
        let topic = app.current.clone().expect("topik terbuka");
        assert!(!topic.blocks.is_empty(), "{id} tanpa materi");
        assert!(!topic.quiz.is_empty(), "{id} tanpa soal");

        app.open_latihan(&id);
        assert_eq!(app.lat_entries.len(), app.current.as_ref().unwrap().lat.len());
        app.go_home();
    }
    let _ = std::fs::remove_dir_all(dir);
}
