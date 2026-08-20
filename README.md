<!-- SPDX-License-Identifier: MIT OR MulanPSL-2.0 -->

# lssbi

`lssbi` lists information about the active RISC-V SBI environment.

The current backend reads values exported by the `lssbi_probe` DKMS module.
The command itself is unprivileged and never loads or unloads kernel modules.

## Build

```sh
git clone https://github.com/rustsbi/lssbi.git
cd lssbi
cargo build
cargo run
```

Building the command does not require kernel headers. The gettext catalogs are
compiled into `target/<profile>/locale`.

## Install the command

```sh
sudo ./install.sh
```

The defaults are `PREFIX=/usr/local` and `PROFILE=debug`; packagers may also
set `DESTDIR`. The executable is installed as a normal mode-`0755` program in
`$PREFIX/bin`.

For a release build:

```sh
cargo build --release
sudo PROFILE=release ./install.sh
```

## Install the DKMS backend

Install DKMS and the headers for the running kernel, then run:

```sh
sudo dkms add .
sudo dkms build lssbi/0.0.0
sudo dkms install lssbi/0.0.0
sudo modprobe lssbi_probe
```

To load the module automatically at boot, install the supplied modules-load
configuration:

```sh
sudo install -Dm644 modules-load.d/lssbi.conf \
    /etc/modules-load.d/lssbi.conf
```

DKMS builds `lssbi_probe.ko` for the running kernel and rebuilds it after
kernel upgrades. Distribution packages should install the module source and
`dkms.conf` under `/usr/src/lssbi-0.0.0/`; a Debian package would normally
split the project into `lssbi` and `lssbi-dkms` binary packages.

## Usage

```sh
lssbi
lssbi --legacy
lssbi --help
lssbi --version
```

Example output:

```text
SBI specification: v2.0 (raw 0x2000000)
SBI implementation: OpenSBI (ID 0x1)
SBI implementation version: v1.6 (raw 0x10006)
Machine vendor ID: SpacemiT (Hangzhou)Technology Co Ltd (raw 0x710)
Machine architecture ID: 0x8000000058000002
Machine implementation ID: 0x33d8a600
SBI extensions:
  Base:                           Supported
  Timer:                          Supported
  Inter-processor Interrupt:      Supported
  Remote Fence:                   Supported
  Hart State Management:          Supported
  System Reset:                   Supported
  Performance Monitoring Unit:    Supported
  Debug Console:                  Supported
  System Suspend:                 Supported
  Collab. Processor Perf. Control: Not supported
  Nested Acceleration:            Not supported
  Steal-time Accounting:          Not supported
  Supervisor Software Events:     Supported
  Firmware Features:              Supported
  Debug Triggers:                 Supported
  Message Proxy:                  Supported
Vulnerabilities:
  PMU2 Crash (CVE-2025-63913):    Affected
Firmware Features:
  Misaligned Exception Delegation: Not supported
  Landing Pad:                    Not supported
  Shadow Stack:                   Not supported
  Double Trap:                    Not supported
  PTE A/D Hardware Updating:      Not supported
  Pointer Masking PMLEN:          Not supported
```

`--legacy` appends probes for the nine deprecated SBI v0.1 calls.

If the module is not loaded, `lssbi` reports the unavailable DKMS backend and
suggests `sudo modprobe lssbi_probe`.

Output follows the invoking user's locale. Use `LC_ALL=C lssbi` to force
English.

## Translation

Gettext sources follow the conventional `po/` layout: `LINGUAS` lists locales,
`POTFILES.in` lists translatable sources, and `<locale>.po` contains each
translation. Cargo compiles them into `target/<profile>/locale`.

The `ar`, `es`, `fr`, `ru`, `zh_CN`, and `zh_TW` catalogs cover every displayed
Base field, extension, vulnerability, FWFT feature, status, and backend error.
Generic language catalogs such as `fr` are also used by regional locales
through gettext fallback.

## Requirements

- `riscv64` Linux booted through SBI
- Rust and Cargo for building the command
- DKMS, GNU Make, and matching kernel headers for building `lssbi_probe`
- Kernel module support and a policy that permits the module to be loaded

## Design

The GPL-2.0-only `lssbi_probe` module reads the immutable SBI Base and machine
identity fields when it is loaded. It caches them in read-only module
parameters under:

```text
/sys/module/lssbi_probe/parameters/spec_version
/sys/module/lssbi_probe/parameters/impl_id
/sys/module/lssbi_probe/parameters/impl_version
/sys/module/lssbi_probe/parameters/mvendorid
/sys/module/lssbi_probe/parameters/marchid
/sys/module/lssbi_probe/parameters/mimpid
```

It also probes the 16 current extensions and nine optional legacy calls once
at load time. Their SBI error/value pairs are exposed together under:

```text
/sys/module/lssbi_probe/parameters/extensions
```

FWFT state is intentionally not cached. Reading this additional parameter
invokes a getter that executes FWFT `GET` for all six standard features:

```text
/sys/module/lssbi_probe/parameters/fwft
```

One read returns a single live sample in the following stable format:

```text
cpu <linux-cpu-id>
misaligned_exc_deleg <sbi-error> <value>
landing_pad <sbi-error> <value>
shadow_stack <sbi-error> <value>
double_trap <sbi-error> <value>
pte_ad_hw_updating <sbi-error> <value>
pointer_masking_pmlen <sbi-error> <value>
```

The getter prevents CPU migration while making the six calls because these
standard FWFT features are local to a hart and may change at runtime. The
reported CPU identifies the hart sampled by that invocation. `lssbi` reads the
parameter once each time it runs and preserves each call's SBI error separately
from its value. The CLI converts those raw results into localized status text
instead of exposing an unexplained SBI error or hexadecimal zero.
The six `GET` calls are consecutive but not an atomic firmware transaction;
FWFT defines only a single-feature `GET` operation.

The module has no writable parameters, device node, ioctl interface, dynamic
allocation, or background activity. It stays loaded so any user can run
`lssbi` without setuid, PAM, or `CAP_SYS_MODULE`.

The module's private SBI call primitive accepts an extension ID, function ID,
and six arguments. It is shared by the cached Base queries and the live FWFT
getter.

The backend boundary is kept separate from presentation. A future native Linux
sysfs backend can be preferred automatically while retaining the DKMS backend
as a compatibility fallback.

## Security

Only an administrator can install or load `lssbi_probe`. Its exported values
are non-secret firmware metadata and have mode `0444`. As with every external
kernel module, loading it marks the kernel with the out-of-tree (`O`) taint.
Systems using kernel lockdown or mandatory module signatures may reject an
unsigned module; use the distribution's DKMS signing workflow in that case.

## License

The Rust program, build support, and documentation are available under either:

- [MIT License](LICENSE-MIT)
- [Mulan PSL v2](LICENSE-MULAN)

The kernel probe is licensed under
[`GPL-2.0-only`](LICENSE-GPL-2.0) and declares `MODULE_LICENSE("GPL")`.
