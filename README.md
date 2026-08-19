<!-- SPDX-License-Identifier: MIT OR MulanPSL-2.0 -->

# sbi-info

`sbi-info` reports the RISC-V SBI specification version, implementation ID,
and implementation version from Linux user space. It is intended for systems
where kernel logs are unavailable and the SBI Base extension cannot be called
directly from U-mode.

The project builds a small Linux kernel probe, embeds the resulting `.ko` in a
Rust executable, authenticates the invoking user through PAM, loads the probe
once, reads its values from sysfs, and immediately unloads it.

> **Security:** the installed executable is setuid root because loading a
> kernel module requires `CAP_SYS_MODULE`. Review the source before installing
> it, and never set the setuid bit on a copy in a user-writable directory.

## Reported information

- SBI specification version from `SBI_EXT_BASE_GET_SPEC_VERSION`
- SBI implementation ID from `SBI_EXT_BASE_GET_IMP_ID`
- Raw implementation version from `SBI_EXT_BASE_GET_IMP_VERSION`
- A decoded OpenSBI version when the implementation ID is `1`

Example output:

```text
SBI specification: v2.0 (raw 0x2000000)
SBI implementation: OpenSBI (ID 0x1)
SBI implementation version: 0x10006
OpenSBI version: v1.6
```

## How it works

1. Kbuild compiles `kernel/sbi_probe.c` for the running kernel.
2. Rust's `include_bytes!` embeds `sbi_probe.ko` in the executable.
3. The installed program verifies that its own file and parent directories are
   root-owned and not writable by unprivileged users.
4. The invoking account is authenticated using the system `sudo` PAM policy.
5. `init_module` loads the embedded bytes without an external module path.
6. The program reads three read-only parameters below
   `/sys/module/sbi_probe/parameters/` and calls `delete_module`.

No kernel log access or external `.ko` file is required at runtime.

## Requirements

- A `riscv64` Linux system booted through an SBI implementation
- A kernel with module support and the RISC-V SBI kernel interface
- Matching kernel build files at `/lib/modules/$(uname -r)/build`
- GNU Make and a native Rust toolchain
- PAM runtime libraries `libpam.so.0` and `libpam_misc.so.0`
- A `sudo` PAM policy and membership in the group selected during installation
- Module loading enabled by the kernel security policy

The build is native and targets the running kernel. Rebuild and reinstall
`sbi-info` after a kernel upgrade because the embedded module is kernel-specific.

## Build

```sh
cd ~/sbi-info
cargo build --release
cargo run --release
```

The unprivileged run does not attempt to load the module. It confirms the build
and reminds you to install the executable in a root-managed directory.

## Install

The program does not depend on a fixed installation path. The following is a
recommended layout:

```sh
sudo install -d -o root -g root -m 0755 /usr/local/libexec/sbi-info
sudo install -o root -g sudo -m 0750 \
  target/release/sbi-info \
  /usr/local/libexec/sbi-info/sbi-info
sudo chmod 4750 /usr/local/libexec/sbi-info/sbi-info
ls -l /usr/local/libexec/sbi-info/sbi-info
```

Expected permissions:

```text
-rwsr-x--- root sudo ... /usr/local/libexec/sbi-info/sbi-info
```

The separate `chmod` is intentional: some `install` implementations clear the
setuid bit while changing ownership.

## Usage

Run the installed copy as a normal member of the selected group:

```sh
/usr/local/libexec/sbi-info/sbi-info
```

PAM asks for the current Linux account password. On success, the program loads
the embedded probe, prints the result, and unloads the probe. Do not run the
Cargo build artifact directly with setuid permissions.

## Security model

- The module image is fixed at compile time; no path, argument, or environment
  variable can select replacement kernel code.
- The executable verifies root ownership, the setuid bit, write permissions,
  and every parent directory of its actual installation path.
- PAM handles password input; the Rust program does not retain the password.
- Catchable signals are blocked during the short load/read/unload window.
- The module exposes only read-only SBI values and is unloaded before output.

Setuid programs have a large security impact. Keep the installed file and all
of its parent directories under root control, and rebuild only from reviewed
source. Systems using `nosuid`, kernel lockdown, mandatory module signatures,
or disabled module loading may reject execution or loading.

## Project layout

```text
.
├── Cargo.toml
├── build.rs
├── kernel
│   ├── Makefile
│   └── sbi_probe.c
└── src
    └── main.rs
```

Kbuild outputs and Cargo's `target/` directory are generated and should not be
committed.

## Contributing

Keep changes focused and retain the privilege boundary described above. Before
submitting a change, run:

```sh
cargo fmt --check
cargo build --release
```

Hardware verification requires a RISC-V Linux machine and an installed,
root-owned executable. Contributions intended for a RustSBI community
repository should include a Developer Certificate of Origin sign-off.

## License

The Rust user-space program, build support, and documentation are available
under either of:

- [MIT License](LICENSE-MIT)
- [Mulan PSL v2](LICENSE-MULAN)

The Linux kernel probe `kernel/sbi_probe.c` remains licensed under
[`GPL-2.0-only`](LICENSE-GPL-2.0), as declared by its SPDX header, and
identifies itself to the kernel with `MODULE_LICENSE("GPL")`.
