// SPDX-License-Identifier: MIT OR MulanPSL-2.0

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Kind {
    Boolean,
    Pmlen,
}

pub(crate) struct Feature {
    pub(crate) key: &'static str,
    pub(crate) message: &'static str,
    pub(crate) kind: Kind,
}

pub(crate) const FEATURES: [Feature; 6] = [
    Feature {
        key: "misaligned_exc_deleg",
        message: "Misaligned Exception Delegation",
        kind: Kind::Boolean,
    },
    Feature {
        key: "landing_pad",
        message: "Landing Pad",
        kind: Kind::Boolean,
    },
    Feature {
        key: "shadow_stack",
        message: "Shadow Stack",
        kind: Kind::Boolean,
    },
    Feature {
        key: "double_trap",
        message: "Double Trap",
        kind: Kind::Boolean,
    },
    Feature {
        key: "pte_ad_hw_updating",
        message: "PTE A/D Hardware Updating",
        kind: Kind::Boolean,
    },
    Feature {
        key: "pointer_masking_pmlen",
        message: "Pointer Masking PMLEN",
        kind: Kind::Pmlen,
    },
];

#[cfg(test)]
mod tests {
    use super::FEATURES;

    #[test]
    fn includes_all_standard_features() {
        assert_eq!(FEATURES.len(), 6);
        assert_eq!(FEATURES[0].key, "misaligned_exc_deleg");
        assert_eq!(FEATURES[5].key, "pointer_masking_pmlen");
    }
}
