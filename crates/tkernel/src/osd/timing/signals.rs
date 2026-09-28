// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! The `OSD` signal statics of OSD_signal.cxx: the recorded signal mode, the stack-trace
//! length, the floating-point trap switch and the Ctrl-C/Ctrl-Break flag.
//!
//! Native OCCT converts SIGSEGV, SIGFPE, SIGILL, SIGBUS, ... (and Windows structured
//! exceptions) into C++ exceptions by throwing or long-jumping out of the signal handler.
//! Rust cannot unwind out of a signal handler, so only the parts that stay sound are
//! ported: the SIGINT (Unix) / console Ctrl-C (Windows) handler that arms the flag
//! `ControlBreak` reads, restoring the default handlers for `OSD_SignalMode_Unset`, and the
//! floating-point trap control. Every other handler is left untouched.
use super::signal_native;
use crate::{
    osd::{SignalMode, exception_ctrl_break},
    standard::Failure,
};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// The host operations `OSD::SetSignal` performs; implement a recording fake for tests.
pub trait SignalHost {
    /// `SetFloatingSignal`: enables or disables SIGFPE on invalid, divide-by-zero and
    /// overflow for the calling thread; `host.set_floating_signal(true)`.
    fn set_floating_signal(&self, enable: bool);
    /// `ToCatchFloatingSignals`: whether those traps are enabled; `host.to_catch_floating_signals()`.
    fn to_catch_floating_signals(&self) -> bool;
    /// Installs (Set/SetUnhandled) or removes (Unset) the process handlers; `AsIs` changes
    /// nothing; `host.set_signal_handlers(SignalMode::Set)`.
    fn set_signal_handlers(&self, mode: SignalMode);
    /// Windows only: whether `CSF_DEBUG_MODE` is set; `host.debug_mode_requested()`.
    fn debug_mode_requested(&self) -> bool;
}

/// The native host: glibc `feenableexcept`/`fedisableexcept` (x86, x86-64, AArch64;
/// other Unix targets do nothing, as natively), `sigaction` for SIGINT and the Unset
/// defaults, and on Windows `_controlfp` and `SetConsoleCtrlHandler`; `NativeSignalHost`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeSignalHost;
impl SignalHost for NativeSignalHost {
    fn set_floating_signal(&self, enable: bool) {
        signal_native::set_floating_signal(enable);
    }
    fn to_catch_floating_signals(&self) -> bool {
        signal_native::to_catch_floating_signals()
    }
    fn set_signal_handlers(&self, mode: SignalMode) {
        signal_native::set_signal_handlers(mode);
    }
    fn debug_mode_requested(&self) -> bool {
        std::env::var_os("CSF_DEBUG_MODE").is_some()
    }
}

/// The process-wide signal statics; `SignalState::global()` backs the `Osd` statics and
/// the native handlers, `SignalState::new()` isolates a test.
#[derive(Debug)]
pub struct SignalState {
    mode: AtomicI32,
    stack_trace_length: AtomicI32,
    ctrl_break: AtomicBool,
}
static GLOBAL_SIGNAL_STATE: SignalState = SignalState::new();

impl Default for SignalState {
    fn default() -> Self {
        Self::new()
    }
}
impl SignalState {
    /// Starts with `OSD_SignalMode_AsIs`, no stack trace and no pending break; `SignalState::new()`.
    pub const fn new() -> Self {
        Self {
            mode: AtomicI32::new(SignalMode::AsIs as i32),
            stack_trace_length: AtomicI32::new(0),
            ctrl_break: AtomicBool::new(false),
        }
    }
    /// The process state the native handlers arm; `SignalState::global().signal_mode()`.
    pub fn global() -> &'static Self {
        &GLOBAL_SIGNAL_STATE
    }
    /// `SignalMode()`: the mode of the last `set_signal`; `state.signal_mode()`.
    pub fn signal_mode(&self) -> SignalMode {
        SignalMode::from_raw(self.mode.load(Ordering::SeqCst)).unwrap_or(SignalMode::AsIs)
    }
    /// `SignalStackTraceLength()`; `state.signal_stack_trace_length()`.
    pub fn signal_stack_trace_length(&self) -> i32 {
        self.stack_trace_length.load(Ordering::SeqCst)
    }
    /// `SetSignalStackTraceLength(int)`: stored as given, even negative; `state.set_signal_stack_trace_length(10)`.
    pub fn set_signal_stack_trace_length(&self, length: i32) {
        self.stack_trace_length.store(length, Ordering::SeqCst);
    }
    /// What the SIGINT (Unix) or console Ctrl-C/Ctrl-Break (Windows) handler does:
    /// arm the break flag; async-signal-safe; `state.arm_control_break()`.
    pub fn arm_control_break(&self) {
        self.ctrl_break.store(true, Ordering::SeqCst);
    }
    /// `ControlBreak()`: consumes an armed break as `OSD_Exception_CTRL_BREAK`
    /// "*** INTERRUPT ***"; `state.control_break()?`.
    pub fn control_break(&self) -> Result<(), Failure> {
        if self.ctrl_break.swap(false, Ordering::SeqCst) {
            return Err(exception_ctrl_break("*** INTERRUPT ***"));
        }
        Ok(())
    }
    /// `SetThreadLocalSignal`: on every host only the floating-point switch is ported (the
    /// Windows `_set_se_translator` is not); `state.set_thread_local_signal(&host, mode, true)`.
    pub fn set_thread_local_signal(
        &self,
        host: &dyn SignalHost,
        _mode: SignalMode,
        floating: bool,
    ) {
        host.set_floating_signal(floating);
    }
    /// `SetSignal(mode, floating)` in the host's native order; `state.set_signal(&host, SignalMode::Set, true)`.
    ///
    /// Unix: floating switch, record the mode, then (unless `AsIs`) the handlers.
    /// Windows: record the mode, honour `CSF_DEBUG_MODE` (message and a stack trace of 10
    /// when none is set), the handlers, clear the break flag, then the thread-local switch.
    pub fn set_signal(&self, host: &dyn SignalHost, mode: SignalMode, floating: bool) {
        if cfg!(windows) {
            self.set_signal_windows(host, mode, floating);
            return;
        }
        host.set_floating_signal(floating);
        self.mode.store(mode as i32, Ordering::SeqCst);
        if mode != SignalMode::AsIs {
            host.set_signal_handlers(mode);
        }
    }
    fn set_signal_windows(&self, host: &dyn SignalHost, mode: SignalMode, floating: bool) {
        self.mode.store(mode as i32, Ordering::SeqCst);
        if host.debug_mode_requested() {
            println!("Environment variable CSF_DEBUG_MODE set.");
            let _ =
                self.stack_trace_length
                    .compare_exchange(0, 10, Ordering::SeqCst, Ordering::SeqCst);
        }
        host.set_signal_handlers(mode);
        self.ctrl_break.store(false, Ordering::SeqCst);
        self.set_thread_local_signal(host, mode, floating);
    }
}
