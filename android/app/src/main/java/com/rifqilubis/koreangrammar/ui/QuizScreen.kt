package com.rifqilubis.koreangrammar.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import uniffi.kg_ffi.QuizStateFfi
import java.util.Locale

/**
 * Layar kuis — `#screen-quiz` + `paintQuestion()` / `pick()`.
 *
 * Status soal (`answered`, `picked`, `answer`, `feedback`, kunci pilihan)
 * seluruhnya milik Rust; layar ini hanya menggambar `QuizStateFfi`.
 */
@Composable
fun QuizScreen(
    c: KgColors,
    state: QuizStateFfi,
    onPick: (Int) -> Unit,
    onNext: () -> Unit,
    onClose: () -> Unit,
) {
    Column(Modifier.fillMaxWidth()) {
        // ------------------------------- quiz-top -------------------------------
        Row(
            Modifier.fillMaxWidth().padding(vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButtonBox(c, onClick = onClose) { CloseIcon(c) }
            Spacer(Modifier.width(12.dp))
            Text(
                state.topicLabel,
                color = c.textDim,
                fontSize = 14.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )
            Spacer(Modifier.width(8.dp))
            ScorePill(state.scoreLabel, c)
        }

        ProgressBar(state.progress.toFloat() / 100f, c)
        Spacer(Modifier.height(16.dp))

        KgCard(c, modifier = Modifier.fillMaxWidth(), padding = 18.dp) {
            Column {
                SectionLabel(state.counter.uppercase(Locale.ROOT), c)
                Spacer(Modifier.height(10.dp))
                Text(
                    state.question,
                    color = c.text,
                    fontSize = KgType.quizQuestion,
                    lineHeight = 27.sp,
                    fontWeight = FontWeight.Bold,
                    fontFamily = KgType.sans,
                )
                Spacer(Modifier.height(18.dp))

                Column(verticalArrangement = Arrangement.spacedBy(9.dp)) {
                    state.options.forEachIndexed { i, text ->
                        OptionRow(
                            c = c,
                            letter = state.letters.getOrElse(i) { "" },
                            text = text,
                            state = state,
                            index = i,
                            onClick = { onPick(i) },
                        )
                    }
                }

                if (state.answered && state.explanation.isNotEmpty()) {
                    Spacer(Modifier.height(14.dp))
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(KgDims.radiusSm))
                            .background(c.surface2)
                            .padding(horizontal = 15.dp, vertical = 13.dp),
                    ) {
                        Box(Modifier.width(3.dp).height(34.dp).background(c.accent, RoundedCornerShape(2.dp)))
                        Spacer(Modifier.width(12.dp))
                        Text(state.explanation, color = c.textDim, fontSize = 14.sp, lineHeight = 21.sp)
                    }
                }

                Spacer(Modifier.height(16.dp))
                Row(
                    Modifier.fillMaxWidth().height(48.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        state.feedback,
                        color = when (state.feedbackKind) {
                            "ok" -> c.ok
                            "bad" -> c.bad
                            else -> c.textDim
                        },
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Bold,
                        lineHeight = 20.sp,
                        modifier = Modifier.weight(1f),
                    )
                    if (state.answered) {
                        Spacer(Modifier.width(12.dp))
                        KgButton(state.nextLabel, c, primary = true, onClick = onNext)
                    }
                }
            }
        }

        Spacer(Modifier.height(14.dp))
        Text(
            "Ketuk pilihan A–D untuk menjawab",
            color = c.textFaint,
            fontSize = 12.sp,
            modifier = Modifier.fillMaxWidth(),
            textAlign = androidx.compose.ui.text.style.TextAlign.Center,
        )
        Spacer(Modifier.height(8.dp))
    }
}

/** `.option` dengan status `.is-correct` / `.is-wrong` / `.is-dim`. */
@Composable
private fun OptionRow(
    c: KgColors,
    letter: String,
    text: String,
    state: QuizStateFfi,
    index: Int,
    onClick: () -> Unit,
) {
    val isRight = state.answered && index == state.answer.toInt()
    val isPickedWrong = state.answered && index == state.picked && !isRight
    val dimmed = state.answered && !isRight && !isPickedWrong

    val border = when {
        isRight -> c.ok
        isPickedWrong -> c.bad
        else -> c.line
    }
    val bg = when {
        isRight -> c.okSoft
        isPickedWrong -> c.badSoft
        else -> c.surface2
    }
    val letterBg = when {
        isRight -> c.ok
        isPickedWrong -> c.bad
        else -> c.surface
    }
    val letterFg = when {
        isRight || isPickedWrong -> Color.White
        else -> c.textFaint
    }

    Row(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(KgDims.radiusSm))
            .background(bg.copy(alpha = if (dimmed) 0.5f else 1f))
            .border(1.5.dp, border.copy(alpha = if (dimmed) 0.5f else 1f), RoundedCornerShape(KgDims.radiusSm))
            .clickable(enabled = !state.answered, role = Role.Button, onClick = onClick)
            .padding(horizontal = 15.dp, vertical = 13.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            Modifier
                .size(26.dp)
                .clip(RoundedCornerShape(8.dp))
                .background(letterBg)
                .border(1.dp, if (isRight) c.ok else if (isPickedWrong) c.bad else c.line, RoundedCornerShape(8.dp)),
            contentAlignment = Alignment.Center,
        ) {
            Text(letter, color = letterFg, fontSize = 11.5.sp, fontWeight = FontWeight.ExtraBold)
        }
        Spacer(Modifier.width(11.dp))
        Text(
            text,
            color = c.text.copy(alpha = if (dimmed) 0.5f else 1f),
            fontSize = 15.sp,
            lineHeight = 22.sp,
            fontFamily = KgType.sans,
        )
    }
}
