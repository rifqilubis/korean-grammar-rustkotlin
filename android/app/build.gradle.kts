// Catatan: berkas ini sengaja TIDAK memakai `import` di bagian atas.
// Pada Gradle 8.7 + AGP 8.5, menaruh import sebelum blok `plugins { }` membuat
// blok `android { }` tidak dikenali ("compileSdkVersion is not specified").
// Karena itu semua tipe di bawah ditulis dengan nama lengkap.

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

/**
 * Apakah AGP bisa menulis folder preferensi Android (~/.android)?
 * Kalau tidak, `validateSigningDebug` gagal karena kunci debug tidak bisa
 * dibuat, jadi APK debug dibiarkan belum ditandatangani.
 */
fun androidPrefsWritable(): Boolean {
    val home = System.getenv("ANDROID_USER_HOME")?.takeIf { it.isNotBlank() }
        ?: System.getProperty("user.home")?.let { "$it/.android" }
        ?: return false
    val dir = File(home)
    return try {
        if (!dir.exists()) dir.mkdirs()
        val probe = File(dir, ".write-probe")
        probe.writeText("ok")
        probe.delete()
        true
    } catch (e: Exception) {
        false
    }
}

android {
    namespace = "com.rifqilubis.koreangrammar"
    compileSdk = 34
    ndkVersion = "27.2.12479018"

    defaultConfig {
        applicationId = "com.rifqilubis.koreangrammar"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "1.0.0"

        ndk {
            // ABI yang didukung: hampir semua ponsel (arm64 + armv7) dan emulator x86_64.
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    buildTypes {
        debug {
            applicationIdSuffix = ".debug"
            versionNameSuffix = "-debug"
            // Bila ~/.android tidak bisa ditulis, APK debug dibiarkan belum
            // ditandatangani (cukup untuk sideload lewat adb install).
            signingConfig = if (androidPrefsWritable()) {
                signingConfigs.getByName("debug")
            } else {
                logger.lifecycle("Signing debug dilewati: folder prefs Android tidak bisa ditulis")
                null
            }
        }
        release {
            isMinifyEnabled = false
            isShrinkResources = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    composeOptions {
        // Pasangan resmi untuk Kotlin 1.9.24
        kotlinCompilerExtensionVersion = "1.5.14"
    }

    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }

    testOptions {
        unitTests {
            isIncludeAndroidResources = true
            all {
                it.testLogging {
                    events("passed", "failed", "skipped")
                    exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
                    showStandardStreams = true
                }
            }
        }
    }
}

/* ---------------------------------------------------------------------
   Kotlin daemon
   ---------------------------------------------------------------------
   Daemon kompilasi Kotlin menulis penanda koneksi ke
   %LOCALAPPDATA%\kotlin\daemon, yang di sebagian mesin tidak bisa ditulis.
   Karena itu kompilasi dijalankan di dalam proses Gradle
   (kotlin.compiler.execution.strategy=in-process).
   --------------------------------------------------------------------- */
tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
    // Tidak memakai Kotlin daemon terpisah.
    if (System.getenv("KG_KOTLIN_INPROCESS") == "1") {
        logger.lifecycle("Kotlin: kompilasi in-process (KG_KOTLIN_INPROCESS=1)")
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.activity:activity-compose:1.9.0")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.2")

    implementation(platform("androidx.compose:compose-bom:2024.06.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.material3:material3")

    debugImplementation("androidx.compose.ui:ui-tooling")

    // Binding UniFFI memakai JNA untuk memanggil pustaka Rust.
    implementation("net.java.dev.jna:jna:5.14.0@aar")

    testImplementation("junit:junit:4.13.2")
    // JNA untuk uji JVM: versi @aar di atas tidak membawa jnidispatch host,
    // jadi tambahkan jar biasa (memuat jnidispatch.dll/.so/.dylib desktop).
    testImplementation("net.java.dev.jna:jna:5.14.0")
}

/* =====================================================================
   Bagian Rust
   ---------------------------------------------------------------------
   Semua logika aplikasi ada di ../kg-core (Rust). Yang dibangun di sini:

     1. `cargo build --release` untuk tiap ABI Android  -> libkg_ffi.so
     2. `uniffi-bindgen generate` dari build host       -> binding Kotlin
     3. pustaka disalin ke build/rustJniLibs, binding ke build/generated/uniffi
        (keduanya didaftarkan sebagai source dir Android di bawah)
   ===================================================================== */

val repoRoot: File = rootDir.resolve("..").canonicalFile

// Cari toolchain Rust: CARGO_HOME -> PATH -> lokasi bawaan rustup.
// CARGO_HOME kosong berarti ~/.cargo (perilaku cargo sendiri).
val cargoExe: File = run {
    val exe = if (System.getProperty("os.name").startsWith("Windows")) "cargo.exe" else "cargo"
    val fromEnv = System.getenv("CARGO_HOME")?.takeIf { it.isNotBlank() }?.let { File(it, "bin/$exe") }
    val home = File(System.getProperty("user.home"))
        .toPath().resolve(".cargo").resolve("bin").resolve(exe).toFile()
    fromEnv?.takeIf { it.isFile } ?: home.takeIf { it.isFile } ?: File(exe)
}

val rustTargets = linkedMapOf(
    "arm64-v8a" to "aarch64-linux-android",
    "armeabi-v7a" to "armv7-linux-androideabi",
    "x86_64" to "x86_64-linux-android"
)

val rustSources = fileTree(repoRoot.resolve("kg-core")) {
    include("src/**", "Cargo.toml", "tests/**")
} + fileTree(repoRoot.resolve("kg-ffi")) {
    include("src/**", "Cargo.toml", "build.rs", "uniffi-bindgen/**")
} + files(repoRoot.resolve("Cargo.toml"), repoRoot.resolve("Cargo.lock"))

/**
 * NDK dipakai `cc-rs` untuk menautkan `.so` Android.
 * Dicari lewat env, lalu folder NDK bawaan SDK, lalu `local.properties`.
 */
fun resolveNdkDir(): String? {
    System.getenv("ANDROID_NDK_HOME")?.takeIf { it.isNotBlank() }?.let { return it }
    System.getenv("ANDROID_NDK_ROOT")?.takeIf { it.isNotBlank() }?.let { return it }
    val sdk = System.getenv("ANDROID_HOME") ?: System.getenv("ANDROID_SDK_ROOT")
    if (!sdk.isNullOrBlank()) {
        val ndkRoot = File(sdk, "ndk")
        val best = ndkRoot.listFiles()
            ?.filter { it.isDirectory }
            ?.maxByOrNull { it.name }
        if (best != null) return best.absolutePath
    }
    val localProps = rootProject.file("local.properties")
    if (localProps.isFile) {
        localProps.readLines().firstOrNull { it.startsWith("ndk.dir=") }
            ?.substringAfter("ndk.dir=")?.trim()?.let { return it }
    }
    return null
}

/**
 * Task yang menjalankan `cargo` di folder repo Rust.
 * Lokasi toolchain dan folder kerja diisi lewat property (agak aman terhadap
 * perubahan versi Gradle dibanding constructor task).
 */
abstract class RustTask : DefaultTask() {
    @get:Inject
    abstract val exec: org.gradle.process.ExecOperations

    @get:Input
    abstract val cargoBin: Property<String>

    @get:Input
    abstract val rustDir: Property<String>

    @get:Internal
    var ndk: String = ""

    fun cargo(vararg args: String): String {
        // Hasil dibaca lewat file sementara: tidak perlu java.io di skrip ini.
        val outFile = project.layout.buildDirectory.file("tmp/rust-log.txt").get().asFile
        outFile.parentFile.mkdirs()
        exec.exec {
            workingDir = File(rustDir.get())
            executable = cargoBin.get()
            this.args = args.toList()
            standardOutput = outFile.outputStream()
            errorOutput = System.err
            // Jangan sampai Gradle memakai ~/.cargo milik pengguna yang mungkin
            // tidak bisa ditulis: samakan dengan folder tempat cargo.exe berada.
            val cargoHome = File(cargoBin.get()).parentFile.parentFile
            environment("CARGO_HOME", cargoHome.absolutePath)
            System.getenv("RUSTUP_HOME")?.let { environment("RUSTUP_HOME", it) }
            if (ndk.isNotEmpty()) {
                environment("ANDROID_NDK_HOME", ndk)
                environment("ANDROID_NDK_ROOT", ndk)
                // cc-rs memakai variabel ini untuk memilih linker tiap ABI.
                val prebuilt = "${ndk}/toolchains/llvm/prebuilt"
                val host = File(prebuilt).listFiles()?.firstOrNull { it.isDirectory }?.absolutePath
                if (host != null) {
                    environment("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", "$host/bin/aarch64-linux-android24-clang.cmd")
                    environment("CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER", "$host/bin/armv7a-linux-androideabi24-clang.cmd")
                    environment("CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER", "$host/bin/x86_64-linux-android24-clang.cmd")
                    environment("CC_aarch64_linux_android", "$host/bin/aarch64-linux-android24-clang.cmd")
                    environment("CC_armv7_linux_androideabi", "$host/bin/armv7a-linux-androideabi24-clang.cmd")
                    environment("CC_x86_64_linux_android", "$host/bin/x86_64-linux-android24-clang.cmd")
                    environment("AR_aarch64_linux_android", "$host/bin/llvm-ar.exe")
                    environment("AR_armv7_linux_androideabi", "$host/bin/llvm-ar.exe")
                    environment("AR_x86_64_linux_android", "$host/bin/llvm-ar.exe")
                }
            }
        }
        return outFile.readText()
    }
}

val cargoBuild = tasks.register<RustTask>("cargoBuild") {
    cargoBin.set(cargoExe.absolutePath)
    rustDir.set(repoRoot.absolutePath)
    group = "rust"
    description = "Kompilasi inti Rust (kg-ffi) untuk semua ABI Android + build host untuk bindgen."
    inputs.files(rustSources).withPropertyName("rustSources")
    outputs.dir(repoRoot.resolve("target/release"))
    doLast {
        val fromAgp = runCatching { android.ndkDirectory.absolutePath }.getOrNull()
        ndk = fromAgp ?: resolveNdkDir() ?: error(
            "NDK tidak ditemukan. Pasang lewat sdkmanager \"ndk;27.2.12479018\" " +
                "atau set ANDROID_NDK_HOME."
        )
        logger.lifecycle("NDK: $ndk")
        // 1) Target Android yang belum terpasang -> pasang lewat rustup.
        //    Hasil `list` ditulis ke file supaya tidak terpotong oleh buffer.
        val rustup = File(cargoExe.parentFile, "rustup.exe").takeIf { it.isFile }
            ?: File(cargoExe.parentFile, "rustup").takeIf { it.isFile }
        val listFile = layout.buildDirectory.file("tmp/rustup-targets.txt").get().asFile
        listFile.parentFile.mkdirs()
        val installed = if (rustup == null) "" else runCatching {
            providers.exec {
                executable = rustup.absolutePath
                args = listOf("target", "list", "--installed")
                standardOutput = listFile.outputStream()
            }.result.get()
            listFile.readText().trim()
        }.getOrDefault("")
        val installedSet = installed.lines().map { it.trim() }.filter { it.isNotEmpty() }.toSet()
        for ((_, triple) in rustTargets) {
            if (triple !in installedSet) {
                logger.lifecycle("rustup target add $triple")
                if (rustup == null) error("rustup tidak ditemukan di samping cargo.exe ($triple belum terpasang)")
                providers.exec {
                    executable = rustup.absolutePath
                    args = listOf("target", "add", triple)
                }.result.get()
            }
            logger.lifecycle("cargo build --release --target $triple -p kg-ffi")
            cargo("build", "--release", "--target", triple, "-p", "kg-ffi")
        }
        // 2) Build host supaya uniffi-bindgen bisa membaca metadata dari pustaka.
        logger.lifecycle("cargo build --release -p kg-ffi (host, untuk bindgen)")
        cargo("build", "--release", "-p", "kg-ffi")
    }
}

val generateUniffiBindings = tasks.register<RustTask>("generateUniffiBindings") {
    cargoBin.set(cargoExe.absolutePath)
    rustDir.set(repoRoot.absolutePath)
    group = "rust"
    description = "Bangkitkan binding Kotlin UniFFI dari pustaka Rust."
    dependsOn(cargoBuild)
    val outDir = layout.buildDirectory.dir("generated/uniffi")
    outputs.dir(outDir)
    doLast {
        val hostLib = when {
            System.getProperty("os.name").startsWith("Windows") -> repoRoot.resolve("target/release/kg_ffi.dll")
            System.getProperty("os.name").startsWith("Mac") -> repoRoot.resolve("target/release/libkg_ffi.dylib")
            else -> repoRoot.resolve("target/release/libkg_ffi.so")
        }
        logger.lifecycle("uniffi-bindgen generate dari ${hostLib.name}")
        cargo(
            "run", "--quiet", "-p", "uniffi-bindgen", "--",
            "generate", "--library", hostLib.absolutePath,
            "--language", "kotlin",
            "--out-dir", outDir.get().asFile.absolutePath,
            "--no-format"
        )
    }
}

val syncRustLibs = tasks.register<Copy>("syncRustLibs") {
    group = "rust"
    description = "Salin libkg_ffi.so hasil build Rust ke folder jniLibs."
    dependsOn(cargoBuild)
    val dest = layout.buildDirectory.dir("rustJniLibs")
    into(dest)
    for ((abi, triple) in rustTargets) {
        from(repoRoot.resolve("target/$triple/release")) {
            include("libkg_ffi.so")
            into(abi)
        }
    }
    outputs.dir(dest)
}

android.sourceSets.getByName("main") {
    jniLibs.srcDir(layout.buildDirectory.dir("rustJniLibs"))
    java.srcDir(layout.buildDirectory.dir("generated/uniffi"))
}

tasks.named("preBuild").configure { dependsOn(syncRustLibs, generateUniffiBindings) }

/* ---------------------------------------------------------------------
   Uji unit JVM (app/src/test)
   ---------------------------------------------------------------------
   Binding UniFFI memuat pustaka host lewat JNA, jadi tes JVM butuh
   kg_ffi.dll / libkg_ffi.so hasil build host. Folder itu dijadikan
   jniLibs untuk varian unit test agar ikut masuk ke java.library.path.
   --------------------------------------------------------------------- */
val hostLibName = when {
    System.getProperty("os.name").startsWith("Windows") -> "kg_ffi.dll"
    System.getProperty("os.name").startsWith("Mac") -> "libkg_ffi.dylib"
    else -> "libkg_ffi.so"
}

val hostRustLibs = tasks.register<Copy>("hostRustLibs") {
    group = "rust"
    description = "Salin pustaka Rust hasil build host ke folder uji JVM."
    dependsOn(cargoBuild)
    val dest = layout.buildDirectory.dir("rustHostLibs")
    into(dest)
    from(repoRoot.resolve("target/release")) { include(hostLibName) }
    outputs.dir(dest)
}

tasks.withType<Test>().configureEach {
    dependsOn(hostRustLibs, generateUniffiBindings)
    val libDir = layout.buildDirectory.dir("rustHostLibs").get().asFile
    // JNA mencari pustaka di properti ini saat tes berjalan.
    doFirst { libDir.mkdirs() }
    systemProperty("jna.library.path", libDir.absolutePath)
    systemProperty("java.library.path", libDir.absolutePath)
    // Pastikan folder berisi kg_ffi terpasang sebelum worker tes dimulai.
    environment("KG_RUST_LIB_DIR", libDir.absolutePath)
}
