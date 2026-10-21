package com.rifqilubis.koreangrammar.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/* =====================================================================
   Komponen kecil yang dipakai di semua layar — padanan kelas CSS:
   .card .btn .icon-btn .badge .topic-icon .mini-stat .score-pill …
   ===================================================================== */

/** `.card` — latar surface, garis tipis, sudut 22dp, bayangan halus. */
@Composable
fun KgCard(
    c: KgColors,
    modifier: Modifier = Modifier,
    padding: Dp = 0.dp,
    content: @Composable () -> Unit,
) {
    Box(
        modifier
            .shadow(
                elevation = if (c.isDark) 6.dp else 4.dp,
                shape = RoundedCornerShape(KgDims.radiusLg),
                ambientColor = Color.Black.copy(alpha = if (c.isDark) 0.45f else 0.10f),
                spotColor = Color.Black.copy(alpha = if (c.isDark) 0.45f else 0.10f),
            )
            .clip(RoundedCornerShape(KgDims.radiusLg))
            .background(c.surface)
            .border(1.dp, c.line, RoundedCornerShape(KgDims.radiusLg))
            .padding(padding),
    ) { content() }
}

/** `.btn` + `.btn-primary` / `.btn-ghost` / `.btn-block`. */
@Composable
fun KgButton(
    text: String,
    c: KgColors,
    modifier: Modifier = Modifier,
    primary: Boolean = false,
    block: Boolean = false,
    enabled: Boolean = true,
    fontSize: Int = 15,
    onClick: () -> Unit,
) {
    val source = remember { MutableInteractionSource() }
    val pressed by source.collectIsPressedAsState()
    val m = modifier
        .then(if (block) Modifier.fillMaxWidth() else Modifier)
        .defaultMinSize(minHeight = KgDims.btnMinHeight)
        .clip(RoundedCornerShape(14.dp))
        .background(
            when {
                primary -> if (pressed) c.accent.copy(alpha = 0.9f) else c.accent
                else -> if (pressed) c.surface2 else c.surface.copy(alpha = 0f)
            }
        )
        .then(
            if (primary) Modifier
            else Modifier.border(1.dp, if (pressed) c.textFaint else c.line, RoundedCornerShape(14.dp))
        )
        .clickable(
            interactionSource = source,
            indication = null,
            enabled = enabled,
            role = Role.Button,
            onClick = onClick,
        )
        .padding(horizontal = 20.dp, vertical = 12.dp)

    Box(m, contentAlignment = Alignment.Center) {
        Text(
            text = text,
            color = if (primary) Color.White else if (pressed) c.text else c.textDim,
            fontSize = fontSize.sp,
            fontFamily = KgType.sans,
            fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.Center,
            maxLines = 2,
        )
    }
}

