//! Penyimpanan progres & tema di berkas.
//!
//! Format isinya **sama persis** dengan versi web (`localStorage`) dan Android
//! (`SharedPreferences`): satu string JSON berkunci `kg-progress-v1`. Bedanya
//! hanya tempat: desktop menaruhnya di folder data pengguna.
//!
//! Karena formatnya identik, berkas `progress.json` di sini bisa disalin ke
//! perangkat Android (atau sebaliknya) tanpa konversi apa pun.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Nama berkas progres (isinya JSON dari `kg_ffi::progress_new()`).
const PROGRESS_FILE: &str = "progress.json";
/// Nama berkas tema ("dark" / "light").
const THEME_FILE: &str = "theme.txt";

/// Folder data aplikasi: `%APPDATA%\korean-grammar` di Windows,
/// `~/.local/share/korean-grammar` di Linux, `~/Library/Application Support/…`
/// di macOS. Bila tidak bisa ditentukan, pakai folder kerja.
pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("KG_DATA_DIR").map(PathBuf::from).or_else(default_base);
    match base {
        Some(dir) => dir.join("korean-grammar"),
        None => PathBuf::from(".korean-grammar"),
    }
}

fn default_base() -> Option<PathBuf> {
    // XDG dulu (Linux/macOS), lalu APPDATA (Windows).
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(xdg));
    }
    if cfg!(windows) {
        return std::env::var_os("APPDATA").map(PathBuf::from);
    }
    if let Some(home) = std::env::var_os("HOME") {
        let h = PathBuf::from(home);
        return Some(if cfg!(target_os = "macos") {
            h.join("Library/Application Support")
        } else {
            h.join(".local/share")
        });
    }
    std::env::var_os("USERPROFILE").map(PathBuf::from)
}

/// Berkas penyimpanan; `root` bisa diganti saat pengujian.
pub struct Store {
    dir: PathBuf,
}

impl Store {
    /// Simpanan di folder data bawaan.
    pub fn open_default() -> Self {
        Self::open(data_dir())
    }

    /// Simpanan di folder tertentu (dipakai uji dengan folder sementara).
    pub fn open(dir: impl Into<PathBuf>) -> Self {
        Store { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// JSON progres tersimpan, atau `None` bila belum pernah ada.
    pub fn load_progress(&self) -> Option<String> {
        read_trimmed(&self.path(PROGRESS_FILE))
    }

    /// Tulis progres ke disk. Kegagalan ditelan: aplikasi tetap jalan
    /// (progres hilang saat keluar, tapi tidak mengganggu pengguna).
    pub fn save_progress(&self, json: &str) -> io::Result<()> {
        self.write(PROGRESS_FILE, json)
    }

    /// Tema tersimpan ("dark"/"light"), atau `None` bila belum ada.
    pub fn load_theme(&self) -> Option<String> {
        read_trimmed(&self.path(THEME_FILE))
    }

    pub fn save_theme(&self, theme: &str) -> io::Result<()> {
        self.write(THEME_FILE, theme)
    }

    /// Hapus berkas progres (tombol "Hapus progres").
    pub fn clear_progress(&self) -> io::Result<()> {
        let p = self.path(PROGRESS_FILE);
        match fs::remove_file(p) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn write(&self, name: &str, body: &str) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        // Tulis ke berkas sementara lalu ganti nama: bila aplikasi mati di
        // tengah penulisan, progres lama tidak ikut rusak.
        let target = self.path(name);
        let tmp = self.path(&format!("{name}.tmp"));
        fs::write(&tmp, body)?;
        fs::rename(&tmp, &target)
    }
}

fn read_trimmed(p: &Path) -> Option<String> {
    let text = fs::read_to_string(p).ok()?;
    let t = text.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> PathBuf {
        let mut p = std::env::temp_dir();
        // nama unik tanpa dependensi tambahan
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        p.push(format!("kg-desktop-test-{n}"));
        p
    }

    #[test]
    fn progress_roundtrip() {
        let dir = temp();
        let s = Store::open(&dir);
        assert!(s.load_progress().is_none(), "awalnya kosong");

        let json = kg_ffi::progress_new();
        s.save_progress(&json).unwrap();
        assert_eq!(s.load_progress().as_deref(), Some(json.as_str()));

        s.clear_progress().unwrap();
        assert!(s.load_progress().is_none(), "terhapus");
        // clear dua kali tidak boleh error
        s.clear_progress().unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn theme_roundtrip() {
        let dir = temp();
        let s = Store::open(&dir);
        assert!(s.load_theme().is_none());
        s.save_theme("light").unwrap();
        assert_eq!(s.load_theme().as_deref(), Some("light"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_file_is_treated_as_missing() {
        let dir = temp();
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(PROGRESS_FILE), "   \n").unwrap();
        let s = Store::open(&dir);
        assert!(s.load_progress().is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn progress_survives_reopen() {
        let dir = temp();
        {
            let s = Store::open(&dir);
            s.save_progress("{\"version\":1,\"topics\":{},\"answered\":3}").unwrap();
        }
        let s = Store::open(&dir);
        assert_eq!(
            s.load_progress().as_deref(),
            Some("{\"version\":1,\"topics\":{},\"answered\":3}")
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
