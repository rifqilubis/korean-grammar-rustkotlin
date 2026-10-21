# 한국어 문법 — Korean Grammar (Rust + Kotlin)

Aplikasi Android native untuk belajar tata bahasa Korea, hasil port dari versi
web/PWA [`korean-grammar`](../korean-grammar).

**Hampir semua logika ditulis di Rust.** Kotlin hanya dipakai untuk hal yang
memang harus di Android: menggambar layar (Jetpack Compose) dan memanggil
mesin suara bawaan sistem (TextToSpeech).

| | |
|---|---|
| Bahasa | Rust 2021 (inti) + Kotlin (UI) |
| Ukuran logika Rust | ~2.800 baris (termasuk materi), **kg-core tanpa dependensi pihak ketiga** |
| Materi | 10 topik, 82 soal kuis, 32 latihan menulis, 61 contoh kalimat |
| Izin | **tidak ada** (offline penuh) |
| ABI | `arm64-v8a`, `armeabi-v7a`, `x86_64` |
| minSdk / targetSdk | 24 / 34 |

---

## 1. Pembagian tugas Rust vs Kotlin

Aturannya sederhana: **kalau bisa diputuskan tanpa menyentuh Android, itu
tugas Rust.**

| Bagian | Dikerjakan oleh | Berkas |
|---|---|---|
| Seluruh materi (teks, aturan, contoh, soal, latihan) | Rust | `kg-core/src/data_gen.rs` |
| Mesin kuis (urutan soal, kunci, skor, umpan balik, durasi) | Rust | `kg-core/src/quiz.rs` |
| Progres & skor terbaik (JSON sama dengan `localStorage` versi web) | Rust | `kg-core/src/progress.rs` |
| Pencocokan latihan menulis (normalisasi teks Korea) | Rust | `kg-core/src/latihan.rs` |
| Teks yang dibacakan mesin suara | Rust | `kg-core/src/tts.rs` |
| Teks UI berbahasa Indonesia, label tombol, badge, statistik | Rust | `kg-core/src/quiz.rs`, `lib.rs` |
| Navigasi antar layar, tema gelap/terang | Kotlin | `android/.../MainActivity.kt` |
| Menggambar layar (Compose), animasi cincin skor | Kotlin | `android/.../ui/*.kt` |
| Menyimpan JSON progres ke disk | Kotlin | `android/.../KgApp.kt` |
| Suara Korea | Kotlin | `android/.../tts/Speaker.kt` |

Kotlin **tidak** berisi aturan permainan: layar hanya menggambar apa yang
dikembalikan Rust. Contohnya, tombol di layar hasil berbunyi
`"Topik berikutnya"` bukan karena Kotlin memutuskan begitu, tetapi karena
`KG.quizResult()` di Rust mengirim `nextLabel`.

## 2. Struktur repo

```
korean-grammar-rust/
├─ Cargo.toml              # workspace: kg-core, kg-ffi, kg-ffi/uniffi-bindgen
├─ kg-core/                # inti Rust murni (tanpa dependensi, forbid(unsafe_code))
│  ├─ src/lib.rs           # model topik, navigasi, tema
│  ├─ src/model.rs         # Rule, Example, Block, Question, Topic
│  ├─ src/quiz.rs          # sesi kuis + teks hasil
│  ├─ src/progress.rs      # encoder/parser JSON progres (dibuat tangan)
│  ├─ src/latihan.rs       # normalize() + penilaian latihan menulis
│  ├─ src/tts.rs           # pembersihan teks pelafalan
│  ├─ src/data_gen.rs      # materi — DIBANGKITKAN, jangan diedit manual
│  └─ tests/flow.rs        # 9 uji alur (meniru tools/smoke-test.js versi web)
├─ kg-ffi/                 # jembatan UniFFI (Record + fungsi untuk Kotlin)
│  ├─ src/lib.rs
│  ├─ src/kg.udl
│  └─ tests/ffi_surface.rs # 3 uji kontrak FFI
├─ tools/
│  ├─ gen-data.cjs         # ubah js/data.js versi web -> data_gen.rs
│  └─ make-icons.py        # buat mipmap dari ikon 512px
└─ android/
   ├─ app/src/main/java/com/rifqilubis/koreangrammar/
   │  ├─ MainActivity.kt   # router + state
   │  ├─ KgApp.kt          # SharedPreferences + akses ke Rust
   │  ├─ tts/Speaker.kt
   │  └─ ui/               # Theme, Components, Home, Lesson, Quiz, Result, Latihan
   └─ app/src/test/        # 8 uji JVM yang memanggil pustaka Rust asli
```

## 3. Cara kerja jembatan Rust → Kotlin