/** `.icon-btn` — kotak 40×40 berbingkai. */
@Composable
fun IconButtonBox(
    c: KgColors,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
    content: @Composable () -> Unit,
) {
    Box(
        modifier
            .size(40.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(c.surface)
            .border(1.dp, c.line, RoundedCornerShape(12.dp))
            .clickable(role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) { content() }
}

/** `.topic-icon` — kotak 46dp berisi huruf Korea. */
@Composable
fun TopicIcon(
    glyph: String,
    c: KgColors,
    size: Dp = 46.dp,
    done: Boolean = false,
    fontSize: Int = 21,
) {
    Box(
        Modifier
            .size(size)
            .clip(RoundedCornerShape(14.dp))
            .background(if (done) c.okSoft else c.accentSoft),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            glyph,
            color = if (done) c.ok else c.accent,
            fontFamily = KgType.sans,
            fontSize = fontSize.sp,
            fontWeight = FontWeight.Bold,
        )
    }
}

/** `.badge` dan variannya. */
@Composable
fun Badge(text: String, c: KgColors, tone: BadgeTone) {
    val (fg, bg) = when (tone) {
        BadgeTone.Neutral -> c.textFaint to Color.Transparent
        BadgeTone.Score -> c.accent to c.accentSoft
        BadgeTone.Done -> c.ok to c.okSoft
    }
    Box(
        Modifier
            .clip(CircleShape)
            .background(bg)
            .border(1.dp, if (tone == BadgeTone.Neutral) c.line else fg, CircleShape)
            .padding(horizontal = 9.dp, vertical = 3.dp),
    ) {
        Text(text, color = fg, fontSize = 11.sp, fontWeight = FontWeight.Bold, maxLines = 1)
    }
}

enum class BadgeTone { Neutral, Score, Done }

/** `.score-pill` — "Benar 3". */
@Composable
fun ScorePill(text: String, c: KgColors) {
    Box(
        Modifier
            .clip(CircleShape)
            .background(c.okSoft)
            .padding(horizontal = 11.dp, vertical = 5.dp),
    ) {
        Text(text, color = c.ok, fontSize = 12.sp, fontWeight = FontWeight.Bold, maxLines = 1)
    }
}

/** `.mini-stat` — angka besar + label kecil. */
@Composable
fun MiniStat(value: String, label: String, c: KgColors, modifier: Modifier = Modifier) {
    Column(
        modifier
            .clip(RoundedCornerShape(KgDims.radiusSm))
            .background(c.surface2)
            .border(1.dp, c.line, RoundedCornerShape(KgDims.radiusSm))
            .padding(horizontal = 14.dp, vertical = 12.dp),
    ) {
        Text(
            value,
            color = c.text,
            fontSize = 19.sp,
            fontWeight = FontWeight.ExtraBold,
            fontFamily = KgType.sans,
            maxLines = 1,
        )
        Spacer(Modifier.height(2.dp))
        Text(label, color = c.textFaint, fontSize = 11.5.sp, maxLines = 1)
    }
}

/** `.progress` — bilah 6dp dengan isian gradien aksen. */
@Composable
fun ProgressBar(fraction: Float, c: KgColors, modifier: Modifier = Modifier) {
    Box(
        modifier
            .fillMaxWidth()
            .height(6.dp)
            .clip(CircleShape)
            .background(c.surface2),
    ) {
        Box(
            Modifier
                .fillMaxWidth(fraction.coerceIn(0f, 1f))
                .height(6.dp)
                .clip(CircleShape)
                .background(
                    androidx.compose.ui.graphics.Brush.horizontalGradient(
                        listOf(c.accent, c.accent2)
                    )
                ),
        )
    }
}

/** Teks "‹ Kembali" yang bisa ditekan (`.back-link`). */
@Composable
fun BackLink(text: String, c: KgColors, onClick: () -> Unit) {
    Text(
        text,
        color = c.textDim,
        fontSize = 14.sp,
        fontWeight = FontWeight.SemiBold,
        modifier = Modifier
            .clip(RoundedCornerShape(6.dp))
            .clickable(role = Role.Button, onClick = onClick)
            .padding(vertical = 4.dp, horizontal = 2.dp),
    )
}

/** Label kecil huruf besar (`.section-title`, `.topic-meta`, `.quiz-count`). */
@Composable
fun SectionLabel(text: String, c: KgColors, modifier: Modifier = Modifier, color: Color = c.textFaint) {
    Text(
        text,
        modifier = modifier,
        color = color,
        fontSize = 12.sp,
        fontWeight = FontWeight.Bold,
        letterSpacing = 1.6.sp,
        fontFamily = KgType.sans,
    )
}

/** Ikon 🔊 (`speak-btn`): segitiga + dua lengkung gelombang. */
@Composable
fun SpeakButton(c: KgColors, onClick: () -> Unit) {
    Box(
        Modifier
            .size(32.dp)
            .clip(RoundedCornerShape(9.dp))
            .background(c.surface2)
            .border(1.dp, c.line, RoundedCornerShape(9.dp))
            .clickable(role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Canvas(Modifier.size(16.dp)) {
            val w = size.width
            val h = size.height
            val color = c.textDim
            // badan speaker
            val body = androidx.compose.ui.graphics.Path().apply {
                moveTo(w * 0.08f, h * 0.38f)
                lineTo(w * 0.30f, h * 0.38f)
                lineTo(w * 0.58f, h * 0.16f)
                lineTo(w * 0.58f, h * 0.84f)
                lineTo(w * 0.30f, h * 0.62f)
                lineTo(w * 0.08f, h * 0.62f)
                close()
            }
            drawPath(body, color)
            // gelombang
            drawArc(
                color = color,
                startAngle = -55f,
                sweepAngle = 110f,
                useCenter = false,
                topLeft = Offset(w * 0.55f, h * 0.30f),
                size = androidx.compose.ui.geometry.Size(w * 0.30f, h * 0.40f),
                style = Stroke(width = w * 0.09f, cap = StrokeCap.Round),
            )
            drawArc(
                color = color,
                startAngle = -52f,
                sweepAngle = 104f,
                useCenter = false,
                topLeft = Offset(w * 0.62f, h * 0.16f),
                size = androidx.compose.ui.geometry.Size(w * 0.36f, h * 0.68f),
                style = Stroke(width = w * 0.09f, cap = StrokeCap.Round),
            )
        }
    }
}

/** Ikon ✕ untuk menutup kuis. */
@Composable
fun CloseIcon(c: KgColors) {
    Canvas(Modifier.size(18.dp)) {
        val s = Stroke(width = size.width * 0.1f, cap = StrokeCap.Round)
        drawLine(c.textDim, Offset(0f, 0f), Offset(size.width, size.height), s.width, StrokeCap.Round)
        drawLine(c.textDim, Offset(size.width, 0f), Offset(0f, size.height), s.width, StrokeCap.Round)
    }
}

/** Ikon matahari (tema gelap aktif) dan bulan (tema terang aktif). */
@Composable
fun ThemeIcon(c: KgColors, dark: Boolean) {
    Canvas(Modifier.size(19.dp)) {
        val center = Offset(size.width / 2f, size.height / 2f)
        val r = size.minDimension
        if (!dark) {
            // bulan sabit
            val path = androidx.compose.ui.graphics.Path().apply {
                moveTo(r * 0.84f, r * 0.61f)
                cubicTo(r * 0.80f, r * 0.92f, r * 0.36f, r * 0.98f, r * 0.16f, r * 0.66f)
                cubicTo(r * 0.02f, r * 0.40f, r * 0.20f, r * 0.08f, r * 0.46f, r * 0.04f)
                cubicTo(r * 0.30f, r * 0.26f, r * 0.40f, r * 0.52f, r * 0.84f, r * 0.61f)
                close()
            }
            drawPath(path, c.textDim)
        } else {
            drawCircle(c.textDim, r * 0.20f, center)
            val stroke = Stroke(width = r * 0.09f, cap = StrokeCap.Round)
            for (i in 0 until 8) {
                val a = Math.toRadians(i * 45.0)
                val dx = Math.cos(a).toFloat()
                val dy = Math.sin(a).toFloat()
                drawLine(
                    c.textDim,
                    Offset(center.x + dx * r * 0.33f, center.y + dy * r * 0.33f),
                    Offset(center.x + dx * r * 0.47f, center.y + dy * r * 0.47f),
                    stroke.width,
                    StrokeCap.Round,
                )
            }
        }
    }
}

/** Baris tipis pemisah untuk tabel aturan. */
@Composable
fun RowScope.KgDivider(c: KgColors) {
    Spacer(Modifier.height(1.dp).fillMaxWidth().background(c.line))
}

/** `.hero-actions` / `.actions`: deretan tombol yang bisa membungkus. */
@Composable
fun ButtonRow(
    modifier: Modifier = Modifier,
    content: @Composable RowScope.() -> Unit,
) {
    Row(
        modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) { content() }
}

/** Spasi vertikal bernama supaya layar mudah dibaca. */
val Gap4 = 4.dp
val Gap8 = 8.dp
val Gap10 = 10.dp
val Gap12 = 12.dp
val Gap14 = 14.dp
val Gap16 = 16.dp
val Gap18 = 18.dp
val Gap22 = 22.dp
val Gap26 = 26.dp

/** Teks isi standar. */
@Composable
fun BodyText(text: String, c: KgColors, style: TextStyle = TextStyle(fontSize = 15.sp, lineHeight = 23.sp)) {
    Text(text, color = c.textDim, style = style, fontFamily = KgType.sans)
}
