// Created on: 1992-11-16
// Created by: Mireille MERCIEN
// Copyright (c) 1992-1999 Matra Datavision
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

//! `OSD_Chronometer`: cumulative CPU time of the process or of the calling thread.
use super::{CpuTimes, NativeCpuTimes, c_format::format_g};
use crate::standard::Failure;
use std::sync::Arc;

/// `OSD_Chronometer`; `let mut chrono = Chronometer::new(false); chrono.start();`.
#[derive(Clone)]
pub struct Chronometer {
    start_cpu_user: f64,
    start_cpu_sys: f64,
    cumul_cpu_user: f64,
    cumul_cpu_sys: f64,
    is_stopped: bool,
    is_thread_only: bool,
    clock: Arc<dyn CpuTimes>,
}
impl Chronometer {
    /// Builds a stopped chronometer on the native CPU clocks; `Chronometer::new(false)`
    /// measures the whole process, `true` the calling thread only.
    pub fn new(this_thread_only: bool) -> Self {
        Self::with_clock(this_thread_only, Arc::new(NativeCpuTimes))
    }
    /// Builds a stopped chronometer on an injected clock; `Chronometer::with_clock(false, clock)`.
    pub fn with_clock(this_thread_only: bool, clock: Arc<dyn CpuTimes>) -> Self {
        Self {
            start_cpu_user: 0.0,
            start_cpu_sys: 0.0,
            cumul_cpu_user: 0.0,
            cumul_cpu_sys: 0.0,
            is_stopped: true,
            is_thread_only: this_thread_only,
            clock,
        }
    }
    /// `GetProcessCPU` on the native clocks: (user, system) seconds; `Chronometer::get_process_cpu()`.
    pub fn get_process_cpu() -> (f64, f64) {
        NativeCpuTimes.process_cpu()
    }
    /// `GetThreadCPU` on the native clocks: (user, system) seconds (Linux reports system 0);
    /// `Chronometer::get_thread_cpu()`.
    pub fn get_thread_cpu() -> (f64, f64) {
        NativeCpuTimes.thread_cpu()
    }
    /// Tells whether the chronometer runs; `chrono.is_started()`.
    pub const fn is_started(&self) -> bool {
        !self.is_stopped
    }
    /// Stops and zeroes the chronometer; `chrono.reset()`.
    pub const fn reset(&mut self) {
        self.is_stopped = true;
        self.start_cpu_user = 0.0;
        self.start_cpu_sys = 0.0;
        self.cumul_cpu_user = 0.0;
        self.cumul_cpu_sys = 0.0;
    }
    /// Resets then starts; `chrono.restart()`.
    pub fn restart(&mut self) {
        self.reset();
        self.start();
    }
    /// Adds the CPU time since the last start; a stopped chronometer ignores it; `chrono.stop()`.
    pub fn stop(&mut self) {
        if self.is_stopped {
            return;
        }
        let (user, sys) = self.read_clock();
        self.cumul_cpu_user += user - self.start_cpu_user;
        self.cumul_cpu_sys += sys - self.start_cpu_sys;
        self.is_stopped = true;
    }
    /// Starts after construction/reset or resumes after stop; `chrono.start()`.
    pub fn start(&mut self) {
        if !self.is_stopped {
            return;
        }
        (self.start_cpu_user, self.start_cpu_sys) = self.read_clock();
        self.is_stopped = false;
    }
    /// `Show()`: prints [`Chronometer::show_to`] on standard output; `chrono.show()`.
    pub fn show(&self) {
        print!("{}", self.show_to());
    }
    /// `Show(Standard_OStream&)`: the two CPU lines at stream precision 12;
    /// `chrono.show_to() == "CPU user time: 0 seconds\nCPU system time: 0 seconds\n"`.
    pub fn show_to(&self) -> String {
        let (user, sys) = self.show_cpu();
        format!(
            "CPU user time: {} seconds\nCPU system time: {} seconds\n",
            format_g(user, 12),
            format_g(sys, 12)
        )
    }
    /// `UserTimeCPU()`: cumulative user seconds, including a running lap; `chrono.user_time_cpu()`.
    pub fn user_time_cpu(&self) -> f64 {
        self.show_cpu().0
    }
    /// `SystemTimeCPU()`: cumulative system seconds; `chrono.system_time_cpu()`.
    pub fn system_time_cpu(&self) -> f64 {
        self.show_cpu().1
    }
    /// `Show(double&)`: the user seconds, as `user_time_cpu`; `chrono.show_user()`.
    pub fn show_user(&self) -> f64 {
        self.user_time_cpu()
    }
    /// `Show(double&, double&)`: (user, system) cumulative seconds, adding the running lap
    /// read from the clock; `let (user, sys) = chrono.show_cpu();`.
    pub fn show_cpu(&self) -> (f64, f64) {
        let (mut user, mut sys) = (self.cumul_cpu_user, self.cumul_cpu_sys);
        if self.is_stopped {
            return (user, sys);
        }
        let (current_user, current_sys) = self.read_clock();
        user += current_user - self.start_cpu_user;
        sys += current_sys - self.start_cpu_sys;
        (user, sys)
    }
    /// Tells whether only the calling thread is measured; `chrono.is_this_thread_only()`.
    pub const fn is_this_thread_only(&self) -> bool {
        self.is_thread_only
    }
    /// Switches between thread and process time; fails with `Standard_ProgramError` while
    /// running; `chrono.set_this_thread_only(true)?`.
    pub fn set_this_thread_only(&mut self, this_thread_only: bool) -> Result<(), Failure> {
        if !self.is_stopped {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "OSD_Chronometer::SetThreadOnly() called for started Timer",
                "",
            ));
        }
        self.is_thread_only = this_thread_only;
        Ok(())
    }
    fn read_clock(&self) -> (f64, f64) {
        if self.is_thread_only {
            self.clock.thread_cpu()
        } else {
            self.clock.process_cpu()
        }
    }
}
impl Default for Chronometer {
    fn default() -> Self {
        Self::new(false)
    }
}
impl std::fmt::Debug for Chronometer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Chronometer")
            .field("cumul_cpu_user", &self.cumul_cpu_user)
            .field("cumul_cpu_sys", &self.cumul_cpu_sys)
            .field("is_stopped", &self.is_stopped)
            .field("is_thread_only", &self.is_thread_only)
            .finish_non_exhaustive()
    }
}
