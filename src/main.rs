// SPDX-License-Identifier: MIT OR MulanPSL-2.0

#![cfg_attr(not(target_arch = "riscv64"), allow(dead_code))]

#[cfg(not(target_arch = "riscv64"))]
compile_error!("lssbi is intentionally restricted to riscv64 Linux");

mod driver;
mod fwft;
mod marchid;
mod mvendorid;
mod sbi_ext;
mod sbi_impl;
mod vuln;

use clap::Parser;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    help_template = "{before-help}{name} {version}\n{author-with-newline}{about-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}"
)]
struct Cli {
    /// Probe deprecated legacy SBI extensions.
    #[arg(long)]
    legacy: bool,
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = driver::run(cli.legacy) {
        eprintln!("lssbi: {error}");
        std::process::exit(1);
    }
}
