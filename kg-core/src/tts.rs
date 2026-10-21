//! Teks untuk mesin suara Android (`TextToSpeech`) — padanan `speakText()`
//! di `js/app.js`.

/// Ambil bagian yang perlu dibacakan dari teks contoh Korea.
///
/// Versi web membuang keterangan dalam tanda kurung di akhir dan — bila ada
/// tanda `→` — hanya membacakan bentuk di kanan panah, karena bentuk itu yang
/// benar-benar diucapkan. Contoh:
///
/// - `"먹다 → 먹어요"` → `"먹어요"`
/// - `"집 (rumah)"` → `"집"`
/// - `"값"` → `"값"`
pub fn speak_text(ko: &str) -> String {
    let mut text = ko.trim();
    // buang satu keterangan dalam kurung di akhir
    if text.ends_with(')') {
        if let Some(open) = text.rfind('(') {
            text = text[..open].trim_end();
        }
    }
    let out = match text.rsplit_once('\u{2192}') {
        Some((_, after)) => after.trim(),
        None => text.trim(),
    };
    if out.is_empty() {
        ko.trim().to_string()
    } else {
        out.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_uses_right_hand_side() {
        assert_eq!(speak_text("먹다 → 먹어요"), "먹어요");
        assert_eq!(speak_text("+ (-) → 가"), "가");
    }

    #[test]
    fn trailing_parenthesis_dropped() {
        assert_eq!(speak_text("집 (rumah)"), "집");
        assert_eq!(speak_text("학교  (sekolah)  "), "학교");
    }

    #[test]
    fn plain_text_untouched() {
        assert_eq!(speak_text("안녕하세요"), "안녕하세요");
        assert_eq!(speak_text("값"), "값");
        assert_eq!(speak_text("  저는 학생이에요  "), "저는 학생이에요");
    }

    #[test]
    fn nothing_to_speak_falls_back() {
        assert_eq!(speak_text("→"), "→");
        assert_eq!(speak_text("(rumah)"), "(rumah)");
    }
}
