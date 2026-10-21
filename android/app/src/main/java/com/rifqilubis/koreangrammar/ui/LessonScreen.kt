package com.rifqilubis.koreangrammar.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import uniffi.kg_ffi.BlockFfi
import uniffi.kg_ffi.ExampleFfi
import uniffi.kg_ffi.TopicFfi
import java.util.Locale

/**
 * Layar materi — `#screen-lesson` + `renderLesson()` / `renderBlock()`.
 *
 * Bentuk setiap blok (heading/text/formula/rules/examples/note) ditentukan
 * Rust lewat field `kind` pada `BlockFfi`.
 */
@Composable
fun LessonScreen(
    c: KgColors,
    topic: TopicFfi,
    ttsAvailable: Boolean,
    prevTitle: String?,
    nextTitle: String?,
    onBack: () -> Unit,
    onSpeak: (String) -> Unit,
    onStartQuiz: () -> Unit,
    onLatihan: () -> Unit,
    onPrev: () -> Unit,
    onNext: () -> Unit,
) {
    Column(Modifier.fillMaxWidth()) {
        Spacer(Modifier.height(4.dp))
        BackLink("‹ Kembali ke daftar", c, onBack)
        Spacer(Modifier.height(12.dp))

        // ------------------------------ header topik ------------------------------
        Row(verticalAlignment = Alignment.Top) {
            TopicIcon(topic.icon, c, size = 54.dp, fontSize = 25)
            Spacer(Modifier.width(15.dp))
            Column(Modifier.weight(1f)) {
                SectionLabel(topic.meta.uppercase(Locale.ROOT), c, color = c.accent)
                Spacer(Modifier.height(2.dp))
                Text(
                    topic.title,
                    color = c.text,
                    fontSize = KgType.lessonTitle,
                    lineHeight = 30.sp,
                    fontWeight = FontWeight.ExtraBold,
                    fontFamily = KgType.sans,
                )
                Text(
                    topic.titleId,
                    color = c.textDim,
                    fontSize = 14.sp,
                    fontFamily = KgType.sans,
                )
            }
        }

        Spacer(Modifier.height(18.dp))
        // `.lesson-intro`: garis aksen kiri + latar aksen lembut
        Row(
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(topEnd = 12.dp, bottomEnd = 12.dp))
                .background(c.accentSoft)
                .padding(14.dp),
        ) {
            Box(Modifier.width(3.dp).height(52.dp).background(c.accent, RoundedCornerShape(2.dp)))
            Spacer(Modifier.width(13.dp))
            Text(topic.intro, color = c.text, fontSize = 14.5.sp, lineHeight = 22.sp)
        }

        Spacer(Modifier.height(22.dp))
        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
            topic.blocks.forEach { block ->
                BlockView(block, c, ttsAvailable, onSpeak)
            }
        }

        Spacer(Modifier.height(26.dp))
        Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            KgButton(topic.quizLabel, c, primary = true, block = true, onClick = onStartQuiz)
            KgButton("Latihan menulis", c, block = true, onClick = onLatihan)
        }
        Spacer(Modifier.height(12.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            if (prevTitle != null) {
                Box(Modifier.weight(1f)) {
                    KgButton("‹ $prevTitle", c, block = true, fontSize = 13, onClick = onPrev)
                }
            }
            if (nextTitle != null) {
                Box(Modifier.weight(1f)) {
                    KgButton("$nextTitle ›", c, block = true, fontSize = 13, onClick = onNext)
                }
            }
        }
        Spacer(Modifier.height(8.dp))
    }
}

