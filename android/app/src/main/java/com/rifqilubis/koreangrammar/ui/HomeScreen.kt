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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import uniffi.kg_ffi.HomeFfi
import uniffi.kg_ffi.TopicCardFfi

/**
 * Layar awal — `#screen-home` di index.html, digambar oleh `renderHome()`.
 *
 * Semua tulisan (statistik, label tombol lanjut, badge kartu) berasal dari
 * `KG.home(json)` di Rust.
 */
@Composable
fun HomeScreen(
    c: KgColors,
    home: HomeFfi,
    onOpenTopic: (String) -> Unit,
    onResume: () -> Unit,
    onReset: () -> Unit,
) {
    var confirmReset by remember { mutableStateOf(false) }

    Column(Modifier.fillMaxWidth()) {
        // ------------------------------- hero card -------------------------------
        KgCard(c, modifier = Modifier.fillMaxWidth(), padding = 22.dp) {
            Column {
                Text(
                    "안녕하세요! Mulai dari sini",
                    color = c.accent,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Bold,
                    fontFamily = KgType.sans,
                )
                Spacer(Modifier.height(10.dp))
                Text(
                    "Grammar Korea untuk pemula",
                    color = c.text,
                    fontSize = KgType.heroTitle,
                    lineHeight = 32.sp,
                    fontWeight = FontWeight.ExtraBold,
                    fontFamily = KgType.sans,
                )
                Spacer(Modifier.height(10.dp))
                BodyText(
                    "10 topik dari nol sampai bisa membuat kalimat sendiri. " +
                        "Baca materinya, lalu uji dengan kuis pilihan ganda.",
                    c,
                    androidx.compose.ui.text.TextStyle(fontSize = 15.sp, lineHeight = 23.sp),
                )

                Spacer(Modifier.height(22.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    Box(Modifier.weight(1f)) {
                        MiniStat(
                            home.topicsDone,
                            "topik tuntas",
                            c,
                            Modifier.fillMaxWidth(),
                        )
                    }
                    Box(Modifier.weight(1f)) {
                        MiniStat(home.quizScore, "rata-rata kuis", c, Modifier.fillMaxWidth())
                    }
                    Box(Modifier.weight(1f)) {
                        MiniStat(home.answered, "soal dijawab", c, Modifier.fillMaxWidth())
                    }
                }

                Spacer(Modifier.height(20.dp))
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    KgButton(
                        home.resumeLabel,
                        c,
                        primary = true,
                        block = true,
                        onClick = onResume,
                    )
                    if (home.hasProgress) {
                        KgButton("Hapus progres", c, block = true) { confirmReset = true }
                    }
                }
            }
        }

        Spacer(Modifier.height(26.dp))
        SectionLabel("Daftar materi", c, modifier = Modifier.padding(start = 2.dp))
        Spacer(Modifier.height(12.dp))

        Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            home.cards.forEach { card ->
                TopicRow(card, c) { onOpenTopic(card.id) }
            }
        }

        Spacer(Modifier.height(22.dp))
        Text(
            "Progresmu disimpan otomatis di perangkat ini.\nSemua materi sudah ada di dalam aplikasi, jadi bisa dibuka tanpa internet.",
            color = c.textFaint,
            fontSize = 12.5.sp,
            lineHeight = 19.sp,
            textAlign = androidx.compose.ui.text.style.TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )
    }

    if (confirmReset) {
        AlertDialog(
            onDismissRequest = { confirmReset = false },
            containerColor = c.surface,
            titleContentColor = c.text,
            textContentColor = c.textDim,
            title = { Text("Hapus progres?", fontWeight = FontWeight.Bold) },
            text = { Text("Hapus semua progres belajar di perangkat ini?") },
            confirmButton = {
                TextButton(onClick = {
                    confirmReset = false
                    onReset()
                }) { Text("Hapus", color = c.bad, fontWeight = FontWeight.Bold) }
            },
            dismissButton = {
                TextButton(onClick = { confirmReset = false }) { Text("Batal", color = c.textDim) }
            },
        )
    }
}

/** Satu kartu `.topic-card`: ikon · nama · badge. */
@Composable
private fun TopicRow(card: TopicCardFfi, c: KgColors, onClick: () -> Unit) {
    Row(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(KgDims.radius))
            .background(c.surface)
            .border(1.dp, c.line, RoundedCornerShape(KgDims.radius))
            .clickable(role = Role.Button, onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 15.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        TopicIcon(card.icon, c, done = card.done)
        Spacer(Modifier.width(14.dp))
        Column(Modifier.weight(1f)) {
            Text(
                card.name,
                color = c.text,
                fontSize = 15.5.sp,
                fontWeight = FontWeight.Bold,
                fontFamily = KgType.sans,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.height(1.dp))
            Text(
                card.titleId,
                color = c.textDim,
                fontSize = 13.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Spacer(Modifier.width(12.dp))
        Badge(
            card.badge,
            c,
            if (card.badgeIsScore) BadgeTone.Score else BadgeTone.Neutral,
        )
    }
}
