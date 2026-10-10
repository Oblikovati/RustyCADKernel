//! Viewer selector state and its virtual sorting seam (OCCT 3d097a0).
use super::*;
use rustycad_tkernel::{
    ncollection::{Array1, DataMap, IndexedDataMap, Sequence},
    standard::{Failure, Handle, SharedState, StandardTransient, Transient},
};
use rustycad_tkmath::{
    bvh::{BvhBinnedBuilder, BvhBuilderHandle, BvhVec3d},
    gp::{Direction3, Point3},
};
use rustycad_tkservice::graphic3d::{Graphic3dStructure, Graphic3dZLayerId};
use std::sync::Arc;

/// Protected ranking storage; `base.fields.write().results.is_sorted = false`.
#[derive(Default)]
pub struct SelectMgrViewerSelectorResults {
    /// `mystored`: owner identity determines the one-based insertion index.
    pub stored: IndexedDataMap<Handle<dyn SelectMgrEntityOwnerSource>, SelectMgrSortCriterion>,
    /// `myIndexes`: sorted references into `stored`.
    pub indexes: Array1<'static, i32>,
    /// `myIsSorted`: explicit invalidation, independent of preference changes.
    pub is_sorted: bool,
}
/// Protected camera and ranking preferences; `base.fields.read().settings.depth_tolerance`.
#[derive(Clone, Copy)]
pub struct SelectMgrViewerSelectorSettings {
    /// `myDepthTolerance`.
    pub depth_tolerance: f64,
    /// `myDepthTolType`.
    pub depth_tolerance_type: SelectMgrTypeOfDepthTolerance,
    /// `myToPreferClosest`.
    pub to_prefer_closest: bool,
    /// `myCameraEye`.
    pub camera_eye: Point3,
    /// `myCameraDir`.
    pub camera_direction: Direction3,
    /// `myCameraScale`.
    pub camera_scale: f64,
    /// `myToPrebuildBVH`.
    pub to_prebuild_bvh: bool,
    /// `myIsLeftChildQueuedFirst`.
    pub is_left_child_queued_first: bool,
}
impl Default for SelectMgrViewerSelectorSettings {
    fn default() -> Self {
        Self {
            depth_tolerance: 0.,
            depth_tolerance_type: SelectMgrTypeOfDepthTolerance::SensitivityFactor,
            to_prefer_closest: true,
            camera_eye: Point3::default(),
            camera_direction: Direction3::default(),
            camera_scale: 1.,
            to_prebuild_bvh: false,
            is_left_child_queued_first: false,
        }
    }
}
/// Mutable protected fields; `base.fields.write().results.stored.add(owner, criterion)`.
pub struct SelectMgrViewerSelectorFields {
    /// Native ranking fields grouped to borrow map and permutation independently.
    pub results: SelectMgrViewerSelectorResults,
    /// Camera and selection preferences.
    pub settings: SelectMgrViewerSelectorSettings,
    /// `myZLayerOrderMap`.
    pub z_layer_order: DataMap<Graphic3dZLayerId, i32>,
    /// `myEntitySetBuilder`.
    pub entity_set_builder: BvhBuilderHandle<BvhVec3d>,
    /// `myBVHThreadPool`.
    pub bvh_thread_pool: Handle<SelectMgrBvhThreadPool>,
    /// `myStructs`.
    pub debug_structures: Sequence<Handle<Graphic3dStructure>>,
}
impl Default for SelectMgrViewerSelectorFields {
    fn default() -> Self {
        Self {
            results: SelectMgrViewerSelectorResults::default(),
            settings: SelectMgrViewerSelectorSettings::default(),
            z_layer_order: DataMap::default(),
            entity_set_builder: Some(Arc::new(BvhBinnedBuilder::<BvhVec3d, 4>::new(
                1, 32, true, 1,
            ))),
            bvh_thread_pool: Handle::default(),
            debug_structures: Sequence::default(),
        }
    }
}
/// Shared base for native selector subclasses; `SelectMgrViewerSelectorBase::new()`.
#[derive(Default)]
pub struct SelectMgrViewerSelectorBase {
    /// Scalar, ranking, layer and background-build state.
    pub fields: SharedState<SelectMgrViewerSelectorFields>,
    /// `mySelectingVolumeMgr`, separately locked across client access.
    pub selecting_volume: SharedState<SelectMgrSelectingVolumeManager>,
    /// `mySelectableObjects`, internally synchronized for virtual bounds callbacks.
    pub selectable_objects: SelectMgrSelectableObjectSet,
    /// `myTolerances`.
    pub tolerances: SharedState<SelectMgrToleranceMap>,
    /// `myMapOfObjectSensitives`; clone a handle before invoking an entity.
    pub object_sensitives: SharedState<
        DataMap<Handle<dyn SelectMgrSelectableObject>, Handle<SelectMgrSensitiveEntitySet>>,
    >,
}
impl SelectMgrViewerSelectorBase {
    /// Initializes native defaults and the regular four-bin builder; `SelectMgrViewerSelectorBase::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}
/// Virtual selector behavior; overriding sort also affects lazy `picked` access.
pub trait SelectMgrViewerSelectorSource: Transient {
    /// Embedded native state; `selector.viewer_selector_base()`.
    fn viewer_selector_base(&self) -> &SelectMgrViewerSelectorBase;
    /// Sort by native priority/depth rules; `selector.sort_result()?`.
    fn sort_result(&self) -> Result<(), Failure> {
        super::viewer_selector_results::select_mgr_sort_selector_results(
            self.viewer_selector_base(),
        )
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn SelectMgrViewerSelectorSource,
    "SelectMgr_ViewerSelector",
    StandardTransient
);

/// Concrete selector; `let selector = SelectMgrViewerSelector::new()`.
pub struct SelectMgrViewerSelector {
    transient: StandardTransient,
    base: SelectMgrViewerSelectorBase,
}
rustycad_tkernel::impl_transient!(SelectMgrViewerSelector, "SelectMgr_ViewerSelector", StandardTransient, transient,
    interfaces: [dyn SelectMgrViewerSelectorSource]);
impl SelectMgrViewerSelector {
    /// Creates the native empty selector state; `SelectMgrViewerSelector::new()`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: SelectMgrViewerSelectorBase::new(),
        }
    }
}
impl Default for SelectMgrViewerSelector {
    fn default() -> Self {
        Self::new()
    }
}
impl SelectMgrViewerSelectorSource for SelectMgrViewerSelector {
    fn viewer_selector_base(&self) -> &SelectMgrViewerSelectorBase {
        &self.base
    }
}
