// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=po/LINGUAS");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    compile_translations(&manifest_dir, &out_dir);
}

fn compile_translations(manifest_dir: &Path, out_dir: &Path) {
    let po_dir = manifest_dir.join("po");
    let linguas = fs::read_to_string(po_dir.join("LINGUAS")).expect("failed to read po/LINGUAS");
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("OUT_DIR does not contain a Cargo profile directory");

    for language in linguas.lines().map(str::trim) {
        if language.is_empty() || language.starts_with('#') {
            continue;
        }

        let input = po_dir.join(format!("{language}.po"));
        let output = profile_dir
            .join("locale")
            .join(language)
            .join("LC_MESSAGES")
            .join("lssbi.mo");
        println!("cargo:rerun-if-changed={}", input.display());
        fs::create_dir_all(output.parent().unwrap()).expect("failed to create locale directory");
        polib::mo_file::compile_from_po(&input, &output)
            .unwrap_or_else(|error| panic!("failed to compile {}: {error}", input.display()));
    }
}
