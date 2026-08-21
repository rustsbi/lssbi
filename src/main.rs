// SPDX-License-Identifier: MIT OR MulanPSL-2.0

#![cfg_attr(
    not(all(target_arch = "riscv64", target_os = "linux")),
    allow(dead_code)
)]

#[cfg(all(not(test), not(all(target_arch = "riscv64", target_os = "linux"))))]
compile_error!("lssbi is intentionally restricted to riscv64 Linux");

mod backend;
#[cfg(all(target_arch = "riscv64", target_os = "linux"))]
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
    /// Run live per-hart probes on Linux CPU N.
    #[arg(long, value_name = "N", allow_negative_numbers = true)]
    cpu: Option<String>,

    /// Probe deprecated legacy SBI extensions.
    #[arg(long)]
    legacy: bool,
}

#[cfg(all(target_arch = "riscv64", target_os = "linux"))]
fn main() {
    let cli = Cli::parse();
    if let Err(error) = driver::run(cli.legacy, cli.cpu.as_deref()) {
        eprintln!("lssbi: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::{CommandFactory, Parser};

    #[test]
    fn parses_cpu_selection() {
        let cli = Cli::try_parse_from(["lssbi", "--cpu", "3"]).unwrap();
        assert_eq!(cli.cpu.as_deref(), Some("3"));
    }

    #[test]
    fn defers_invalid_cpu_values_until_localized_validation() {
        for value in ["-1", "first"] {
            let cli = Cli::try_parse_from(["lssbi", "--cpu", value]).unwrap();
            assert_eq!(cli.cpu.as_deref(), Some(value));
        }
    }

    #[test]
    fn cli_version_matches_package_version() {
        assert_eq!(
            Cli::command().get_version(),
            Some(env!("CARGO_PKG_VERSION"))
        );
    }
}
