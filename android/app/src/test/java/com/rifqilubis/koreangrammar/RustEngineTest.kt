package com.rifqilubis.koreangrammar

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.kg_ffi.BlockFfi
import uniffi.kg_ffi.allTopics
import uniffi.kg_ffi.coreVersion
import uniffi.kg_ffi.home
import uniffi.kg_ffi.latCheck
import uniffi.kg_ffi.progressAddAnswered
import uniffi.kg_ffi.progressHasAny
import uniffi.kg_ffi.progressNew
import uniffi.kg_ffi.quizAdvance
import uniffi.kg_ffi.quizEnd
import uniffi.kg_ffi.quizPick
import uniffi.kg_ffi.quizResult
import uniffi.kg_ffi.quizStart
import uniffi.kg_ffi.quizState
import uniffi.kg_ffi.sanitizeTheme
import uniffi.kg_ffi.storeKey
import uniffi.kg_ffi.themeKey
import uniffi.kg_ffi.ttsText

/**
 * Uji JVM terhadap binding UniFFI yang benar-benar dipakai aplikasi.
 *
 * Pustaka Rust host (.dll/.so/.dylib) otomatis dimuat oleh task Gradle
 * `hostRustLibs` sebelum tes berjalan, jadi tes ini membuktikan bahwa
 * `.so` yang dipaketkan ke APK punya API dan perilaku yang sama.
 */
class RustEngineTest {

    @Test
    fun topikLengkapDanBerurutan() {
        val topics = allTopics()
        assertEquals(10, topics.size)
        assertEquals(listOf(1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u, 9u, 10u), topics.map { it.no })
        assertEquals("hangeul", topics.first().id)
        assertEquals("lampau", topics.last().id)
        assertTrue(coreVersion().isNotEmpty())
        assertTrue(topics.all { it.quiz.isNotEmpty() && it.lat.isNotEmpty() })
        // Setiap blok punya jenis yang bisa digambar layar materi.
        val kinds = setOf("heading", "text", "formula", "rules", "examples", "note")
        topics.forEach { t ->
            t.blocks.forEach { b: BlockFfi ->
                assertTrue("jenis blok tak dikenal: ${b.kind}", b.kind in kinds)
            }
        }
    }

    @Test
    fun kunciPenyimpananSamaDenganVersiWeb() {
        assertEquals("kg-progress-v1", storeKey())
        assertEquals("kg-theme-v1", themeKey())
    }

    @Test
    fun progresBaruKosongLaluTerisi() {
        val empty = progressNew()
        assertFalse(progressHasAny(empty))
        val one = progressAddAnswered(empty, 1)
        assertTrue(progressHasAny(one))
        assertEquals("1", home(one).answered)
    }

    @Test
    fun kuisPenuhMemberiSkorSeratus() {
        assertTrue(quizStart("hangeul", "home", 0uL))
        var guard = 0
        while (true) {
            val st = quizState()!!
            if (!st.answered) assertTrue(quizPick(st.answer.toInt()))
            guard++
            assertTrue(guard < 100)
            if (quizAdvance() == "finished") break
        }
        val r = quizResult(3_600_000uL)!!
        assertEquals(100u, r.percent)
        assertEquals(10u, r.correct)
        assertEquals("Sempurna!", r.headline)
        assertEquals("ok", r.ringTone)
        assertEquals("60:00", r.time)
        quizEnd()
    }

    @Test
    fun salahMenjawabMenampilkanPembahasan() {
        assertTrue(quizStart("hangeul", "lesson", 0uL))
        val st = quizState()!!
        val wrong = ((st.answer.toInt() + 1) % 4)
        assertTrue(quizPick(wrong))
        val after = quizState()!!
        assertTrue(after.answered)
        assertFalse(after.letters.isEmpty())
        // Teks umpan balik & penjelasan datang dari Rust.
        assertTrue(after.feedback.startsWith("Belum tepat"))
        assertEquals("bad", after.feedbackKind)
        assertTrue(after.explanation.isNotEmpty())
        assertEquals("Benar 0", after.scoreLabel)
        quizEnd()
    }

    @Test
    fun latihanMenulisMencocokkanSepertiWeb() {
        val t = allTopics().first { it.id == "urutan" }
        val item = t.lat.first()
        assertEquals("ok", latCheck("urutan", 0u, item.ko))
        assertEquals("", latCheck("urutan", 0u, ""))
        assertEquals("bad", latCheck("urutan", 0u, "bukan jawaban"))
        // Indeks di luar batas tidak dilaporkan salah.
        assertEquals("", latCheck("urutan", 999u, item.ko))
    }

    @Test
    fun temaDibatasiRust() {
        assertEquals("dark", sanitizeTheme("dark"))
        assertEquals("light", sanitizeTheme("light"))
        assertEquals("dark", sanitizeTheme("aneh"))
    }

    @Test
    fun teksSuaraDirapikan() {
        assertEquals("안녕하세요", ttsText("안녕하세요 (annyeonghaseyo)"))
        assertEquals("saya", ttsText("저 → saya"))
        assertNotNull(ttsText("먹어요."))
    }
}
