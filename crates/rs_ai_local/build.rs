#![allow(clippy::disallowed_methods)]

use std::process::Command;

fn main() {
    // --- Foundation Models (macOS) ---
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "macos" {
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
                    "cargo::warning=rs_ai_local: Swift bridge compilation failed \
                     (requires Xcode with macOS 26+ SDK). All APIs will return \
                     Err(Error::Unavailable) at runtime."
                );
            }
            Err(e) => {
                println!(
                    "cargo::warning=rs_ai_local: xcrun not found, \
                     skipping Swift bridge: {e}"
                );
            }
        }
    }

    // --- Phi Silica (Windows) ---
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let csproj = format!("{manifest_dir}/csharp/PhiSilicaNative.csproj");
        let lib_path = format!("{out_dir}/PhiSilicaNative.dll");

        println!("cargo:rerun-if-changed=csharp/PhiSilicaNative.cs");
        println!("cargo:rerun-if-changed=csharp/PhiSilicaNative.csproj");
        println!("cargo::rustc-check-cfg=cfg(phi_silica_bridge)");

        let dotnet_check = Command::new("dotnet").arg("--version").output();
        if dotnet_check.is_err() {
            println!(
                "cargo::warning=rs_ai_local: dotnet SDK not found. \
                 C# bridge will not be available."
            );
            return;
        }

        let status = Command::new("dotnet")
            .args([
                "publish",
                &csproj,
                "-c",
                "Release",
                "-r",
                "win-x64",
                "--self-contained",
                "true",
                "-p:PublishAot=true",
                "-p:PublishSingleFile=false",
                "-o",
                &out_dir,
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                if std::path::Path::new(&lib_path).exists() {
                    println!("cargo:rustc-cfg=phi_silica_bridge");
                    println!("cargo:rustc-link-search=native={out_dir}");
                    println!("cargo:rustc-link-lib=dylib=PhiSilicaNative");
                } else {
                    println!(
                        "cargo::warning=rs_ai_local: C# bridge compilation succeeded but \
                         library not found at {lib_path}."
                    );
                }
            }
            Ok(_) => {
                println!(
                    "cargo::warning=rs_ai_local: C# bridge compilation failed. \
                     This typically means the Windows App SDK is not installed."
                );
            }
            Err(e) => {
                println!("cargo::warning=rs_ai_local: Failed to run dotnet: {e}.");
            }
        }
    }
}
