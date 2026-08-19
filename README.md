<!-- SPDX-License-Identifier: MIT OR MulanPSL-2.0 -->

# sbi-info

`sbi-info` reports the SBI specification version, implementation ID, and
firmware version from RISC-V Linux without kernel-log access.

## Build

```sh
git clone https://github.com/rustsbi/sbi-info.git
cd sbi-info
cargo build
cargo run
```

The build targets the running kernel, embeds `sbi_probe.ko`, and compiles the
gettext catalogs. Rebuild after a kernel upgrade.

## Install

The installer copies the PAM policy, every catalog listed in `po/LINGUAS`, and
the executable into root-owned locations. Do not set setuid on the copy under
`target/`, because that directory is controlled by the build user.

```sh
sudo ./install.sh
```

The defaults are `PREFIX=/usr/local`, `PROFILE=debug`, and `ADMIN_GROUP=sudo`;
packagers may also set `DESTDIR`. The installed executable has mode `4750`.

## Usage

```sh
sbi-info
sbi-info --help
sbi-info --version
```

Help and version output exit before PAM authentication. Invoking the program
without arguments runs the probe and requests the current account password.
Successful authentication is cached for five minutes on the current terminal:

```text
Password:
SBI specification: v2.0 (raw 0x2000000)
SBI implementation: OpenSBI (ID 0x1)
SBI implementation version: v1.6 (raw 0x10006)
```

Output follows the invoking user's locale. Use `LC_ALL=C sbi-info` to force
English.

## Translation

Gettext sources follow the conventional `po/` layout: `LINGUAS` lists locales,
`POTFILES.in` lists translatable sources, and `<locale>.po` contains each
translation. Cargo compiles them into `target/<profile>/locale`.

## Requirements

- `riscv64` Linux booted through SBI
- Kernel module support and matching files at
  `/lib/modules/$(uname -r)/build`
- Rust, Cargo, GNU Make, `libpam.so.0`, and `libpam_misc.so.0`
- `pam_timestamp.so` and a working system PAM policy
- A kernel security policy that permits module loading

## Design

Kbuild compiles the small C probe because the target kernel does not enable
Rust support. `include_bytes!` embeds the module in `sbi-info`; gettext selects
messages from the current locale. After PAM authentication, `init_module` loads
the probe directly from memory. The program reads three read-only values from
sysfs and immediately calls `delete_module`.

The `sbi-info` PAM service uses `pam_timestamp` for a root-owned, per-terminal
authentication cache with a five-minute timeout.

There is no external module path and no runtime `.ko` file to replace. The
program verifies that its installed executable and parent directories are
root-owned and not writable by unprivileged users.

## Security

`sbi-info` is setuid root because module loading requires `CAP_SYS_MODULE`.
Review changes before installation. Systems using `nosuid`, kernel lockdown,
mandatory module signatures, or disabled module loading may reject it.

## License

The Rust program, PAM policy, build support, and documentation are available
under either:

- [MIT License](LICENSE-MIT)
- [Mulan PSL v2](LICENSE-MULAN)

The kernel probe is licensed under
[`GPL-2.0-only`](LICENSE-GPL-2.0) and declares `MODULE_LICENSE("GPL")`.
