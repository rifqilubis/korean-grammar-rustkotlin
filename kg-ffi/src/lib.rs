//! Jembatan UniFFI — semua yang bisa dipanggil Kotlin dari Rust.
//!
//! Kotlin tidak menyimpan aturan aplikasi sama sekali: ia hanya menggambar
//! apa yang dikembalikan fungsi-fungsi di sini. Tombol kuis, kunci jawaban,
//! perhitungan skor, penyimpanan progres (JSON), pencocokan latihan menulis,
//! dan teks pelafalan semuanya jalan di Rust (`kg-core`).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError};

use kg_core::model::Block;
use kg_core::progress::ProgressStore;
use kg_core::quiz::{Feedback, QuizSession};
use kg_core::{find_topic, latihan, level_name, quiz, tts, TOPICS, THEME_KEY};
use kg_core::{Advance, Check, QuizResult, Reveal, Topic};

uniffi::setup_scaffolding!();

/* =========================================================== konversi == */

/// Satu baris tabel aturan.
#[derive(Clone, Debug, uniffi::Record)]
pub struct RuleRowFfi {
    pub a: String,
    pub b: String,
    pub c: String,
}

/// Satu contoh kalimat.
#[derive(Clone, Debug, uniffi::Record)]
pub struct ExampleFfi {
    pub ko: String,
    pub rom: String,
    pub id: String,
    pub why: String,
}

/// Satu blok isi materi; `kind` = heading | text | formula | rules | examples | note.
#[derive(Clone, Debug, uniffi::Record)]
pub struct BlockFfi {
    pub kind: String,
    pub text: String,
    pub parts: Vec<String>,
    pub rules: Vec<RuleRowFfi>,
    pub examples: Vec<ExampleFfi>,
}

impl BlockFfi {
    fn from_block(b: &Block) -> Self {
        let mut out = BlockFfi {
            kind: String::new(),
            text: String::new(),
            parts: Vec::new(),
            rules: Vec::new(),
            examples: Vec::new(),
        };
        match b {
            Block::Heading { text } => {
                out.kind = "heading".into();
                out.text = (*text).into();
            }
            Block::Text { text } => {
                out.kind = "text".into();
                out.text = (*text).into();
            }
            Block::Note { text } => {
                out.kind = "note".into();
                out.text = (*text).into();
            }
            Block::Formula { parts } => {
                out.kind = "formula".into();
                out.parts = parts.iter().map(|s| (*s).to_string()).collect();
            }
            Block::Rules { items } => {
                out.kind = "rules".into();
                out.rules = items
                    .iter()
                    .map(|r| RuleRowFfi { a: r.a.into(), b: r.b.into(), c: r.c.into() })
                    .collect();
            }
            Block::Examples { items } => {
                out.kind = "examples".into();
                out.examples = items
                    .iter()
                    .map(|e| ExampleFfi {
                        ko: e.ko.into(),
                        rom: e.rom.into(),
                        id: e.id.into(),
                        why: e.why.into(),
                    })
                    .collect();
            }
        }
        out
    }
}

/// Satu soal kuis (selalu 4 pilihan).
#[derive(Clone, Debug, uniffi::Record)]
pub struct QuestionFfi {
    pub q: String,
    pub options: Vec<String>,
    pub answer: u32,
    pub why: String,
}

/// Satu soal latihan menulis.
#[derive(Clone, Debug, uniffi::Record)]
pub struct ExerciseFfi {
    pub ko: String,
    pub rom: String,
    pub id: String,
}

/// Materi satu topik lengkap (kalimat pembuka sampai latihan).
#[derive(Clone, Debug, uniffi::Record)]
pub struct TopicFfi {
    pub id: String,
    pub no: u32,
    pub title: String,
    pub title_id: String,
    pub level: u32,
    pub level_name: String,
    pub icon: String,
    pub intro: String,
    pub meta: String,
    pub quiz_label: String,
    pub blocks: Vec<BlockFfi>,
    pub quiz: Vec<QuestionFfi>,
    pub lat: Vec<ExerciseFfi>,
}

