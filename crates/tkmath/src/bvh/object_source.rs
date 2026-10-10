//! Handle-facing virtual interface of BVH_ObjectTransient, separate from value state.
use super::BvhProperties;
use rustycad_tkernel::standard::{StandardTransient, Transient};
use std::sync::Arc;

/// Native shared-object operations, usable through a base handle.
/// Example: `let base = set.cast::<dyn BvhObjectTransientSource>(); base.mark_dirty();`.
pub trait BvhObjectTransientSource: Transient {
    /// Read shared geometric properties; `object.properties()`.
    fn properties(&self) -> Option<Arc<dyn BvhProperties>>;
    /// Replace shared properties without marking dirty; `object.set_properties(None)`.
    fn set_properties(&self, properties: Option<Arc<dyn BvhProperties>>);
    /// Read the BVH rebuild flag; `object.is_dirty()`.
    fn is_dirty(&self) -> bool;
    /// Require reconstruction on the next tree request; `object.mark_dirty()`.
    fn mark_dirty(&self);
}
rustycad_tkernel::impl_transient_interface!(
    dyn BvhObjectTransientSource,
    "BVH_ObjectTransient",
    StandardTransient
);
