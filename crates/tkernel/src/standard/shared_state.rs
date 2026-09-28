// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2023 OPEN CASCADE SAS
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
// Rust support for Standard_Transient shared mutation (see docs/geom-design.md, section 3)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Interior state of a transient that every handle may change.
//!
//! OCCT mutates a `Standard_Transient` through any `occ::handle` that shares it, so
//! `c->SetRadius(5)` is visible through every other handle. A [`Handle`](super::Handle)
//! only lends `&T`, so a transient keeps its mutable fields in one [`SharedState`] and
//! every native member, setters included, takes `&self`. Readers on several threads do
//! not block each other, and the object stays `Send + Sync`.
//!
//! A member takes the lock once and never calls another handle while it holds it,
//! because that handle can be the same object.
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// The locked fields of one transient; `SharedState::new(5.0).read()`.
///
/// A poisoned lock is recovered: the port never panics while it holds a guard, so the
/// state is always consistent.
#[derive(Debug, Default)]
pub struct SharedState<S> {
    lock: RwLock<S>,
}
impl<S> SharedState<S> {
    /// Wraps the initial fields; `SharedState::new(fields)`.
    pub const fn new(state: S) -> Self {
        Self {
            lock: RwLock::new(state),
        }
    }
    /// Shared access for const members; `state.read().radius`.
    pub fn read(&self) -> RwLockReadGuard<'_, S> {
        self.lock.read().unwrap_or_else(PoisonError::into_inner)
    }
    /// Exclusive access for mutating members; `state.write().radius = 2.0`.
    pub fn write(&self) -> RwLockWriteGuard<'_, S> {
        self.lock.write().unwrap_or_else(PoisonError::into_inner)
    }
}
impl<S: Clone> SharedState<S> {
    /// Copies the whole state, as a native copy constructor reads it; `state.snapshot()`.
    pub fn snapshot(&self) -> S {
        self.read().clone()
    }
}