impl TopicFfi {
    fn from_topic(t: &Topic) -> Self {
        TopicFfi {
            id: t.id.into(),
            no: t.no,
            title: t.title.into(),
            title_id: t.title_id.into(),
            level: t.level as u32,
            level_name: level_name(t.level).into(),
            icon: t.icon.into(),
            intro: t.intro.into(),
            meta: quiz::topic_meta(t),
            quiz_label: quiz::quiz_button_label(t),
            blocks: t.blocks.iter().map(BlockFfi::from_block).collect(),
            quiz: t
                .quiz
                .iter()
                .map(|q| QuestionFfi {
                    q: q.q.into(),
                    options: q.options.iter().map(|s| (*s).to_string()).collect(),
                    answer: q.answer as u32,
                    why: q.why.into(),
                })
                .collect(),
            lat: t
                .lat
                .iter()
                .map(|l| ExerciseFfi { ko: l.ko.into(), rom: l.rom.into(), id: l.id.into() })
                .collect(),
        }
    }
}

/// Kartu di layar awal + statistik header.
#[derive(Clone, Debug, uniffi::Record)]
pub struct HomeFfi {
    pub topics_done: String,
    pub quiz_score: String,
    pub answered: String,
    pub resume_label: String,
    pub resume_topic: String,
    pub has_progress: bool,
    pub cards: Vec<TopicCardFfi>,
}

/// Satu kartu topik di daftar materi.
#[derive(Clone, Debug, uniffi::Record)]
pub struct TopicCardFfi {
    pub id: String,
    pub no: u32,
    pub title: String,
    pub title_id: String,
    pub icon: String,
    pub name: String,
    pub done: bool,
    /// `"85%"` bila sudah dikerjakan, atau nama level bila belum.
    pub badge: String,
    pub badge_is_score: bool,
    /// `"Topik 1 · Hangeul: dasar membaca"`.
    pub label: String,
}

/// Data satu soal untuk layar kuis.
#[derive(Clone, Debug, uniffi::Record)]
pub struct QuizStateFfi {
    pub topic_id: String,
    pub index: u32,
    pub total: u32,
    pub counter: String,
    pub topic_label: String,
    pub score_label: String,
    pub progress: u32,
    pub question: String,
    pub options: Vec<String>,
    pub letters: Vec<String>,
    pub picked: i32,
    pub answer: u32,
    pub answered: bool,
    pub explanation: String,
    pub feedback: String,
    pub feedback_kind: String,
    pub next_label: String,
    pub is_last: bool,
}

/// Satu jawaban salah di layar pembahasan.
#[derive(Clone, Debug, uniffi::Record)]
pub struct WrongAnswerFfi {
    pub q: String,
    pub picked: String,
    pub right: String,
    pub why: String,
}

/// Hasil kuis.
#[derive(Clone, Debug, uniffi::Record)]
pub struct QuizResultFfi {
    pub topic_id: String,
    pub correct: u32,
    pub total: u32,
    pub wrong: u32,
    pub percent: u32,
    pub headline: String,
    pub message: String,
    /// `"1:05"`
    pub time: String,
    /// Hijau bila ≥80%, aksen bila ≥50%, merah bila di bawahnya.
    pub ring_tone: String,
    pub wrong_answers: Vec<WrongAnswerFfi>,
}

/// Progres satu topik dari penyimpanan.
#[derive(Clone, Debug, uniffi::Record)]
pub struct ProgressEntryFfi {
    pub id: String,
    pub best_pct: i32,
    pub best_correct: i32,
    pub total: i32,
    pub done: i32,
    pub last_at: i64,
}

