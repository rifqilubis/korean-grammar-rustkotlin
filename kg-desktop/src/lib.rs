//! Aplikasi desktop 한국어 문법.
//!
//! **Tidak ada logika aplikasi di crate ini.** Semua keputusan (materi, soal,
//! skor, progres, pencocokan latihan, teks UI berbahasa Indonesia) diambil
//! dari `kg-ffi`, yang membungkus `kg-core` — sumber yang sama dengan versi
//! Android. Crate ini hanya:
//!
//! 1. menyimpan string JSON progres ke berkas ([`store`]),
//! 2. memegang status layar ([`app`]),
//! 3. menggambar jendela dengan egui ([`ui`]).
//!
//! Karena keduanya memakai fungsi Rust yang sama, versi desktop tidak mungkin
//! berperilaku berbeda dari versi Android.

pub mod app;
pub mod audio;
pub mod palette;
pub mod store;
pub mod ui;

pub use app::{App, Route, LatEntry, LatCheck};
pub use palette::Palette;

/// Versi tampilan aplikasi (dipakai footer "About").
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Judul jendela.
pub const TITLE: &str = "한국어 문법 — Korean Grammar";
