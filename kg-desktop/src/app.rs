//! Status aplikasi + jembatan ke Rust.
//!
//! Semua angka dan teks di sini berasal dari `kg_ffi`. Kelas ini hanya:
//! menyimpan transisi layar (Route), menyimpan ketikan latihan, dan
//! menuliskan JSON progres yang diterima Rust ke berkas.

use std::collections::HashMap;

use kg_ffi as ffi;
use kg_ffi::{HomeFfi, QuizResultFfi, QuizStateFfi, RevealAllFfi, TopicFfi};

use crate::audio::Speaker;
use crate::store::Store;

/// Layar yang sedang dibuka.
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Home,
    Lesson { topic_id: String },
    Latihan { topic_id: String },
    Quiz { topic_id: String, from: String },
    Result { topic_id: String, from: String },
}

/// Status satu kotak latihan, hasil `ffi::lat_check()`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum LatCheck {
    #[default]
    Empty,
    Ok,
    Bad,
}

impl LatCheck {
    pub fn of(raw: &str) -> Self {
        match raw {
            "ok" => LatCheck::Ok,
            "bad" => LatCheck::Bad,
            _ => LatCheck::Empty,
        }
    }
}

/// Isi kotak: ketikan, status Rust, dan teks kecil di bawahnya.
#[derive(Clone, Debug, Default)]
pub struct LatEntry {
    pub typed: String,
    pub check: LatCheck,
    pub inline: String,
}

pub struct App {
    pub store: Store,
    pub speaker: Speaker,

    /// JSON progres (format sama dengan web/Android).
    progress_json: String,
    /// "dark" atau "light" — sudah dibatasi `ffi::sanitize_theme`.
    pub theme: String,

    pub route: Route,
    pub topics: Vec<TopicFfi>,
    /// Topik dibuka; ada saat Route::Lesson/Latihan.
    pub current: Option<TopicFfi>,
    /// Status kuis terakhir yang dibaca dari Rust.
    pub quiz: Option<QuizStateFfi>,
    pub result: Option<QuizResultFfi>,
    pub lat_entries: Vec<LatEntry>,
    pub lat_status: String,
    /// Teks asli contoh -> teks yang dibacakan (dari Rust).
    pub tts_map: HashMap<String, String>,

    /// Status sementara: pesan galat penyimpanan dsb.
    pub toast: Option<String>,
}

impl App {
    /// Siapkan aplikasi: baca berkas, minta data awal ke Rust.
    pub fn new(store: Store, speaker: Speaker) -> Self {
        let progress_json = store.load_progress().unwrap_or_else(ffi::progress_new);
        // Tema awal: pilihan tersimpan, kalau belum ada pakai gelap (bawaan web).
        let theme = ffi::sanitize_theme(store.load_theme().unwrap_or_else(|| "dark".into()));
        let mut app = App {
            store,
            speaker,
            progress_json,
            theme,
            route: Route::Home,
            topics: Vec::new(),
            current: None,
            quiz: None,
            result: None,
            lat_entries: Vec::new(),
            lat_status: String::new(),
            tts_map: HashMap::new(),
            toast: None,
        };
        app.reload_topics();
        app
    }

    /// Muat ulang daftar topik dari Rust (sekali saat start).
    pub fn reload_topics(&mut self) {
        self.topics = ffi::all_topics();
        self.tts_map = ffi::tts_map();
    }

    pub fn palette(&self) -> crate::Palette {
        crate::Palette::of(&self.theme)
    }

    pub fn progress_json(&self) -> &str {
        &self.progress_json
    }

    /// Data layar awal (statistik + kartu), dihitung Rust.
    pub fn home(&self) -> HomeFfi {
        ffi::home(self.progress_json.clone())
    }

    /* ----------------------------------------------------------- tema -- */

    pub fn toggle_theme(&mut self) {
        let next = if self.theme == "light" { "dark" } else { "light" };
        self.theme = ffi::sanitize_theme(next.to_string());
        if let Err(e) = self.store.save_theme(&self.theme) {
            self.toast = Some(format!("Tema tidak bisa disimpan: {e}"));
        }
    }

    /* -------------------------------------------------------- progres -- */

    fn set_progress(&mut self, json: String) {
        if let Err(e) = self.store.save_progress(&json) {
            self.toast = Some(format!("Progres tidak bisa disimpan: {e}"));
        }
        self.progress_json = json;
    }

    pub fn reset_progress(&mut self) {
        self.set_progress(ffi::progress_reset());
        if let Err(e) = self.store.clear_progress() {
            self.toast = Some(format!("Progres tidak bisa dihapus: {e}"));
        }
        self.route = Route::Home;
    }

    /* ------------------------------------------------------ navigasi -- */

    pub fn open_topic(&mut self, id: &str) {
        self.current = ffi::topic(id.to_string()).or_else(|| self.topics.iter().find(|t| t.id == id).cloned());
        self.lat_entries.clear();
        self.lat_status.clear();
        self.route = Route::Lesson { topic_id: id.to_string() };
    }

    pub fn go_home(&mut self) {
        if ffi::quiz_active() {
            ffi::quiz_end();
        }
        self.quiz = None;
        self.result = None;
        self.route = Route::Home;
    }

    /// Judul topik sebelumnya/berikutnya untuk tombol navigasi di layar materi.
    pub fn neighbours(&self, topic_id: &str) -> (Option<String>, Option<String>) {
        let idx = self.topics.iter().position(|t| t.id == topic_id);
        match idx {
            Some(i) => (
                self.topics.get(i.wrapping_sub(1)).filter(|_| i > 0).map(|t| t.title.clone()),
                self.topics.get(i + 1).map(|t| t.title.clone()),
            ),
            None => (None, None),
        }
    }

