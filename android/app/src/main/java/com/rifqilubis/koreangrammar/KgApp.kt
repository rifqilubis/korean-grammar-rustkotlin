package com.rifqilubis.koreangrammar

import android.content.Context
import android.content.SharedPreferences
import com.rifqilubis.koreangrammar.tts.Speaker
import uniffi.kg_ffi.home
import uniffi.kg_ffi.progressNew
import uniffi.kg_ffi.sanitizeTheme
import uniffi.kg_ffi.storeKey
import uniffi.kg_ffi.themeKey

/**
 * Status aplikasi yang dipakai bersama layar Compose.
 *
 * Penting: **tidak ada aturan aplikasi di sini.** Semua keputusan (skor,
 * progres, teks badge, pencocokan latihan, jadwal soal) dihitung Rust
 * (`kg-core`); kelas ini hanya menyimpan hasilnya sebagai state Compose dan
 * menulis/membaca string yang diterima ke `SharedPreferences`.
 */
class KgApp(context: Context) {

    private val ctx: Context = context.applicationContext

    private val prefs: SharedPreferences =
        ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    val speaker = Speaker(ctx)

    /** JSON progres (format & kunci sama dengan localStorage versi web). */
    var progressJson: String = ""
        private set

    /** "dark" atau "light" — dibatasi Rust lewat sanitizeTheme(). */
    var theme: String = "dark"
        private set

    /** Daftar id topik yang punya latihan menulis (dari Rust saat boot). */
    var topicsWithLatihan: Set<String> = emptySet()
        private set

    init {
        reload()
    }

    fun reload() {
        progressJson = prefs.getString(storeKey(), null)?.takeIf { it.isNotEmpty() }
            ?: progressNew()
        theme = sanitizeTheme(
            prefs.getString(themeKey(), null)
                ?: if (contextResolvesLight(ctx)) "light" else "dark"
        )
    }

    fun setProgress(json: String) {
        progressJson = json
        prefs.edit().putString(storeKey(), json).apply()
    }

    fun setTheme(name: String) {
        theme = sanitizeTheme(name)
        prefs.edit().putString(themeKey(), theme).apply()
    }

    /** Data kartu + statistik layar awal, dihitung Rust. */
    fun home() = home(progressJson)

    fun setTopicsWithLatihan(ids: Set<String>) {
        topicsWithLatihan = ids
    }

    companion object {
        private const val PREFS = "kg"

        /** Ikuti mode gelap/terang sistem untuk tema pertama kali (CSS `prefers-color-scheme`). */
        private fun contextResolvesLight(context: Context): Boolean {
            val mode = context.resources.configuration.uiMode and
                android.content.res.Configuration.UI_MODE_NIGHT_MASK
            return mode == android.content.res.Configuration.UI_MODE_NIGHT_NO
        }

        @Volatile
        private var instance: KgApp? = null

        fun get(context: Context): KgApp {
            return instance ?: synchronized(this) {
                instance ?: KgApp(context.applicationContext).also { instance = it }
            }
        }
    }
}
