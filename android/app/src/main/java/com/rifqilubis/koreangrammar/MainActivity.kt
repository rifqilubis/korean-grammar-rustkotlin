package com.rifqilubis.koreangrammar

import android.graphics.Color as AndroidColor
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.view.WindowCompat
import com.rifqilubis.koreangrammar.ui.HomeScreen
import com.rifqilubis.koreangrammar.ui.IconButtonBox
import com.rifqilubis.koreangrammar.ui.KgColors
import com.rifqilubis.koreangrammar.ui.KgDims
import com.rifqilubis.koreangrammar.ui.KgType
import com.rifqilubis.koreangrammar.ui.LatCheck
import com.rifqilubis.koreangrammar.ui.LatEntry
import com.rifqilubis.koreangrammar.ui.LatihanScreen
import com.rifqilubis.koreangrammar.ui.LessonScreen
import com.rifqilubis.koreangrammar.ui.QuizScreen
import com.rifqilubis.koreangrammar.ui.ResultScreen
import com.rifqilubis.koreangrammar.ui.ThemeIcon
import com.rifqilubis.koreangrammar.ui.latCheckOf
import uniffi.kg_ffi.HomeFfi
import uniffi.kg_ffi.QuizResultFfi
import uniffi.kg_ffi.QuizStateFfi
import uniffi.kg_ffi.allTopics
import uniffi.kg_ffi.coreVersion
import uniffi.kg_ffi.latCheck
import uniffi.kg_ffi.latInlineResult
import uniffi.kg_ffi.latReveal
import uniffi.kg_ffi.progressAddAnswered
import uniffi.kg_ffi.progressRecord
import uniffi.kg_ffi.progressReset
import uniffi.kg_ffi.quizAdvance
import uniffi.kg_ffi.quizEnd
import uniffi.kg_ffi.quizPick
import uniffi.kg_ffi.quizResult
import uniffi.kg_ffi.quizStart
import uniffi.kg_ffi.quizState
import uniffi.kg_ffi.ttsText

/**
 * Satu Activity untuk seluruh aplikasi (versi web: router hash `#/topik/…`).
 *
 * Yang ada di sini hanya navigasi dan gambar. Semua aturan — daftar topik,
 * soal, kunci jawaban, skor, progres, latihan, teks pelafalan — dipanggil
 * dari pustaka Rust lewat binding UniFFI (`uniffi.kg_ffi`).
 */
class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge(
            statusBarStyle = SystemBarStyle.dark(AndroidColor.TRANSPARENT),
            navigationBarStyle = SystemBarStyle.dark(AndroidColor.TRANSPARENT),
        )
        val app = KgApp.get(this)

        setContent {
            var colors by remember { mutableStateOf(KgColors.of(app.theme)) }
            var ttsAvailable by remember { mutableStateOf(app.speaker.available) }
            // Versi inti Rust, ditampilkan di kaki halaman.
            val rustVersion = remember { coreVersion() }

            // Mesin suara bisa siap sesudah layar pertama muncul.
            DisposableEffect(Unit) {
                app.speaker.onChanged = { ttsAvailable = app.speaker.available }
                onDispose { app.speaker.onChanged = null }
            }

            ApplySystemBars(dark = colors.isDark)

            KgRoot(
                app = app,
                colors = colors,
                ttsAvailable = ttsAvailable,
                rustVersion = rustVersion,
                onToggleTheme = {
                    app.setTheme(if (app.theme == "dark") "light" else "dark")
                    colors = KgColors.of(app.theme)
                },
            )
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        if (isFinishing) KgApp.get(this).speaker.shutdown()
    }
}

/** Ikon bilah status & navigasi: gelap di tema terang, terang di tema gelap. */
@Composable
private fun ApplySystemBars(dark: Boolean) {
    // minSdk 24, jadi tidak perlu cek SDK_INT.
    val view = LocalView.current
    SideEffect {
        val activity = view.context as? ComponentActivity ?: return@SideEffect
        val controller = WindowCompat.getInsetsController(activity.window, activity.window.decorView)
        controller.isAppearanceLightStatusBars = !dark
        controller.isAppearanceLightNavigationBars = !dark
    }
}

/** Halaman yang sedang tampil — padanan hash router versi web. */
private sealed interface Route {
    data object Home : Route
    data class Lesson(val topicId: String) : Route
    data class Latihan(val topicId: String) : Route
    data object Quiz : Route
    data object Result : Route
}

