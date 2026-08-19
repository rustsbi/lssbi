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

The build targets the running kernel and embeds `sbi_probe.ko` in the Rust
executable. Rebuild after a kernel upgrade.

## Install

Install the executable in a root-owned directory. Do not set setuid on the copy
under `target/`, because that directory is controlled by the build user.

```sh
sudo install -d -o root -g root -m 0755 /usr/local/libexec/sbi-info
sudo install -o root -g sudo -m 0750 \
  target/debug/sbi-info \
  /usr/local/libexec/sbi-info/sbi-info
sudo chmod 4750 /usr/local/libexec/sbi-info/sbi-info
```

Replace the `sudo` group if your system uses a different administrative group.
The installed mode should be `-rwsr-x---` (`root:sudo`, `4750`).

## Usage

```sh
/usr/local/libexec/sbi-info/sbi-info
```

Authenticate with the current Linux account password when prompted. Example:

```text
Password:
SBI specification: v2.0 (raw 0x2000000)
SBI implementation: OpenSBI (ID 0x1)
SBI implementation version: 0x10006
OpenSBI version: v1.6
```

## Requirements

- `riscv64` Linux booted through SBI
- Kernel module support and matching files at
  `/lib/modules/$(uname -r)/build`
- Rust, Cargo, GNU Make, `libpam.so.0`, and `libpam_misc.so.0`
- A working `sudo` PAM policy
- A kernel security policy that permits module loading

## Design

Kbuild compiles the small C probe because the target kernel does not enable
Rust support. `include_bytes!` embeds the module in `sbi-info`; after PAM
authentication, `init_module` loads it directly from memory. The program reads
three read-only values from sysfs and immediately calls `delete_module`.

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
