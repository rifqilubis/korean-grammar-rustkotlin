//! Mesin kuis pilihan ganda — padanan `startQuiz` / `paintQuestion` / `pick` /
//! `nextQuestion` / `finishQuiz` di `js/app.js`.
//!
//! Sesi menyimpan indeks soal yang sedang dibuka, jumlah benar, dan daftar
//! jawaban salah (untuk layar pembahasan). Tampilannya digambar di sisi UI;
//! semua aturan mainnya ada di sini.

use crate::model::{level_name, option_letter, Question, Topic};

/// Satu jawaban yang belum tepat, disimpan untuk layar hasil.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrongAnswer {
    /// Pertanyaan.
    pub q: String,
    /// Kata yang dipilih pengguna.
    pub picked: String,
    /// Jawaban yang seharusnya.
    pub right: String,
    /// Pembahasan.
    pub why: String,
}

/// Ringkasan setelah kuis selesai (`finishQuiz`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuizResult {
    pub topic_id: String,
    pub correct: u32,
    pub total: u32,
    /// 0–100, dibulatkan.
    pub percent: u32,
    /// Sekon, minimal 1.
    pub seconds: u64,
    /// Judul besar di layar hasil (`"Sempurna!"`, `"Bagus sekali!"`, …).
    pub headline: String,
    /// Kalimat ajakan di bawah judul.
    pub message: String,
    /// Hanya jawaban yang salah, urut seperti dijawab.
    pub wrong: Vec<WrongAnswer>,
}

/// Status satu soal pada saat digambar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuizState {
    pub topic_id: String,
    pub topic_no: u32,
    pub topic_title_id: String,
    /// Indeks soal (0-based).
    pub index: u32,
    pub total: u32,
    /// Teks `"Soal 3 dari 8"`.
    pub counter: String,
    /// Teks `"Topik 1 · Hangeul: dasar membaca"`.
    pub topic_label: String,
    /// Teks `"Benar 2"` atau `"Benar 2 dari 8"` (lihat [`QuizSession::score_label`]).
    pub score_label: String,
    /// Persentase kemajuan sebelum soal ini dijawab.
    pub progress: u32,
    pub question: String,
    pub options: Vec<String>,
    /// Indeks yang dipilih; `-1` bila belum menjawab.
    pub picked: i32,
    /// Indeks jawaban benar.
    pub answer: u32,
    pub answered: bool,
    /// Hanya diisi setelah menjawab.
    pub explanation: Option<String>,
    pub feedback: Option<Feedback>,
    /// `"Lihat hasil"` pada soal terakhir, `"Lanjut"` untuk lainnya.
    pub next_label: String,
    pub is_last: bool,
}

/// Umpan balik setelah memilih jawaban.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Feedback {
    /// `"잘했어요! Benar."`
    Correct,
    /// `"Belum tepat. Jawaban benar: …"`
    Wrong { right: String },
}

impl Feedback {
    pub fn text(&self) -> String {
        match self {
            Feedback::Correct => "잘했어요! Benar.".to_string(),
            Feedback::Wrong { right } => format!("Belum tepat. Jawaban benar: {right}"),
        }
    }
}

/// Sesi kuis yang sedang berjalan.
#[derive(Clone, Debug)]
pub struct QuizSession {
    topic: &'static Topic,
    index: u32,
    correct: u32,
    answered: bool,
    picked: i32,
    wrong: Vec<WrongAnswer>,
  /// Waktu mulai dalam milidetik (epoch).
    pub started_at_ms: u64,
    pub from: String,
}

impl QuizSession {
    /// Mulai kuis untuk sebuah topik (`startQuiz`).
    pub fn start(topic: &'static Topic, from: &str, started_at_ms: u64) -> Self {
        Self {
            topic,
            index: 0,
            correct: 0,
            answered: false,
            picked: -1,
            wrong: Vec::new(),
            started_at_ms,
            from: if from == "lesson" { "lesson".to_string() } else { "home".to_string() },
        }
    }

    pub fn topic(&self) -> &'static Topic {
        self.topic
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn total(&self) -> u32 {
        self.topic.quiz.len() as u32
    }

    pub fn correct(&self) -> u32 {
        self.correct
    }

    pub fn picked(&self) -> i32 {
        self.picked
    }

    pub fn answered(&self) -> bool {
        self.answered
    }

    pub fn wrong(&self) -> &[WrongAnswer] {
        &self.wrong
    }