@Composable
private fun KgRoot(
    app: KgApp,
    colors: KgColors,
    ttsAvailable: Boolean,
    rustVersion: String,
    onToggleTheme: () -> Unit,
) {
    // Daftar topik dari Rust, sekali saat aplikasi dibuka (`window.KG_TOPICS`).
    val topics = remember { allTopics() }
    val byId = remember(topics) { topics.associateBy { it.id } }

    var route by remember { mutableStateOf<Route>(Route.Home) }

    // ------------------------------- state kuis -------------------------------
    var quizState by remember { mutableStateOf<QuizStateFfi?>(null) }
    var quizTopicId by remember { mutableStateOf<String?>(null) }
    var quizFromLesson by remember { mutableStateOf(false) }
    var result by remember { mutableStateOf<QuizResultFfi?>(null) }

    // ------------------------------ state latihan -----------------------------
    val latEntries = remember { mutableStateListOf<LatEntry>() }
    var latStatus by remember { mutableStateOf("") }

    // Statistik & kartu layar awal — dihitung Rust dari JSON progres.
    var home by remember { mutableStateOf<HomeFfi>(app.home()) }

    fun refreshHome() {
        home = app.home()
    }

    fun openTopic(id: String) {
        if (byId.containsKey(id)) route = Route.Lesson(id)
    }

    fun startQuiz(topicId: String, fromLesson: Boolean) {
        val from = if (fromLesson) "lesson" else "home"
        if (!quizStart(topicId, from, System.currentTimeMillis().toULong())) return
        quizTopicId = topicId
        quizFromLesson = fromLesson
        quizState = quizState()
        result = null
        route = Route.Quiz
    }

    fun openLatihan(topicId: String) {
        val t = byId[topicId] ?: return
        latEntries.clear()
        repeat(t.lat.size) { latEntries.add(LatEntry()) }
        latStatus = ""
        route = Route.Latihan(topicId)
    }

    fun pickOption(index: Int) {
        if (quizPick(index)) {
            app.setProgress(progressAddAnswered(app.progressJson, 1))
            quizState = quizState()
            refreshHome()
        }
    }

    fun nextQuestion() {
        when (quizAdvance()) {
            "more" -> quizState = quizState()
            "finished" -> {
                val now = System.currentTimeMillis()
                val r = quizResult(now.toULong()) ?: return
                app.setProgress(progressRecord(app.progressJson, r, now))
                result = r
                refreshHome()
                route = Route.Result
            }
        }
    }

    fun closeQuiz() {
        quizEnd()
        quizState = null
        val id = quizTopicId
        route = if (quizFromLesson && id != null) Route.Lesson(id) else Route.Home
    }

    fun typeLatihan(index: Int, text: String) {
        val id = (route as? Route.Latihan)?.topicId ?: return
        if (index !in latEntries.indices) return
        latEntries[index] = LatEntry(
            typed = text,
            check = latCheckOf(latCheck(id, index.toUInt(), text)),
            inline = latInlineResult(id, index.toUInt(), text),
        )
    }

    fun revealLatihan() {
        val id = (route as? Route.Latihan)?.topicId ?: return
        val out = latReveal(id, latEntries.map { it.typed }) ?: return
        out.lines.forEach { line ->
            val i = line.index.toInt()
            if (i in latEntries.indices) {
                latEntries[i] = if (latCheckOf(line.check) == LatCheck.Ok) {
                    LatEntry(latEntries[i].typed, LatCheck.Ok, "Tepat!")
                } else {
                    LatEntry(latEntries[i].typed, LatCheck.Bad, line.text)
                }
            }
        }
        latStatus = out.status
    }

    // ------------------------------ tombol kembali ------------------------------
    BackHandler(enabled = route != Route.Home) {
        route = when (val r = route) {
            is Route.Lesson -> Route.Home
            is Route.Latihan -> Route.Lesson(r.topicId)
            Route.Quiz -> {
                val id = quizTopicId
                if (quizFromLesson && id != null) Route.Lesson(id) else Route.Home
            }
            Route.Result -> {
                val id = result?.topicId
                if (id != null && quizFromLesson) Route.Lesson(id) else Route.Home
            }
            Route.Home -> Route.Home
        }
    }

    // Ruang aman: poni (notch) di atas, bilah gestur di bawah.
    val insets = WindowInsets.safeDrawing
    val top = insets.only(WindowInsetsSides.Top).asPaddingValues().calculateTopPadding()
    val bottom = insets.only(WindowInsetsSides.Bottom).asPaddingValues().calculateBottomPadding()
    // Kiri/kanan dipalikasikan lewat `safeContentPadding` di kontainer layar.

    Box(
        Modifier
            .fillMaxSize()
            .background(colors.bg)
            // body { radial-gradient(… var(--accent-soft) …, transparent) }
            .background(
                Brush.radialGradient(
                    colors = listOf(colors.accentSoft, Color.Transparent),
                    center = Offset(1400f, -160f),
                    radius = 1500f,
                )
            ),
    ) {
        Column(
            Modifier
                .fillMaxSize()
                .windowInsetsPadding(
                    WindowInsets.safeDrawing.only(WindowInsetsSides.Horizontal)
                ),
        ) {
            TopBar(
                colors,
                onToggleTheme,
                modifier = Modifier.padding(top = top + 14.dp),
            )

            Column(
                Modifier
                    .fillMaxSize()
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = KgDims.gutterSmall)
                    .padding(bottom = bottom + KgDims.bottomPadding)
                    .widthIn(max = KgDims.maxWidth)
                    .align(Alignment.CenterHorizontally),
            ) {
                AnimatedContent(
                    targetState = route,
                    transitionSpec = { fadeIn(tween(200)) togetherWith fadeOut(tween(120)) },
                    label = "screen",
                ) { current ->
                    when (current) {
                        Route.Home -> HomeScreen(
                            c = colors,
                            home = home,
                            onOpenTopic = ::openTopic,
                            onResume = { openTopic(home.resumeTopic) },
                            onReset = {
                                app.setProgress(progressReset())
                                refreshHome()
                            },
                        )

                        is Route.Lesson -> {
                            val t = byId.getValue(current.topicId)
                            val idx = topics.indexOfFirst { it.id == t.id }
                            LessonScreen(
                                c = colors,
                                topic = t,
                                ttsAvailable = ttsAvailable,
                                prevTitle = topics.getOrNull(idx - 1)?.title,
                                nextTitle = topics.getOrNull(idx + 1)?.title,
                                onBack = { route = Route.Home },
                                onSpeak = { ko -> app.speaker.say(ttsText(ko)) },
                                onStartQuiz = { startQuiz(t.id, fromLesson = true) },
                                onLatihan = { openLatihan(t.id) },
                                onPrev = { topics.getOrNull(idx - 1)?.let { route = Route.Lesson(it.id) } },
                                onNext = { topics.getOrNull(idx + 1)?.let { route = Route.Lesson(it.id) } },
                            )
                        }

                        is Route.Latihan -> {
                            val t = byId.getValue(current.topicId)
                            LatihanScreen(
                                c = colors,
                                topic = t,
                                entries = latEntries.toList(),
                                status = latStatus,
                                onBack = { route = Route.Lesson(t.id) },
                                onType = ::typeLatihan,
                                onReveal = ::revealLatihan,
                                onReset = { openLatihan(t.id) },
                            )
                        }

                        Route.Quiz -> quizState?.let { st ->
                            QuizScreen(
                                c = colors,
                                state = st,
                                onPick = ::pickOption,
                                onNext = ::nextQuestion,
                                onClose = ::closeQuiz,
                            )
                        }

                        Route.Result -> result?.let { r ->
                            val idx = topics.indexOfFirst { it.id == r.topicId }
                            ResultScreen(
                                c = colors,
                                result = r,
                                hasNext = idx >= 0 && idx + 1 < topics.size,
                                onRetry = { startQuiz(r.topicId, quizFromLesson) },
                                onBackLesson = { route = Route.Lesson(r.topicId) },
                                onNextTopic = {
                                    topics.getOrNull(idx + 1)?.let { route = Route.Lesson(it.id) }
                                },
                            )
                        }
                    }
                }

                Spacer(Modifier.height(8.dp))
                AboutFooter(colors, rustVersion)
            }
        }
    }
}

