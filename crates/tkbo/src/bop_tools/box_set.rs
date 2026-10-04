use rustycad_tkmath::bvh::{BvhBoxSet, BvhBuilderHandle, BvhLinearBuilder, BvhVector};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
/// BVH box set with OCCT Boolean toolkit's linear-builder default.
pub struct BoxSet<V: BvhVector, D = i32>(BvhBoxSet<V, D>);
impl<V: BvhVector, D> Default for BoxSet<V, D> {
    fn default() -> Self {
        Self::new(None)
    }
}
impl<V: BvhVector, D> BoxSet<V, D> {
    /// Retain a supplied builder or select the native linear default; `BoxSet::<BvhVec3d>::new(None)`.
    pub fn new(builder: BvhBuilderHandle<V>) -> Self {
        let builder = builder.unwrap_or_else(|| Arc::new(BvhLinearBuilder::<V>::default()));
        Self(BvhBoxSet::new(Some(builder)))
    }
}
impl<V: BvhVector, D> Deref for BoxSet<V, D> {
    type Target = BvhBoxSet<V, D>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<V: BvhVector, D> DerefMut for BoxSet<V, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
