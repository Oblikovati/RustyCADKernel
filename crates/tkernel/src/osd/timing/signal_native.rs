//! Narrow platform FFI boundary for the portable parts of OSD_signal.cxx.
#![allow(unsafe_code)]
use super::SignalState;
use crate::osd::SignalMode;

// glibc floating-point trap control (GNU extensions, not bound by the libc crate).
#[cfg(all(
    target_os = "linux",
    target_env = "gnu",
    any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")
))]
mod fenv {
    unsafe extern "C" {
        pub fn feclearexcept(excepts: i32) -> i32;
        pub fn feenableexcept(excepts: i32) -> i32;
        pub fn fedisableexcept(excepts: i32) -> i32;
        pub fn fegetexcept() -> i32;
    }
    // <fenv.h> values; `_OSD_FPX` is FE_INVALID | FE_DIVBYZERO | FE_OVERFLOW.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub const OSD_FPX: i32 = 0x01 | 0x04 | 0x08;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub const FE_ALL_EXCEPT: i32 = 0x3f;
    #[cfg(target_arch = "aarch64")]
    pub const OSD_FPX: i32 = 0x01 | 0x02 | 0x04;
    #[cfg(target_arch = "aarch64")]
    pub const FE_ALL_EXCEPT: i32 = 0x1f;
}

#[cfg(all(
    target_os = "linux",
    target_env = "gnu",
    any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")
))]
pub(super) fn set_floating_signal(enable: bool) {
    // SAFETY: the fenv calls only change the calling thread's floating-point environment.
    unsafe {
        fenv::feclearexcept(fenv::FE_ALL_EXCEPT);
        if enable {
            fenv::feenableexcept(fenv::OSD_FPX);
        } else {
            fenv::fedisableexcept(fenv::OSD_FPX);
        }
    }
}

#[cfg(all(
    target_os = "linux",
    target_env = "gnu",
    any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")
))]
pub(super) fn to_catch_floating_signals() -> bool {
    // SAFETY: fegetexcept only reads the calling thread's trap mask.
    (unsafe { fenv::fegetexcept() } & fenv::OSD_FPX) != 0
}

// Native `#else` branch (macOS, musl, other targets): no trap control, never catching.
#[cfg(all(
    unix,
    not(all(
        target_os = "linux",
        target_env = "gnu",
        any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")
    ))
))]
pub(super) fn set_floating_signal(_enable: bool) {}

#[cfg(all(
    unix,
    not(all(
        target_os = "linux",
        target_env = "gnu",
        any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")
    ))
))]
pub(super) fn to_catch_floating_signals() -> bool {
    false
}

/// The signals `OSD::SetSignal` manages on Unix, in native order (SIGSEGV last).
#[cfg(unix)]
const MANAGED_SIGNALS: [i32; 8] = [
    libc::SIGFPE,
    libc::SIGHUP,
    libc::SIGINT,
    libc::SIGQUIT,
    libc::SIGILL,
    libc::SIGBUS,
    libc::SIGSYS,
    libc::SIGSEGV,
];

#[cfg(unix)]
extern "C" fn interrupt_handler(
    _signal: i32,
    _info: *mut libc::siginfo_t,
    _context: *mut libc::c_void,
) {
    // Native SIGINT arm: set the flag, never long-jump ("we are out of context anyway").
    SignalState::global().arm_control_break();
}

#[cfg(unix)]
fn action(handler: libc::sighandler_t, flags: i32) -> libc::sigaction {
    // SAFETY: sigaction is plain data; all-zero is a valid empty value on every Unix.
    let mut value: libc::sigaction = unsafe { std::mem::zeroed() };
    value.sa_sigaction = handler;
    value.sa_flags = flags;
    // SAFETY: sa_mask is a valid, writable sigset_t.
    unsafe { libc::sigemptyset(&mut value.sa_mask) };
    value
}

