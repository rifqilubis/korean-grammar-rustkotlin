//! Palet warna & ukuran — terjemahan langsung dari `css/style.css` versi web,
//! sama dengan `ui/Theme.kt` di aplikasi Android.
//!
//! Menjaga nilai di sini identik dengan Android membuat kedua aplikasi terlihat
//! seperti satu produk.

use egui::Color32;

/// Satu set warna untuk satu tema (`html[data-theme="dark"]` / `"light"`).
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub accent: Color32,
    pub accent_soft: Color32,
    pub accent2: Color32,
    pub ok: Color32,
    pub ok_soft: Color32,
    pub bad: Color32,
    pub bad_soft: Color32,
    pub warn: Color32,
    pub bg: Color32,
    pub bg_soft: Color32,
    pub surface: Color32,
    pub surface2: Color32,
    pub line: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_faint: Color32,
    pub is_dark: bool,
}

impl Palette {
    pub const DARK: Palette = Palette {
        accent: Color32::from_rgb(0xE1, 0x4B, 0x6A),
        accent_soft: Color32::from_rgba_premultiplied(
            // rgba(225,75,106,.14) di atas latar gelap
            0x2A, 0x16, 0x1D, 0x2A,
        ),
        accent2: Color32::from_rgb(0x3D, 0x8B, 0xFD),
        ok: Color32::from_rgb(0x17, 0xA6, 0x73),
        ok_soft: Color32::from_rgb(0x14, 0x2A, 0x27),
        bad: Color32::from_rgb(0xE5, 0x48, 0x4D),
        bad_soft: Color32::from_rgb(0x2E, 0x19, 0x1F),
        warn: Color32::from_rgb(0xD9, 0x8A, 0x00),
        bg: Color32::from_rgb(0x0C, 0x0D, 0x16),
        bg_soft: Color32::from_rgb(0x12, 0x14, 0x1F),
        surface: Color32::from_rgb(0x17, 0x1A, 0x28),
        surface2: Color32::from_rgb(0x1E, 0x22, 0x33),
        line: Color32::from_rgb(0x27, 0x2C, 0x40),
        text: Color32::from_rgb(0xEC, 0xEE, 0xF8),
        text_dim: Color32::from_rgb(0xA2, 0xA8, 0xC0),
        text_faint: Color32::from_rgb(0x6C, 0x73, 0x90),
        is_dark: true,
    };

    pub const LIGHT: Palette = Palette {
        accent: Color32::from_rgb(0xE1, 0x4B, 0x6A),
        accent_soft: Color32::from_rgb(0xFB, 0xE6, 0xEB),
        accent2: Color32::from_rgb(0x3D, 0x8B, 0xFD),
        ok: Color32::from_rgb(0x0D, 0x8F, 0x62),
        ok_soft: Color32::from_rgb(0xE4, 0xF5, 0xEE),
        bad: Color32::from_rgb(0xD0, 0x35, 0x3A),
        bad_soft: Color32::from_rgb(0xFC, 0xE8, 0xE9),
        warn: Color32::from_rgb(0xD9, 0x8A, 0x00),
        bg: Color32::from_rgb(0xF3, 0xF4, 0xF9),
        bg_soft: Color32::from_rgb(0xEA, 0xEC, 0xF4),
        surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        surface2: Color32::from_rgb(0xF5, 0xF6, 0xFB),
        line: Color32::from_rgb(0xE0, 0xE3, 0xEE),
        text: Color32::from_rgb(0x19, 0x1C, 0x2B),
        text_dim: Color32::from_rgb(0x5C, 0x64, 0x80),
        text_faint: Color32::from_rgb(0x8B, 0x92, 0xA9),
        is_dark: false,
    };

    /// Pilih palet dari nama tema (`kg_ffi::sanitize_theme` sudah membatasinya).
    pub fn of(theme: &str) -> Palette {
        if theme == "light" {
            Palette::LIGHT
        } else {
            Palette::DARK
        }
    }
}

/// Ukuran tata letak (`--radius-*`, `--maxw`, jarak tepi `.app`).
pub mod dims {
    pub const RADIUS_LG: f32 = 22.0;
    pub const RADIUS: f32 = 16.0;
    pub const RADIUS_SM: f32 = 12.0;
    /// Lebar kolom isi (`.app { max-width: 720px }`).
    pub const MAX_WIDTH: f32 = 720.0;
    /// Lebar layar kuis di ponsel.
    pub const QUIZ_MAX_WIDTH: f32 = 620.0;
    pub const GUTTER: f32 = 18.0;
    pub const BOTTOM_PADDING: f32 = 40.0;
    pub const BTN_MIN_HEIGHT: f32 = 44.0;
}

/// Ukuran teks yang dipakai layar (clamp() di CSS untuk lebar ponsel).
pub mod text_size {
    pub const HERO_TITLE: f32 = 27.0;
    pub const LESSON_TITLE: f32 = 24.0;
    pub const QUIZ_QUESTION: f32 = 19.0;
    pub const BODY: f32 = 15.0;
    pub const EXAMPLE_KO: f32 = 18.0;
    pub const SMALL: f32 = 13.0;
    pub const TINY: f32 = 12.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_names_select_matching_palette() {
        assert!(Palette::of("dark").is_dark);
        assert!(!Palette::of("light").is_dark);
        // nilai asing jatuh ke gelap, sama seperti sanitize_theme() Rust
        assert!(Palette::of("").is_dark);
    }

    #[test]
    fn palette_matches_android_values() {
        // Sampel dari Theme.kt: accent #E14B6A, bg gelap #0C0D16, bg terang #F3F4F9
        assert_eq!(Palette::DARK.accent, Color32::from_rgb(0xE1, 0x4B, 0x6A));
        assert_eq!(Palette::DARK.bg, Color32::from_rgb(0x0C, 0x0D, 0x16));
        assert_eq!(Palette::LIGHT.bg, Color32::from_rgb(0xF3, 0xF4, 0xF9));
        assert_eq!(Palette::DARK.text, Color32::from_rgb(0xEC, 0xEE, 0xF8));
        assert_eq!(Palette::LIGHT.text, Color32::from_rgb(0x19, 0x1C, 0x2B));
    }
}
