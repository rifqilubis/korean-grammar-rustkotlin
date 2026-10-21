fn main() {
    // `#[uniffi::export]` menghasilkan kode pembungkus secara otomatis;
    // baris ini menyiapkan tabel FFI + pengecekan versi kontrak UniFFI.
    uniffi::generate_scaffolding("src/kg.udl").expect("gagal membuat scaffolding UniFFI");
}