/// Unix: `Unset` restores `SIG_DFL` on all eight managed signals (this also removes the
/// Rust runtime's SIGSEGV stack-overflow reporter, as it removes any handler natively);
/// `Set`/`SetUnhandled` install only the SIGINT break handler, and `SetUnhandled` puts a
/// previous non-default SIGINT handler back.
#[cfg(unix)]
pub(super) fn set_signal_handlers(mode: SignalMode) {
    match mode {
        SignalMode::AsIs => {}
        SignalMode::Unset => {
            let default: libc::sigaction = action(libc::SIG_DFL, libc::SA_RESTART);
            MANAGED_SIGNALS
                .iter()
                .for_each(|&signal| install(signal, &default, mode));
        }
        SignalMode::Set | SignalMode::SetUnhandled => {
            let handler: libc::sighandler_t = interrupt_handler as *const () as usize;
            let set: libc::sigaction = action(handler, libc::SA_RESTART | libc::SA_SIGINFO);
            install(libc::SIGINT, &set, mode);
        }
    }
}

#[cfg(unix)]
fn install(signal: i32, wanted: &libc::sigaction, mode: SignalMode) {
    let mut old: libc::sigaction = action(libc::SIG_DFL, 0);
    // SAFETY: both structures are valid for the call; the handler only stores an atomic.
    let status: i32 = unsafe { libc::sigaction(signal, wanted, &mut old) };
    if mode == SignalMode::SetUnhandled && status == 0 && old.sa_sigaction != libc::SIG_DFL {
        let mut replaced: libc::sigaction = action(libc::SIG_DFL, 0);
        // SAFETY: `old` came from the kernel for this signal and is valid to reinstall.
        unsafe { libc::sigaction(signal, &old, &mut replaced) };
    }
}

// Windows: the MSVC floating-point control word.
#[cfg(windows)]
mod float_control {
    unsafe extern "C" {
        pub fn _fpreset();
        pub fn _clearfp() -> u32;
        pub fn _controlfp(new: u32, mask: u32) -> u32;
    }
    // _EM_INVALID | _EM_DENORMAL | _EM_ZERODIVIDE | _EM_OVERFLOW; a zero bit raises.
    pub const OSD_FPX: u32 = 0x10 | 0x0008_0000 | 0x08 | 0x04;
}

#[cfg(windows)]
pub(super) fn set_floating_signal(enable: bool) {
    // SAFETY: these CRT calls only change the calling thread's floating-point state.
    unsafe {
        float_control::_fpreset();
        float_control::_clearfp();
        float_control::_controlfp(
            if enable { 0 } else { float_control::OSD_FPX },
            float_control::OSD_FPX,
        );
    }
}

#[cfg(windows)]
pub(super) fn to_catch_floating_signals() -> bool {
    // SAFETY: a zero mask only reads the control word.
    let control: u32 = unsafe { float_control::_controlfp(0, 0) };
    (float_control::OSD_FPX & !control) != 0
}

#[cfg(windows)]
unsafe extern "system" fn console_break_handler(code: u32) -> windows_sys::core::BOOL {
    use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT};
    if code == CTRL_C_EVENT || code == CTRL_BREAK_EVENT {
        // Native also calls MessageBeep(MB_ICONEXCLAMATION) here; the beep is not ported.
        SignalState::global().arm_control_break();
        return 1;
    }
    std::process::exit(254);
}

/// Windows: only the console Ctrl-C/Ctrl-Break handler is ported; the unhandled-exception
/// filter, the `signal()` translators and the SE translator would raise C++ exceptions.
#[cfg(windows)]
pub(super) fn set_signal_handlers(mode: SignalMode) {
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
    let add: windows_sys::core::BOOL = match mode {
        SignalMode::Set | SignalMode::SetUnhandled => 1,
        SignalMode::Unset => 0,
        SignalMode::AsIs => return,
    };
    // SAFETY: the handler is a 'static function valid for the process lifetime.
    unsafe { SetConsoleCtrlHandler(Some(console_break_handler), add) };
}
