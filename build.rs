use std::{env, fs, path::PathBuf};
fn main() {
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("tools/presentmon/PresentMon.exe");
    println!("cargo:rerun-if-changed={}", source.display());
    let path = if source.exists() {
        source
    } else {
        let empty = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("presentmon-empty.bin");
        fs::write(&empty, []).unwrap();
        empty
    };
    println!("cargo:rustc-env=SELA_PRESENTMON_BUNDLE={}", path.display());
}
