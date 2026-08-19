// SPDX-License-Identifier: MIT OR MulanPSL-2.0

use crate::sbi_impl;
use libc::{c_int, c_void};
use nix::sys::signal::{SigSet, SigmaskHow};
use nix::unistd::{Uid, User};
use pam_sys::{
    PAM_SUCCESS, pam_acct_mgmt, pam_authenticate, pam_conv, pam_end, pam_handle_t, pam_message,
    pam_response, pam_start, pam_strerror,
};
use sbi_spec::base::Version;
use std::ffi::{CStr, CString};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::ptr;

const MODULE_NAME: &str = "sbi_probe";
const EMBEDDED_MODULE: &[u8] = include_bytes!("../kernel/sbi_probe.ko");

unsafe extern "C" {
    fn misc_conv(
        num_msg: c_int,
        msg: *mut *const pam_message,
        response: *mut *mut pam_response,
        appdata_ptr: *mut c_void,
    ) -> c_int;
}

pub(crate) fn run() -> Result<(), String> {
    let (real_uid, effective_uid) = (Uid::current(), Uid::effective());

    if !effective_uid.is_root() {
        print_install_instructions();
        return Ok(());
    }

    verify_privileged_install()?;

    if !real_uid.is_root() {
        let username = username_for_uid(real_uid)?;
        authenticate(&username)?;
    }

    // Block catchable signals only for the short load/read/unload window so a
    // normal Ctrl-C cannot strand the probe module. SIGKILL cannot be blocked.
    let signals = SignalMaskGuard::block_all()?;
    let mut loaded = load_module()?;

    let spec_raw = read_parameter("spec_raw")?;
    let impl_id = read_parameter("impl_id")?;
    let impl_version = read_parameter("impl_version")?;

    loaded.unload()?;
    drop(signals);
    print_result(spec_raw, impl_id, impl_version);
    Ok(())
}

fn print_install_instructions() {
    println!("编译完成，但当前文件没有 setuid-root，尚未执行任何特权操作。");
    println!("模块已内联；请按 README 将本程序安装到 root 管理的目录，并设置 root:sudo、4750。");
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

fn authenticate(username: &CString) -> Result<(), String> {
    let conversation = pam_conv {
        conv: Some(misc_conv),
        appdata_ptr: ptr::null_mut(),
    };
    let mut handle: *mut pam_handle_t = ptr::null_mut();

    // SAFETY: all strings and the conversation object remain live until
    // pam_end, and handle is initialized as required by pam_start.
    let mut status = unsafe {
        pam_start(
            c"sudo".as_ptr(),
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

    let message = if status == PAM_SUCCESS {
        String::new()
    } else {
        // SAFETY: pam_strerror returns a PAM-owned NUL-terminated string.
        unsafe { CStr::from_ptr(pam_strerror(handle, status)) }
            .to_string_lossy()
            .into_owned()
    };
    if !handle.is_null() {
        // SAFETY: handle was created by pam_start and is ended exactly once.
        unsafe { pam_end(handle, status) };
    }

    if status == PAM_SUCCESS {
        Ok(())
    } else {
        Err(format!("PAM authentication failed: {message}"))
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

fn load_module() -> Result<LoadedModule, String> {
    // SAFETY: the embedded byte slice and NUL-terminated parameters stay valid
    // for this synchronous syscall.
    let result = unsafe {
        libc::syscall(
            libc::SYS_init_module,
            EMBEDDED_MODULE.as_ptr(),
            EMBEDDED_MODULE.len(),
            c"".as_ptr(),
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

fn print_result(spec_raw: i64, impl_id: i64, impl_version: i64) {
    let spec_raw = spec_raw as usize;
    let spec = Version::from_raw(spec_raw);
    let implementation = impl_id as usize;
    let version = impl_version as usize;

    println!("SBI specification: v{spec} (raw {spec_raw:#x})");
    println!(
        "SBI implementation: {} (ID {implementation:#x})",
        sbi_impl::name(implementation)
    );
    println!(
        "SBI implementation version: v{} (raw {version:#x})",
        sbi_impl::version(implementation, version)
    );
}