@Composable
private fun BlockView(
    block: BlockFfi,
    c: KgColors,
    ttsAvailable: Boolean,
    onSpeak: (String) -> Unit,
) {
    when (block.kind) {
        "heading" -> Column(
            Modifier
                .fillMaxWidth()
                .padding(top = 8.dp),
        ) {
            Text(
                block.text,
                color = c.text,
                fontSize = 16.sp,
                lineHeight = 22.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(bottom = 7.dp),
            )
            // border-bottom 1px
            Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
        }

        "text" -> SelectionContainer {
            Text(block.text, color = c.textDim, fontSize = 15.sp, lineHeight = 23.sp)
        }

        "formula" -> Row(
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(KgDims.radius))
                .background(c.surface)
                .border(1.dp, c.line, RoundedCornerShape(KgDims.radius))
                .padding(horizontal = 16.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            block.parts.forEachIndexed { i, part ->
                if (i > 0) Text("+", color = c.textFaint, fontWeight = FontWeight.Bold)
                Box(
                    Modifier
                        .clip(RoundedCornerShape(10.dp))
                        .background(c.surface2)
                        .border(1.dp, c.line, RoundedCornerShape(10.dp))
                        .padding(horizontal = 13.dp, vertical = 7.dp),
                ) {
                    Text(part, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, fontFamily = KgType.sans)
                }
            }
        }

        "rules" -> Column(
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(KgDims.radius))
                .background(c.line)
                .border(1.dp, c.line, RoundedCornerShape(KgDims.radius)),
            verticalArrangement = Arrangement.spacedBy(1.dp),
        ) {
            block.rules.forEach { row ->
                Row(
                    Modifier
                        .fillMaxWidth()
                        .background(c.surface)
                        .padding(horizontal = 14.dp, vertical = 11.dp),
                ) {
                    Text(
                        row.a,
                        color = c.text,
                        fontSize = 13.5.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.weight(1.1f),
                        fontFamily = KgType.sans,
                    )
                    Text(
                        row.b,
                        color = c.textDim,
                        fontSize = 13.5.sp,
                        modifier = Modifier.weight(1f),
                        fontFamily = KgType.sans,
                    )
                    Text(
                        row.c,
                        color = c.textDim,
                        fontSize = 13.5.sp,
                        modifier = Modifier.weight(1.25f),
                        fontFamily = KgType.sans,
                    )
                }
            }
        }

        "examples" -> Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            block.examples.forEach { ex -> ExampleCard(ex, c, ttsAvailable, onSpeak) }
        }

        "note" -> {
            // `.block-note::before { content: "Catatan: " }`
            val text = buildAnnotatedNote("Catatan: ", block.text, c)
            Box(
                Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(KgDims.radius))
                    .background(c.surface2)
                    .border(1.dp, c.line, RoundedCornerShape(KgDims.radius))
                    .padding(horizontal = 15.dp, vertical = 13.dp),
            ) { Text(text, fontSize = 14.sp, lineHeight = 21.sp) }
        }
    }
}

private fun buildAnnotatedNote(prefix: String, text: String, c: KgColors) =
    androidx.compose.ui.text.buildAnnotatedString {
        // withStyle adalah ekstensi pada builder, jadi dipanggil langsung.
        withStyle(androidx.compose.ui.text.SpanStyle(fontWeight = FontWeight.Bold, color = c.warn)) {
            append(prefix)
        }
        withStyle(androidx.compose.ui.text.SpanStyle(color = c.textDim)) { append(text) }
    }

@Composable
private fun ExampleCard(
    ex: ExampleFfi,
    c: KgColors,
    ttsAvailable: Boolean,
    onSpeak: (String) -> Unit,
) {
    Column(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(KgDims.radius))
            .background(c.surface)
            .border(1.dp, c.line, RoundedCornerShape(KgDims.radius))
            .padding(horizontal = 15.dp, vertical = 13.dp),
    ) {
        Row(verticalAlignment = Alignment.Top) {
            Box(Modifier.weight(1f)) {
                Text(
                    ex.ko,
                    color = c.text,
                    fontSize = KgType.exampleKo,
                    lineHeight = 26.sp,
                    fontWeight = FontWeight.SemiBold,
                    fontFamily = KgType.sans,
                )
            }
            if (ttsAvailable) {
                Spacer(Modifier.width(12.dp))
                // Teks yang dibacakan TIDAK dikirim dari sini: aplikasi meminta
                // Rust merapikannya lewat `tts_text(ex.ko)`.
                SpeakButton(c) { onSpeak(ex.ko) }
            }
        }
        if (ex.rom.isNotEmpty()) {
            Spacer(Modifier.height(2.dp))
            Text(
                ex.rom,
                color = c.textFaint,
                fontSize = 13.sp,
                fontStyle = FontStyle.Italic,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        }
        if (ex.id.isNotEmpty()) {
            Spacer(Modifier.height(5.dp))
            Text(ex.id, color = c.textDim, fontSize = 14.5.sp)
        }
        if (ex.why.isNotEmpty()) {
            Spacer(Modifier.height(7.dp))
            Text(ex.why, color = c.accent, fontSize = 13.sp, lineHeight = 19.sp)
        }
    }
}
