// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use super::{FwftInformation, ProbeError, SbiCallResult, SbiInformation};
use crate::{fwft, sbi_ext};
use std::fs;
use std::path::Path;

const PARAMETER_DIRECTORY: &str = "/sys/module/lssbi_probe/parameters";

pub(super) fn probe() -> Result<SbiInformation, ProbeError> {
    let directory = Path::new(PARAMETER_DIRECTORY);
    if !directory.is_dir() {
        return Err(ProbeError::ModuleNotLoaded);
    }

    Ok(SbiInformation {
        spec_version: read_parameter(directory, "spec_version")?,
        impl_id: read_parameter(directory, "impl_id")?,
        impl_version: read_parameter(directory, "impl_version")?,
        mvendorid: read_parameter(directory, "mvendorid")?,
        marchid: read_parameter(directory, "marchid")?,
        mimpid: read_parameter(directory, "mimpid")?,
        extensions: read_extensions(directory)?,
        fwft: read_fwft(directory)?,
    })
}

fn read_parameter(directory: &Path, name: &str) -> Result<u64, ProbeError> {
    let path = directory.join(name);
    let text = fs::read_to_string(&path)
        .map_err(|error| ProbeError::Message(format!("cannot read {}: {error}", path.display())))?;
    parse_parameter(&text).map_err(|error| {
        ProbeError::Message(format!("invalid value in {}: {error}", path.display()))
    })
}

fn parse_parameter(text: &str) -> Result<u64, std::num::ParseIntError> {
    text.trim().parse()
}

fn read_extensions(
    directory: &Path,
) -> Result<[SbiCallResult; sbi_ext::EXTENSIONS.len()], ProbeError> {
    let path = directory.join("extensions");
    let text = fs::read_to_string(&path)
        .map_err(|error| ProbeError::Message(format!("cannot read {}: {error}", path.display())))?;
    let keys = sbi_ext::EXTENSIONS.map(|extension| extension.key);
    parse_records(&text, &keys).map_err(|error| {
        ProbeError::Message(format!("invalid value in {}: {error}", path.display()))
    })
}

fn read_fwft(directory: &Path) -> Result<FwftInformation, ProbeError> {
    let path = directory.join("fwft");
    let text = fs::read_to_string(&path)
        .map_err(|error| ProbeError::Message(format!("cannot read {}: {error}", path.display())))?;
    parse_fwft(&text).map_err(|error| {
        ProbeError::Message(format!("invalid value in {}: {error}", path.display()))
    })
}

fn parse_fwft(text: &str) -> Result<FwftInformation, String> {
    let mut lines = text.lines();
    let cpu_line = lines
        .next()
        .ok_or_else(|| "missing CPU record".to_owned())?;
    let mut cpu_fields = cpu_line.split_ascii_whitespace();
    if cpu_fields.next() != Some("cpu") {
        return Err("the first record is not a CPU record".to_owned());
    }
    let cpu = cpu_fields
        .next()
        .ok_or_else(|| "missing CPU number".to_owned())?
        .parse()
        .map_err(|error| format!("invalid CPU number: {error}"))?;
    if cpu_fields.next().is_some() {
        return Err("unexpected field in CPU record".to_owned());
    }

    let keys = fwft::FEATURES.map(|feature| feature.key);
    let remaining = lines.collect::<Vec<_>>().join("\n");
    let results = parse_records(&remaining, &keys)?;
    Ok(FwftInformation { cpu, results })
}

fn parse_records<const N: usize>(
    text: &str,
    expected_keys: &[&str; N],
) -> Result<[SbiCallResult; N], String> {
    let mut lines = text.lines();
    let mut results = [SbiCallResult { error: 0, value: 0 }; N];

    for (index, expected_key) in expected_keys.iter().enumerate() {
        let line = lines
            .next()
            .ok_or_else(|| format!("missing {expected_key} record"))?;
        let mut fields = line.split_ascii_whitespace();
        let key = fields
            .next()
            .ok_or_else(|| format!("missing {expected_key} key"))?;
        if key != *expected_key {
            return Err(format!("expected {expected_key} record, found {key}"));
        }
        let error = fields
            .next()
            .ok_or_else(|| format!("missing {expected_key} error"))?
            .parse()
            .map_err(|parse_error| format!("invalid {expected_key} error: {parse_error}"))?;
        let value = fields
            .next()
            .ok_or_else(|| format!("missing {expected_key} value"))?
            .parse()
            .map_err(|parse_error| format!("invalid {expected_key} value: {parse_error}"))?;
        if fields.next().is_some() {
            return Err(format!("unexpected field in {expected_key} record"));
        }
        results[index] = SbiCallResult { error, value };
    }

    if lines.any(|line| !line.trim().is_empty()) {
        return Err("unexpected trailing record".to_owned());
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::{parse_fwft, parse_parameter, parse_records};
    use crate::backend::{FwftInformation, SbiCallResult};
    use crate::{fwft, sbi_ext};

    const FWFT_SAMPLE: &str = "\
cpu 3
misaligned_exc_deleg 0 1
landing_pad -2 0
shadow_stack 0 0
double_trap 0 1
pte_ad_hw_updating 0 1
pointer_masking_pmlen 0 7
";

    #[test]
    fn parses_unsigned_module_parameter() {
        assert_eq!(
            parse_parameter("9223372038331170818\n"),
            Ok(0x8000_0000_5800_0002)
        );
    }

    #[test]
    fn rejects_non_numeric_parameter() {
        assert!(parse_parameter("not-a-number\n").is_err());
    }

    #[test]
    fn parses_live_fwft_sample() {
        assert_eq!(
            parse_fwft(FWFT_SAMPLE),
            Ok(FwftInformation {
                cpu: 3,
                results: [
                    SbiCallResult { error: 0, value: 1 },
                    SbiCallResult {
                        error: -2,
                        value: 0,
                    },
                    SbiCallResult { error: 0, value: 0 },
                    SbiCallResult { error: 0, value: 1 },
                    SbiCallResult { error: 0, value: 1 },
                    SbiCallResult { error: 0, value: 7 },
                ],
            })
        );
    }

    #[test]
    fn rejects_out_of_order_fwft_sample() {
        let malformed = FWFT_SAMPLE.replacen("landing_pad", "shadow_stack", 1);
        assert!(parse_fwft(&malformed).is_err());
    }

    #[test]
    fn parses_all_extension_records() {
        let keys = sbi_ext::EXTENSIONS.map(|extension| extension.key);
        let text = keys
            .iter()
            .map(|key| format!("{key} 0 1"))
            .collect::<Vec<_>>()
            .join("\n");
        let results = parse_records(&text, &keys).unwrap();
        assert_eq!(results.len(), sbi_ext::EXTENSIONS.len());
        assert!(
            results
                .iter()
                .all(|result| result.error == 0 && result.value == 1)
        );
    }

    #[test]
    fn fwft_keys_match_feature_table() {
        let keys = fwft::FEATURES.map(|feature| feature.key);
        assert_eq!(keys[0], "misaligned_exc_deleg");
        assert_eq!(keys[5], "pointer_masking_pmlen");
    }
}
