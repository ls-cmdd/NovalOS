use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/ap_trampoline.asm");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR missing"));
    let output = out_dir.join("ap_trampoline.bin");
    let mut nasm = Command::new("nasm");
    nasm.args(["-f", "bin"]);
    if env::var_os("CARGO_FEATURE_SMP_LOW_HANDSHAKE").is_some() {
        nasm.arg("-dLOW_HANDSHAKE");
    }
    let status = nasm
        .arg("src/ap_trampoline.asm")
        .arg("-o")
        .arg(&output)
        .status()
        .expect("failed to invoke nasm for AP trampoline");
    assert!(status.success(), "nasm failed to assemble AP trampoline");
    println!("cargo:rustc-env=AP_TRAMPOLINE_BIN={}", output.display());
}
