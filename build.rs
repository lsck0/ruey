use std::{fs, process::Command};

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let dotenv = fs::read_to_string(".env").expect("Failed to read .env");

    // make env variables available to the compiler to embed
    for line in dotenv.lines() {
        if let Some((key, value)) = line.split_once('=') {
            println!("cargo:rustc-env={}={}", key, value);
        }
    }

    // add icon to the windows executable
    if target_os == "windows" {
        println!("cargo:rerun-if-changed=./assets/icon.rc");
        println!("cargo:rerun-if-changed=./assets/icon.ico");

        Command::new("x86_64-w64-mingw32-windres")
            .args(["./assets/icon.rc", "-O", "coff", "-o", "./assets/icon.res"])
            .status()
            .expect("failed to run windres");

        println!("cargo:rustc-link-arg=./assets/icon.res");
    }
}
