//! 한국어 문법 — inti aplikasi (Rust).
//!
//! Semua logika aplikasi yang dulu ada di `js/app.js` + bank materi dari
//! `js/data.js` tinggal di sini: topik, blok materi, mesin kuis, progres,
//! latihan menulis, dan teks pelafalan.
//!
//! Crate ini **tanpa satu pun dependensi** (tidak ada crate pihak ketiga,
//! termasuk untuk JSON — parser progres di `progress.rs` ditulis sendiri),
//! dan tidak memakai `unsafe`. Ia dipakai langsung oleh `kg-ffi` (UniFFI)
//! sehingga Kotlin cukup memanggil fungsi Rust ini.
//!
//! Kode UI (Compose) memang bahasa Kotlin — itu satu-satunya bagian yang
//! ditulis di luar Rust, sesuai permintaan ("rust bila memungkinkan").
#![forbid(unsafe_code)]

pub mod latihan;
pub mod model;
pub mod progress;
pub mod quiz;
pub mod tts;

mod data_gen;

/// Seluruh topik, urut nomor (`window.KG_TOPICS`).
pub use data_gen::TOPICS;
pub use latihan::{check, inline_result, is_correct, normalize, reveal, Check, Reveal, RevealLine};
pub use model::{level_name, option_letter, Block, Example, Exercise, Question, Rule, Topic};
pub use progress::{ProgressStore, TopicProgress, STORE_KEY};
pub use quiz::{
    format_duration, letter, percent, quiz_button_label, topic_meta, Advance, Feedback, QuizResult,
    QuizSession, QuizState, WrongAnswer,
};
pub use tts::speak_text;

/// Cari topik lewat id (`findTopic` di `js/app.js`).
pub fn find_topic(id: &str) -> Option<&'static Topic> {
    TOPICS.iter().find(|t| t.id == id)
}

/// Topik pertama yang belum dikerjakan, atau topik 1 bila semuanya tuntas
/// (dipakai tombol "Lanjut topik N" dan "Mulai dari topik 1").
pub fn next_topic(store: &ProgressStore) -> &'static Topic {
    TOPICS
        .iter()
        .find(|t| store.get(t.id).is_none())
        .unwrap_or(&TOPICS[0])
}

/// Topik setelah `topic` (`null` bila sudah yang terakhir).
pub fn topic_after(topic: &Topic) -> Option<&'static Topic> {
    let i = TOPICS.iter().position(|t| t.id == topic.id)?;
    TOPICS.get(i + 1)
}

/// Topik sebelum `topic`.
pub fn topic_before(topic: &Topic) -> Option<&'static Topic> {
    let i = TOPICS.iter().position(|t| t.id == topic.id)?;
    i.checked_sub(1).and_then(|j| TOPICS.get(j))
}

/// Versi kunci penyimpanan tema (`THEME_KEY` di `js/app.js`).
pub const THEME_KEY: &str = "kg-theme-v1";
/// Nilai tema yang dikenal.
pub const THEMES: [&str; 2] = ["dark", "light"];

/// Normalisasi pilihan tema; nilai asing dianggap gelap (bawaan web).
pub fn sanitize_theme(value: &str) -> &'static str {
    match value {
        "light" => "light",
        _ => "dark",
    }
}

/// Judul teks untuk tombol "Lanjut" di layar awal.
pub fn resume_label(store: &ProgressStore) -> (String, &'static Topic) {
    let next = next_topic(store);
    if store.count_done() >= TOPICS.len() {
        ("Ulangi dari topik 1".into(), next)
    } else {
        (format!("Lanjut topik {}: {}", next.no, next.title_id), next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_lookup_and_navigation() {
        assert_eq!(TOPICS.len(), 10);
        assert_eq!(find_topic("hangeul").unwrap().no, 1);
        assert!(find_topic("tidak-ada").is_none());

        let first = find_topic("hangeul").unwrap();
        assert!(topic_before(first).is_none());
        assert_eq!(topic_after(first).unwrap().no, 2);

        let last = &TOPICS[TOPICS.len() - 1];
        assert!(topic_after(last).is_none());
        assert_eq!(topic_before(last).unwrap().no, last.no - 1);
    }

    #[test]
    fn resume_points_to_first_unfinished_topic() {
        let mut store = ProgressStore::new();
        let (label, topic) = resume_label(&store);
        assert_eq!(topic.no, 1);
        assert_eq!(label, format!("Lanjut topik 1: {}", topic.title_id));

        // tandai semua tuntas
        for t in TOPICS {
            store.topics.push((
                t.id.into(),
                TopicProgress { best_pct: 100, best_correct: 8, total: 8, done: 1, last_at: 0 },
            ));
        }
        let (label, topic) = resume_label(&store);
        assert_eq!(label, "Ulangi dari topik 1");
        assert_eq!(topic.no, 1);
    }

    #[test]
    fn theme_values_sanitized() {
        assert_eq!(sanitize_theme("light"), "light");
        assert_eq!(sanitize_theme("dark"), "dark");
        assert_eq!(sanitize_theme("purple"), "dark");
        assert_eq!(sanitize_theme(""), "dark");
    }
}
