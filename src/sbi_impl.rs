// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use sbi_spec::base::{Version, impl_id};

pub(crate) fn name(id: usize) -> &'static str {
    match id {
        impl_id::BBL => "Berkeley Boot Loader (BBL)",
        impl_id::OPEN_SBI => "OpenSBI",
        impl_id::XVISOR => "Xvisor",
        impl_id::KVM => "KVM",
        impl_id::RUST_SBI => "RustSBI",
        impl_id::DIOSIX => "Diosix",
        impl_id::COFFER => "Coffer",
        impl_id::XEN => "Xen Project",
        impl_id::POLARFIRE_HSS => "PolarFire Hart Software Services",
        impl_id::COREBOOT => "coreboot",
        impl_id::OREBOOT => "oreboot",
        11 => "bhyve",
        _ => "unknown",
    }
}

pub(crate) fn version(id: usize, raw: usize) -> Version {
    let (major, minor) = match id {
        impl_id::OPEN_SBI => (raw >> 16, raw & 0xffff),
        impl_id::RUST_SBI => (raw >> 16, (raw >> 8) & 0xff),
        _ => return Version::from_raw(raw),
    };
    Version::from_raw((major << 24) | minor)
}

#[cfg(test)]
mod tests {
    use super::{impl_id, version};

    #[test]
    fn formats_opensbi_version() {
        let version = version(impl_id::OPEN_SBI, 0x10006);
        assert_eq!((version.major(), version.minor()), (1, 6));
    }

    #[test]
    fn formats_rustsbi_version() {
        let version = version(impl_id::RUST_SBI, 0x0004_0102);
        assert_eq!((version.major(), version.minor()), (4, 1));
    }

    #[test]
    fn preserves_other_version_encodings() {
        let version = version(impl_id::BBL, 42);
        assert_eq!((version.major(), version.minor()), (0, 42));
    }
}
