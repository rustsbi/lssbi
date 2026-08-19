// SPDX-License-Identifier: MIT OR MulanPSL-2.0

// Canonical architecture IDs from:
// https://github.com/riscv/riscv-isa-manual/blob/main/marchid.md
const PROJECTS: [&str; 55] = [
    "Rocket",
    "BOOM",
    "CVA6",
    "CV32E40P",
    "Spike",
    "E-Class",
    "ORCA",
    "SCR1",
    "YARVI",
    "RVBS",
    "SweRV EH1",
    "MSCC",
    "BlackParrot",
    "BaseJump Manycore",
    "C-Class",
    "SweRV EL2",
    "SweRV EH2",
    "SERV",
    "NEORV32",
    "CV32E40X",
    "CV32E40S",
    "Ibex",
    "RudolV",
    "Steel Core",
    "XiangShan",
    "Hummingbirdv2 E203",
    "Hazard3",
    "CV32E41P",
    "Rift",
    "RISu064",
    "AIRISC",
    "Proteus",
    "VexRiscv",
    "Shuttle",
    "CV32E2",
    "Wally",
    "Boa32",
    "WIV64",
    "RV6",
    "ApogeoRV",
    "MicroRV32",
    "QEMU",
    "KianV",
    "Coreblocks",
    "rrv32",
    "VexiiRiscv",
    "Wildcat",
    "CVA5",
    "River",
    "Raptor",
    "Sargantana",
    "KianV Stealth",
    "RVController",
    "aRVern",
    "SARV",
];

pub(crate) fn project_name(id: usize) -> Option<&'static str> {
    PROJECTS.get(id.checked_sub(1)?).copied()
}

#[cfg(test)]
mod tests {
    use super::project_name;

    #[test]
    fn decodes_registered_ids() {
        assert_eq!(project_name(1), Some("Rocket"));
        assert_eq!(project_name(42), Some("QEMU"));
        assert_eq!(project_name(55), Some("SARV"));
    }

    #[test]
    fn rejects_unregistered_ids() {
        assert_eq!(project_name(0), None);
        assert_eq!(project_name(56), None);
        assert_eq!(project_name(usize::MAX), None);
    }
}
