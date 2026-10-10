//! Selector configuration and direct access to its native owned components.
use super::*;
use crate::select3d::Select3DSensitiveEntity;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bvh::{BvhBuilderHandle, BvhVec3d};
use std::sync::RwLockWriteGuard;

/// Native nonvirtual configuration members; `selector.set_pixel_tolerance(4)`.
pub trait SelectMgrViewerSelectorMembers: SelectMgrViewerSelectorSource {
    /// Erase, clear and remove debug structures, then update a non-null view.
    /// Example: `selector.clear_sensitive(&view)?`.
    fn clear_sensitive(&self, view: &Handle<crate::v3d::V3dView>) -> Result<(), Failure> {
        super::viewer_selector_debug::clear_selector_sensitive(self.viewer_selector_base(), view)
    }

    /// Dump native selector fields with depth-limited children; `selector.dump_json(&mut stream, -1)?`.
    fn dump_json(
        &self,
        stream: &mut rustycad_tkernel::standard::dump::DumpStream,
        depth: i32,
    ) -> std::io::Result<()> {
        super::viewer_selector_dump::dump_selector(self.viewer_selector_base(), stream, depth)
    }
    /// Read the custom offset, including the unset sentinel; `selector.custom_pixel_tolerance()`.
    fn custom_pixel_tolerance(&self) -> i32 {
        self.viewer_selector_base()
            .tolerances
            .read()
            .custom_tolerance()
    }
    /// Preserve the native early return against effective tolerance; `selector.set_pixel_tolerance(3)`.
    fn set_pixel_tolerance(&self, tolerance: i32) {
        let mut tolerances = self.viewer_selector_base().tolerances.write();
        if tolerances.tolerance() == tolerance {
            return;
        }
        if tolerance < 0 {
            tolerances.reset_defaults();
        } else {
            tolerances.set_custom_tolerance(tolerance);
        }
    }
    /// Return effective sensitivity as a real value; `selector.sensitivity()`.
    fn sensitivity(&self) -> f64 {
        f64::from(self.pixel_tolerance())
    }
    /// Return effective pixel tolerance; `selector.pixel_tolerance()`.
    fn pixel_tolerance(&self) -> i32 {
        self.viewer_selector_base().tolerances.read().tolerance()
    }
    /// Read the depth-before-priority preference; `selector.to_pick_closest()`.
    fn to_pick_closest(&self) -> bool {
        self.viewer_selector_base()
            .fields
            .read()
            .settings
            .to_prefer_closest
    }
    /// Change preference without invalidating cached ranks; `selector.set_pick_closest(false)`.
    fn set_pick_closest(&self, prefer: bool) {
        self.viewer_selector_base()
            .fields
            .write()
            .settings
            .to_prefer_closest = prefer;
    }
    /// Read how depth tolerance is evaluated; `selector.depth_tolerance_type()`.
    fn depth_tolerance_type(&self) -> SelectMgrTypeOfDepthTolerance {
        self.viewer_selector_base()
            .fields
            .read()
            .settings
            .depth_tolerance_type
    }
    /// Read the retained depth tolerance; `selector.depth_tolerance()`.
    fn depth_tolerance(&self) -> f64 {
        self.viewer_selector_base()
            .fields
            .read()
            .settings
            .depth_tolerance
    }
    /// Store both tolerance values without validation; `selector.set_depth_tolerance(kind, -1.)`.
    fn set_depth_tolerance(&self, kind: SelectMgrTypeOfDepthTolerance, tolerance: f64) {
        let mut fields = self.viewer_selector_base().fields.write();
        fields.settings.depth_tolerance_type = kind;
        fields.settings.depth_tolerance = tolerance;
    }
    /// Access the actual selecting volume; `selector.get_manager().init_axis_selecting_volume(axis)`.
    fn get_manager(&self) -> RwLockWriteGuard<'_, SelectMgrSelectingVolumeManager> {
        self.viewer_selector_base().selecting_volume.write()
    }
    /// Access the internally synchronized object set; `selector.selectable_objects().contains(&object)`.
    fn selectable_objects(&self) -> &SelectMgrSelectableObjectSet {
        &self.viewer_selector_base().selectable_objects
    }
    /// Share the current entity builder; `selector.entity_set_builder()`.
    fn entity_set_builder(&self) -> BvhBuilderHandle<BvhVec3d> {
        self.viewer_selector_base()
            .fields
            .read()
            .entity_set_builder
            .clone()
    }
    /// Update existing sets without invalidating their trees; `selector.set_entity_set_builder(builder)`.
    fn set_entity_set_builder(&self, builder: BvhBuilderHandle<BvhVec3d>) {
        self.viewer_selector_base()
            .fields
            .write()
            .entity_set_builder = builder.clone();
        let sets: Vec<_> = self
            .viewer_selector_base()
            .object_sensitives
            .read()
            .items()
            .map(|(_, set)| set.clone())
            .collect();
        for set in sets {
            set.set_builder(builder.clone());
        }
    }
    /// Configure rectangular overlap detection; `selector.allow_overlap_detection(false)`.
    fn allow_overlap_detection(&self, allowed: bool) {
        self.get_manager().allow_overlap_detection(allowed);
    }
    /// Read whether background BVH building is enabled; `selector.to_prebuild_bvh()`.
    fn to_prebuild_bvh(&self) -> bool {
        self.viewer_selector_base()
            .fields
            .read()
            .settings
            .to_prebuild_bvh
    }
    /// Enabling always replaces the pool, including repeated calls; `selector.set_to_prebuild_bvh(true, 2)`.
    fn set_to_prebuild_bvh(&self, enabled: bool, threads: i32) {
        let pool = if enabled {
            Handle::new(SelectMgrBvhThreadPool::new(threads))
        } else {
            Handle::default()
        };
        let old = std::mem::replace(
            &mut self.viewer_selector_base().fields.write().bvh_thread_pool,
            pool,
        );
        // A destructor joins workers; it must not hold the selector's state lock.
        drop(old);
        self.viewer_selector_base()
            .fields
            .write()
            .settings
            .to_prebuild_bvh = enabled;
    }
    /// Queue only when background construction is enabled; `selector.queue_bvh_build(&entity)?`.
    fn queue_bvh_build(&self, entity: &Handle<dyn Select3DSensitiveEntity>) -> Result<(), Failure> {
        if !self.to_prebuild_bvh() {
            return Ok(());
        }
        let pool = self
            .viewer_selector_base()
            .fields
            .read()
            .bvh_thread_pool
            .clone();
        pool.add_entity(entity)
    }
    /// Wait through the configured pool; `selector.wait_for_bvh_build()`.
    fn wait_for_bvh_build(&self) {
        if !self.to_prebuild_bvh() {
            return;
        }
        let pool = self
            .viewer_selector_base()
            .fields
            .read()
            .bvh_thread_pool
            .clone();
        pool.wait_threads();
    }
}
impl<T: SelectMgrViewerSelectorSource + ?Sized> SelectMgrViewerSelectorMembers for T {}
