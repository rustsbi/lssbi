// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=kernel/sbi_probe.c");
    println!("cargo:rerun-if-changed=kernel/Makefile");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    link_runtime_library(&out_dir, "pam");
    link_runtime_library(&out_dir, "pam_misc");
    println!("cargo:rustc-link-search=native={}", out_dir.display());

    let release = Command::new("uname")
        .arg("-r")
        .output()
        .expect("failed to run uname -r");
    assert!(release.status.success(), "uname -r failed");
    let release = String::from_utf8(release.stdout)
        .expect("uname output was not UTF-8")
        .trim()
        .to_owned();

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let kernel_dir = manifest_dir.join("kernel");
    let kernel_build = format!("/lib/modules/{release}/build");

    let status = Command::new("/usr/bin/make")
        .arg("-C")
        .arg(kernel_build)
        .arg(format!("M={}", kernel_dir.display()))
        .arg("modules")
        .status()
        .expect("failed to invoke the kernel module build");
    assert!(status.success(), "kernel module build failed");
}

fn link_runtime_library(out_dir: &Path, name: &str) {
    let soname = format!("lib{name}.so.0");
    let output = Command::new("cc")
        .arg(format!("-print-file-name={soname}"))
        .output()
        .unwrap_or_else(|error| panic!("failed to locate {soname}: {error}"));
    assert!(output.status.success(), "failed to locate {soname}");

    let source = String::from_utf8(output.stdout).expect("compiler path was not UTF-8");
    let source = PathBuf::from(source.trim());
    assert!(
        source.is_absolute() && source.is_file(),
        "{soname} is not installed"
    );

    let link = out_dir.join(format!("lib{name}.so"));
    let _ = fs::remove_file(&link);
    symlink(source, link).unwrap_or_else(|error| panic!("failed to link {soname}: {error}"));
}