/// Hasil tombol "Tampilkan jawaban".
#[derive(Clone, Debug, uniffi::Record)]
pub struct RevealFfi {
    pub index: u32,
    pub check: String,
    pub text: String,
}

/// Ringkasan tombol "Tampilkan jawaban".
#[derive(Clone, Debug, uniffi::Record)]
pub struct RevealAllFfi {
    pub lines: Vec<RevealFfi>,
    pub status: String,
}

/* ============================================================ topik == */

/// Semua topik (10 buah). Dipakai sekali saat aplikasi dibuka.
#[uniffi::export]
pub fn all_topics() -> Vec<TopicFfi> {
    TOPICS.iter().map(TopicFfi::from_topic).collect()
}

/// Satu topik berdasarkan id; `None` bila tidak ada.
#[uniffi::export]
pub fn topic(id: String) -> Option<TopicFfi> {
    find_topic(&id).map(TopicFfi::from_topic)
}

/// Versi inti Rust (ditampilkan di layar About).
#[uniffi::export]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Kunci penyimpanan yang dipakai versi web, supaya progres lama tetap bisa dibaca.
#[uniffi::export]
pub fn store_key() -> String {
    kg_core::STORE_KEY.to_string()
}

/// Kunci penyimpanan tema.
#[uniffi::export]
pub fn theme_key() -> String {
    THEME_KEY.to_string()
}

/* ========================================================== progres == */

/// Progres kosong/enkode.
#[uniffi::export]
pub fn progress_new() -> String {
    ProgressStore::new().to_json()
}

/// Simpan hasil kuis ke progres; mengembalikan JSON progres yang baru.
#[uniffi::export]
pub fn progress_record(json: String, result: QuizResultFfi, now_ms: i64) -> String {
    let mut store = ProgressStore::from_json(&json);
    store.record(&core_result(result), now_ms);
    store.to_json()
}

/// Tambah jumlah soal dijawab (+1 tiap pilihan).
#[uniffi::export]
pub fn progress_add_answered(json: String, count: i64) -> String {
    let mut store = ProgressStore::from_json(&json);
    store.add_answered(count);
    store.to_json()
}

/// Hapus semua progres.
#[uniffi::export]
pub fn progress_reset() -> String {
    ProgressStore::new().to_json()
}

/// Apakah ada progres yang perlu ditawari tombol "Hapus progres".
#[uniffi::export]
pub fn progress_has_any(json: String) -> bool {
    let store = ProgressStore::from_json(&json);
    store.count_done() > 0 || store.answered > 0
}

/// Isi progres per topik (untuk layar About / uji).
#[uniffi::export]
pub fn progress_entries(json: String) -> Vec<ProgressEntryFfi> {
    ProgressStore::from_json(&json)
        .topics
        .iter()
        .map(|(id, p)| ProgressEntryFfi {
            id: id.clone(),
            best_pct: p.best_pct,
            best_correct: p.best_correct,
            total: p.total,
            done: p.done,
            last_at: p.last_at,
        })
        .collect()
}

/// Data layar awal (`renderHome`).
#[uniffi::export]
pub fn home(json: String) -> HomeFfi {
    let store = ProgressStore::from_json(&json);
    let done = store.count_done();
    let (resume_label, resume_topic) = kg_core::resume_label(&store);
    let avg = store.average_score();
    HomeFfi {
        topics_done: format!("{}/{}", done, TOPICS.len()),
        quiz_score: match avg {
            Some(v) => format!("{v}%"),
            None => "\u{2013}".to_string(),
        },
        answered: store.answered.to_string(),
        resume_label,
        resume_topic: resume_topic.id.to_string(),
        has_progress: done > 0 || store.answered > 0,
        cards: TOPICS
            .iter()
            .map(|t| {
                let prog = store.get(t.id);
                TopicCardFfi {
                    id: t.id.into(),
                    no: t.no,
                    title: t.title.into(),
                    title_id: t.title_id.into(),
                    icon: t.icon.into(),
                    name: format!("{}. {}", t.no, t.title),
                    done: prog.is_some(),
                    badge: match prog {
                        Some(p) => format!("{}%", p.best_pct.max(0)),
                        None => level_name(t.level).to_string(),
                    },
                    badge_is_score: prog.is_some(),
                    label: format!("Topik {} · {}", t.no, t.title_id),
                }
            })
            .collect(),
    }
}