Memakai [UniFFI](https://mozilla.github.io/uniffi-rs/) 0.32 dalam mode
proc-macro. Alurnya sepenuhnya otomatis lewat Gradle (`android/app/build.gradle.kts`):

1. `cargoBuild` — kompilasi `kg-ffi` untuk tiga target Android **dan** untuk
   host. Variabel linker NDK diisi otomatis dari NDK di dalam SDK.
2. `syncRustLibs` — salin `libkg_ffi.so` ke `build/rustJniLibs/<abi>/` yang
   menjadi `jniLibs`.
3. `generateUniffiBindings` — jalankan `uniffi-bindgen` (dibangun dari
   workspace) untuk menghasilkan `kg_ffi.kt` di
   `build/generated/uniffi/`. Berkas ini **tidak** di-commit.
4. `hostRustLibs` — salin `kg_ffi.dll`/`.so`/`.dylib` untuk uji JVM.

Binding UniFFI memanggil fungsi Rust lewat JNA, jadi `net.java.dev.jna:jna`
dipaketkan di APK dan `libjnidispatch.so` ikut masuk ke tiap ABI.

### Contoh pemakaian dari Kotlin

```kotlin
import uniffi.kg_ffi.quizStart
import uniffi.kg_ffi.quizState
import uniffi.kg_ffi.quizPick

quizStart("hangeul", "lesson", System.currentTimeMillis().toULong())
val st = quizState()!!            // pertanyaan, pilihan, kunci, label — dari Rust
st.question                        // "Huruf ㅎ dibaca…"
st.options                         // ["h", "n", "m", "r"]
quizPick(0)                        // true bila diterima
```

Progres disimpan sebagai satu string JSON dengan bentuk yang **identik**
dengan versi web, sehingga data dari `localStorage` bisa dipindahkan apa adanya:

```json
{"version":1,"topics":{"hangeul":{"bestPct":90,"bestCorrect":9,"total":10,"done":1,"lastAt":1700000000000}},"answered":42}
```

## 4. Membangun sendiri

### Yang dibutuhkan

* Rust (stable) + target Android:
  `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`
* JDK 17
* Android SDK: `platforms;android-34`, `build-tools;34.0.0`,
  `ndk;27.2.12479018`
* Gradle 8.7 (atau `./gradlew`)

### Perintah

```bash
# uji inti Rust
cargo test --workspace

# APK debug (Rust, binding, dan Kotlin dibangun otomatis)
cd android
./gradlew assembleDebug

# APK rilis (belum ditandatangani)
./gradlew assembleRelease

# uji unit Kotlin terhadap pustaka Rust
./gradlew testDebugUnitTest
```

Hasilnya di `android/app/build/outputs/apk/debug/app-debug.apk`.

Tanpa `./gradlew`? Pakai `gradle assembleDebug` dari folder `android`.
NDK dicari lewat `ANDROID_NDK_HOME`, lalu `$ANDROID_HOME/ndk/*`, lalu
`local.properties` (`ndk.dir=...`).

> Catatan lingkungan: bila `~/.android` atau `~/.local/share/kotlin` tidak bisa
> ditulis, APK debug tetap bisa dibangun — tanda tangan debug dilewati
> otomatis dan kompilasi Kotlin berjalan di dalam proses Gradle
> (`-Pkotlin.compiler.execution.strategy=in-process`).

## 5. Memperbarui materi

Materi berasal dari aplikasi web, tidak ditulis dua kali. Setelah mengubah
`../korean-grammar/js/data.js`:

```bash
node tools/gen-data.cjs            # tulis ulang kg-core/src/data_gen.rs
cargo test --workspace             # 36 uji harus hijau
```

Skrip ini memvalidasi data lebih dulu (jumlah topik, 4 pilihan per soal, indeks
jawaban yang sah) lalu membangkitkan Rust. Ikon peluncur dibuat ulang dengan
`python tools/make-icons.py`.

## 6. Pengujian

| Tingkat | Jumlah | Perintah |
|---|---|---|
| Unit inti Rust | 19 | `cargo test -p kg-core --lib` |
| Alur Rust (meniru smoke test web) | 9 | `cargo test -p kg-core --test flow` |
| Kontrak FFI | 3 | `cargo test -p kg-ffi` |
| JVM (memanggil `.so`/`.dll` Rust) | 8 | `./gradlew testDebugUnitTest` |

Uji alur mencakup: kelengkapan 10 topik, menjawab 100% benar, pembahasan
jawaban salah, perjalanan JSON progres, skor terbaik yang tidak pernah turun,
latihan menulis, dan navigasi topik. Uji JVM membuktikan pustaka yang
dipaketkan ke APK punya API dan perilaku yang sama.

## 7. Berkas yang dibangkitkan (jangan diedit)

* `kg-core/src/data_gen.rs` (oleh `tools/gen-data.cjs`)
* `android/app/build/generated/uniffi/uniffi/kg_ffi/kg_ffi.kt` (oleh UniFFI)
* `android/app/src/main/res/mipmap-*/ic_launcher*.png` (oleh `tools/make-icons.py`)

## 8. Lisensi

Materi dan kode mengikuti repositori asal `korean-grammar`.
