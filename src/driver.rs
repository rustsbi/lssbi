// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use crate::{fwft, marchid, mvendorid, sbi_ext, sbi_impl, vuln};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, gettext, textdomain};
use libc::{c_int, c_void};
use nix::sys::signal::{SigSet, SigmaskHow};
use nix::unistd::{Uid, User};
use pam_sys::{
    PAM_MAX_NUM_MSG, PAM_PROMPT_ECHO_OFF, PAM_SUCCESS, pam_acct_mgmt, pam_authenticate,
    pam_close_session, pam_conv, pam_end, pam_handle_t, pam_message, pam_open_session,
    pam_response, pam_start, pam_strerror,
};
use sbi_spec::{base::Version, fwft::EID_FWFT};
use std::ffi::{CStr, CString};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::ptr;
use unicode_width::UnicodeWidthStr;

const MODULE_NAME: &str = "sbi_probe";
const TEXT_DOMAIN: &str = "lssbi";
const EMBEDDED_MODULE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sbi_probe.ko"));

unsafe extern "C" {
    fn misc_conv(
        num_msg: c_int,
        msg: *mut *const pam_message,
        response: *mut *mut pam_response,
        appdata_ptr: *mut c_void,
    ) -> c_int;
}

unsafe extern "C" fn conversation(
    num_msg: c_int,
    msg: *mut *const pam_message,
    response: *mut *mut pam_response,
    appdata_ptr: *mut c_void,
) -> c_int {
    if (1..=PAM_MAX_NUM_MSG).contains(&num_msg) && !msg.is_null() {
        // SAFETY: Linux-PAM supplies num_msg pointers for the duration of the
        // conversation callback.
        let messages = unsafe { std::slice::from_raw_parts(msg, num_msg as usize) };
        if messages.iter().any(|message| {
            let message = *message;
            !message.is_null()
                // SAFETY: non-null message pointers are owned by Linux-PAM and
                // remain valid for the duration of this callback.
                && unsafe { (*message).msg_style == PAM_PROMPT_ECHO_OFF }
        }) {
            eprintln!(
                "{}",
                gettext(
                    "Querying SBI information requires elevated privileges; enter your password to continue."
                )
            );
        }
    }

    // SAFETY: forward the unmodified Linux-PAM conversation arguments to
    // libpam_misc, which performs the actual terminal interaction.
    unsafe { misc_conv(num_msg, msg, response, appdata_ptr) }
}

pub(crate) fn run(include_legacy: bool) -> Result<(), String> {
    // Match the invoking user's locale before PAM translates its messages.
    // SAFETY: run is called once, before any PAM transaction or thread exists.
    unsafe {
        libc::setlocale(libc::LC_ALL, c"".as_ptr());
    }
    init_gettext()?;

    let (real_uid, effective_uid) = (Uid::current(), Uid::effective());

    if !effective_uid.is_root() {
        print_install_instructions();
        return Ok(());
    }

    verify_privileged_install()?;

    let _pam_session = if !real_uid.is_root() {
        let username = username_for_uid(real_uid)?;
        Some(authenticate(&username)?)
    } else {
        None
    };

    // Block catchable signals only for the short load/read/unload window so a
    // normal Ctrl-C cannot strand the probe module. SIGKILL cannot be blocked.
    let signals = SignalMaskGuard::block_all()?;
    let extensions = sbi_ext::selected(include_legacy);
    let fwft_features = fwft::features();
    let extension_parameters = sbi_ext::module_parameters(&extensions);
    let module_parameters = CString::new(format!(
        "{} {}",
        extension_parameters.to_str().unwrap(),
        fwft::module_parameters(&fwft_features)
    ))
    .unwrap();
    let mut loaded = load_module(&module_parameters)?;

    let base = BaseInfo::read()?;
    let extension_values = read_parameter_list("extension_values")?;
    if extension_values.len() != extensions.len() {
        return Err(format!(
            "kernel probe returned {} extension values; expected {}",
            extension_values.len(),
            extensions.len()
        ));
    }
    let fwft_results = if extensions
        .iter()
        .zip(&extension_values)
        .any(|(extension, value)| extension.id == EID_FWFT && *value != 0)
    {
        let errors = read_parameter_list("fwft_errors")?;
        let values = read_parameter_list("fwft_values")?;
        if errors.len() != fwft_features.len() || values.len() != fwft_features.len() {
            return Err(format!(
                "kernel probe returned {} FWFT errors and {} values; expected {}",
                errors.len(),
                values.len(),
                fwft_features.len()
            ));
        }
        Some((errors, values))
    } else {
        None
    };

    loaded.unload()?;
    drop(signals);
    print_result(&base, &extensions, &extension_values);
    if let Some((errors, values)) = fwft_results {
        print_fwft(&fwft_features, &errors, &values);
    }
    Ok(())
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

fn print_install_instructions() {
    println!(
        "{}",
        gettext(
            "Build complete, but this executable is not setuid root; no privileged operation was performed."
        )
    );
    println!(
        "{}",
        gettext(
            "The module is embedded; follow the README to install it at /usr/local/sbin/lssbi with owner root:sudo and mode 4750."
        )
    );
}

fn verify_privileged_install() -> Result<(), String> {
    let executable = fs::canonicalize("/proc/self/exe")
        .map_err(|e| format!("cannot resolve the running executable: {e}"))?;
    let metadata = fs::metadata("/proc/self/exe")
        .map_err(|e| format!("cannot inspect the running executable: {e}"))?;
    if metadata.uid() != 0 || metadata.mode() & 0o4022 != 0o4000 {
        return Err("executable must be root-owned, setuid, and not group/other-writable".into());
    }

    for directory in executable
        .parent()
        .into_iter()
        .flat_map(|path| path.ancestors())
    {
        let metadata = fs::metadata(directory)
            .map_err(|e| format!("cannot inspect {}: {e}", directory.display()))?;
        if metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
            return Err(format!(
                "{} must be root-owned and not group/other-writable",
                directory.display()
            ));
        }
    }
    Ok(())
}

