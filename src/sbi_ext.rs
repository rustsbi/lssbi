// SPDX-License-Identifier: MIT OR MulanPSL-2.0

pub(crate) struct Extension {
    pub(crate) key: &'static str,
    pub(crate) message: &'static str,
}

pub(crate) const STANDARD_EXTENSION_COUNT: usize = 16;

pub(crate) const EXTENSIONS: [Extension; 25] = [
    Extension {
        key: "base",
        message: "Base",
    },
    Extension {
        key: "time",
        message: "Timer",
    },
    Extension {
        key: "ipi",
        message: "Inter-processor Interrupt",
    },
    Extension {
        key: "rfence",
        message: "Remote Fence",
    },
    Extension {
        key: "hsm",
        message: "Hart State Management",
    },
    Extension {
        key: "srst",
        message: "System Reset",
    },
    Extension {
        key: "pmu",
        message: "Performance Monitoring Unit",
    },
    Extension {
        key: "dbcn",
        message: "Debug Console",
    },
    Extension {
        key: "susp",
        message: "System Suspend",
    },
    Extension {
        key: "cppc",
        message: "Collab. Processor Perf. Control",
    },
    Extension {
        key: "nacl",
        message: "Nested Acceleration",
    },
    Extension {
        key: "sta",
        message: "Steal-time Accounting",
    },
    Extension {
        key: "sse",
        message: "Supervisor Software Events",
    },
    Extension {
        key: "fwft",
        message: "Firmware Features",
    },
    Extension {
        key: "dbtr",
        message: "Debug Triggers",
    },
    Extension {
        key: "mpxy",
        message: "Message Proxy",
    },
    Extension {
        key: "legacy_set_timer",
        message: "Legacy Set Timer",
    },
    Extension {
        key: "legacy_console_putchar",
        message: "Legacy Console Put Character",
    },
    Extension {
        key: "legacy_console_getchar",
        message: "Legacy Console Get Character",
    },
    Extension {
        key: "legacy_clear_ipi",
        message: "Legacy Clear Inter-processor Interrupt",
    },
    Extension {
        key: "legacy_send_ipi",
        message: "Legacy Send Inter-processor Interrupt",
    },
    Extension {
        key: "legacy_remote_fence_i",
        message: "Legacy Remote Instruction Fence",
    },
    Extension {
        key: "legacy_remote_sfence_vma",
        message: "Legacy Remote Virtual-memory Fence",
    },
    Extension {
        key: "legacy_remote_sfence_vma_asid",
        message: "Legacy Remote Virtual-memory Fence with ASID",
    },
    Extension {
        key: "legacy_shutdown",
        message: "Legacy Shutdown",
    },
];

pub(crate) fn selected(legacy: bool) -> &'static [Extension] {
    if legacy {
        &EXTENSIONS
    } else {
        &EXTENSIONS[..STANDARD_EXTENSION_COUNT]
    }
}

#[cfg(test)]
mod tests {
    use super::{EXTENSIONS, STANDARD_EXTENSION_COUNT, selected};

    #[test]
    fn extension_keys_are_unique() {
        assert_eq!(EXTENSIONS.len(), 25);
        for (index, extension) in EXTENSIONS.iter().enumerate() {
            assert!(
                EXTENSIONS[..index]
                    .iter()
                    .all(|other| other.key != extension.key)
            );
        }
    }

    #[test]
    fn legacy_extensions_are_opt_in() {
        assert_eq!(selected(false).len(), STANDARD_EXTENSION_COUNT);
        assert_eq!(selected(true).len(), EXTENSIONS.len());
    }
}