    fn current(&self) -> &'static Question {
        &self.topic.quiz[self.index as usize]
    }

    /// `"Benar 0"` sebelum selesai, `"Benar 7 dari 8"` setelah selesai.
    pub fn score_label(&self) -> String {
        if self.answered && self.index as usize == self.topic.quiz.len() - 1 {
            format!("Benar {} dari {}", self.correct, self.total())
        } else {
            format!("Benar {}", self.correct)
        }
    }

    /// Data yang dibutuhkan layar kuis (`paintQuestion`).
    pub fn state(&self) -> QuizState {
        let q = self.current();
        let total = self.total();
        let is_last = self.index as usize + 1 == total as usize;
        QuizState {
            topic_id: self.topic.id.to_string(),
            topic_no: self.topic.no,
            topic_title_id: self.topic.title_id.to_string(),
            index: self.index,
            total,
            counter: format!("Soal {} dari {}", self.index + 1, total),
            topic_label: format!("Topik {} · {}", self.topic.no, self.topic.title_id),
            score_label: self.score_label(),
            progress: percent(self.index, total),
            question: q.q.to_string(),
            options: q.options.iter().map(|s| s.to_string()).collect(),
            picked: self.picked,
            answer: q.answer as u32,
            answered: self.answered,
            explanation: if self.answered && !q.why.is_empty() { Some(q.why.to_string()) } else { None },
            feedback: if !self.answered {
                None
            } else if self.picked == q.answer as i32 {
                Some(Feedback::Correct)
            } else {
                Some(Feedback::Wrong { right: q.options[q.answer as usize].to_string() })
            },
            next_label: if is_last { "Lihat hasil".to_string() } else { "Lanjut".to_string() },
            is_last,
        }
    }

    /// Pilih jawaban. Mengembalikan `false` bila soal ini sudah dijawab
    /// (atau indeks di luar 0–3) sehingga klik diabaikan, persis seperti
    /// `if (!quiz || quiz.answered) return;` di versi web.
    pub fn pick(&mut self, index: i32) -> bool {
        if self.answered || !(0..4).contains(&index) {
            return false;
        }
        let q = self.current();
        self.answered = true;
        self.picked = index;
        if index == q.answer as i32 {
            self.correct += 1;
        } else {
            self.wrong.push(WrongAnswer {
                q: q.q.to_string(),
                picked: q.options[index as usize].to_string(),
                right: q.options[q.answer as usize].to_string(),
                why: q.why.to_string(),
            });
        }
        true
    }

    /// Maju ke soal berikutnya. `None` bila belum menjawab; `Some(state)`
    /// untuk soal berikutnya; `Some(result)` (lewat [`QuizSession::result`])
    /// saat soal terakhir sudah dijawab — lihat [`QuizSession::advance`].
    pub fn advance(&mut self) -> Advance {
        if !self.answered {
            return Advance::Blocked;
        }
        if self.index as usize + 1 < self.topic.quiz.len() {
            self.index += 1;
            self.answered = false;
            self.picked = -1;
            Advance::More
        } else {
            Advance::Finished
        }
    }

    /// Ringkasan akhir (`finishQuiz`), dihitung dari jawaban yang diberikan.
    pub fn result(&self, now_ms: u64) -> QuizResult {
        let total = self.total();
        let pct = percent(self.correct, total);
        let seconds = ((now_ms.saturating_sub(self.started_at_ms)) / 1000).max(1);
        let (headline, message) = if pct == 100 {
            ("Sempurna!", "Semua jawaban tepat. Topik ini sudah dikuasai.")
        } else if pct >= 80 {
            ("Bagus sekali!", "Tinggal sedikit lagi. Baca ulang bagian yang salah, lalu coba lagi.")
        } else if pct >= 50 {
            ("Sudah lumayan", "Beberapa pola belum melekat. Buka materinya sebentar, lalu ulangi kuisnya.")
        } else {
            ("Ayo coba lagi", "Tidak apa-apa, ini topik baru. Baca materinya dulu, lalu kerjakan kuis sekali lagi.")
        };
        QuizResult {
            topic_id: self.topic.id.to_string(),
            correct: self.correct,
            total,
            percent: pct,
            seconds,
            headline: headline.to_string(),
            message: message.to_string(),
            wrong: self.wrong.clone(),
        }
    }
}

/// Hasil [`QuizSession::advance`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Advance {
    /// Belum menjawab soal ini; tombol "Lanjut" tidak melakukan apa-apa.
    Blocked,
    /// Sudah pindah ke soal berikutnya.
    More,
    /// Soal terakhir sudah dijawab; lanjut ke layar hasil.
    Finished,
}

/// `Math.round(a / b * 100)`, 0 bila `b == 0`. Dibuat dalam bilangan bulat
/// supaya sama persis dengan hasil JavaScript pada semua kasus.
pub fn percent(part: u32, total: u32) -> u32 {
    if total == 0 {
        return 0;
    }
    // round(part * 100 / total) tanpa pecahan floating point:
    // round(x) = floor(x + 1/2) = floor((2*num + den) / (2*den))
    let num = part as u64 * 100;
    let den = total as u64;
    ((2 * num + den) / (2 * den)) as u32
}

/// `"1.2"`-gaya mm:ss untuk kolom waktu (`Math.floor(s/60) + ':' + pad(s%60)`).
pub fn format_duration(seconds: u64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// Teks ringkas untuk tombol "Kerjakan kuis": `"Kuis topik ini (8 soal)"`.
pub fn quiz_button_label(topic: &Topic) -> String {
    format!("Kuis topik ini ({} soal)", topic.quiz_len())
}

/// Teks metadata materi: `"Topik 1 · Dasar"`.
pub fn topic_meta(topic: &Topic) -> String {
    format!("Topik {} · {}", topic.no, level_name(topic.level))
}

/// Huruf A–D untuk sebuah indeks pilihan.
pub fn letter(index: u32) -> &'static str {
    option_letter(index as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_rounds_like_js() {
        assert_eq!(percent(0, 8), 0);
        assert_eq!(percent(1, 8), 13); // 12.5 -> 13 (JS Math.round membulatkan ke atas)
        assert_eq!(percent(2, 3), 67);
        assert_eq!(percent(7, 8), 88); // 87.5 -> 88
        assert_eq!(percent(8, 8), 100);
        assert_eq!(percent(1, 0), 0);
    }

    #[test]
    fn duration_like_js() {
        assert_eq!(format_duration(0), "0:00");
        assert_eq!(format_duration(5), "0:05");
        assert_eq!(format_duration(61), "1:01");
        assert_eq!(format_duration(600), "10:00");
    }
}
