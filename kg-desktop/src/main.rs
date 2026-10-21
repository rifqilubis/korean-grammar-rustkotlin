//! Titik masuk aplikasi desktop.
//!
//! Tidak ada logika di sini: buat [`App`], lalu serahkan ke [`ui::run`].

fn main() -> eframe::Result<()> {
    kg_desktop::ui::run()
}
