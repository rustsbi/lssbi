<!-- SPDX-License-Identifier: MIT OR MulanPSL-2.0 -->

# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Add `--json` with a versioned, locale-independent schema that preserves raw
  SBI values and probe error/value pairs.
- Add `--cpu <N>` for selecting the Linux CPU used by live FWFT queries and
  report the sampled Linux CPU ID and SBI hart ID.

### Fixed

- Export SBI probe values as unsigned kernel module fields so high-bit XLEN
  values remain parseable.
- Preserve 64-bit SBI register values exactly in JSON by encoding XLEN values as
  hexadecimal strings.
- Localize CPU selection errors and explain malformed, out-of-range, offline,
  and affinity-restricted values.
- Distinguish unsupported SBI extensions from failed probes and retain
  implementation-defined nonzero probe values in command output.
- Format known SBI implementation version encodings, including the RustSBI
  patch component, while leaving unknown encodings undecoded.

## [0.0.0] - 2026-08-20

### Added

- Add a DKMS backend that builds `lssbi_probe` for each installed kernel and
  exports SBI information through read-only module parameters.
- Add a live FWFT sample parameter that executes all six standard FWFT `GET`
  calls on one CPU every time it is read.
- Report RISC-V machine vendor, architecture, and implementation identifiers.
- Probe 16 current SBI extensions and optionally nine legacy calls with
  `--legacy`.
- Report OpenSBI PMU2 Crash (CVE-2025-63913) exposure.
- Add a backend boundary for a future native Linux sysfs implementation.
- Provide CLI help and version output, gettext localization, and a POSIX
  installer with selectable debug and release profiles.
- Add complete Arabic, French, Russian, Spanish, Simplified Chinese, and
  Traditional Chinese translations for all displayed information and errors.

### Changed

- Rename the project and command from `sbi-info` to `lssbi`.
- Build the command independently from the running kernel.
- Keep the probe module loaded so the command can run without privileges.

### Removed

- Remove the embedded kernel module, setuid installation, and project-owned
  PAM authentication policy.

[Unreleased]: https://github.com/rustsbi/lssbi/compare/v0.0.0...HEAD
[0.0.0]: https://github.com/rustsbi/lssbi/releases/tag/v0.0.0
