fn main() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let contract_src = manifest_dir.join("../contract/src");
    println!("cargo::rerun-if-changed={}", contract_src.display());
    let source = vastrum_abi_codegen::generate(&contract_src)
        .expect("failed to generate ABI code from contract source");
    let generated = manifest_dir.join("src/generated.rs");
    let current = std::fs::read_to_string(&generated).unwrap_or_default();
    if current != source {
        std::fs::write(&generated, &source).unwrap();
    }
}
