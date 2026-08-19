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

Create the root-owned directory and copy `target/debug/sbi-info` into it with
`install`. Do not set setuid on the copy under `target/`, because that directory
is controlled by the build user.

```sh
sudo install -d -o root -g root -m 0755 /usr/local/sbin
sudo install -d -o root -g root -m 0755 \
  /usr/local/share/locale/zh_CN/LC_MESSAGES
sudo install -o root -g root -m 0644 \
  target/debug/locale/zh_CN/LC_MESSAGES/sbi-info.mo \
  /usr/local/share/locale/zh_CN/LC_MESSAGES/sbi-info.mo
sudo install -o root -g sudo -m 0750 \
  target/debug/sbi-info \
  /usr/local/sbin/sbi-info
sudo chmod 4750 /usr/local/sbin/sbi-info
```

Replace the `sudo` group if your system uses a different administrative group.
The installed mode should be `-rwsr-x---` (`root:sudo`, `4750`).

## Usage

```sh
sbi-info
sbi-info --help
sbi-info --version
```

Help and version output exit before PAM authentication. Invoking the program
without arguments runs the probe and requests the current account password:

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
- A working `sudo` PAM policy
- A kernel security policy that permits module loading

## Design

Kbuild compiles the small C probe because the target kernel does not enable
Rust support. `include_bytes!` embeds the module in `sbi-info`; gettext selects
messages from the current locale. After PAM authentication, `init_module` loads
the probe directly from memory. The program reads three read-only values from
sysfs and immediately calls `delete_module`.

There is no external module path and no runtime `.ko` file to replace. The
program verifies that its installed executable and parent directories are
root-owned and not writable by unprivileged users.

## Security

`sbi-info` is setuid root because module loading requires `CAP_SYS_MODULE`.
Review changes before installation. Systems using `nosuid`, kernel lockdown,
mandatory module signatures, or disabled module loading may reject it.

## License

The Rust program, build support, and documentation are available under either:

- [MIT License](LICENSE-MIT)
- [Mulan PSL v2](LICENSE-MULAN)

The kernel probe is licensed under
[`GPL-2.0-only`](LICENSE-GPL-2.0) and declares `MODULE_LICENSE("GPL")`.
