//! Private scoped topology projection keeps native const-reference reads uncounted; every value copy acquires ownership.
use super::TShape;
use rustycad_tkernel::standard::{Handle, HandleObserver};
use std::ops::Deref;
pub(super) enum ShapeTopologyStorage {
    Owned(Handle<dyn TShape>),
    Observed(HandleObserver<dyn TShape>),
}
impl Default for ShapeTopologyStorage {
    fn default() -> Self {
        let topology = Handle::default();
        Self::Owned(topology)
    }
}
impl Clone for ShapeTopologyStorage {
    fn clone(&self) -> Self {
        // Native TopoDS value copies retain even when their source is a const reference.
        let topology = Handle::clone(self);
        Self::Owned(topology)
    }
}
impl Deref for ShapeTopologyStorage {
    type Target = Handle<dyn TShape>;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(topology) => topology,
            Self::Observed(topology) => topology,
        }
    }
}
impl ShapeTopologyStorage {
    pub(super) fn nullify(&mut self) {
        // Rust assignment destroys the prior variant before replacing its pointer field.
        *self = Self::default();
    }
}
