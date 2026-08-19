// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=kernel/sbi_probe.c");
    println!("cargo:rerun-if-changed=kernel/Makefile");

    // The target only ships versioned PAM runtime libraries, without the
    // unversioned development symlinks normally consumed by -lpam.
    println!("cargo:rustc-link-arg=-Wl,-l:libpam.so.0");
    println!("cargo:rustc-link-arg=-Wl,-l:libpam_misc.so.0");

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