/* ============================================================= kuis == */

fn slot() -> &'static Mutex<Option<QuizSession>> {
    static SESSION: OnceLock<Mutex<Option<QuizSession>>> = OnceLock::new();
    SESSION.get_or_init(|| Mutex::new(None))
}

fn with_session<T>(f: impl FnOnce(&mut QuizSession) -> T) -> Option<T> {
    let mut guard = slot().lock().unwrap_or_else(PoisonError::into_inner);
    guard.as_mut().map(f)
}

/// Mulai kuis untuk sebuah topik (`startQuiz`). `from` = "lesson" atau "home".
/// Mengembalikan `false` bila id topik tidak dikenal.
#[uniffi::export]
pub fn quiz_start(topic_id: String, from: String, now_ms: u64) -> bool {
    match find_topic(&topic_id) {
        Some(t) => {
            let mut guard = slot().lock().unwrap_or_else(PoisonError::into_inner);
            *guard = Some(QuizSession::start(t, &from, now_ms));
            true
        }
        None => false,
    }
}

/// Tutup sesi kuis (`quiz = null`).
#[uniffi::export]
pub fn quiz_end() {
    let mut guard = slot().lock().unwrap_or_else(PoisonError::into_inner);
    *guard = None;
}

/// Apakah ada kuis yang sedang berjalan.
#[uniffi::export]
pub fn quiz_active() -> bool {
    slot().lock().unwrap_or_else(PoisonError::into_inner).is_some()
}

/// Data soal yang sedang dibuka (`paintQuestion`).
#[uniffi::export]
pub fn quiz_state() -> Option<QuizStateFfi> {
    with_session(|s| {
        let st = s.state();
        QuizStateFfi {
            topic_id: st.topic_id,
            index: st.index,
            total: st.total,
            counter: st.counter,
            topic_label: st.topic_label,
            score_label: st.score_label,
            progress: st.progress,
            question: st.question,
            letters: (0..st.options.len())
                .map(|i| quiz::letter(i as u32).to_string())
                .collect(),
            options: st.options,
            picked: st.picked,
            answer: st.answer,
            answered: st.answered,
            explanation: st.explanation.unwrap_or_default(),
            feedback_kind: match st.feedback {
                None => "none".to_string(),
                Some(Feedback::Correct) => "ok".to_string(),
                Some(Feedback::Wrong { .. }) => "bad".to_string(),
            },
            feedback: st.feedback.map(|f| f.text()).unwrap_or_default(),
            next_label: st.next_label,
            is_last: st.is_last,
        }
    })
}

/// Pilih jawaban; `false` bila klik diabaikan (sudah dijawab).
#[uniffi::export]
pub fn quiz_pick(index: i32) -> bool {
    with_session(|s| s.pick(index)).unwrap_or(false)
}

/// Tombol "Lanjut": "more" | "finished" | "blocked".
#[uniffi::export]
pub fn quiz_advance() -> String {
    match with_session(|s| s.advance()) {
        None => "blocked".into(),
        Some(Advance::Blocked) => "blocked".into(),
        Some(Advance::More) => "more".into(),
        Some(Advance::Finished) => "finished".into(),
    }
}

/// Ringkasan akhir (`finishQuiz`). `None` bila tidak ada kuis.
#[uniffi::export]
pub fn quiz_result(now_ms: u64) -> Option<QuizResultFfi> {
    with_session(|s| to_ffi_result(&s.result(now_ms)))
}

