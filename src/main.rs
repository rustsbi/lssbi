// SPDX-License-Identifier: MIT OR MulanPSL-2.0

#![cfg_attr(not(target_arch = "riscv64"), allow(dead_code))]

#[cfg(not(target_arch = "riscv64"))]
compile_error!("sbi-info is intentionally restricted to riscv64 Linux");

use std::ffi::{CStr, CString, c_char, c_int, c_long, c_void};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::ptr;

const MODULE_NAME: &str = "sbi_probe";
const EMBEDDED_MODULE: &[u8] = include_bytes!("../kernel/sbi_probe.ko");

// asm-generic syscall numbers, used by riscv64 Linux.
const SYS_INIT_MODULE: c_long = 105;
const SYS_DELETE_MODULE: c_long = 106;

const PAM_SUCCESS: c_int = 0;
const SIG_BLOCK: c_int = 0;
const SIG_SETMASK: c_int = 2;

#[repr(C)]
struct PamHandle {
    _private: [u8; 0],
}

#[repr(C)]
struct PamMessage {
    msg_style: c_int,
    msg: *const c_char,
}

#[repr(C)]
struct PamResponse {
    resp: *mut c_char,
    resp_retcode: c_int,
}

type PamConvFn = unsafe extern "C" fn(
    c_int,
    *mut *const PamMessage,
    *mut *mut PamResponse,
    *mut c_void,
) -> c_int;

#[repr(C)]
struct PamConv {
    conv: Option<PamConvFn>,
    appdata_ptr: *mut c_void,
}

#[repr(C)]
struct Passwd {
    pw_name: *mut c_char,
    pw_passwd: *mut c_char,
    pw_uid: u32,
    pw_gid: u32,
    pw_gecos: *mut c_char,
    pw_dir: *mut c_char,
    pw_shell: *mut c_char,
}

// glibc uses 1024 signal bits for sigset_t on Linux.
#[repr(C)]
struct SigSet {
    words: [u64; 16],
}

unsafe extern "C" {
    fn getuid() -> u32;
    fn geteuid() -> u32;
    fn getpwuid(uid: u32) -> *mut Passwd;
    fn syscall(number: c_long, ...) -> c_long;

    fn sigfillset(set: *mut SigSet) -> c_int;
    fn sigprocmask(how: c_int, set: *const SigSet, oldset: *mut SigSet) -> c_int;

    fn misc_conv(
        num_msg: c_int,
        msg: *mut *const PamMessage,
        response: *mut *mut PamResponse,
        appdata_ptr: *mut c_void,
    ) -> c_int;
    fn pam_start(
        service_name: *const c_char,
        user: *const c_char,
        pam_conversation: *const PamConv,
        pamh: *mut *mut PamHandle,
    ) -> c_int;
    fn pam_authenticate(pamh: *mut PamHandle, flags: c_int) -> c_int;
    fn pam_acct_mgmt(pamh: *mut PamHandle, flags: c_int) -> c_int;
    fn pam_end(pamh: *mut PamHandle, status: c_int) -> c_int;
    fn pam_strerror(pamh: *mut PamHandle, status: c_int) -> *const c_char;
}

fn main() {
    if let Err(error) = run() {
        eprintln!("sbi-info: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    // SAFETY: getuid/geteuid take no pointers and have no preconditions.
    let (real_uid, effective_uid) = unsafe { (getuid(), geteuid()) };

    if effective_uid != 0 {
        print_install_instructions();
        return Ok(());
    }

    verify_privileged_install()?;

    if real_uid != 0 {
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

fn username_for_uid(uid: u32) -> Result<CString, String> {
    // SAFETY: getpwuid returns NSS-owned storage. This single-threaded program
    // copies pw_name before invoking PAM or making another NSS call.
    let record = unsafe { getpwuid(uid) };
    if record.is_null() {
        return Err(format!("no account exists for real uid {uid}"));
    }
    // SAFETY: a successful getpwuid result has a NUL-terminated pw_name.
    let name = unsafe { CStr::from_ptr((*record).pw_name) }
        .to_bytes()
        .to_vec();
    CString::new(name).map_err(|_| "account name contains an embedded NUL".into())
}

fn authenticate(username: &CString) -> Result<(), String> {
    let conversation = PamConv {
        conv: Some(misc_conv),
        appdata_ptr: ptr::null_mut(),
    };
    let mut handle: *mut PamHandle = ptr::null_mut();

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
        let mut all = SigSet { words: [0; 16] };
        let mut old = SigSet { words: [0; 16] };
        // SAFETY: both objects are valid sigset_t-compatible storage.
        if unsafe { sigfillset(&mut all) } != 0
            || unsafe { sigprocmask(SIG_BLOCK, &all, &mut old) } != 0
        {
            return Err(format!(
                "cannot block signals: {}",
                io::Error::last_os_error()
            ));
        }
        Ok(Self { old })
    }
}

impl Drop for SignalMaskGuard {
    fn drop(&mut self) {
        // SAFETY: old was filled by sigprocmask in block_all.
        unsafe {
            sigprocmask(SIG_SETMASK, &self.old, ptr::null_mut());
        }
    }
}

struct LoadedModule {
    active: bool,
}

fn load_module() -> Result<LoadedModule, String> {
    // SAFETY: the embedded byte slice and NUL-terminated parameters stay valid
    // for this synchronous syscall; the number is riscv64 asm-generic.
    let result = unsafe {
        syscall(
            SYS_INIT_MODULE,
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
        let result = unsafe { syscall(SYS_DELETE_MODULE, c"sbi_probe".as_ptr(), 0 as c_int) };
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
    let spec = spec_raw as u64;
    let implementation = impl_id as u64;
    let version = impl_version as u64;

    println!(
        "SBI specification: v{}.{} (raw {spec:#x})",
        (spec >> 24) & 0x7f,
        spec & 0x00ff_ffff
    );
    println!(
        "SBI implementation: {} (ID {implementation:#x})",
        implementation_name(implementation)
    );
    println!("SBI implementation version: {version:#x}");

    if implementation == 1 {
        println!(
            "OpenSBI version: v{}.{}",
            (version >> 16) & 0xffff,
            version & 0xffff
        );
    }
}

fn implementation_name(id: u64) -> &'static str {
    match id {
        0 => "Berkeley Boot Loader (BBL)",
        1 => "OpenSBI",
        2 => "Xvisor",
        3 => "KVM",
        4 => "RustSBI",
        5 => "Diosix",
        6 => "Coffer",
        7 => "Xen Project",
        8 => "PolarFire Hart Software Services",
        9 => "coreboot",
        10 => "oreboot",
        11 => "bhyve",
        _ => "unknown",
    }
}
