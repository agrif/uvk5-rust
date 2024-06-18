//! This build script will automatically build the bins of this crate,
//! with the feature flag "intern-compile" set, in order to include
//! those compiled bins in the crate lib.
//!
//! This calls both cargo (via $CARGO) and llvm-objcopy (via llvm_tools).
//!
//! The bins are compiled with the linker scripts link.x and memory.x,
//! and they are compiled as position-independent code.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const BINS: &[&str] = &["dp32g030-hal-flash"];
const TARGETDIR: &str = "dp32g030-hal-flash-target";
const TARGET: &str = "thumbv6m-none-eabi";

fn set_linker_script(out: &Path) {
    // Put `link.x` in our output directory and ensure it's
    // on the linker search path.
    File::create(out.join("link.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .unwrap();
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `link.x`
    // here, we ensure the build script is only re-run when
    // `link.x` is changed.
    println!("cargo:rerun-if-changed=link.x");
    println!("cargo:rerun-if-changed=memory.x");

    // Set the linker script to the one provided.
    println!("cargo:rustc-link-arg=-Tlink.x");
}

fn make_dummy_bins(out: &Path) {
    for bin in BINS {
        File::create(out.join(format!("{bin}.bin")))
            .unwrap()
            .write_all(b"")
            .unwrap();
    }
}

fn flatten(objcopy: &Path, src: &Path, dest: &Path) {
    Command::new(objcopy)
        .arg("-O")
        .arg("binary")
        .arg(src)
        .arg(dest)
        .status()
        .unwrap()
        .success()
        .then_some(())
        .unwrap();
}

fn build_and_flatten_bins(out: &Path) {
    let cargo = env::var_os("CARGO").unwrap();
    let llvm = llvm_tools::LlvmTools::new().expect("failed to get llvm tools");
    let objcopy = llvm
        .tool(&llvm_tools::exe("llvm-objcopy"))
        .expect("llvm-objcopy not found");

    // I would love to forward feature flags here, but there does not
    // appear to be any way to grab a comma-seperated list of them.
    // I would have to manually build such a list... and the bins don't
    // yet use any features.
    //
    // If you add this capability, change the CI script to also build
    // with features.
    let targetdir = out.join(TARGETDIR);
    Command::new(cargo)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .arg("build")
        .arg("--release")
        .arg("--bins")
        .arg("--features")
        .arg("intern-compile")
        .arg("--target")
        .arg(TARGET)
        .arg("--config")
        .arg("build.rustflags = \"-C relocation-model=pic\"")
        .arg("--target-dir")
        .arg(&targetdir)
        .status()
        .unwrap()
        .success()
        .then_some(())
        .unwrap();

    // copy the bins into OUT_DIR
    for bin in BINS {
        let inpath = targetdir.join(TARGET).join("release").join(bin);
        let outpath = out.join(format!("{bin}.bin"));
        flatten(&objcopy, &inpath, &outpath);
    }
}

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    if cfg!(feature = "intern-compile") {
        // we are compiling the bins
        make_dummy_bins(out);
        set_linker_script(out);
    } else {
        // we are compiling the lib. shell out to compile the bins
        build_and_flatten_bins(out);
    }
}
