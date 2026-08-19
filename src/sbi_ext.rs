// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use gettextrs::gettext;
use sbi_spec::{base, cppc, dbcn, dbtr, fwft, hsm, legacy, mpxy, nacl, pmu, rfnc, spi};
use sbi_spec::{srst, sse, sta, susp, time};
use std::ffi::CString;
use std::fmt::Write;

pub(crate) struct Extension {
    pub(crate) id: usize,
    pub(crate) name: String,
}

fn ext(id: usize, name: String) -> Extension {
    Extension { id, name }
}

pub(crate) fn selected(legacy: bool) -> Vec<Extension> {
    let mut extensions = vec![
        ext(base::EID_BASE, gettext("Base")),
        ext(time::EID_TIME, gettext("Timer")),
        ext(spi::EID_SPI, gettext("Inter-processor Interrupt")),
        ext(rfnc::EID_RFNC, gettext("Remote Fence")),
        ext(hsm::EID_HSM, gettext("Hart State Management")),
        ext(srst::EID_SRST, gettext("System Reset")),
        ext(pmu::EID_PMU, gettext("Performance Monitoring Unit")),
        ext(dbcn::EID_DBCN, gettext("Debug Console")),
        ext(susp::EID_SUSP, gettext("System Suspend")),
        ext(cppc::EID_CPPC, gettext("Collab. Processor Perf. Control")),
        ext(nacl::EID_NACL, gettext("Nested Acceleration")),
        ext(sta::EID_STA, gettext("Steal-time Accounting")),
        ext(sse::EID_SSE, gettext("Supervisor Software Events")),
        ext(fwft::EID_FWFT, gettext("Firmware Features")),
        ext(dbtr::EID_DBTR, gettext("Debug Triggers")),
        ext(mpxy::EID_MPXY, gettext("Message Proxy")),
    ];
    if legacy {
        extensions.extend([
            ext(legacy::LEGACY_SET_TIMER, gettext("Legacy Set Timer")),
            ext(
                legacy::LEGACY_CONSOLE_PUTCHAR,
                gettext("Legacy Console Put Character"),
            ),
            ext(
                legacy::LEGACY_CONSOLE_GETCHAR,
                gettext("Legacy Console Get Character"),
            ),
            ext(
                legacy::LEGACY_CLEAR_IPI,
                gettext("Legacy Clear Inter-processor Interrupt"),
            ),
            ext(
                legacy::LEGACY_SEND_IPI,
                gettext("Legacy Send Inter-processor Interrupt"),
            ),
            ext(
                legacy::LEGACY_REMOTE_FENCE_I,
                gettext("Legacy Remote Instruction Fence"),
            ),
            ext(
                legacy::LEGACY_REMOTE_SFENCE_VMA,
                gettext("Legacy Remote Virtual-memory Fence"),
            ),
            ext(
                legacy::LEGACY_REMOTE_SFENCE_VMA_ASID,
                gettext("Legacy Remote Virtual-memory Fence with ASID"),
            ),
            ext(legacy::LEGACY_SHUTDOWN, gettext("Legacy Shutdown")),
        ]);
    }
    extensions
}

pub(crate) fn module_parameters(extensions: &[Extension]) -> CString {
    let mut parameters = String::from("extension_ids=");
    for (index, extension) in extensions.iter().enumerate() {
        if index != 0 {
            parameters.push(',');
        }
        write!(parameters, "{:#x}", extension.id).unwrap();
    }
    CString::new(parameters).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_ids_are_unique_and_names_are_readable() {
        let extensions = selected(true);
        assert_eq!(extensions.len(), 25);
        for (index, extension) in extensions.iter().enumerate() {
            assert!(
                extensions[..index]
                    .iter()
                    .all(|other| other.id != extension.id),
                "duplicate extension ID {:#x}",
                extension.id
            );
            assert!(!extension.name.contains('_'));
        }
    }

    #[test]
    fn legacy_extensions_are_opt_in() {
        assert_eq!(selected(false).len(), 16);
        assert_eq!(selected(true).len(), 25);
    }

    #[test]
    fn module_parameters_include_selected_extensions() {
        let extensions = selected(false);
        assert_eq!(
            module_parameters(&extensions)
                .to_str()
                .unwrap()
                .strip_prefix("extension_ids=")
                .unwrap()
                .split(',')
                .count(),
            extensions.len()
        );
    }
}
