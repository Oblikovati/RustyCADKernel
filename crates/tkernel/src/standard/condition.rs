// Created by: Kirill Gavrilov
// Copyright (c) 2018 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_Condition.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_Condition`: a manual-reset event shared between threads.
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// A boolean flag threads set, reset and wait on, like a WinAPI event; `condition.set()`.
pub struct StandardCondition {
    flag: Mutex<bool>,
    signal: Condvar,
}

impl StandardCondition {
    /// Creates the event in the given state; `StandardCondition::new(false)`.
    pub fn new(is_set: bool) -> Self {
        Self {
            flag: Mutex::new(is_set),
            signal: Condvar::new(),
        }
    }

    /// Signals the event and wakes every waiter; `condition.set()`.
    pub fn set(&self) {
        *self.flag() = true;
        self.signal.notify_all();
    }

    /// Clears the signal; `condition.reset()`.
    pub fn reset(&self) {
        *self.flag() = false;
    }

    /// Blocks until the event is signaled; `condition.wait()`.
    pub fn wait(&self) {
        let guard = self.flag();
        let _signaled = self
            .signal
            .wait_while(guard, |flag| !*flag)
            .unwrap_or_else(PoisonError::into_inner);
    }

    /// Waits at most `milliseconds` for the signal and reports it; `condition.wait_timeout(10)`.
    /// A zero or negative limit only tests the flag, as `wait_for` does with such durations.
    pub fn wait_timeout(&self, milliseconds: i32) -> bool {
        let limit = Duration::from_millis(u64::try_from(milliseconds).unwrap_or(0));
        let guard = self.flag();
        let (guard, _) = self
            .signal
            .wait_timeout_while(guard, limit, |flag| !*flag)
            .unwrap_or_else(PoisonError::into_inner);
        *guard
    }

    /// Reports the flag without waiting; `condition.check()`.
    pub fn check(&self) -> bool {
        *self.flag()
    }

    /// Clears the flag and reports whether it was signaled; `condition.check_reset()`.
    pub fn check_reset(&self) -> bool {
        std::mem::replace(&mut *self.flag(), false)
    }

    fn flag(&self) -> MutexGuard<'_, bool> {
        self.flag.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for StandardCondition {
    /// The header's default argument: not signaled; `StandardCondition::default().check() == false`.
    fn default() -> Self {
        Self::new(false)
    }
}
