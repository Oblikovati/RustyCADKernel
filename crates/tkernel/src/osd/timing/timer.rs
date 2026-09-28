// Created on: 1992-12-04
// Created by: Didier PIFFAULT , Mireille MERCIEN
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

//! `OSD_Timer`: an `OSD_Chronometer` that also accumulates wall-clock time.
//!
//! C++ inheritance becomes composition: the timer owns its chronometer, exposes the
//! chronometer's read-only members through `Deref`, and re-implements the virtual
//! members it overrides in the native call order.
use super::{Chronometer, CpuTimes, NativeWallClock, WallClock, c_format::format_g};
use crate::standard::Failure;
use std::{ops::Deref, sync::Arc};

/// `OSD_Timer`; `let mut timer = Timer::new(false); timer.start(); timer.elapsed_time()`.
#[derive(Clone)]
pub struct Timer {
    chronometer: Chronometer,
    time_start: f64,
    time_cumul: f64,
    wall: Arc<dyn WallClock>,
}
impl Timer {
    /// `GetWallClockTime`: native wall-clock seconds; `Timer::get_wall_clock_time()`.
    pub fn get_wall_clock_time() -> f64 {
        NativeWallClock.now()
    }
    /// Builds a stopped timer on the native clocks; `Timer::new(false)`. The flag only
    /// selects the chronometer's CPU scope, never the elapsed time.
    pub fn new(this_thread_only: bool) -> Self {
        Self::with_clocks(
            Chronometer::new(this_thread_only),
            Arc::new(NativeWallClock),
        )
    }
    /// Builds a stopped timer on injected clocks; `Timer::with_clocks(chronometer, wall)`.
    pub fn with_clocks(chronometer: Chronometer, wall: Arc<dyn WallClock>) -> Self {
        let mut timer: Self = Self {
            chronometer,
            time_start: 0.0,
            time_cumul: 0.0,
            wall,
        };
        timer.chronometer.reset();
        timer
    }
    /// Builds a stopped timer whose chronometer reads `cpu`; `Timer::with_cpu(false, cpu, wall)`.
    pub fn with_cpu(
        this_thread_only: bool,
        cpu: Arc<dyn CpuTimes>,
        wall: Arc<dyn WallClock>,
    ) -> Self {
        Self::with_clocks(Chronometer::with_clock(this_thread_only, cpu), wall)
    }
    /// `Reset(double)`: stops and restarts from `elapsed_seconds`; `timer.reset_to(3725.5)`.
    pub const fn reset_to(&mut self, elapsed_seconds: f64) {
        self.time_start = 0.0;
        self.time_cumul = elapsed_seconds;
        self.chronometer.reset();
    }
    /// `Reset()`: stops and zeroes the timer; `timer.reset()`.
    pub const fn reset(&mut self) {
        self.reset_to(0.0);
    }
    /// `Restart()`: zeroes and starts. Native order kept: the wall clock is read, then the
    /// inherited restart calls the overridden `Reset` and `Start`, which reads it again;
    /// `timer.restart()`.
    pub fn restart(&mut self) {
        self.time_start = self.wall.now();
        self.time_cumul = 0.0;
        self.reset();
        self.start();
    }
    /// `Show()`: prints [`Timer::show_to`] on standard output; `timer.show()`.
    pub fn show(&self) {
        print!("{}", self.show_to());
    }
    /// `Show(Standard_OStream&)`: elapsed hours, minutes and seconds, then the CPU lines,
    /// at stream precision 12; `Timer::new(false).show_to()` starts "Elapsed time: 0 Hours".
    pub fn show_to(&self) -> String {
        let (hours, minutes, seconds) = hours_minutes_seconds(self.elapsed_time());
        format!(
            "Elapsed time: {hours} Hours {minutes} Minutes {} Seconds\n{}",
            format_g(seconds, 12),
            self.chronometer.show_to()
        )
    }
    /// `Show(double&, int&, int&, double&)`: (seconds, minutes, hours, CPU user seconds);
    /// `let (seconds, minutes, hours, cpu) = timer.show_values();`.
    pub fn show_values(&self) -> (f64, i32, i32, f64) {
        let (hours, minutes, seconds) = hours_minutes_seconds(self.elapsed_time());
        (seconds, minutes, hours, self.chronometer.show_user())
    }
    /// `Stop()`: accumulates the running lap; a stopped timer ignores it; `timer.stop()`.
    pub fn stop(&mut self) {
        if self.chronometer.is_started() {
            self.time_cumul += self.wall.now() - self.time_start;
            self.chronometer.stop();
        }
    }
    /// `Start()`: starts or resumes; a running timer ignores it; `timer.start()`.
    pub fn start(&mut self) {
        if !self.chronometer.is_started() {
            self.time_start = self.wall.now();
            self.chronometer.start();
        }
    }
    /// `ElapsedTime()`: accumulated wall seconds, including a running lap; `timer.elapsed_time()`.
    pub fn elapsed_time(&self) -> f64 {
        if !self.chronometer.is_started() {
            return self.time_cumul;
        }
        self.time_cumul + self.wall.now() - self.time_start
    }
    /// The inherited `SetThisThreadOnly`; fails while running; `timer.set_this_thread_only(true)?`.
    pub fn set_this_thread_only(&mut self, this_thread_only: bool) -> Result<(), Failure> {
        self.chronometer.set_this_thread_only(this_thread_only)
    }
}
impl Deref for Timer {
    type Target = Chronometer;
    /// The inherited read-only chronometer members (`is_started`, `user_time_cpu`, ...).
    fn deref(&self) -> &Chronometer {
        &self.chronometer
    }
}
impl Default for Timer {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Native `timeToHoursMinutesSeconds`: truncates to whole seconds in `int`, splits hours
/// and minutes in integer arithmetic, and keeps the fractional remainder in the seconds.
pub(crate) fn hours_minutes_seconds(time_seconds: f64) -> (i32, i32, f64) {
    let whole: i32 = truncate_to_int(time_seconds);
    let hours: i32 = whole / 3600;
    let minutes: i32 = (whole - hours * 3600) / 60;
    let seconds: f64 = time_seconds - f64::from(hours * 3600) - f64::from(minutes * 60);
    (hours, minutes, seconds)
}

/// The `(int)` cast of the native build. Out-of-range and NaN casts are undefined in C++;
/// x86 `cvttsd2si` yields `INT_MIN` for them (kept on x86), other targets saturate like
/// Rust's `as`.
fn truncate_to_int(value: f64) -> i32 {
    let in_range: bool = value > -2_147_483_649.0 && value < 2_147_483_648.0;
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) && !in_range {
        return i32::MIN;
    }
    value as i32
}
