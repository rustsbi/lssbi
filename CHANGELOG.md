<!-- SPDX-License-Identifier: MIT OR MulanPSL-2.0 -->

# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Report the SBI specification, implementation ID, and firmware version from
  RISC-V Linux without kernel-log access.
- Embed the kernel probe in the executable and unload it immediately after
  collecting the SBI information.
- Provide CLI help and version output, gettext localization, PAM authentication
  caching, and a POSIX installer with selectable debug and release profiles.
- Add Arabic, French, Russian, Spanish, and Traditional Chinese translations,
  covering all six official United Nations languages alongside English and
  Simplified Chinese.
- Explain the privilege requirement only when PAM requests a password.
- Probe and list implemented SBI 3.0 extensions using IDs provided by
  `sbi-spec`, with deprecated legacy extensions available through `--legacy`.
- Localize extension names and report a supported or not-supported status for
  every probed extension.
- Report OpenSBI PMU2 Crash (CVE-2025-63913) exposure from the firmware version.

### Changed

- Rename the project, binary, PAM service, and gettext domain to `lssbi`.

### Fixed

- Apply the executable's setuid mode after setting its ownership so mode `4750`
  is preserved across `install` implementations.
