//! Copies `memory.x` into OUT_DIR so the linker finds it, and rebuilds when it changes.
//! The linker scripts themselves (`link.x`, `defmt.x`) are selected in `.cargo/config.toml`.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("memory.x"), include_bytes!("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
}
