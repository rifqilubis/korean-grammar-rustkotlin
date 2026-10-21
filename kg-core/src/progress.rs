//! Progres belajar — padanan `loadStore` / `saveStore` / `recordResult` /
//! `countDone` / `averageScore` di `js/app.js`.
//!
//! Versi web menyimpannya sebagai JSON di `localStorage`. Di Android, kode ini
//! meng-encode dan meng-decode format yang **sama persis** — `{"version":1,
//! "topics":{…},"answered":…}` — sehingga ia tetap berupa string JSON yang
//! tinggal ditulis ke `SharedPreferences`, tanpa perlu pustaka JSON.

use crate::quiz::{percent, QuizResult};

/// Kunci penyimpanan, sama dengan versi web.
pub const STORE_KEY: &str = "kg-progress-v1";

/// Rekor satu topik.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopicProgress {
    /// Skor terbaik 0–100 (`bestPct`).
    pub best_pct: i32,
    /// Jumlah jawaban benar terbanyak (`bestCorrect`).
    pub best_correct: i32,
    /// Jumlah soal saat rekor dibuat (`total`).
    pub total: i32,
    /// Selalu ≥ 1 bila ada rekaman (`done`).
    pub done: i32,
    /// Waktu `Date.now()` terakhir (`lastAt`), milidetik epoch.
    pub last_at: i64,
}

/// Seluruh progres satu perangkat.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProgressStore {
    /// Disimpan sebagai `Vec` agar urutannya stabil saat di-encode.
    pub topics: Vec<(String, TopicProgress)>,
    /// Berapa soal yang pernah dijawab.
    pub answered: i64,
}

impl ProgressStore {
    /// Progres kosong (`blankStore`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Ambil rekor sebuah topik.
    pub fn get(&self, id: &str) -> Option<&TopicProgress> {
        self.topics.iter().find(|(k, _)| k == id).map(|(_, v)| v)
    }

    /// Jumlah topik yang sudah dikerjakan (`countDone`).
    pub fn count_done(&self) -> usize {
        self.topics.len()
    }

    /// Rata-rata skor terbaik, dibulatkan; `None` bila belum ada.
    pub fn average_score(&self) -> Option<i32> {
        if self.topics.is_empty() {
            return None;
        }
        let sum: i64 = self.topics.iter().map(|(_, p)| p.best_pct as i64).sum();
        let n = self.topics.len() as i64;
        Some(((2 * sum + n) / (2 * n)) as i32) // Math.round(sum / n)
    }

    /// Catat jawaban satu soal (`store.answered = (store.answered || 0) + 1`).
    pub fn add_answered(&mut self, n: i64) {
        self.answered += n;
    }

    /// Simpan hasil satu kuis (`recordResult`). Skor terbaik tidak pernah turun.
    pub fn record(&mut self, result: &QuizResult, now_ms: i64) {
        let pct = percent(result.correct, result.total) as i32;
        let prev = self.get(&result.topic_id).cloned();
        let best_pct = prev.as_ref().map(|p| p.best_pct).unwrap_or(-1).max(pct);
        let best_correct = prev
            .as_ref()
            .map(|p| p.best_correct)
            .unwrap_or(0)
            .max(result.correct as i32);
        let done = prev.as_ref().map(|p| p.done).unwrap_or(0).max(1);
        let entry = TopicProgress {
            best_pct,
            best_correct,
            total: result.total as i32,
            done,
            last_at: now_ms,
        };
        match self.topics.iter_mut().find(|(k, _)| k == &result.topic_id) {
            Some((_, slot)) => *slot = entry,
            None => self.topics.push((result.topic_id.clone(), entry)),
        }
    }

    /// Hapus semua progres (`store = blankStore(); saveStore();`).
    pub fn reset(&mut self) {
        self.topics.clear();
        self.answered = 0;
    }

    /// Tampilkan sebagai JSON gaya `JSON.stringify` versi web.
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(128 + self.topics.len() * 96);
        out.push_str("{\"version\":1,\"topics\":{");
        for (i, (id, p)) in self.topics.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&json_string(id));
            out.push_str(":{\"bestPct\":");
            out.push_str(&p.best_pct.to_string());
            out.push_str(",\"bestCorrect\":");
            out.push_str(&p.best_correct.to_string());
            out.push_str(",\"total\":");
            out.push_str(&p.total.to_string());
            out.push_str(",\"done\":");
            out.push_str(&p.done.to_string());
            out.push_str(",\"lastAt\":");
            out.push_str(&p.last_at.to_string());
            out.push('}');
        }
        out.push_str("},\"answered\":");
        out.push_str(&self.answered.to_string());
        out.push('}');
        out
    }

    /// Baca JSON hasil [`ProgressStore::to_json`]. Bila rusak, kembalikan
    /// progres kosong seperti `loadStore()` versi web — aplikasi tidak boleh
    /// gagal jalan gara-gara data lama.
    pub fn from_json(raw: &str) -> Self {
        match Json::new(raw).parse_store() {
            Some(store) => store,
            None => Self::new(),
        }
    }
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/* ------------------------------------------------------------------------
   Parser JSON kecil — cukup untuk format di atas (objek, angka, string),
   ditulis sendiri supaya `kg-core` tetap tanpa dependensi.
   ------------------------------------------------------------------------ */
