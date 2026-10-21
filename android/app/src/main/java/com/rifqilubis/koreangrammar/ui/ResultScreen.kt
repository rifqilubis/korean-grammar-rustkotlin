package com.rifqilubis.koreangrammar.ui

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import uniffi.kg_ffi.QuizResultFfi

/**
 * Layar hasil — `#screen-result` + `finishQuiz()`.
 *
 * Cincin skor dianimasikan seperti `stroke-dashoffset` di CSS;
 * warnanya dipilih Rust (`ring_tone`).
 */
@Composable
fun ResultScreen(
    c: KgColors,
    result: QuizResultFfi,
    hasNext: Boolean,
    onRetry: () -> Unit,
    onBackLesson: () -> Unit,
    onNextTopic: () -> Unit,
) {
    val ringColor = when (result.ringTone) {
        "ok" -> c.ok
        "bad" -> c.bad
        else -> c.accent
    }
    val target = result.percent.coerceIn(0u, 100u).toFloat() / 100f
    val progress by animateFloatAsState(
        targetValue = target,
        animationSpec = tween(durationMillis = 800, easing = LinearEasing),
        label = "ring",
    )

    Column(Modifier.fillMaxWidth()) {
        Spacer(Modifier.height(20.dp))
        KgCard(c, modifier = Modifier.fillMaxWidth(), padding = 22.dp) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                // ---------------------------- cincin skor ----------------------------
                Box(Modifier.size(132.dp), contentAlignment = Alignment.Center) {
                    Canvas(Modifier.size(132.dp)) {
                        val stroke = 9.dp.toPx()
                        val inset = stroke / 2f
                        val arcSize = Size(size.width - stroke, size.height - stroke)
                        drawArc(
                            color = c.surface2,
                            startAngle = 0f,
                            sweepAngle = 360f,
                            useCenter = false,
                            topLeft = Offset(inset, inset),
                            size = arcSize,
                            style = Stroke(width = stroke),
                        )
                        drawArc(
                            color = ringColor,
                            startAngle = -90f,
                            sweepAngle = 360f * progress,
                            useCenter = false,
                            topLeft = Offset(inset, inset),
                            size = arcSize,
                            style = Stroke(width = stroke, cap = StrokeCap.Round),
                        )
                    }
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(
                            "${result.percent}%",
                            color = c.text,
                            fontSize = 28.sp,
                            fontWeight = FontWeight.ExtraBold,
                            letterSpacing = (-1).sp,
                        )
                        Text(
                            "${result.correct} / ${result.total}",
                            color = c.textFaint,
                            fontSize = 12.sp,
                        )
                    }
                }

                Spacer(Modifier.height(18.dp))
                Text(
                    result.headline,
                    color = c.text,
                    fontSize = 21.sp,
                    fontWeight = FontWeight.ExtraBold,
                    lineHeight = 27.sp,
                    textAlign = TextAlign.Center,
                )
                Spacer(Modifier.height(8.dp))
                Text(
                    result.message,
                    color = c.textDim,
                    fontSize = 14.5.sp,
                    lineHeight = 22.sp,
                    textAlign = TextAlign.Center,
                )

                Spacer(Modifier.height(22.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    ResultStat(result.correct.toString(), "Benar", c, c.ok, Modifier.weight(1f))
                    ResultStat(result.wrong.toString(), "Salah", c, c.bad, Modifier.weight(1f))
                    ResultStat(result.time, "Waktu", c, c.text, Modifier.weight(1f))
                }

                Spacer(Modifier.height(22.dp))
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    KgButton("Ulangi kuis", c, primary = true, block = true, onClick = onRetry)
                    KgButton("Baca materi lagi", c, block = true, onClick = onBackLesson)
                    if (hasNext) {
                        KgButton("Topik berikutnya", c, block = true, onClick = onNextTopic)
                    }
                }
            }
        }

        if (result.wrongAnswers.isNotEmpty()) {
            Spacer(Modifier.height(14.dp))
            KgCard(c, modifier = Modifier.fillMaxWidth(), padding = 20.dp) {
                Column(Modifier.fillMaxWidth()) {
                    Text(
                        "Pembahasan jawaban yang belum tepat",
                        color = c.text,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Bold,
                    )
                    Spacer(Modifier.height(12.dp))
                    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        result.wrongAnswers.forEach { w ->
                            Column(
                                Modifier
                                    .fillMaxWidth()
                                    .clip(RoundedCornerShape(KgDims.radiusSm))
                                    .background(c.surface2)
                                    .border(1.dp, c.line, RoundedCornerShape(KgDims.radiusSm))
                                    .padding(horizontal = 14.dp, vertical = 12.dp),
                            ) {
                                Text(
                                    w.q,
                                    color = c.text,
                                    fontSize = 14.sp,
                                    lineHeight = 21.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    fontFamily = KgType.sans,
                                )
                                Spacer(Modifier.height(6.dp))
                                Text(
                                    buildAnnotatedString {
                                        withStyle(SpanStyle(color = c.textDim)) { append("Jawabanmu: ") }
                                        withStyle(SpanStyle(color = c.bad, fontWeight = FontWeight.Bold)) {
                                            append(w.picked)
                                        }
                                    },
                                    fontSize = 13.5.sp,
                                    lineHeight = 20.sp,
                                    fontFamily = KgType.sans,
                                )
                                Text(
                                    buildAnnotatedString {
                                        withStyle(SpanStyle(color = c.textDim)) { append("Yang benar: ") }
                                        withStyle(SpanStyle(color = c.ok, fontWeight = FontWeight.Bold)) {
                                            append(w.right)
                                        }
                                    },
                                    fontSize = 13.5.sp,
                                    lineHeight = 20.sp,
                                    fontFamily = KgType.sans,
                                )
                                if (w.why.isNotEmpty()) {
                                    Spacer(Modifier.height(7.dp))
                                    Text(w.why, color = c.textFaint, fontSize = 13.sp, lineHeight = 19.sp)
                                }
                            }
                        }
                    }
                }
            }
        }
        Spacer(Modifier.height(12.dp))
    }
}

@Composable
private fun ResultStat(
    value: String,
    label: String,
    c: KgColors,
    valueColor: Color,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier
            .clip(RoundedCornerShape(KgDims.radiusSm))
            .background(c.surface2)
            .border(1.dp, c.line, RoundedCornerShape(KgDims.radiusSm))
            .padding(12.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(value, color = valueColor, fontSize = 20.sp, fontWeight = FontWeight.ExtraBold, maxLines = 1)
        Text(label, color = c.textFaint, fontSize = 11.5.sp, maxLines = 1)
    }
}
