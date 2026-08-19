// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use gettextrs::gettext;
use sbi_spec::base::impl_id::OPEN_SBI;

pub(crate) fn pmu2_crash_name() -> String {
    gettext("PMU2 Crash")
}

pub(crate) fn pmu2_crash(impl_id: usize, version: usize) -> bool {
    // Fixed in OpenSBI v1.7 by 69a0f0245f39ea3af4685cab4cb2dda90acd17cd.
    impl_id == OPEN_SBI && ((1 << 16) | 3..=(1 << 16) | 6).contains(&version)
}

#[cfg(test)]
mod tests {
    use super::pmu2_crash;
    use sbi_spec::base::impl_id;

    #[test]
    fn opensbi_1_3_through_1_6_are_affected() {
        assert!(pmu2_crash(impl_id::OPEN_SBI, 0x1_0003));
        assert!(pmu2_crash(impl_id::OPEN_SBI, 0x1_0006));
    }

    #[test]
    fn other_versions_and_implementations_are_not_affected() {
        assert!(!pmu2_crash(impl_id::OPEN_SBI, 0x1_0002));
        assert!(!pmu2_crash(impl_id::OPEN_SBI, 0x1_0007));
        assert!(!pmu2_crash(impl_id::RUST_SBI, 0x1_0006));
    }
}
