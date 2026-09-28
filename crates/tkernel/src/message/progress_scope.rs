// Created on: 2002-02-22
// Created by: Andrey BETENEV
// Copyright (c) 2002-2014 OPEN CASCADE SAS
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

//! Message_ProgressScope: a local `[0, max]` scale mapped onto a range of its parent.
use super::{ProgressRange, progress_indicator::IndicatorShared};
use crate::tcollection::AsciiString;
use std::{
    borrow::Cow,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
mod scale;

/// Message_ProgressScope: iterates a local scale of `max` steps whose whole span is
/// the part of the global scale given by the range it was created from.
///
/// `next` hands out ranges for sub-steps; dropping or closing the scope advances the
/// indicator to the scope end. Scopes borrow their parent, so a sub-scope cannot
/// outlive it. The position and armed flag are atomics so ranges of one scope can
/// be handed to parallel tasks, as the native class allows.
/// Example:
/// `let scope = ProgressScope::with_literal(&range, "Outer", 10., false); while scope.more() { drop(scope.next(1.)); }`.
pub struct ProgressScope<'a> {
    progress: Option<Arc<IndicatorShared>>,
    parent: Option<&'a ProgressScope<'a>>,
    // `None` is the native null `myName`; a literal name is kept even when empty.
    name: Option<Cow<'static, [u8]>>,
    start: f64,
    portion: f64,
    max: f64,
    value: AtomicU64,
    active: AtomicBool,
    infinite: bool,
}
impl Default for ProgressScope<'_> {
    /// The dummy scope bound to no indicator: portion 1, max 1, inactive;
    /// `ProgressScope::default().next(1.).is_active() == false`.
    fn default() -> Self {
        Self::detached(None, None, 0., 1., 1., false, false)
    }
}
impl<'a> ProgressScope<'a> {
    #[allow(clippy::too_many_arguments)]
    fn detached(
        progress: Option<Arc<IndicatorShared>>,
        parent: Option<&'a ProgressScope<'a>>,
        start: f64,
        portion: f64,
        max: f64,
        active: bool,
        infinite: bool,
    ) -> Self {
        Self {
            progress,
            parent,
            name: None,
            start,
            portion,
            max,
            value: AtomicU64::new(0f64.to_bits()),
            active: AtomicBool::new(active),
            infinite,
        }
    }
    /// Private native constructor of the indicator's root scope.
    pub(super) fn root(progress: Arc<IndicatorShared>) -> ProgressScope<'static> {
        ProgressScope::detached(Some(progress), None, 0., 1., 1., true, false)
    }
    /// Shared body of the three range constructors: take the range's parent, start
    /// and delta, clamp `max` to at least 1e-6 (zero-range protection) and disarm it.
    ///
    /// Native debug builds assert the range was not used before; release builds (the
    /// oracle) only create an inactive scope, as here.
    fn from_range(range: &ProgressRange<'a>, max: f64, infinite: bool) -> Self {
        let (parent, start, delta, used) = range.disarm_for_scope();
        let progress = parent.and_then(|scope| scope.progress.clone());
        let active = progress.is_some() && !used;
        Self::detached(
            progress,
            parent,
            start,
            delta,
            1.0e-6_f64.max(max),
            active,
            infinite,
        )
    }
    /// Native `(range, const TCollection_AsciiString&, max, isInfinite)`: the name is
    /// copied, an empty one becomes null;
    /// `ProgressScope::new(&range, &AsciiString::new("Step"), 5., false)`.
    pub fn new(range: &ProgressRange<'a>, name: &AsciiString, max: f64, infinite: bool) -> Self {
        let mut scope = Self::from_range(range, max, infinite);
        scope.set_name(name);
        scope
    }
    /// Native `(range, const char (&)[N], max, isInfinite)`: the literal is stored as
    /// is, so `""` stays a non-null empty name;
    /// `ProgressScope::with_literal(&range, "Step", 5., false)`.
    pub fn with_literal(
        range: &ProgressRange<'a>,
        name: &'static str,
        max: f64,
        infinite: bool,
    ) -> Self {
        let mut scope = Self::from_range(range, max, infinite);
        scope.name = Some(Cow::Borrowed(name.as_bytes()));
        scope
    }
    /// Native `(range, const NullString*, max, isInfinite)`: a scope with a null name;
    /// `ProgressScope::unnamed(&range, 5., false)`.
    pub fn unnamed(range: &ProgressRange<'a>, max: f64, infinite: bool) -> Self {
        Self::from_range(range, max, infinite)
    }
    /// Copies `name`; an empty string makes the name null; `scope.set_name(&name)`.
    pub fn set_name(&mut self, name: &AsciiString) {
        self.name = (!name.is_empty()).then(|| Cow::Owned(name.as_bytes().to_vec()));
    }
    /// Stores the literal itself, even when empty; `scope.set_literal_name("Step")`.
    pub fn set_literal_name(&mut self, name: &'static str) {
        self.name = Some(Cow::Borrowed(name.as_bytes()));
    }
    /// True when the indicator signals a user break; `scope.user_break()`.
    pub fn user_break(&self) -> bool {
        self.indicator_break()
    }
    /// False when the indicator signals a user break; `while scope.more() {}`.
    pub fn more(&self) -> bool {
        !self.user_break()
    }
    /// Forces `show(self, force = true)` while armed; `scope.show()`.
    pub fn show(&self) {
        if let (true, Some(progress)) = (self.is_active(), &self.progress) {
            progress.show(self, true);
        }
    }
    /// True while armed and bound to an indicator; `scope.is_active()`.
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
    /// The name as the C string `Name()` returns (bytes up to the first NUL), or
    /// `None` for a null name; `scope.name() == Some(&b"Step"[..])`.
    pub fn name(&self) -> Option<&[u8]> {
        let bytes = self.name.as_deref()?;
        Some(bytes.split(|byte| *byte == 0).next().unwrap_or(bytes))
    }
    /// The parent scope; `None` only for the root and dummy scopes; `scope.parent()`.
    pub fn parent(&self) -> Option<&'a ProgressScope<'a>> {
        self.parent
    }
    /// The maximal local value (at least 1e-6 for range-built scopes); `scope.max_value()`.
    pub fn max_value(&self) -> f64 {
        self.max
    }
    /// The infinite (hyperbolic advance) flag; `scope.is_infinite()`.
    pub fn is_infinite(&self) -> bool {
        self.infinite
    }
    /// The part of the global scale this scope covers; `scope.get_portion()`.
    pub fn get_portion(&self) -> f64 {
        self.portion
    }
    fn local_value(&self) -> f64 {
        f64::from_bits(self.value.load(Ordering::Acquire))
    }
    fn set_local_value(&self, value: f64) {
        self.value.store(value.to_bits(), Ordering::Release);
    }
    pub(super) fn reset_value(&self) {
        self.set_local_value(0.);
    }
    pub(super) fn disarm(&self) {
        self.active.store(false, Ordering::Release);
    }
    pub(super) fn has_indicator(&self) -> bool {
        self.progress.is_some()
    }
    pub(super) fn indicator_break(&self) -> bool {
        self.progress
            .as_ref()
            .is_some_and(|progress| progress.user_break())
    }
    pub(super) fn increment_indicator(&self, step: f64) {
        if let Some(progress) = &self.progress {
            progress.increment(step, self);
        }
    }
}
impl Drop for ProgressScope<'_> {
    /// Native destructor: `Close()` (the owned name is released by Rust).
    fn drop(&mut self) {
        self.close();
    }
}
