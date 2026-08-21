// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use sbi_spec::base::impl_id;

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

pub(crate) fn version(id: usize, raw: usize) -> Option<String> {
    match id {
        impl_id::OPEN_SBI | impl_id::POLARFIRE_HSS => {
            Some(format!("{}.{}", raw >> 16, raw & 0xffff))
        }
        impl_id::KVM | impl_id::RUST_SBI => Some(format!(
            "{}.{}.{}",
            raw >> 16,
            (raw >> 8) & 0xff,
            raw & 0xff
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{impl_id, version};

    #[test]
    fn formats_known_version_encodings() {
        for (id, raw, expected) in [
            (impl_id::OPEN_SBI, 0x0001_0006, "1.6"),
            (impl_id::POLARFIRE_HSS, 0x0001_0006, "1.6"),
            (impl_id::KVM, 0x0004_0102, "4.1.2"),
            (impl_id::RUST_SBI, 0x0004_0102, "4.1.2"),
        ] {
            assert_eq!(version(id, raw).as_deref(), Some(expected));
        }
    }

    #[test]
    fn leaves_unknown_version_encoding_raw() {
        assert_eq!(version(impl_id::BBL, 42), None);
    }
}
