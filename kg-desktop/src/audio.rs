//! Pelafalan Korea memakai mesin suara bawaan sistem operasi.
//!
//! Ini pengganti `TextToSpeech` (Android) dan Web Speech API (web). Yang
//! dibacakan bukan teks mentah: aplikasi meminta Rust merapikannya lebih dulu
//! lewat `kg_ffi::tts_text()` — aturan yang sama persis dengan Android.
//!
//! Bila mesin suara tidak ada, [`Speaker::available`] bernilai `false` dan
//! tombol 🔊 tidak ditampilkan. Tidak ada error yang sampai ke pengguna.

use std::process::{Command, Stdio};

pub struct Speaker {
    /// Perintah yang terbukti ada di mesin ini.
    backend: Option<Backend>,
    /// Suara Korea benar-benar terpasang (bukan sekadar perintahnya ada).
    korean: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Backend {
    /// Windows: PowerShell + System.Speech (SAPI).
    WindowsSapi,
    /// macOS: `say`.
    MacSay,
    /// Linux: `espeak-ng` / `espeak`.
    Espeak(&'static str),
}

impl Speaker {
    /// Deteksi sekali saat aplikasi dibuka.
    pub fn detect() -> Self {
        let backend = detect_backend();
        let korean = match backend {
            Some(Backend::WindowsSapi) => windows_has_korean(),
            Some(Backend::MacSay) => mac_has_korean(),
            Some(Backend::Espeak(_)) => true, // espeak membunyikan Hangul apa adanya
            None => false,
        };
        Speaker { backend, korean }
    }

    /// Speaker tanpa mesin suara (untuk pengujian).
    pub fn silent() -> Self {
        Speaker { backend: None, korean: false }
    }

    /// Tombol 🔊 hanya muncul bila ini `true`.
    pub fn available(&self) -> bool {
        self.backend.is_some() && self.korean
    }

    /// Bacakan satu kalimat (sudah dirapikan Rust). Diam bila tidak tersedia.
    pub fn say(&self, text: &str) -> bool {
        if !self.available() || text.trim().is_empty() {
            return false;
        }
        match self.backend {
            Some(Backend::WindowsSapi) => {
                // SpeechSynthesizer dimuat lewat PowerShell; tidak butuh paket.
                let script = "Add-Type -AssemblyName System.Speech; \
                     $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
                     $s.Rate = -1; \
                     [Console]::InputEncoding = [Text.Encoding]::UTF8; \
                     $t = [Console]::In.ReadToEnd(); \
                     $s.Speak($t); $s.Dispose()";
                spawn(
                    "powershell",
                    &[
                        "-NoProfile",
                        "-NonInteractive",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-Command",
                        script,
                    ],
                    Some(text),
                )
            }
            Some(Backend::MacSay) => spawn("say", &["-v", "Yuna", text], None),
            Some(Backend::Espeak(name)) => spawn(name, &["-v", "ko", text], None),
            None => false,
        }
    }
}

/// Jalankan perintah tanpa menunggu selesai, tanpa jendela konsol di Windows.
fn spawn(exe: &str, args: &[&str], stdin_text: Option<&str>) -> bool {
    let mut cmd = Command::new(exe);
    cmd.args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(if stdin_text.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: jangan kedipkan jendela konsol.
        cmd.creation_flags(0x0800_0000);
    }

    match cmd.spawn() {
        Ok(mut child) => {
            if let Some(text) = stdin_text {
                use std::io::Write;
                if let Some(mut si) = child.stdin.take() {
                    let _ = si.write_all(text.as_bytes());
                    // tutup stdin agar ReadToEnd() di PowerShell selesai
                }
            }
            true
        }
        Err(_) => false,
    }
}

fn detect_backend() -> Option<Backend> {
    if cfg!(windows) {
        return if command_exists("powershell") {
            Some(Backend::WindowsSapi)
        } else {
            None
        };
    }
    if cfg!(target_os = "macos") {
        return if command_exists("say") { Some(Backend::MacSay) } else { None };
    }
    if command_exists("espeak-ng") {
        Some(Backend::Espeak("espeak-ng"))
    } else if command_exists("espeak") {
        Some(Backend::Espeak("espeak"))
    } else {
        None
    }
}

fn command_exists(name: &str) -> bool {
    let probe = if cfg!(windows) { "where" } else { "which" };
    Command::new(probe)
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Cek apakah Windows punya voice Korea (mis. "Microsoft Heami").
fn windows_has_korean() -> bool {
    let script = "Add-Type -AssemblyName System.Speech; \
         $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
         $ok = ($s.GetInstalledVoices() | Where-Object { $_.VoiceInfo.Culture.Name -like 'ko*' }).Count -gt 0; \
         $s.Dispose(); if ($ok) { 'KR' }";
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains("KR"),
        Err(_) => false,
    }
}

fn mac_has_korean() -> bool {
    let out = Command::new("say").arg("-v").arg("?").stdout(Stdio::piped()).stderr(Stdio::null()).output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_lowercase().contains("ko_kr"),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_speaker_is_never_available() {
        let s = Speaker::silent();
        assert!(!s.available());
        // memanggil say() pada speaker bisu harus aman dan mengembalikan false
        assert!(!s.say("안녕하세요"));
    }

    #[test]
    fn empty_text_is_ignored() {
        let s = Speaker::detect();
        assert!(!s.say(""), "teks kosong tidak dibacakan");
        assert!(!s.say("   "), "spasi saja juga diabaikan");
    }
}
