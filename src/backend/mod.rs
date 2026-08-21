// SPDX-License-Identifier: MIT OR MulanPSL-2.0

mod dkms;

use crate::{fwft, sbi_ext};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SbiCallResult {
    pub(crate) error: i64,
    pub(crate) value: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FwftInformation {
    pub(crate) cpu: u32,
    pub(crate) hart_id: u64,
    pub(crate) results: [SbiCallResult; fwft::FEATURES.len()],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SbiInformation {
    pub(crate) spec_version: u64,
    pub(crate) impl_id: u64,
    pub(crate) impl_version: u64,
    pub(crate) mvendorid: u64,
    pub(crate) marchid: u64,
    pub(crate) mimpid: u64,
    pub(crate) extensions: [SbiCallResult; sbi_ext::EXTENSIONS.len()],
    pub(crate) fwft: FwftInformation,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ProbeError {
    ModuleNotLoaded,
    CpuOutOfRange { cpu: usize, max: usize },
    CpuNotAllowed(usize),
    CpuAffinity { cpu: usize, error: String },
    Message(String),
}

pub(crate) fn probe(cpu: Option<usize>) -> Result<SbiInformation, ProbeError> {
    dkms::probe(cpu)
}