/** `.topbar`: 한국어 문법 + tombol ganti tema. */
@Composable
private fun TopBar(
    colors: KgColors,
    onToggleTheme: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier
            .fillMaxWidth()
            .padding(start = 16.dp, end = 16.dp, bottom = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                "한국어",
                color = colors.accent,
                fontSize = 21.sp,
                fontWeight = FontWeight.ExtraBold,
                fontFamily = KgType.sans,
            )
            Spacer(Modifier.width(7.dp))
            Text(
                "문법",
                color = colors.textFaint,
                fontSize = 13.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 2.sp,
            )
        }
        IconButtonBox(colors, onClick = onToggleTheme) {
            ThemeIcon(colors, dark = colors.isDark)
        }
    }
}

/** Kaki halaman: keterangan singkat + versi inti Rust. */
@Composable
private fun AboutFooter(colors: KgColors, rustVersion: String) {
    Column(
        Modifier.fillMaxWidth().padding(top = 20.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            "Logika aplikasi berjalan di Rust · inti $rustVersion",
            color = colors.textFaint,
            fontSize = 11.5.sp,
        )
        Spacer(Modifier.height(2.dp))
        Text(
            "Tanpa izin · 100% offline",
            color = colors.textFaint,
            fontSize = 11.5.sp,
        )
    }
}
