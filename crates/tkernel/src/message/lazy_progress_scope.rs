// Copyright (c) 2017-2021 OPEN CASCADE SAS
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

//! Message_LazyProgressScope: a scope that advances and polls for breaks in patches.
use super::{ProgressRange, ProgressScope};
use crate::tcollection::AsciiString;

/// Message_LazyProgressScope: counts unit steps and advances the underlying scope
/// (and polls the user break) only once per `patch_step` steps. The native class
/// inherits the scope protectedly, so only `next`, `more` and `is_aborted` are public.
/// Example: `let mut lazy = LazyProgressScope::new(&range, "Loop", 1000., 50., false); while lazy.more() { lazy.next(); }`.
pub struct LazyProgressScope<'a> {
    scope: ProgressScope<'a>,
    patch_step: f64,
    patch_progress: f64,
    lazy_aborted: bool,
}
impl<'a> LazyProgressScope<'a> {
    /// Creates the scope (name copied, empty meaning null) with a patch of
    /// `patch_step` unit steps; `LazyProgressScope::new(&range, "Loop", 1000., 50., false)`.
    pub fn new(
        range: &ProgressRange<'a>,
        name: &str,
        max: f64,
        patch_step: f64,
        infinite: bool,
    ) -> Self {
        Self {
            scope: ProgressScope::new(range, &AsciiString::new(name), max, infinite),
            patch_step,
            patch_progress: 0.0,
            lazy_aborted: false,
        }
    }
    /// Counts one step; each full patch advances the scope by `patch_step` (the range
    /// closes at once) and refreshes the break state; `lazy.next()`.
    pub fn next(&mut self) {
        self.patch_progress += 1.0;
        if self.patch_progress < self.patch_step {
            return;
        }
        self.patch_progress = 0.0;
        drop(self.scope.next(self.patch_step));
        self.is_aborted();
    }
    /// The cached (lazily refreshed) negated break state; `while lazy.more() {}`.
    pub fn more(&self) -> bool {
        !self.lazy_aborted
    }
    /// Polls the indicator; once aborted the state sticks; `lazy.is_aborted()`.
    pub fn is_aborted(&mut self) -> bool {
        self.lazy_aborted = self.lazy_aborted || !self.scope.more();
        self.lazy_aborted
    }
}
