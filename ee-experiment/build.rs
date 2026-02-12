//! SPDX-License-Identifier: MIT
//!
//! Copyright (c) 2021–2024 The rp-rs Developers
//! Copyright (c) 2021 rp-rs organization
//! Copyright (c) 2025 Raspberry Pi Ltd.
//!
//! Set up linker scripts

use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::PathBuf;

use regex::Regex;

fn main() {
    // If building for the host (not cross-compiling), set a cfg flag for host testing
    let target = std::env::var("TARGET").unwrap_or_default();
    let host = std::env::var("HOST").unwrap_or_default();
    if target == host {
        println!("cargo:rustc-cfg=host_test");
    }

    println!("cargo::rustc-check-cfg=cfg(rp2350)");

    // Put the linker script somewhere the linker can find it
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=.pico-rs");
    let contents = read_to_string(".pico-rs")
        .map(|s| s.trim().to_string().to_lowercase())
        .unwrap_or_else(|e| {
            eprintln!("Failed to read file: {}", e);
            String::new()
        });

    // The file `memory.x` is loaded by cortex-m-rt's `link.x` script, which
    // is what we specify in `.cargo/config.toml` for Arm builds
    let target_arch;

    if contents.contains("riscv") {
        target_arch = "riscv32imac-unknown-none-elf";
    } else {
        target_arch = "thumbv8m.main-none-eabihf";
    }
    let memory_x = include_bytes!("rp2350.x");
    let mut f = File::create(out.join("memory.x")).unwrap();
    f.write_all(memory_x).unwrap();
    println!("cargo::rustc-cfg=rp2350");
    println!("cargo:rerun-if-changed=rp2350.x");

    let re = Regex::new(r"target = .*").unwrap();
    let config_toml = include_str!(".cargo/config.toml");
    let result = re.replace(config_toml, format!("target = \"{}\"", target_arch));
    let mut f = File::create(".cargo/config.toml").unwrap();
    f.write_all(result.as_bytes()).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
