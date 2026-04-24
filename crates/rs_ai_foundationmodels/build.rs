#![allow(clippy::disallowed_methods)]

use std::process::Command;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "macos" {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let swift_file = format!("{manifest_dir}/bridge.swift");
    let lib_path = format!("{out_dir}/librusty_fm_bridge.a");

    println!("cargo:rerun-if-changed=bridge.swift");
    println!("cargo::rustc-check-cfg=cfg(foundation_models_bridge)");

    let status = Command::new("xcrun")
        .args([
            "swiftc",
            "-emit-library",
            "-static",
            "-parse-as-library",
            "-module-name",
            "RustyFMBridge",
            "-target",
            "arm64-apple-macos15.0",
            "-o",
            &lib_path,
            &swift_file,
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:rustc-cfg=foundation_models_bridge");
            println!("cargo:rustc-link-search=native={out_dir}");
            println!("cargo:rustc-link-lib=static=rusty_fm_bridge");
            println!("cargo:rustc-link-arg=-Wl,-weak_framework,FoundationModels");
            println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
        }
        Ok(_) => {
            println!(
                "cargo::warning=rs_ai_foundationmodels: Swift bridge compilation failed \
                 (requires Xcode with macOS 26+ SDK). All APIs will return \
                 Err(Error::Unavailable) at runtime."
            );
        }
        Err(e) => {
            println!(
                "cargo::warning=rs_ai_foundationmodels: xcrun not found, \
                 skipping Swift bridge: {e}"
            );
        }
    }
}