fn username_for_uid(uid: Uid) -> Result<CString, String> {
    let user = User::from_uid(uid)
        .map_err(|error| format!("cannot resolve real uid {}: {error}", uid.as_raw()))?
        .ok_or_else(|| format!("no account exists for real uid {}", uid.as_raw()))?;
    CString::new(user.name).map_err(|_| "account name contains an embedded NUL".into())
}

fn authenticate(username: &CString) -> Result<PamSession, String> {
    let conversation = pam_conv {
        conv: Some(conversation),
        appdata_ptr: ptr::null_mut(),
    };
    let mut handle: *mut pam_handle_t = ptr::null_mut();

    // SAFETY: all strings and the conversation object remain live until
    // pam_end, and handle is initialized as required by pam_start.
    let mut status = unsafe {
        pam_start(
            c"lssbi".as_ptr(),
            username.as_ptr(),
            &conversation,
            &mut handle,
        )
    };
    if status == PAM_SUCCESS {
        // libpam_misc performs terminal echo suppression and passes the secret
        // directly to PAM; this Rust program never stores the password.
        status = unsafe { pam_authenticate(handle, 0) };
    }
    if status == PAM_SUCCESS {
        status = unsafe { pam_acct_mgmt(handle, 0) };
    }
    if status == PAM_SUCCESS {
        status = unsafe { pam_open_session(handle, 0) };
    }

    let message = if status == PAM_SUCCESS {
        String::new()
    } else {
        // SAFETY: pam_strerror returns a PAM-owned NUL-terminated string.
        unsafe { CStr::from_ptr(pam_strerror(handle, status)) }
            .to_string_lossy()
            .into_owned()
    };
    if status != PAM_SUCCESS && !handle.is_null() {
        // SAFETY: handle was created by pam_start and is ended exactly once.
        unsafe { pam_end(handle, status) };
    }

    if status == PAM_SUCCESS {
        Ok(PamSession { handle })
    } else {
        Err(format!("PAM authentication failed: {message}"))
    }
}

struct PamSession {
    handle: *mut pam_handle_t,
}

impl Drop for PamSession {
    fn drop(&mut self) {
        // SAFETY: authenticate opened this session and transferred its sole
        // live handle to this guard.
        let status = unsafe { pam_close_session(self.handle, 0) };
        unsafe { pam_end(self.handle, status) };
    }
}

struct SignalMaskGuard {
    old: SigSet,
}

impl SignalMaskGuard {
    fn block_all() -> Result<Self, String> {
        let old = SigSet::all()
            .thread_swap_mask(SigmaskHow::SIG_BLOCK)
            .map_err(|error| format!("cannot block signals: {error}"))?;
        Ok(Self { old })
    }
}

impl Drop for SignalMaskGuard {
    fn drop(&mut self) {
        let _ = self.old.thread_set_mask();
    }
}

struct LoadedModule {
    active: bool,
}

fn load_module(parameters: &CStr) -> Result<LoadedModule, String> {
    // SAFETY: the embedded byte slice and NUL-terminated parameters stay valid
    // for this synchronous syscall.
    let result = unsafe {
        libc::syscall(
            libc::SYS_init_module,
            EMBEDDED_MODULE.as_ptr(),
            EMBEDDED_MODULE.len(),
            parameters.as_ptr(),
        )
    };
    if result != 0 {
        return Err(format!(
            "init_module failed: {} (a stale {MODULE_NAME} module may already be loaded)",
            io::Error::last_os_error()
        ));
    }
    Ok(LoadedModule { active: true })
}

