// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use crate::{backend, fwft, marchid, mvendorid, sbi_ext, sbi_impl, vuln};
use gettextrs::{
    LocaleCategory, bind_textdomain_codeset, bindtextdomain, gettext, setlocale, textdomain,
};
use sbi_spec::base::Version;
use std::fs;
use unicode_width::UnicodeWidthStr;

const TEXT_DOMAIN: &str = "lssbi";

pub(crate) fn run(legacy: bool) -> Result<(), String> {
    // SAFETY: run is called once from the single-threaded program entry point,
    // before any other code can read or change the process-wide locale.
    unsafe {
        setlocale(LocaleCategory::LcAll, "");
    }
    init_gettext()?;

    let information = backend::probe().map_err(localize_probe_error)?;
    print_result(information, legacy);
    Ok(())
}

fn localize_probe_error(error: backend::ProbeError) -> String {
    match error {
        backend::ProbeError::ModuleNotLoaded => gettext(
            "DKMS backend unavailable: the lssbi_probe module is not loaded; run `sudo modprobe lssbi_probe`",
        ),
        backend::ProbeError::Message(message) => message,
    }
}

fn init_gettext() -> Result<(), String> {
    let executable = fs::canonicalize("/proc/self/exe")
        .map_err(|error| format!("cannot resolve the running executable: {error}"))?;
    let executable_dir = executable
        .parent()
        .ok_or_else(|| "the running executable has no parent directory".to_owned())?;
    let locale_dir = match executable_dir.file_name().and_then(|name| name.to_str()) {
        Some("bin" | "sbin") => executable_dir
            .parent()
            .ok_or_else(|| "the executable directory has no installation prefix".to_owned())?
            .join("share/locale"),
        _ => executable_dir.join("locale"),
    };

    bindtextdomain(TEXT_DOMAIN, &locale_dir)
        .map_err(|error| format!("cannot bind the translation directory: {error}"))?;
    bind_textdomain_codeset(TEXT_DOMAIN, "UTF-8")
        .map_err(|error| format!("cannot select the translation encoding: {error}"))?;
    textdomain(TEXT_DOMAIN).map_err(|error| format!("cannot select the text domain: {error}"))?;
    Ok(())
}

fn print_result(information: backend::SbiInformation, legacy: bool) {
    let spec_raw = information.spec_version as usize;
    let spec = Version::from_raw(spec_raw);
    let impl_id = information.impl_id as usize;
    let impl_version = information.impl_version as usize;
    let mvendorid = information.mvendorid as usize;
    let marchid = information.marchid as usize;
    let mimpid = information.mimpid as usize;

    let raw = gettext("raw");
    let id = gettext("ID");
    println!(
        "{}: v{spec} ({raw} {spec_raw:#x})",
        gettext("SBI specification")
    );
    println!(
        "{}: {} ({id} {impl_id:#x})",
        gettext("SBI implementation"),
        sbi_impl::name(impl_id)
    );
    println!(
        "{}: v{} ({raw} {impl_version:#x})",
        gettext("SBI implementation version"),
        sbi_impl::version(impl_id, impl_version)
    );

    if let Some(vendor) = mvendorid::vendor_name(mvendorid) {
        println!(
            "{}: {vendor} ({raw} {mvendorid:#x})",
            gettext("Machine vendor ID")
        );
    } else {
        println!("{}: {mvendorid:#x}", gettext("Machine vendor ID"));
    }
    if let Some(project) = marchid::project_name(marchid) {
        println!(
            "{}: {project} ({raw} {marchid:#x})",
            gettext("Machine architecture ID")
        );
    } else {
        println!("{}: {marchid:#x}", gettext("Machine architecture ID"));
    }
    println!("{}: {mimpid:#x}", gettext("Machine implementation ID"));

    print_extensions(&information.extensions, legacy);
    print_vulnerabilities(impl_id, impl_version);
    print_fwft(&information.fwft.results);
}

fn print_extensions(results: &[backend::SbiCallResult; sbi_ext::EXTENSIONS.len()], legacy: bool) {
    println!("{}:", gettext("SBI extensions"));
    let extensions = sbi_ext::selected(legacy);
    let names = extensions
        .iter()
        .map(|extension| gettext(extension.message))
        .collect::<Vec<_>>();
    let width = status_width(&names);

    for (name, result) in names.iter().zip(&results[..extensions.len()]) {
        let status = if result.error == 0 && result.value != 0 {
            gettext("Supported")
        } else {
            gettext("Not supported")
        };
        print_status(&format!("{name}:"), &status, width);
    }
}

fn print_vulnerabilities(impl_id: usize, impl_version: usize) {
    println!("{}:", gettext("Vulnerabilities"));
    let status = if vuln::pmu2_crash(impl_id, impl_version) {
        gettext("Affected")
    } else {
        gettext("Not affected")
    };
    let label = format!("{} (CVE-2025-63913):", gettext("PMU2 Crash"));
    print_status(&label, &status, 32);
}

fn print_fwft(results: &[backend::SbiCallResult; fwft::FEATURES.len()]) {
    println!("{}:", gettext("Firmware Features"));
    let names = fwft::FEATURES.map(|feature| gettext(feature.message));
    let width = status_width(&names);

    for ((name, feature), result) in names.iter().zip(fwft::FEATURES.iter()).zip(results) {
        let status = if result.error != 0 {
            gettext("Not supported")
        } else {
            match feature.kind {
                fwft::Kind::Boolean => {
                    if result.value != 0 {
                        gettext("Supported")
                    } else {
                        gettext("Not supported")
                    }
                }
                fwft::Kind::Pmlen if result.value == 0 => gettext("Disabled"),
                fwft::Kind::Pmlen => result.value.to_string(),
            }
        };
        print_status(&format!("{name}:"), &status, width);
    }
}

fn status_width(names: &[String]) -> usize {
    names
        .iter()
        .map(|name| UnicodeWidthStr::width(name.as_str()) + 2)
        .max()
        .unwrap_or(0)
        .clamp(32, 48)
}

fn print_status(label: &str, status: &str, width: usize) {
    let padding = width.saturating_sub(UnicodeWidthStr::width(label)).max(1);
    println!("  {label}{}{status}", " ".repeat(padding));
}
