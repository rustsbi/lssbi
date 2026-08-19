// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use gettextrs::gettext;
use sbi_spec::fwft::feature_type;
use std::fmt::Write;

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Boolean,
    Pmlen,
}

pub(crate) struct Feature {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) kind: Kind,
}

fn feature(id: usize, name: String, kind: Kind) -> Feature {
    Feature { id, name, kind }
}

pub(crate) fn features() -> [Feature; 6] {
    [
        feature(
            feature_type::MISALIGNED_EXC_DELEG,
            gettext("Misaligned Exception Delegation"),
            Kind::Boolean,
        ),
        feature(
            feature_type::LANDING_PAD,
            gettext("Landing Pad"),
            Kind::Boolean,
        ),
        feature(
            feature_type::SHADOW_STACK,
            gettext("Shadow Stack"),
            Kind::Boolean,
        ),
        feature(
            feature_type::DOUBLE_TRAP,
            gettext("Double Trap"),
            Kind::Boolean,
        ),
        feature(
            feature_type::PTE_AD_HW_UPDATING,
            gettext("PTE A/D Hardware Updating"),
            Kind::Boolean,
        ),
        feature(
            feature_type::POINTER_MASKING_PMLEN,
            gettext("Pointer Masking PMLEN"),
            Kind::Pmlen,
        ),
    ]
}

pub(crate) fn module_parameters(features: &[Feature]) -> String {
    let mut parameters = String::from("fwft_ids=");
    for (index, feature) in features.iter().enumerate() {
        if index != 0 {
            parameters.push(',');
        }
        write!(parameters, "{:#x}", feature.id).unwrap();
    }
    parameters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_every_standard_fwft_feature() {
        let features = features();
        assert_eq!(features.len(), 6);
        assert_eq!(
            module_parameters(&features),
            "fwft_ids=0x0,0x1,0x2,0x3,0x4,0x5"
        );
    }
}