impl LoadedModule {
    fn unload(&mut self) -> Result<(), String> {
        if !self.active {
            return Ok(());
        }
        // SAFETY: the name is NUL-terminated and flags=0 requests a normal unload.
        let result =
            unsafe { libc::syscall(libc::SYS_delete_module, c"sbi_probe".as_ptr(), 0 as c_int) };
        if result != 0 {
            return Err(format!(
                "delete_module({MODULE_NAME}) failed: {}",
                io::Error::last_os_error()
            ));
        }
        self.active = false;
        Ok(())
    }
}

impl Drop for LoadedModule {
    fn drop(&mut self) {
        let _ = self.unload();
    }
}

fn read_parameter(name: &str) -> Result<i64, String> {
    let path = PathBuf::from("/sys/module")
        .join(MODULE_NAME)
        .join("parameters")
        .join(name);
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    text.trim()
        .parse()
        .map_err(|e| format!("invalid value in {}: {e}", path.display()))
}

fn read_parameter_list(name: &str) -> Result<Vec<i64>, String> {
    let path = PathBuf::from("/sys/module")
        .join(MODULE_NAME)
        .join("parameters")
        .join(name);
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    text.trim()
        .split(',')
        .map(|value| {
            value
                .parse()
                .map_err(|e| format!("invalid value in {}: {e}", path.display()))
        })
        .collect()
}

struct BaseInfo {
    spec_raw: i64,
    impl_id: i64,
    impl_version: i64,
    mvendorid: i64,
    marchid: i64,
    mimpid: i64,
}

impl BaseInfo {
    fn read() -> Result<Self, String> {
        Ok(Self {
            spec_raw: read_parameter("spec_raw")?,
            impl_id: read_parameter("impl_id")?,
            impl_version: read_parameter("impl_version")?,
            mvendorid: read_parameter("mvendorid")?,
            marchid: read_parameter("marchid")?,
            mimpid: read_parameter("mimpid")?,
        })
    }
}

fn print_result(base: &BaseInfo, extensions: &[sbi_ext::Extension], extension_values: &[i64]) {
    let spec_raw = base.spec_raw as usize;
    let spec = Version::from_raw(spec_raw);
    let impl_id = base.impl_id as usize;
    let version = base.impl_version as usize;
    let (mvendorid, marchid, mimpid) = (
        base.mvendorid as usize,
        base.marchid as usize,
        base.mimpid as usize,
    );

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
        "{}: v{} ({raw} {version:#x})",
        gettext("SBI implementation version"),
        sbi_impl::version(impl_id, version)
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
    println!("{}:", gettext("SBI extensions"));
    let extension_width = extensions
        .iter()
        .map(|extension| UnicodeWidthStr::width(extension.name.as_str()) + 2)
        .max()
        .unwrap_or(0)
        .clamp(32, 48);
    for (extension, value) in extensions.iter().zip(extension_values) {
        let label = format!("{}:", extension.name);
        let status = if *value != 0 {
            gettext("Supported")
        } else {
            gettext("Not supported")
        };
        print_status(&label, &status, extension_width);
    }
    println!("{}:", gettext("Vulnerabilities"));
    let status = gettext(if vuln::pmu2_crash(impl_id, version) {
        "Affected"
    } else {
        "Not affected"
    });
    let label = format!("{} (CVE-2025-63913):", vuln::pmu2_crash_name());
    print_status(&label, &status, 32);
}

fn print_status(label: &str, status: &str, width: usize) {
    let padding = width.saturating_sub(UnicodeWidthStr::width(label)).max(1);
    println!("  {label}{}{status}", " ".repeat(padding));
}

fn print_fwft(features: &[fwft::Feature], errors: &[i64], values: &[i64]) {
    println!("{}:", gettext("Firmware Features"));
    let width = features
        .iter()
        .map(|feature| UnicodeWidthStr::width(feature.name.as_str()) + 2)
        .max()
        .unwrap_or(0)
        .clamp(32, 48);
    for ((feature, error), value) in features.iter().zip(errors).zip(values) {
        let label = format!("{}:", feature.name);
        let status = if *error != 0 {
            gettext("Not supported")
        } else {
            match feature.kind {
                fwft::Kind::Boolean => gettext(if *value != 0 {
                    "Supported"
                } else {
                    "Not supported"
                }),
                fwft::Kind::Pmlen => value.to_string(),
            }
        };
        print_status(&label, &status, width);
    }
}
