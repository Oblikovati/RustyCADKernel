// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG2d/Geom2d (shared-mutation support, see docs/geom-design.md) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Interior mutability of one transient geometry object.
//!
//! OCCT mutates a geometry through any handle that shares it; a Rust handle only
//! gives `&T`. Each Geom2d object therefore keeps its fields in one
//! [`GeometryCell`], and every native member (setters too) takes `&self`.
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// The locked state of one geometry object; `GeometryCell::new(5.0).read()`.
///
/// A poisoned lock is recovered: the port never panics while it holds a lock,
/// so the state is always consistent.
#[derive(Debug, Default)]
pub struct GeometryCell<S> {
    lock: RwLock<S>,
}
impl<S> GeometryCell<S> {
    /// Wrap the initial state; `GeometryCell::new(state)`.
    pub fn new(state: S) -> Self {
        Self {
            lock: RwLock::new(state),
        }
    }
    /// Shared access for evaluators and getters; `cell.read().radius`.
    pub fn read(&self) -> RwLockReadGuard<'_, S> {
        self.lock.read().unwrap_or_else(PoisonError::into_inner)
    }
    /// Exclusive access for setters; `cell.write().radius = 2.0`.
    pub fn write(&self) -> RwLockWriteGuard<'_, S> {
        self.lock.write().unwrap_or_else(PoisonError::into_inner)
    }
}
impl<S: Clone> GeometryCell<S> {
    /// Copy the whole state, as a native copy constructor reads it; `cell.snapshot()`.
    pub fn snapshot(&self) -> S {
        self.read().clone()
    }
}

/// Proof that a caller is inside this crate, for the native `protected` members
/// that abstract classes share with their subclasses (`Geom2d_Vector::gpVec2d`,
/// `Geom2d_Conic::pos`). Other crates cannot build a value of it.
pub struct ProtectedAccess {
    _private: (),
}
pub(crate) const PROTECTED: ProtectedAccess = ProtectedAccess { _private: () };
