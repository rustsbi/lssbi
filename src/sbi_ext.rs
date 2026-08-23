// SPDX-License-Identifier: MIT OR MulanPSL-2.0

pub(crate) struct Extension {
    pub(crate) key: &'static str,
    pub(crate) message: &'static str,
}

impl Extension {
    const fn new(key: &'static str, message: &'static str) -> Self {
        Self { key, message }
    }
}

pub(crate) const STANDARD_EXTENSION_COUNT: usize = 16;

pub(crate) const EXTENSIONS: [Extension; 25] = [
    Extension::new("base", "Base"),
    Extension::new("time", "Timer"),
    Extension::new("ipi", "Inter-processor Interrupt"),
    Extension::new("rfence", "Remote Fence"),
    Extension::new("hsm", "Hart State Management"),
    Extension::new("srst", "System Reset"),
    Extension::new("pmu", "Performance Monitoring Unit"),
    Extension::new("dbcn", "Debug Console"),
    Extension::new("susp", "System Suspend"),
    Extension::new("cppc", "Collab. Processor Perf. Control"),
    Extension::new("nacl", "Nested Acceleration"),
    Extension::new("sta", "Steal-time Accounting"),
    Extension::new("sse", "Supervisor Software Events"),
    Extension::new("fwft", "Firmware Features"),
    Extension::new("dbtr", "Debug Triggers"),
    Extension::new("mpxy", "Message Proxy"),
    Extension::new("legacy_set_timer", "Legacy Set Timer"),
    Extension::new("legacy_console_putchar", "Legacy Console Put Character"),
    Extension::new("legacy_console_getchar", "Legacy Console Get Character"),
    Extension::new("legacy_clear_ipi", "Legacy Clear Inter-processor Interrupt"),
    Extension::new("legacy_send_ipi", "Legacy Send Inter-processor Interrupt"),
    Extension::new("legacy_remote_fence_i", "Legacy Remote Instruction Fence"),
    Extension::new(
        "legacy_remote_sfence_vma",
        "Legacy Remote Virtual-memory Fence",
    ),
    Extension::new(
        "legacy_remote_sfence_vma_asid",
        "Legacy Remote Virtual-memory Fence with ASID",
    ),
    Extension::new("legacy_shutdown", "Legacy Shutdown"),
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
