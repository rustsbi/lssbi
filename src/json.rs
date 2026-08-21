// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use crate::{backend, fwft, marchid, mvendorid, sbi_ext, sbi_impl, vuln};
use sbi_spec::base::Version;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::{self, Write};

const SCHEMA_VERSION: u8 = 1;

#[derive(Serialize)]
struct Output<'a> {
    schema_version: u8,
    sbi_specification: RawVersion,
    sbi_implementation: Implementation,
    machine: Machine,
    extensions: BTreeMap<&'static str, &'a backend::SbiCallResult>,
    vulnerabilities: Vulnerabilities,
    firmware_features: FirmwareFeatures<'a>,
}

#[derive(Serialize)]
struct RawVersion {
    raw: u64,
    version: Option<String>,
}

#[derive(Serialize)]
struct Implementation {
    id: ImplementationId,
    version: RawVersion,
}

#[derive(Serialize)]
struct ImplementationId {
    raw: u64,
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
    raw: u64,
    name: Option<&'static str>,
}

#[derive(Serialize)]
struct Raw {
    raw: u64,
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
struct FirmwareFeatures<'a> {
    linux_cpu_id: u32,
    sbi_hart_id: u64,
    features: BTreeMap<&'static str, &'a backend::SbiCallResult>,
}

pub(crate) fn print(information: &backend::SbiInformation, legacy: bool) -> Result<(), String> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    serde_json::to_writer(&mut stdout, &output(information, legacy))
        .map_err(|error| format!("cannot write JSON output: {error}"))?;
    writeln!(stdout).map_err(|error| format!("cannot write JSON output: {error}"))
}

fn output(information: &backend::SbiInformation, legacy: bool) -> Output<'_> {
    let impl_id = information.impl_id as usize;
    let impl_version = information.impl_version as usize;
    let mvendorid = information.mvendorid as usize;
    let marchid = information.marchid as usize;
    let extensions = sbi_ext::selected(legacy)
        .iter()
        .zip(&information.extensions)
        .map(|(extension, result)| (extension.key, result))
        .collect();
    let features = fwft::FEATURES
        .iter()
        .zip(&information.fwft.results)
        .map(|(feature, result)| (feature.key, result))
        .collect();

    Output {
        schema_version: SCHEMA_VERSION,
        sbi_specification: RawVersion {
            raw: information.spec_version,
            version: Some(Version::from_raw(information.spec_version as usize).to_string()),
        },
        sbi_implementation: Implementation {
            id: ImplementationId {
                raw: information.impl_id,
                name: sbi_impl::name(impl_id),
            },
            version: RawVersion {
                raw: information.impl_version,
                version: sbi_impl::version(impl_id, impl_version),
            },
        },
        machine: Machine {
            vendor_id: NamedRaw {
                raw: information.mvendorid,
                name: mvendorid::vendor_name(mvendorid),
            },
            architecture_id: NamedRaw {
                raw: information.marchid,
                name: marchid::project_name(marchid),
            },
            implementation_id: Raw {
                raw: information.mimpid,
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
            linux_cpu_id: information.fwft.cpu,
            sbi_hart_id: information.fwft.hart_id,
            features,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::output;
    use crate::backend::{FwftInformation, SbiCallResult, SbiInformation};
    use crate::{fwft, sbi_ext};
    use sbi_spec::base::impl_id;
    use serde_json::json;

    #[test]
    fn schema_preserves_raw_probe_results() {
        let mut extensions = [SbiCallResult { error: 0, value: 0 }; sbi_ext::EXTENSIONS.len()];
        extensions[0] = SbiCallResult {
            error: -4,
            value: 2,
        };
        let mut features = [SbiCallResult { error: 0, value: 0 }; fwft::FEATURES.len()];
        features[0] = SbiCallResult {
            error: -2,
            value: 7,
        };
        let information = SbiInformation {
            spec_version: 0x0200_0000,
            impl_id: impl_id::RUST_SBI as u64,
            impl_version: 0x0004_0102,
            mvendorid: 0x144,
            marchid: 42,
            mimpid: 0x1234,
            extensions,
            fwft: FwftInformation {
                cpu: 3,
                hart_id: 7,
                results: features,
            },
        };

        let value = serde_json::to_value(output(&information, false)).unwrap();
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
            json!({ "raw": 0x0200_0000, "version": "2.0" })
        );
        assert_eq!(value["sbi_implementation"]["id"]["name"], "RustSBI");
        assert_eq!(value["sbi_implementation"]["version"]["raw"], 0x0004_0102);
        assert_eq!(value["sbi_implementation"]["version"]["version"], "4.1.2");
        assert_eq!(value["machine"]["implementation_id"]["raw"], 0x1234);
        assert_eq!(value["extensions"].as_object().unwrap().len(), 16);
        assert_eq!(
            value["extensions"]["base"],
            json!({ "error": -4, "value": 2 })
        );
        assert_eq!(value["firmware_features"]["linux_cpu_id"], 3);
        assert_eq!(value["firmware_features"]["sbi_hart_id"], 7);
        assert_eq!(
            value["firmware_features"]["features"]["misaligned_exc_deleg"],
            json!({ "error": -2, "value": 7 })
        );

        let legacy = serde_json::to_value(output(&information, true)).unwrap();
        assert_eq!(legacy["extensions"].as_object().unwrap().len(), 25);
    }
}
