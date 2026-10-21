//! Tipe dasar bank materi — cermin struktur `window.KG_TOPICS` di `js/data.js`.
//!
//! Semua tipe di sini `Copy` dan berupa data statis (`&'static str`, slice),
//! jadi bisa dikembalikan ke Kotlin lewat FFI tanpa alokasi.

/// Satu baris tabel aturan (`{ a, b, c }` di data web).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    /// Kolom kiri — biasanya huruf/partikel, ditebalkan.
    pub a: &'static str,
    /// Kolom tengah — keterangan bunyi atau aturan.
    pub b: &'static str,
    /// Kolom kanan — huruf Korea atau contoh.
    pub c: &'static str,
}

/// Satu contoh kalimat (`{ ko, rom, id, why }`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Example {
    /// Kalimat Korea.
    pub ko: &'static str,
    /// Romanisasi.
    pub rom: &'static str,
    /// Terjemahan Indonesia.
    pub id: &'static str,
    /// Kenapa bentuknya begitu (opsional).
    pub why: &'static str,
}

/// Satu blok isi materi. Padanan `renderBlock()` di `js/app.js`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Block {
    /// Sub-judul bagian.
    Heading { text: &'static str },
    /// Paragraf biasa.
    Text { text: &'static str },
    /// Rumus pola kalimat: `parts` digabung dengan tanda `+`.
    Formula { parts: &'static [&'static str] },
    /// Tabel aturan tiga kolom.
    Rules { items: &'static [Rule] },
    /// Daftar contoh kalimat (dengan tombol pelafalan).
    Examples { items: &'static [Example] },
    /// Kotak "Catatan:".
    Note { text: &'static str },
}

/// Satu soal pilihan ganda. Selalu 4 pilihan; `answer` adalah indeks yang benar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Question {
    pub q: &'static str,
    pub options: [&'static str; 4],
    pub answer: u8,
    pub why: &'static str,
}

/// Satu soal latihan menulis (`{ ko, rom, id }`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exercise {
    /// Jawaban yang diharapkan.
    pub ko: &'static str,
    /// Romanisasi, ditampilkan saat "Tampilkan jawaban".
    pub rom: &'static str,
    /// Arti dalam bahasa Indonesia — petunjuk soal.
    pub id: &'static str,
}

/// Satu topik materi beserta kuis dan latihannya.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Topic {
    /// Kunci stabil, dipakai untuk progres (mis. `"hangeul"`).
    pub id: &'static str,
    /// Nomor urut yang ditampilkan (1–10).
    pub no: u32,
    /// Judul Korea, mis. `"한글 — Huruf Korea"`.
    pub title: &'static str,
    /// Judul bahasa Indonesia, mis. `"Hangeul: dasar membaca"`.
    pub title_id: &'static str,
    /// 1 = Dasar, 2 = Menengah, 3 = Lanjutan.
    pub level: u8,
    /// Huruf ikon, mis. `"한"`.
    pub icon: &'static str,
    /// Pengantar singkat topik.
    pub intro: &'static str,
    pub blocks: &'static [Block],
    pub quiz: &'static [Question],
    pub lat: &'static [Exercise],
}

impl Topic {
    /// Jumlah blok yang akan dirender (dipakai uji alur).
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Jumlah soal kuis.
    pub fn quiz_len(&self) -> usize {
        self.quiz.len()
    }

    /// Jumlah soal latihan menulis.
    pub fn lat_len(&self) -> usize {
        self.lat.len()
    }
}

/// Nama level dalam bahasa Indonesia (`LEVELS` di `js/app.js`).
pub fn level_name(level: u8) -> &'static str {
    match level {
        2 => "Menengah",
        3 => "Lanjutan",
        _ => "Dasar",
    }
}

/// Huruf pilihan A–D (`LETTERS` di `js/app.js`).
pub fn option_letter(index: usize) -> &'static str {
    match index {
        0 => "A",
        1 => "B",
        2 => "C",
        3 => "D",
        _ => "",
    }
}
