//! BVH adapter: stable tree identity and the native named base interface.
use super::SelectMgrSensitiveEntitySet;
use super::sensitive_entity_set::set_base_sensitive;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{
    BvhBox, BvhBuilderHandle, BvhObjectTransientSource, BvhPrimitiveSet3d, BvhPrimitiveSetAccess,
    BvhProperties, BvhSet, BvhTreeHandle, BvhVec3d,
};
use std::sync::Arc;

struct EntitySetPrimitives<'a> {
    set: &'a SelectMgrSensitiveEntitySet,
    content: &'a mut BvhPrimitiveSet3d,
}
impl BvhSet<BvhVec3d> for EntitySetPrimitives<'_> {
    fn size(&self) -> i32 {
        self.set.size()
    }
    fn element_box(&self, index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.set.element_box(index)
    }
    fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        self.set.center(index, axis)
    }
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.set.swap(first, second)
    }
    fn bounding_box(&self) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.cached_box()
    }
}
impl BvhPrimitiveSetAccess<BvhVec3d> for EntitySetPrimitives<'_> {
    fn primitive_set(&self) -> &BvhPrimitiveSet3d {
        self.content
    }
    fn primitive_set_mut(&mut self) -> &mut BvhPrimitiveSet3d {
        self.content
    }
}
impl BvhObjectTransientSource for SelectMgrSensitiveEntitySet {
    fn properties(&self) -> Option<Arc<dyn BvhProperties>> {
        self.lock_content().object_transient().properties.clone()
    }
    fn set_properties(&self, properties: Option<Arc<dyn BvhProperties>>) {
        self.lock_content().object_transient_mut().properties = properties;
    }
    fn is_dirty(&self) -> bool {
        self.lock_content().object_transient().is_dirty
    }
    fn mark_dirty(&self) {
        self.lock_content().object_transient_mut().is_dirty = true;
    }
}
impl SelectMgrSensitiveEntitySet {
    /// Read an entity's bounds, void for transform persistence; `set.element_box(0)?`.
    pub fn element_box(&self, index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        use crate::select3d::Select3DSensitiveEntityMembers;
        let sensitive = set_base_sensitive(&self.get_sensitive_by_id(index)?)?;
        if !sensitive.transform_persistence().is_null() {
            return Ok(BvhBox::default());
        }
        Ok(sensitive.bounding_box())
    }
    /// Read geometry center; every axis other than 0 or 1 selects Z; `set.center(0,2)?`.
    pub fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        let point = set_base_sensitive(&self.get_sensitive_by_id(index)?)?.center_of_geometry();
        Ok(match axis {
            0 => point.x(),
            1 => point.y(),
            _ => point.z(),
        })
    }
    /// Read the dirty-aware cached aggregate box; `set.bounding_box()?`.
    pub fn bounding_box(&self) -> Result<BvhBox<BvhVec3d>, Failure> {
        let mut content = self.lock_content();
        EntitySetPrimitives {
            set: self,
            content: &mut content,
        }
        .cached_box()
    }
    /// Rebuild if dirty and retain the live tree; `set.bvh()?.read().length()`.
    pub fn bvh(&self) -> Result<BvhTreeHandle<BvhVec3d>, Failure> {
        let mut content = self.lock_content();
        EntitySetPrimitives {
            set: self,
            content: &mut content,
        }
        .bvh_handle()
    }
    /// Read the shared builder, including null; `set.builder()`.
    pub fn builder(&self) -> BvhBuilderHandle<BvhVec3d> {
        let mut content = self.lock_content();
        EntitySetPrimitives {
            set: self,
            content: &mut content,
        }
        .builder()
        .clone()
    }
    /// Replace the builder without changing the dirty flag; `set.set_builder(Some(builder))`.
    pub fn set_builder(&self, builder: BvhBuilderHandle<BvhVec3d>) {
        let mut content = self.lock_content();
        EntitySetPrimitives {
            set: self,
            content: &mut content,
        }
        .set_builder(builder);
    }
}
