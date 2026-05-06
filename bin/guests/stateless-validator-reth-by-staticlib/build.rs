use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    let host = env::var("HOST").unwrap_or_default();

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let zisk_dir = PathBuf::from(&manifest_dir)
        .join("../../../../zisk")
        .canonicalize()
        .expect("zisk directory not found at ../../../../zisk");

    // Re-run when RUSTFLAGS change so cfg flags (e.g. --cfg zisk_hints) propagate to the library.
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");

    // Emit a unified cfg for both ZisK guest targets so source can use #[cfg(zisk_guest)]
    // instead of repeating the full target tuple condition everywhere.
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or_default();
    println!("cargo:rustc-check-cfg=cfg(zisk_guest)");
    if (target_os == "zkvm" && target_vendor == "zisk")
        || (target_arch == "riscv64" && target_os == "none")
    {
        println!("cargo:rustc-cfg=zisk_guest");
    }

    let mut cmd = Command::new("cargo");

    const RISCV_BARE: &str = "riscv64im-unknown-none-elf";

    let lib_dir = if target.contains("zisk") {
        let target_triple = "riscv64ima-zisk-zkvm-elf";
        cmd.args([
            "+zisk",
            "build",
            "-p",
            "ziskos-staticlib",
            "--release",
            "--target",
            target_triple,
            "--config",
            r#"profile.release.lto="fat""#,
        ]);
        zisk_dir.join(format!("target/{target_triple}/release"))
    } else if target == RISCV_BARE {
        // Build ziskos-staticlib for the bare RISC-V target using the custom target spec.
        // The JSON spec lowers atomics via -C passes=lower-atomic (required by the stateless
        // crate's use of atomics, which have no native support on RV64IM bare metal).
        let target_spec = PathBuf::from(&manifest_dir)
            .join("riscv64im-unknown-none-elf.json")
            .canonicalize()
            .expect("riscv64im target spec not found at riscv64im-unknown-none-elf.json");
        cmd.args([
            "+nightly",
            "build",
            "-p",
            "ziskos-staticlib",
            "--release",
            "--target",
            target_spec.to_str().unwrap(),
            "-Zbuild-std=core,alloc",
            "-Zjson-target-spec",
        ]);
        zisk_dir.join(format!("target/{RISCV_BARE}/release"))
    } else {
        cmd.args(["build", "-p", "ziskos-staticlib", "--release", "--target", &host]);
        zisk_dir.join(format!("target/{host}/release"))
    };

    cmd.current_dir(&zisk_dir);

    // Forward RUSTFLAGS so cfg flags like --cfg zisk_hints are applied to the library.
    // Cargo sets CARGO_ENCODED_RUSTFLAGS (unit separator 0x1F) in build scripts; plain
    // RUSTFLAGS is NOT propagated by cargo to build scripts.
    let encoded_flags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    if !encoded_flags.is_empty() {
        let rustflags = encoded_flags.split('\x1f').collect::<Vec<_>>().join(" ");
        cmd.env("RUSTFLAGS", rustflags);
    }

    let status = cmd.status().expect("failed to spawn cargo build");
    assert!(status.success(), "cargo build -p ziskos-staticlib failed");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=ziskos_staticlib");

    if target == RISCV_BARE {
        // Pass the linker script that defines memory layout and heap/stack symbols.
        let ld = PathBuf::from(&manifest_dir).join("link.x");
        println!("cargo:rerun-if-changed=link.x");
        println!("cargo:rustc-link-arg=-T{}", ld.display());
        // libziskos_staticlib.a (built with -Zbuild-std=core,alloc) may define alloc symbols
        // that duplicate those from the binary's own -Zbuild-std alloc. Allow overlap.
        // rust-lld is invoked directly (not through cc), so the flag needs no -Wl, wrapper.
        println!("cargo:rustc-link-arg=--allow-multiple-definition");
    } else if !target.contains("zisk") {
        // libziskos_staticlib.a references libziskc.a (the C/C++ elliptic-curve implementations
        // from the zisk lib-c crate). Link it and its system dependencies explicitly, since
        // they are not propagated through the external staticlib.
        let lib_c_dir = zisk_dir.join("lib-c/c/lib");
        println!("cargo:rustc-link-search=native={}", lib_c_dir.display());
        println!("cargo:rustc-link-lib=static=ziskc");
        for lib in &["pthread", "gmp", "gmpxx", "stdc++"] {
            println!("cargo:rustc-link-lib={lib}");
        }
        // With user-hints, libziskos_staticlib.a pulls in proofman which uses OpenMP, MPI,
        // and libsodium (for randombytes_buf in SNARK proof generation).
        if env::var("CARGO_CFG_ZISK_HINTS").is_ok() {
            for lib in &["gomp", "mpi", "sodium"] {
                println!("cargo:rustc-link-lib={lib}");
            }
        }
        // libziskos_staticlib.a bundles std (ziskos uses std on non-zkvm targets).
        // Allow duplicate std symbols; the binary's own copy takes precedence.
        // Use -Wl, to pass the flag through the cc linker driver to the actual linker.
        println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    } // end else if !target.contains("zisk")
}
