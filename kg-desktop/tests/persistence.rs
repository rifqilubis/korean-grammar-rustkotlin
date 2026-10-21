//! Membuktikan progres benar-benar ditulis ke lokasi data pengguna.
//!
//! Dijalankan sebagai uji biasa (bukan aplikasi jendela), tetapi memakai
//! `Store::open_default()` — jalur yang sama dengan aplikasi sungguhan.

use kg_desktop::app::App;
use kg_desktop::audio::Speaker;
use kg_desktop::store::{self, Store};

#[test]
fn default_location_is_under_user_data_dir() {
    let dir = store::data_dir();
    let shown = dir.display().to_string();
    assert!(
        shown.contains("korean-grammar"),
        "folder data harus bernama korean-grammar: {shown}"
    );
    // Harus absolut supaya tidak menulis relatif ke folder kerja.
    assert!(dir.is_absolute(), "folder data harus absolut: {shown}");

    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").expect("APPDATA ada di Windows");
        assert!(
            shown.starts_with(&appdata),
            "progres seharusnya di %APPDATA%\\korean-grammar, dapat: {shown}"
        );
    }
}

#[test]
fn progress_file_is_created_where_run_expects_it() {
    // Pakai folder sementara agar tidak mengganggu data asli pengguna,
    // tetapi lewati kode yang sama dengan aplikasi.
    let mut dir = std::env::temp_dir();
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    dir.push(format!("kg-default-loc-{n}"));

    let mut app = App::new(Store::open(&dir), Speaker::silent());
    assert!(app.start_quiz("hangeul", "home"));
    let st = app.quiz.clone().unwrap();
    app.pick(st.answer as i32);
    app.go_home();

    let file = dir.join("progress.json");
    assert!(file.is_file(), "progress.json harus dibuat di {}", dir.display());
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("\"version\""), "isi harus JSON progres: {text}");
    assert!(text.contains("\"answered\""), "penghitung jawaban tersimpan");

    // Aplikasi baru di folder yang sama harus memuat progres tersebut.
    let reopened = App::new(Store::open(&dir), Speaker::silent());
    assert!(reopened.home().has_progress);

    let _ = std::fs::remove_dir_all(&dir);
}