struct Json<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Json<'a> {
    fn new(s: &'a str) -> Self {
        Self { b: s.as_bytes(), i: 0 }
    }

    fn skip_ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.skip_ws();
        if self.i < self.b.len() && self.b[self.i] == c {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        self.b.get(self.i).copied()
    }

    fn string(&mut self) -> Option<String> {
        if !self.eat(b'"') {
            return None;
        }
        let mut out = String::new();
        while let Some(&c) = self.b.get(self.i) {
            self.i += 1;
            match c {
                b'"' => return Some(out),
                b'\\' => {
                    let e = *self.b.get(self.i)?;
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'u' => {
                            let hex = std::str::from_utf8(self.b.get(self.i..self.i + 4)?).ok()?;
                            self.i += 4;
                            let code = u32::from_str_radix(hex, 16).ok()?;
                            if (0xD800..0xDC00).contains(&code) {
                                // pasangan surrogate: \uD83D\uDE00
                                if self.b.get(self.i) == Some(&b'\\') && self.b.get(self.i + 1) == Some(&b'u') {
                                    let low_hex =
                                        std::str::from_utf8(self.b.get(self.i + 2..self.i + 6)?).ok()?;
                                    let low = u32::from_str_radix(low_hex, 16).ok()?;
                                    if (0xDC00..0xE000).contains(&low) {
                                        self.i += 6;
                                        let cp = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                                        out.push(char::from_u32(cp)?);
                                        continue;
                                    }
                                }
                                out.push('\u{FFFD}');
                            } else {
                                out.push(char::from_u32(code)?);
                            }
                        }
                        _ => return None,
                    }
                }
                c if c < 0x80 => out.push(c as char),
                _ => {
                    // UTF-8 multi-byte: tempel kembali byte awalannya.
                    self.i -= 1;
                    let rest = std::str::from_utf8(&self.b[self.i..]).ok()?;
                    let ch = rest.chars().next()?;
                    out.push(ch);
                    self.i += ch.len_utf8();
                }
            }
        }
        None
    }

    fn number(&mut self) -> Option<i64> {
        self.skip_ws();
        let start = self.i;
        if self.b.get(self.i) == Some(&b'-') {
            self.i += 1;
        }
        let mut digits = 0;
        while let Some(&c) = self.b.get(self.i) {
            if c.is_ascii_digit() {
                self.i += 1;
                digits += 1;
            } else if c == b'.' || c == b'e' || c == b'E' || c == b'+' || c == b'-' {
                self.i += 1; // abaikan pecahan/eksponen; semua angka di sini bulat
            } else {
                break;
            }
        }
        if digits == 0 {
            return None;
        }
        let text: String = self.b[start..self.i].iter().map(|&c| c as char).collect();
        text.split(['.', 'e', 'E']).next()?.parse::<i64>().ok()
    }

    fn skip_value(&mut self) -> Option<()> {
        match self.peek()? {
            b'"' => self.string().map(|_| ()),
            b'{' => {
                self.eat(b'{');
                loop {
                    match self.peek()? {
                        b'}' => {
                            self.eat(b'}');
                            return Some(());
                        }
                        b',' => {
                            self.eat(b',');
                        }
                        _ => {
                            self.string()?;
                            if !self.eat(b':') {
                                return None;
                            }
                            self.skip_value()?;
                        }
                    }
                }
            }
            b'[' => {
                self.eat(b'[');
                loop {
                    match self.peek()? {
                        b']' => {
                            self.eat(b']');
                            return Some(());
                        }
                        b',' => {
                            self.eat(b',');
                        }
                        _ => self.skip_value()?,
                    }
                }
            }
            b't' | b'f' | b'n' => {
                while matches!(self.peek(), Some(c) if c.is_ascii_alphabetic()) {
                    self.i += 1;
                }
                Some(())
            }
            _ => self.number().map(|_| ()),
        }
    }

    /// `{ "version":1, "topics":{ id: {...} }, "answered":N }`
    fn parse_store(&mut self) -> Option<ProgressStore> {
        let mut store = ProgressStore::new();
        let mut saw_topics = false;
        if !self.eat(b'{') {
            return None;
        }
        loop {
            match self.peek()? {
                b'}' => {
                    self.eat(b'}');
                    break;
                }
                b',' => {
                    self.eat(b',');
                }
                _ => {
                    let key = self.string()?;
                    if !self.eat(b':') {
                        return None;
                    }
                    match key.as_str() {
                        "topics" => {
                            if !self.eat(b'{') {
                                return None;
                            }
                            saw_topics = true;
                            loop {
                                match self.peek()? {
                                    b'}' => {
                                        self.eat(b'}');
                                        break;
                                    }
                                    b',' => {
                                        self.eat(b',');
                                    }
                                    _ => {
                                        let id = self.string()?;
                                        if !self.eat(b':') {
                                            return None;
                                        }
                                        let p = self.parse_progress()?;
                                        store.topics.push((id, p));
                                    }
                                }
                            }
                        }
                        "answered" => store.answered = self.number()?,
                        _ => self.skip_value()?,
                    }
                }
            }
        }
        if !saw_topics {
            return None;
        }
        Some(store)
    }

    fn parse_progress(&mut self) -> Option<TopicProgress> {
        let mut p = TopicProgress {
            best_pct: -1,
            best_correct: 0,
            total: 0,
            done: 0,
            last_at: 0,
        };
        let mut saw_pct = false;
        if !self.eat(b'{') {
            return None;
        }
        loop {
            match self.peek()? {
                b'}' => {
                    self.eat(b'}');
                    break;
                }
                b',' => {
                    self.eat(b',');
                }
                _ => {
                    let key = self.string()?;
                    if !self.eat(b':') {
                        return None;
                    }
                    let v = self.number()?;
                    match key.as_str() {
                        "bestPct" => {
                            p.best_pct = v as i32;
                            saw_pct = true;
                        }
                        "bestCorrect" => p.best_correct = v as i32,
                        "total" => p.total = v as i32,
                        "done" => p.done = v as i32,
                        "lastAt" => p.last_at = v,
                        _ => {}
                    }
                }
            }
        }
        if !saw_pct {
            return None;
        }
        Some(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quiz::QuizResult;

    fn result(id: &str, correct: u32, total: u32) -> QuizResult {
        QuizResult {
            topic_id: id.to_string(),
            correct,
            total,
            percent: 0,
            seconds: 1,
            headline: String::new(),
            message: String::new(),
            wrong: Vec::new(),
        }
    }

    #[test]
    fn record_keeps_best() {
        let mut s = ProgressStore::new();
        s.record(&result("hangeul", 5, 8), 1000);
        assert_eq!(s.get("hangeul").unwrap().best_pct, 63);
        s.record(&result("hangeul", 3, 8), 2000);
        let p = s.get("hangeul").unwrap();
        assert_eq!(p.best_pct, 63, "skor terbaik tidak boleh turun");
        assert_eq!(p.total, 8);
        assert_eq!(p.last_at, 2000, "waktu terakhir tetap diperbarui");
        assert_eq!(s.count_done(), 1);
    }

    #[test]
    fn average_rounds_like_js() {
        let mut s = ProgressStore::new();
        for (id, pct) in [("a", 50), ("b", 75)] {
            s.record(&result(id, pct, 100), 0);
        }
        assert_eq!(s.average_score(), Some(63)); // 62.5 -> 63
        assert_eq!(ProgressStore::new().average_score(), None);
    }

    #[test]
    fn json_round_trip() {
        let mut s = ProgressStore::new();
        s.record(&result("hangeul", 8, 8), 1_700_000_000_000);
        s.record(&result("eun-neun", 7, 8), 1_700_000_001_000);
        s.add_answered(15);
        let raw = s.to_json();
        assert!(raw.starts_with("{\"version\":1,\"topics\":{\"hangeul\":{\"bestPct\":100"));
        let back = ProgressStore::from_json(&raw);
        assert_eq!(back, s);
    }

    #[test]
    fn broken_json_is_blank() {
        assert_eq!(ProgressStore::from_json(""), ProgressStore::new());
        assert_eq!(ProgressStore::from_json("nope"), ProgressStore::new());
        assert_eq!(ProgressStore::from_json("{\"a\":1}"), ProgressStore::new());
        assert!(ProgressStore::from_json(
            "{\"version\":1,\"topics\":{\"x\":{\"bestPct\":50,\"bestCorrect\":1,\"total\":2,\"done\":1,\"lastAt\":9}},\"answered\":3}"
        )
        .get("x")
        .is_some());
    }

    #[test]
    fn unicode_topic_ids_survive() {
        let mut s = ProgressStore::new();
        s.record(&result("한글/quiz", 1, 4), 5);
        let back = ProgressStore::from_json(&s.to_json());
        assert_eq!(back.get("한글/quiz").unwrap().best_pct, 25);
    }

    #[test]
    fn reset_clears() {
        let mut s = ProgressStore::new();
        s.record(&result("a", 1, 1), 0);
        s.add_answered(1);
        s.reset();
        assert_eq!(s, ProgressStore::new());
    }
}
