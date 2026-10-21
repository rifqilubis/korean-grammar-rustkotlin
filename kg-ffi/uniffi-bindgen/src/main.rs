//! Pembangkit binding Kotlin: `cargo run -p uniffi-bindgen -- generate …`.
//!
//! Versi UniFFI di sini sengaja dikunci sama dengan yang dipakai `kg-ffi`,
//! supaya binding yang dihasilkan selalu cocok dengan pustaka aslinya
//! (tidak bergantung pada paket `uniffi-bindgen` global yang versinya bisa beda).

fn main() {
    uniffi::uniffi_bindgen_main();
}
