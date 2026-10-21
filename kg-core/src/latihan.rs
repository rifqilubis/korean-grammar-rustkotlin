//! Latihan menulis — padanan `normalize()` dan pengecekan per item di
//! `renderLat()` pada `js/app.js`.

use crate::model::Topic;

/// Status satu kotak latihan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    /// Belum ada isi — kotak netral.
    Empty,
    /// Jawaban cocok.
    Ok,
    /// Ada isi tetapi belum cocok.
    Bad,
}

impl Check {
    /// Kelas CSS versi web, dipakai untuk memilih warna di Compose.
    pub fn css_class(&self) -> &'static str {
        match self {
            Check::Empty => "",
            Check::Ok => "ok",
            Check::Bad => "bad",
        }
    }
}

/// Hasil menekan "Tampilkan jawaban".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevealLine {
    /// Indeks soal.
    pub index: usize,
    pub check: Check,
    /// Teks bantuan: `"Jawaban: 먹어요 (meogeoyo.)"` atau `"Tepat!"`.
    pub text: String,
}

/// Hasil keseluruhan tombol "Tampilkan jawaban".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reveal {
    pub lines: Vec<RevealLine>,
    /// `"4 dari 8 tepat."`
    pub status: String,
}

/// Samakan kalimat sebelum dibandingkan — **sama persis** dengan `normalize()`:
/// spasi ganda diringkas, spasi ideografis (U+3000) jadi spasi biasa, tanda
/// baca `. ! ? 。 ！ ？` di akhir dibuang, lalu huruf kecil dan trim.
pub fn normalize(s: &str) -> String {
    let collapsed: String = s.chars().map(|c| if c == '\u{3000}' { ' ' } else { c }).collect();
    let mut words: Vec<&str> = Vec::new();
    for part in collapsed.split_whitespace() {
        words.push(part);
    }
    let joined = words.join(" ");
    let trimmed = joined.trim_end_matches(['.', '!', '?', '\u{3002}', '\u{FF01}', '\u{FF1F}']);
    trimmed.trim().to_lowercase()
}

/// Apakah ketikan pengguna sama dengan jawaban soal ke-`index`.
pub fn is_correct(topic: &Topic, index: usize, typed: &str) -> bool {
    match topic.lat.get(index) {
        Some(item) => !typed.trim().is_empty() && normalize(typed) == normalize(item.ko),
        None => false,
    }
}

/// Status satu kotak berdasarkan ketikan pengguna.
pub fn check(topic: &Topic, index: usize, typed: &str) -> Check {
    if typed.is_empty() {
        Check::Empty
    } else if is_correct(topic, index, typed) {
        Check::Ok
    } else {
        Check::Bad
    }
}

/// Teks di bawah kotak saat mengetik (`check()` di `renderLat`): `"Tepat!"`
/// bila cocok, kosong bila belum cocok atau masih kosong.
pub fn inline_result(check: Check) -> String {
    match check {
        Check::Ok => "Tepat!".to_string(),
        _ => String::new(),
    }
}

/// Tombol "Tampilkan jawaban": jawab semua kotak sekaligus.
pub fn reveal(topic: &Topic, typed: &[String]) -> Reveal {
    let mut lines = Vec::with_capacity(topic.lat.len());
    let mut ok_count = 0usize;
    for (i, item) in topic.lat.iter().enumerate() {
        let input = typed.get(i).map(|s| s.as_str()).unwrap_or("");
        let c = check(topic, i, input);
        if c == Check::Ok {
            ok_count += 1;
            lines.push(RevealLine {
                index: i,
                check: Check::Ok,
                text: "Tepat!".to_string(),
            });
        } else {
            let text = if item.rom.is_empty() {
                format!("Jawaban: {}", item.ko)
            } else {
                format!("Jawaban: {} ({})", item.ko, item.rom)
            };
            lines.push(RevealLine { index: i, check: Check::Bad, text });
        }
    }
    Reveal {
        lines,
        status: format!("{} dari {} tepat.", ok_count, topic.lat.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_matches_web_rules() {
        assert_eq!(normalize("  안녕하세요!  "), "안녕하세요");
        assert_eq!(normalize("저는  학생이에요."), "저는 학생이에요");
        assert_eq!(normalize("가\u{3000}나"), "가 나");
        assert_eq!(normalize("ABC?"), "abc");
        assert_eq!(normalize("정말요？"), "정말요");
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("뭐?"), "뭐");
        // hanya tanda baca penutup; tanda di tengah dipertahankan
        assert_eq!(normalize("a, b."), "a, b");
    }

    #[test]
    fn empty_is_not_correct() {
        // Kotak kosong tidak pernah dianggap benar walau jawabannya kosong.
        assert!(!is_correct(&FAKE, 0, ""));
        assert!(!is_correct(&FAKE, 0, "   "));
        assert!(is_correct(&FAKE, 0, " 안녕하세요 "));
        assert!(is_correct(&FAKE, 0, "안녕하세요!"));
        assert_eq!(check(&FAKE, 0, ""), Check::Empty);
        assert_eq!(check(&FAKE, 0, "안녕"), Check::Bad);
        assert_eq!(inline_result(Check::Ok), "Tepat!");
        assert_eq!(inline_result(Check::Bad), "");
    }

    #[test]
    fn reveal_counts_and_labels() {
        let typed = vec!["안녕하세요".to_string(), "salah".to_string()];
        let r = reveal(&FAKE, &typed);
        assert_eq!(r.status, "1 dari 2 tepat.");
        assert_eq!(r.lines[0].check, Check::Ok);
        assert_eq!(r.lines[1].check, Check::Bad);
        assert_eq!(r.lines[1].text, "Jawaban: 먹어요. (meogeoyo.)");
    }

    #[test]
    fn reveal_shows_answer_when_empty() {
        let r = reveal(&FAKE, &[]);
        assert_eq!(r.status, "0 dari 2 tepat.");
        assert_eq!(r.lines[0].text, "Jawaban: 안녕하세요! (annyeonghaseyo.)");
        assert_eq!(r.lines[1].check, Check::Bad);
    }

    /// Topik tiruan minimal untuk uji.
    static LAT: [crate::model::Exercise; 2] = [
        crate::model::Exercise { ko: "안녕하세요!", rom: "annyeonghaseyo.", id: "Halo." },
        crate::model::Exercise { ko: "먹어요.", rom: "meogeoyo.", id: "Saya makan." },
    ];
    static FAKE: crate::model::Topic = crate::model::Topic {
        id: "uji",
        no: 99,
        title: "",
        title_id: "",
        level: 1,
        icon: "",
        intro: "",
        blocks: &[],
        quiz: &[],
        lat: &LAT,
    };
}