    pub fn open_prev_topic(&mut self, topic_id: &str) {
        if let Some(i) = self.topics.iter().position(|t| t.id == topic_id) {
            if i > 0 {
                let id = self.topics[i - 1].id.clone();
                self.open_topic(&id);
            }
        }
    }

    pub fn open_next_topic(&mut self, topic_id: &str) {
        if let Some(i) = self.topics.iter().position(|t| t.id == topic_id) {
            if let Some(t) = self.topics.get(i + 1) {
                let id = t.id.clone();
                self.open_topic(&id);
            }
        }
    }

    /* ---------------------------------------------------------- kuis -- */

    /// Mulai kuis; `from` = "home" atau "lesson" (menentukan tombol kembali).
    pub fn start_quiz(&mut self, topic_id: &str, from: &str) -> bool {
        let now = now_ms();
        if !ffi::quiz_start(topic_id.to_string(), from.to_string(), now) {
            self.toast = Some("Kuis tidak bisa dimulai".into());
            return false;
        }
        self.result = None;
        self.quiz = ffi::quiz_state();
        self.route = Route::Quiz { topic_id: topic_id.to_string(), from: from.to_string() };
        true
    }

    /// Pilih jawaban; kembalikan `true` bila diterima Rust.
    pub fn pick(&mut self, index: i32) -> bool {
        let ok = ffi::quiz_pick(index);
        if ok {
            // Soal yang dijawab menambah penghitung "soal dijawab".
            let json = ffi::progress_add_answered(self.progress_json.clone(), 1);
            self.set_progress(json);
            self.quiz = ffi::quiz_state();
        }
        ok
    }

    /// Tombol "Lanjut"/"Lihat hasil". Mengembalikan true bila pindah layar.
    pub fn advance(&mut self) -> bool {
        match ffi::quiz_advance().as_str() {
            "more" => {
                self.quiz = ffi::quiz_state();
                true
            }
            "finished" => {
                if let Some(res) = ffi::quiz_result(now_ms()) {
                    // Simpan skor terbaik lewat Rust (best-score tidak pernah turun).
                    let json = ffi::progress_record(self.progress_json.clone(), res.clone(), now_ms() as i64);
                    self.set_progress(json);
                    self.result = Some(res);
                }
                if let Route::Quiz { topic_id, from } = self.route.clone() {
                    self.route = Route::Result { topic_id, from };
                }
                true
            }
            _ => false,
        }
    }

    /// Keluar dari kuis tanpa menyimpan (tombol ✕).
    pub fn quit_quiz(&mut self) {
        ffi::quiz_end();
        self.quiz = None;
        match self.route.clone() {
            Route::Quiz { topic_id, from } => {
                if from == "home" {
                    self.route = Route::Home;
                } else {
                    self.current = self.topics.iter().find(|t| t.id == topic_id).cloned();
                    self.route = Route::Lesson { topic_id };
                }
            }
            _ => self.route = Route::Home,
        }
    }

    /* ------------------------------------------------------- latihan -- */

    pub fn open_latihan(&mut self, topic_id: &str) {
        self.current = self.topics.iter().find(|t| t.id == topic_id).cloned();
        self.lat_entries = self
            .current
            .as_ref()
            .map(|t| vec![LatEntry::default(); t.lat.len()])
            .unwrap_or_default();
        self.lat_status.clear();
        self.route = Route::Latihan { topic_id: topic_id.to_string() };
    }

    /// Ketikan berubah: minta Rust menilai, simpan hasilnya.
    pub fn type_latihan(&mut self, topic_id: &str, index: usize, typed: String) {
        let check = ffi::lat_check(topic_id.to_string(), index as u32, typed.clone());
        let inline = ffi::lat_inline_result(topic_id.to_string(), index as u32, typed.clone());
        if let Some(e) = self.lat_entries.get_mut(index) {
            e.typed = typed;
            e.check = LatCheck::of(&check);
            e.inline = inline;
        }
    }

    /// Tombol "Tampilkan jawaban".
    pub fn reveal_latihan(&mut self, topic_id: &str) {
        let typed: Vec<String> = self.lat_entries.iter().map(|e| e.typed.clone()).collect();
        if let Some(all) = ffi::lat_reveal(topic_id.to_string(), typed) {
            self.apply_reveal(all);
        }
    }

    fn apply_reveal(&mut self, all: RevealAllFfi) {
        self.lat_status = all.status;
        for line in all.lines {
            let i = line.index as usize;
            if let Some(e) = self.lat_entries.get_mut(i) {
                e.check = LatCheck::of(&line.check);
                if e.typed.trim().is_empty() {
                    e.typed = line.text.clone();
                }
                e.inline = line.text;
            }
        }
    }

    /// Tombol "Kosongkan".
    pub fn clear_latihan(&mut self) {
        for e in &mut self.lat_entries {
            *e = LatEntry::default();
        }
        self.lat_status.clear();
    }

    /* ------------------------------------------------------------- tts */

    /// Bacakan teks Korea. Teks dirapikan Rust lebih dulu.
    pub fn speak(&self, ko: &str) {
        let text = self
            .tts_map
            .get(ko)
            .cloned()
            .unwrap_or_else(|| ffi::tts_text(ko.to_string()));
        self.speaker.say(&text);
    }
}

/// Waktu sekarang dalam milidetik (dipakai Rust untuk durasi kuis).
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
