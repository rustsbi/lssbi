// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use crate::{backend, fwft, marchid, mvendorid, sbi_ext, sbi_impl, vuln};
use sbi_spec::base::Version;
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;
use std::io::{self, Write};

const SCHEMA_VERSION: u8 = 1;

#[derive(Serialize)]
struct Output {
    schema_version: u8,
    sbi_specification: RawVersion,
    sbi_implementation: Implementation,
    machine: Machine,
    extensions: BTreeMap<&'static str, CallResult>,
    vulnerabilities: Vulnerabilities,
    firmware_features: FirmwareFeatures,
}

#[derive(Serialize)]
struct RawVersion {
    raw: HexValue,
    version: Option<String>,
}

#[derive(Serialize)]
struct Implementation {
    id: ImplementationId,
    version: RawVersion,
}

#[derive(Serialize)]
struct ImplementationId {
    raw: HexValue,
    name: &'static str,
}

#[derive(Serialize)]
struct Machine {
    vendor_id: NamedRaw,
    architecture_id: NamedRaw,
    implementation_id: Raw,
}

#[derive(Serialize)]
struct NamedRaw {
    raw: HexValue,
    name: Option<&'static str>,
}

#[derive(Serialize)]
struct Raw {
    raw: HexValue,
}

#[derive(Serialize)]
struct CallResult {
    error: i64,
    value: HexValue,
}

impl From<&backend::SbiCallResult> for CallResult {
    fn from(result: &backend::SbiCallResult) -> Self {
        Self {
            error: result.error,
            value: HexValue(result.value),
        }
    }
}

struct HexValue(u64);

impl Serialize for HexValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&format_args!("{:#x}", self.0))
    }
}

#[derive(Serialize)]
struct Vulnerabilities {
    pmu2_crash: Vulnerability,
}

#[derive(Serialize)]
struct Vulnerability {
    id: &'static str,
    affected: bool,
}

#[derive(Serialize)]
struct FirmwareFeatures {
    linux_cpu_id: u32,
    sbi_hart_id: HexValue,
    features: BTreeMap<&'static str, CallResult>,
}

pub(crate) fn print(info: &backend::SbiInfo, legacy: bool) -> Result<(), String> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    serde_json::to_writer(&mut stdout, &output(info, legacy))
        .map_err(|error| format!("cannot write JSON output: {error}"))?;
    writeln!(stdout).map_err(|error| format!("cannot write JSON output: {error}"))
}

fn output(info: &backend::SbiInfo, legacy: bool) -> Output {
    let impl_id = info.impl_id as usize;
    let impl_version = info.impl_version as usize;
    let mvendorid = info.mvendorid as usize;
    let marchid = info.marchid as usize;
    let extensions = sbi_ext::selected(legacy)
        .iter()
        .zip(&info.extensions)
        .map(|(extension, result)| (extension.key, result.into()))
        .collect();
    let features = fwft::FEATURES
        .iter()
        .zip(&info.fwft.results)
        .map(|(feature, result)| (feature.key, result.into()))
        .collect();

    Output {
        schema_version: SCHEMA_VERSION,
        sbi_specification: RawVersion {
            raw: HexValue(info.spec_version),
            version: Some(Version::from_raw(info.spec_version as usize).to_string()),
        },
        sbi_implementation: Implementation {
            id: ImplementationId {
                raw: HexValue(info.impl_id),
                name: sbi_impl::name(impl_id),
            },
            version: RawVersion {
                raw: HexValue(info.impl_version),
                version: sbi_impl::version(impl_id, impl_version),
            },
        },
        machine: Machine {
            vendor_id: NamedRaw {
                raw: HexValue(info.mvendorid),
                name: mvendorid::vendor_name(mvendorid),
            },
            architecture_id: NamedRaw {
                raw: HexValue(info.marchid),
                name: marchid::project_name(marchid),
            },
            implementation_id: Raw {
                raw: HexValue(info.mimpid),
            },
        },
        extensions,
        vulnerabilities: Vulnerabilities {
            pmu2_crash: Vulnerability {
                id: "CVE-2025-63913",
                affected: vuln::pmu2_crash(impl_id, impl_version),
            },
        },
        firmware_features: FirmwareFeatures {
            linux_cpu_id: info.fwft.cpu,
            sbi_hart_id: HexValue(info.fwft.hart_id),
            features,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::output;
    use crate::backend::{FwftInfo, SbiCallResult, SbiInfo};
    use crate::{fwft, sbi_ext};
    use sbi_spec::base::impl_id;
    use serde_json::json;

    #[test]
    fn schema_preserves_xlen_values() {
        const LARGE_XLEN: u64 = 0x8000_0000_5800_0002;
        let mut extensions = [SbiCallResult { error: 0, value: 0 }; sbi_ext::EXTENSIONS.len()];
        extensions[0] = SbiCallResult {
            error: -4,
            value: LARGE_XLEN,
        };
        let mut features = [SbiCallResult { error: 0, value: 0 }; fwft::FEATURES.len()];
        features[0] = SbiCallResult {
            error: -2,
            value: u64::MAX,
        };
        let info = SbiInfo {
            spec_version: 0x0200_0000,
            impl_id: impl_id::RUST_SBI as u64,
            impl_version: 0x0004_0102,
            mvendorid: 0x144,
            marchid: LARGE_XLEN,
            mimpid: 0x1234,
            extensions,
            fwft: FwftInfo {
                cpu: 3,
                hart_id: LARGE_XLEN,
                results: features,
            },
        };

        let value = serde_json::to_value(output(&info, false)).unwrap();
        let object = value.as_object().unwrap();
        assert_eq!(object.len(), 7);
        for key in [
            "schema_version",
            "sbi_specification",
            "sbi_implementation",
            "machine",
            "extensions",
            "vulnerabilities",
            "firmware_features",
        ] {
            assert!(object.contains_key(key));
        }
        assert_eq!(value["schema_version"], 1);
        assert_eq!(
            value["sbi_specification"],
            json!({ "raw": "0x2000000", "version": "2.0" })
        );
        assert_eq!(value["sbi_implementation"]["id"]["raw"], "0x4");
        assert_eq!(value["sbi_implementation"]["id"]["name"], "RustSBI");
        assert_eq!(value["sbi_implementation"]["version"]["raw"], "0x40102");
        assert_eq!(value["sbi_implementation"]["version"]["version"], "4.1.2");
        assert_eq!(value["machine"]["vendor_id"]["raw"], "0x144");
        assert_eq!(
            value["machine"]["architecture_id"]["raw"],
            "0x8000000058000002"
        );
        assert_eq!(value["machine"]["implementation_id"]["raw"], "0x1234");
        assert_eq!(value["extensions"].as_object().unwrap().len(), 16);
        assert_eq!(
            value["extensions"]["base"],
            json!({ "error": -4, "value": "0x8000000058000002" })
        );
        assert_eq!(value["firmware_features"]["linux_cpu_id"], 3);
        assert_eq!(
            value["firmware_features"]["sbi_hart_id"],
            "0x8000000058000002"
        );
        assert_eq!(
            value["firmware_features"]["features"]["misaligned_exc_deleg"],
            json!({ "error": -2, "value": "0xffffffffffffffff" })
        );

        let legacy = serde_json::to_value(output(&info, true)).unwrap();
        assert_eq!(legacy["extensions"].as_object().unwrap().len(), 25);
    }
}
