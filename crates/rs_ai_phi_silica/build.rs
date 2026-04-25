#![allow(clippy::disallowed_methods)]

use std::process::Command;

fn main() {
    // Only build the C# bridge on Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let csproj = format!("{manifest_dir}/csharp/PhiSilicaNative.csproj");
    let lib_path = format!("{out_dir}/PhiSilicaNative.dll");

    println!("cargo:rerun-if-changed=csharp/PhiSilicaNative.cs");
    println!("cargo:rerun-if-changed=csharp/PhiSilicaNative.csproj");
    println!("cargo::rustc-check-cfg=cfg(phi_silica_bridge)");

    // Check if dotnet is available
    let dotnet_check = Command::new("dotnet").arg("--version").output();
    if dotnet_check.is_err() {
        println!(
            "cargo::warning=rs_ai_phi_silica: dotnet SDK not found. \
             C# bridge will not be available. Use a custom PhiSilicaBridge implementation."
        );
        return;
    }

    // Publish the C# project with Native AOT
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
                    "cargo::warning=rs_ai_phi_silica: C# bridge compilation succeeded but \
                     library not found at {lib_path}. Use a custom PhiSilicaBridge implementation."
                );
            }
        }
        Ok(_) => {
            println!(
                "cargo::warning=rs_ai_phi_silica: C# bridge compilation failed. \
                 This typically means the Windows App SDK is not installed. \
                 Use a custom PhiSilicaBridge implementation."
            );
        }
        Err(e) => {
            println!(
                "cargo::warning=rs_ai_phi_silica: Failed to run dotnet: {e}. \
             Use a custom PhiSilicaBridge implementation."
            );
        }
    }
}
