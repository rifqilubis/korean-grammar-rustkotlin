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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import uniffi.kg_ffi.TopicFfi

/**
 * Status satu kotak latihan, hasil `KG.latCheck(topicId, index, typed)`.
 * String "ok"/"bad"/"" diterjemahkan kembali menjadi enum di sini — nilai
 * aslinya tetap berasal dari Rust.
 */
enum class LatCheck { Empty, Ok, Bad }

fun latCheckOf(raw: String): LatCheck = when (raw) {
    "ok" -> LatCheck.Ok
    "bad" -> LatCheck.Bad
    else -> LatCheck.Empty
}

/** Isi kotak: ketikan, status Rust, dan teks kecil di bawahnya. */
data class LatEntry(
    val typed: String = "",
    val check: LatCheck = LatCheck.Empty,
    val inline: String = "",
)

/**
 * Layar latihan menulis — `#screen-lat` + `renderLat()`.
 *
 * Setiap perubahan ketikan dikirim ke Rust (`latCheck`) untuk dibandingkan
 * dengan `normalize()` yang sama persis dengan versi web.
 */
@Composable
fun LatihanScreen(
    c: KgColors,
    topic: TopicFfi,
    entries: List<LatEntry>,
    status: String,
    onBack: () -> Unit,
    onType: (Int, String) -> Unit,
    onReveal: () -> Unit,
    onReset: () -> Unit,
) {
    Column(Modifier.fillMaxWidth()) {
        Spacer(Modifier.height(4.dp))
        BackLink("‹ Kembali ke ${topic.titleId}", c, onBack)
        Spacer(Modifier.height(12.dp))

        KgCard(c, modifier = Modifier.fillMaxWidth(), padding = 22.dp) {
            Column {
                SectionLabel("LATIHAN MENULIS", c, color = c.accent)
                Spacer(Modifier.height(4.dp))
                Text(
                    "Tulis dalam bahasa Korea",
                    color = c.text,
                    fontSize = KgType.lessonTitle,
                    lineHeight = 30.sp,
                    fontWeight = FontWeight.ExtraBold,
                    fontFamily = KgType.sans,
                )
                Spacer(Modifier.height(6.dp))
                Text(
                    "Ketikan atau tempel kalimatnya. Cocokkan tanpa melihat romanisasi.",
                    color = c.textDim,
                    fontSize = 14.sp,
                    lineHeight = 21.sp,
                )

                Spacer(Modifier.height(20.dp))
                Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
                    topic.lat.forEachIndexed { i, item ->
                        val entry = entries.getOrNull(i) ?: LatEntry()
                        LatItem(c, item.id, entry) { onType(i, it) }
                    }
                }

                Spacer(Modifier.height(16.dp))
                Text(
                    status,
                    color = c.textDim,
                    fontSize = 13.5.sp,
                    minLines = 1,
                    modifier = Modifier.fillMaxWidth(),
                )
                Spacer(Modifier.height(10.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    Box(Modifier.weight(1f)) {
                        KgButton("Tampilkan jawaban", c, block = true, fontSize = 13, onClick = onReveal)
                    }
                    Box(Modifier.weight(1f)) {
                        KgButton("Kosongkan", c, block = true, fontSize = 13, onClick = onReset)
                    }
                }
            }
        }
        Spacer(Modifier.height(12.dp))
    }
}

@Composable
private fun LatItem(
    c: KgColors,
    meaning: String,
    entry: LatEntry,
    onType: (String) -> Unit,
) {
    val borderColor = when (entry.check) {
        LatCheck.Ok -> c.ok
        LatCheck.Bad -> c.bad
        LatCheck.Empty -> c.line
    }
    Column(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(KgDims.radius))
            .background(c.surface2)
            .border(1.dp, borderColor, RoundedCornerShape(KgDims.radius))
            .padding(horizontal = 15.dp, vertical = 14.dp),
    ) {
        Text(
            "Artinya: $meaning",
            color = c.textDim,
            fontSize = 13.5.sp,
        )
        Spacer(Modifier.height(7.dp))
        OutlinedTextField(
            value = entry.typed,
            onValueChange = onType,
            modifier = Modifier.fillMaxWidth(),
            textStyle = TextStyle(
                color = c.text,
                fontSize = 16.5.sp,
                fontFamily = KgType.sans,
            ),
            placeholder = {
                Text("Tulis kalimat Korea…", color = c.textFaint, fontSize = 16.sp)
            },
            singleLine = true,
            shape = RoundedCornerShape(11.dp),
            keyboardOptions = KeyboardOptions(
                capitalization = KeyboardCapitalization.None,
                autoCorrect = false,
                imeAction = ImeAction.Next,
            ),
            colors = TextFieldDefaults.colors(
                focusedContainerColor = c.surface,
                unfocusedContainerColor = c.surface,
                focusedIndicatorColor = c.accent,
                unfocusedIndicatorColor = c.line,
                cursorColor = c.accent,
                focusedTextColor = c.text,
                unfocusedTextColor = c.text,
            ),
        )
        if (entry.inline.isNotEmpty()) {
            Spacer(Modifier.height(6.dp))
            Text(
                entry.inline,
                color = if (entry.check == LatCheck.Ok) c.ok else c.bad,
                fontSize = 13.sp,
                fontWeight = FontWeight.SemiBold,
            )
        }
    }
}