fn to_ffi_result(r: &QuizResult) -> QuizResultFfi {
    QuizResultFfi {
        topic_id: r.topic_id.clone(),
        correct: r.correct,
        total: r.total,
        wrong: r.total.saturating_sub(r.correct),
        percent: r.percent,
        headline: r.headline.clone(),
        message: r.message.clone(),
        time: quiz::format_duration(r.seconds),
        ring_tone: if r.percent >= 80 {
            "ok".into()
        } else if r.percent >= 50 {
            "accent".into()
        } else {
            "bad".into()
        },
        wrong_answers: r
            .wrong
            .iter()
            .map(|w| WrongAnswerFfi {
                q: w.q.clone(),
                picked: w.picked.clone(),
                right: w.right.clone(),
                why: w.why.clone(),
            })
            .collect(),
    }
}

fn core_result(r: QuizResultFfi) -> QuizResult {
    QuizResult {
        topic_id: r.topic_id,
        correct: r.correct,
        total: r.total,
        percent: r.percent,
        seconds: 0,
        headline: r.headline,
        message: r.message,
        wrong: Vec::new(),
    }
}

/* ===================================================== latihan menulis == */

/// Pemeriksaan satu kotak saat mengetik: "", "ok", atau "bad".
#[uniffi::export]
pub fn lat_check(topic_id: String, index: u32, typed: String) -> String {
    match find_topic(&topic_id) {
        // Soal di luar jangkauan tidak boleh dilaporkan sebagai "salah".
        Some(t) if (index as usize) < t.lat.len() => {
            match latihan::check(t, index as usize, &typed) {
                Check::Ok => "ok".into(),
                Check::Bad => "bad".into(),
                Check::Empty => String::new(),
            }
        }
        _ => String::new(),
    }
}

/// Teks kecil di bawah kotak: "Tepat!" atau kosong.
#[uniffi::export]
pub fn lat_inline_result(topic_id: String, index: u32, typed: String) -> String {
    match find_topic(&topic_id) {
        Some(t) if (index as usize) < t.lat.len() => {
            latihan::inline_result(latihan::check(t, index as usize, &typed))
        }
        _ => String::new(),
    }
}

/// Tombol "Tampilkan jawaban" untuk semua kotak.
#[uniffi::export]
pub fn lat_reveal(topic_id: String, typed: Vec<String>) -> Option<RevealAllFfi> {
    find_topic(&topic_id).map(|t| {
        let Reveal { lines, status } = latihan::reveal(t, &typed);
        RevealAllFfi {
            lines: lines
                .into_iter()
                .map(|l| RevealFfi {
                    index: l.index as u32,
                    check: l.check.css_class().to_string(),
                    text: l.text,
                })
                .collect(),
            status,
        }
    })
}

/* ============================================================== suara == */

/// Teks yang dibacakan mesin suara untuk sebuah contoh Korea.
#[uniffi::export]
pub fn tts_text(ko: String) -> String {
    tts::speak_text(&ko)
}

/// Pemetaan semua contoh: teks asli -> teks yang dibacakan.
#[uniffi::export]
pub fn tts_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for t in TOPICS {
        for b in t.blocks {
            if let Block::Examples { items } = b {
                for ex in items.iter() {
                    map.insert(ex.ko.to_string(), tts::speak_text(ex.ko));
                }
            }
        }
    }
    map
}

/* ============================================================== tema == */

