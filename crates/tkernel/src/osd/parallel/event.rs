// Created by: Kirill Gavrilov
// Copyright (c) 2018-2020 OPEN CASCADE SAS
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

//! The manual-reset event the pool uses (`Standard_Condition`, which the Standard package
//! port does not expose yet): `set` wakes every waiter and stays set until `reset`.
use std::{
    sync::{Condvar, Mutex, MutexGuard},
    time::Duration,
};

/// A manual-reset event; `let event = ManualResetEvent::default(); event.set(); event.wait()`.
#[derive(Default)]
pub(super) struct ManualResetEvent {
    flag: Mutex<bool>,
    condition: Condvar,
}
impl ManualResetEvent {
    fn guard(&self) -> MutexGuard<'_, bool> {
        // A panicking job never holds this lock, so a poisoned flag is still consistent.
        self.flag
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
    /// Signals the event and wakes every waiter.
    pub(super) fn set(&self) {
        *self.guard() = true;
        self.condition.notify_all();
    }
    /// Clears the signal.
    pub(super) fn reset(&self) {
        *self.guard() = false;
    }
    /// Blocks until the event is signalled.
    pub(super) fn wait(&self) {
        let guard = self.guard();
        drop(
            self.condition
                .wait_while(guard, |flag| !*flag)
                .unwrap_or_else(|poison| poison.into_inner()),
        );
    }
    /// Blocks at most `timeout`; tells whether the event is set.
    pub(super) fn wait_for(&self, timeout: Duration) -> bool {
        let (guard, _) = self
            .condition
            .wait_timeout_while(self.guard(), timeout, |flag| !*flag)
            .unwrap_or_else(|poison| poison.into_inner());
        *guard
    }
}
