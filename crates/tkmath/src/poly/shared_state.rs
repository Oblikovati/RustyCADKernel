//! Interior state of a Poly transient that every handle may change.
//!
//! OCCT mutates a `Poly_Triangulation` or `Poly_Polygon3D` through any handle that shares
//! it. The TKernel `Handle` lends `&T`, so each transient keeps its fields in one
//! `SharedState` and every native member, setters included, takes `&self` (the rule of
//! docs/geom-design.md, section 3). A member takes the lock once and never calls another
//! handle while it holds it.
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
/// A reader-writer lock that ignores poisoning, because the port never panics while it
/// holds a guard; `SharedState::new(0).read()`.
#[derive(Debug, Default)]
pub(crate) struct SharedState<T>(RwLock<T>);
impl<T> SharedState<T> {
    /// Wraps the initial fields; `SharedState::new(fields)`.
    pub(crate) const fn new(value: T) -> Self {
        Self(RwLock::new(value))
    }
    /// Shared access for const members; `state.read().field`.
    pub(crate) fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().unwrap_or_else(PoisonError::into_inner)
    }
    /// Exclusive access for mutating members; `state.write().field = value`.
    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().unwrap_or_else(PoisonError::into_inner)
    }
}