/// Normalisasi nama tema ("light" atau "dark").
#[uniffi::export]
pub fn sanitize_theme(value: String) -> String {
    kg_core::sanitize_theme(&value).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topics_cross_ffi_intact() {
        let all = all_topics();
        assert_eq!(all.len(), 10);
        let total_quiz: usize = all.iter().map(|t| t.quiz.len()).sum();
        assert_eq!(total_quiz, 82);
        let t = topic("hangeul".into()).unwrap();
        assert_eq!(t.title, "한글 — Huruf Korea");
        assert!(t.blocks.iter().any(|b| b.kind == "examples"));
        assert_eq!(t.lat.len(), t.lat.len());
    }

    #[test]
    fn quiz_session_over_ffi() {
        assert!(quiz_start("hangeul".into(), "lesson".into(), 0));
        assert!(quiz_active());
        let st = quiz_state().unwrap();
        assert_eq!(st.total, 10);
        assert_eq!(st.topic_label, "Topik 1 · Hangeul: dasar membaca");
        assert!(quiz_pick(st.answer as i32));
        assert!(!quiz_pick(0));
        let after = quiz_state().unwrap();
        assert_eq!(after.feedback_kind, "ok");
        assert_eq!(after.feedback, "잘했어요! Benar.");
        assert_eq!(quiz_advance(), "more");
        assert_eq!(quiz_state().unwrap().index, 1);
        let r = quiz_result(5000).unwrap();
        assert_eq!(r.topic_id, "hangeul");
        assert_eq!(r.correct, 1);
        quiz_end();
        assert!(!quiz_active());
        assert!(quiz_state().is_none());
        assert!(!quiz_start("tidak-ada".into(), "home".into(), 0));
    }

    #[test]
    fn progress_and_home_over_ffi() {
        let json = progress_new();
        assert!(!progress_has_any(json.clone()));
        let h = home(json.clone());
        assert_eq!(h.topics_done, "0/10");
        assert_eq!(h.quiz_score, "\u{2013}");
        assert_eq!(h.cards.len(), 10);
        assert!(h.cards.iter().all(|c| !c.done && !c.badge_is_score));
        assert_eq!(h.cards[0].badge, "Dasar");

        let result = QuizResultFfi {
            topic_id: "hangeul".into(),
            correct: 8,
            total: 8,
            wrong: 0,
            percent: 100,
            headline: "Sempurna!".into(),
            message: String::new(),
            time: "0:20".into(),
            ring_tone: "ok".into(),
            wrong_answers: Vec::new(),
        };
        let json = progress_record(json, result, 1_700_000_000_000);
        let json = progress_add_answered(json, 8);
        let h = home(json.clone());
        assert_eq!(h.topics_done, "1/10");
        assert_eq!(h.quiz_score, "100%");
        assert_eq!(h.answered, "8");
        assert!(h.has_progress);
        assert_eq!(h.resume_topic, "urutan");
        let card = &h.cards[0];
        assert!(card.done && card.badge_is_score);
        assert_eq!(card.badge, "100%");
        assert_eq!(card.name, "1. 한글 — Huruf Korea");
        let entries = progress_entries(json);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].best_correct, 8);
        assert!(!progress_has_any(progress_reset()));
    }

    #[test]
    fn latihan_over_ffi() {
        assert_eq!(lat_check("hangeul".into(), 0, String::new()), "");
        assert_eq!(lat_check("hangeul".into(), 0, "salah".into()), "bad");
        let ko = topic("hangeul".into()).unwrap().lat[0].ko.clone();
        assert_eq!(lat_check("hangeul".into(), 0, ko.clone()), "ok");
        let typed = vec![ko, String::new()];
        let r = lat_reveal("hangeul".into(), typed).unwrap();
        assert!(r.status.contains("tepat."));
        assert_eq!(r.lines[0].check, "ok");
        assert!(lat_reveal("tidak-ada".into(), Vec::new()).is_none());
    }

    #[test]
    fn misc_over_ffi() {
        let map = tts_map();
        assert!(map.len() >= 50);
        assert_eq!(tts_text("먹다 → 먹어요".into()), "먹어요");
        assert_eq!(sanitize_theme("light".into()), "light");
        assert_eq!(sanitize_theme("x".into()), "dark");
        assert_eq!(store_key(), "kg-progress-v1");
        assert_eq!(theme_key(), "kg-theme-v1");
        assert!(!core_version().is_empty());
    }
}
