package com.rifqilubis.koreangrammar.tts

import android.content.Context
import android.speech.tts.TextToSpeech
import android.speech.tts.UtteranceProgressListener
import java.util.Locale

/**
 * Pelafalan Korea memakai TextToSpeech bawaan Android (`Locale.KOREAN`),
 * pengganti Web Speech API versi web.
 *
 * Aplikasi hanya memanggil [say]; teks yang dibacakan sudah dirapikan Rust
 * (`tts_text()`), mis. "먹다 → 먹어요" menjadi "먹어요".
 */
class Speaker(context: Context) {

    private var tts: TextToSpeech? = null

    /** True bila mesin suara Korea benar-benar tersedia (tombol 🔊 baru muncul). */
    @Volatile
    var available: Boolean = false
        private set

    /** Dipanggil sekali setiap status ketersediaan berubah. */
    var onChanged: (() -> Unit)? = null

    init {
        try {
            tts = TextToSpeech(context.applicationContext) { status ->
                val engine = tts
                if (status == TextToSpeech.SUCCESS && engine != null) {
                    val res = engine.setLanguage(Locale.KOREAN)
                    available = res != TextToSpeech.LANG_MISSING_DATA &&
                        res != TextToSpeech.LANG_NOT_SUPPORTED
                    if (available) engine.setSpeechRate(0.92f)
                }
                onChanged?.invoke()
            }
            tts?.setOnUtteranceProgressListener(object : UtteranceProgressListener() {
                override fun onStart(utteranceId: String?) {}
                override fun onDone(utteranceId: String?) {}
                @Deprecated("Dipanggil mesin suara lama")
                override fun onError(utteranceId: String?) {}
            })
        } catch (_: Throwable) {
            available = false
        }
    }

    /** Bacakan satu kalimat Korea. Diabaikan bila mesin suara tidak tersedia. */
    fun say(text: String) {
        val engine = tts ?: return
        if (!available || text.isBlank()) return
        try {
            engine.stop()
            engine.speak(text, TextToSpeech.QUEUE_FLUSH, null, "kg")
        } catch (_: Throwable) {
            // suara tidak tersedia — tidak perlu mengganggu pengguna
        }
    }

    fun shutdown() {
        try {
            tts?.stop()
            tts?.shutdown()
        } catch (_: Throwable) {
        }
        tts = null
        available = false
    }
}
