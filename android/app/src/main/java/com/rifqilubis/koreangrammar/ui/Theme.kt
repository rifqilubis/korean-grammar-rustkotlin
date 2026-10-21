package com.rifqilubis.koreangrammar.ui

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/* =====================================================================
   Palet & ukuran — terjemahan langsung dari css/style.css.

   :root          -> aksen, radius, lebar maksimum
   [data-theme=…] -> Colors.dark / Colors.light
   ===================================================================== */

/** Satu set warna untuk satu tema (`html[data-theme="dark"]` / `"light"`). */
@Immutable
data class KgColors(
    val accent: Color,
    val accentSoft: Color,
    val accent2: Color,
    val ok: Color,
    val okSoft: Color,
    val bad: Color,
    val badSoft: Color,
    val warn: Color,
    val bg: Color,
    val bgSoft: Color,
    val surface: Color,
    val surface2: Color,
    val line: Color,
    val text: Color,
    val textDim: Color,
    val textFaint: Color,
    val isDark: Boolean,
) {
    companion object {
        val dark = KgColors(
            accent = Color(0xFFE14B6A),
            accentSoft = Color(0x24E14B6A), // rgba(225,75,106,.14)
            accent2 = Color(0xFF3D8BFD),
            ok = Color(0xFF17A673),
            okSoft = Color(0x2217A673),
            bad = Color(0xFFE5484D),
            badSoft = Color(0x1FE5484D),
            warn = Color(0xFFD98A00),
            bg = Color(0xFF0C0D16),
            bgSoft = Color(0xFF12141F),
            surface = Color(0xFF171A28),
            surface2 = Color(0xFF1E2233),
            line = Color(0xFF272C40),
            text = Color(0xFFECEEF8),
            textDim = Color(0xFFA2A8C0),
            textFaint = Color(0xFF6C7390),
            isDark = true,
        )

        val light = KgColors(
            accent = Color(0xFFE14B6A),
            accentSoft = Color(0x24E14B6A),
            accent2 = Color(0xFF3D8BFD),
            ok = Color(0xFF0D8F62), // tema terang menimpa --ok
            okSoft = Color(0x2217A673),
            bad = Color(0xFFD0353A), // tema terang menimpa --bad
            badSoft = Color(0x1FE5484D),
            warn = Color(0xFFD98A00),
            bg = Color(0xFFF3F4F9),
            bgSoft = Color(0xFFEAECF4),
            surface = Color(0xFFFFFFFF),
            surface2 = Color(0xFFF5F6FB),
            line = Color(0xFFE0E3EE),
            text = Color(0xFF191C2B),
            textDim = Color(0xFF5C6480),
            textFaint = Color(0xFF8B92A9),
            isDark = false,
        )

        fun of(theme: String): KgColors = if (theme == "light") light else dark
    }
}

/** --radius-lg / --radius / --radius-sm dan --maxw. */
object KgDims {
    val radiusLg = 22.dp
    val radius = 16.dp
    val radiusSm = 12.dp
    val maxWidth = 720.dp
    val quizMaxWidth = 620.dp
    /** Jarak tepi `.app` (18px desktop, 14px di layar ≤480px). */
    val gutter = 18.dp
    val gutterSmall = 14.dp
    /** Ruang aman bawah `.app`: 40px + safe-area. */
    val bottomPadding = 40.dp
    val btnMinHeight = 48.dp
}

/** Ukuran teks mengikuti `clamp()` di CSS untuk layar ponsel (± 390dp). */
object KgType {
    /** Font sistem memakai Malgun Gothic / Noto Sans KR bila tersedia. */
    val sans = FontFamily.SansSerif

    val heroTitle = 25.sp // clamp(1.55rem, 5.4vw, 2.05rem)
    val lessonTitle = 23.sp // clamp(1.4rem, 5vw, 1.8rem)
    val quizQuestion = 18.sp // clamp(1.12rem, 3.8vw, 1.34rem)
    val body = 16.sp
    val bodySmall = 15.sp
    val exampleKo = 18.sp
    val optionKo = 16.sp
}

/** Bobot huruf yang dipakai CSS (650/700/750/800). */
object KgWeight {
    val semiBold = FontWeight.SemiBold
    val bold = FontWeight.Bold
    val extraBold = FontWeight.ExtraBold
}
