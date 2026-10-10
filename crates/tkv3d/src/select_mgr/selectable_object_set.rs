//! Indexed selectable subsets and stable shared BVH ownership.
use super::{SelectMgrBvhSubset, SelectMgrSelectableObject};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::{
    ncollection::IndexedMap,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkmath::bvh::{
    BvhBinnedBuilder, BvhBuilder, BvhLinearBuilder, BvhTreeHandle, BvhVec3d,
};
use rustycad_tkservice::graphic3d::{
    Graphic3dCStructureMembers, Graphic3dStructureMembers, Graphic3dTransModeFlags,
    Graphic3dTransformPersMembers, Graphic3dWorldViewProjState,
};
use std::sync::Arc;

pub(super) type SelectableMap = IndexedMap<Handle<dyn SelectMgrSelectableObject>>;
#[derive(Clone)]
pub(super) struct SelectableSubsetState {
    pub objects: SelectableMap,
    pub tree: BvhTreeHandle<BvhVec3d>,
    pub builder: Arc<dyn BvhBuilder<BvhVec3d>>,
    pub dirty: bool,
}
impl SelectableSubsetState {
    fn new(index: usize) -> Self {
        let builder: Arc<dyn BvhBuilder<BvhVec3d>> = if index == 0 {
            // OCCT 3d097a0 constructs four bins, despite the header's 32-bin comment.
            Arc::new(BvhBinnedBuilder::<BvhVec3d, 4>::new(1, 32, true, 1))
        } else {
            Arc::new(BvhLinearBuilder::<BvhVec3d>::new(1, 32))
        };
        Self {
            objects: IndexedMap::default(),
            tree: BvhTreeHandle::default(),
            builder,
            dirty: false,
        }
    }
}
#[derive(Clone, Default)]
pub(super) struct SelectableViewState {
    pub camera: Graphic3dWorldViewProjState,
    pub size: [i32; 2],
}
/// Five native indexed subsets and their retained trees; `SelectMgrSelectableObjectSet::new()`.
pub struct SelectMgrSelectableObjectSet {
    pub(super) subsets: [SharedState<SelectableSubsetState>; SelectMgrBvhSubset::COUNT],
    pub(super) view: SharedState<SelectableViewState>,
}
impl Clone for SelectMgrSelectableObjectSet {
    /// Copy indexed maps and flags while sharing tree and builder handles; `set.clone()`.
    fn clone(&self) -> Self {
        Self {
            subsets: std::array::from_fn(|i| SharedState::new(self.subsets[i].snapshot())),
            view: SharedState::new(self.view.snapshot()),
        }
    }
}
impl Default for SelectMgrSelectableObjectSet {
    fn default() -> Self {
        Self::new()
    }
}
impl SelectMgrSelectableObjectSet {
    /// Start with five clean empty trees; `let set = SelectMgrSelectableObjectSet::new()`.
    pub fn new() -> Self {
        Self {
            subsets: std::array::from_fn(|i| SharedState::new(SelectableSubsetState::new(i))),
            view: SharedState::default(),
        }
    }
    /// Add a previously absent object to its current persistence class; `set.append(&object)?`.
    pub fn append(&self, object: &Handle<dyn SelectMgrSelectableObject>) -> Result<bool, Failure> {
        let subset = appropriate_object_subset(object)?;
        if self.contains(object) {
            return Ok(false);
        }
        let mut state = self.subsets[subset as usize].write();
        let previous = state.objects.len();
        let inserted = previous < state.objects.add(object.clone());
        state.dirty |= inserted;
        Ok(inserted)
    }
    /// Remove from its stored subset, swapping the last key into its slot; `set.remove(&object)?`.
    pub fn remove(&self, object: &Handle<dyn SelectMgrSelectableObject>) -> Result<bool, Failure> {
        let Some(subset) = self.current_subset(object) else {
            return Ok(false);
        };
        let mut state = self.subsets[subset as usize].write();
        let index = state.objects.find_index(object);
        let last = state.objects.len();
        if index != last {
            state.objects.swap(index, last)?;
        }
        state.objects.remove_last()?;
        state.dirty = true;
        Ok(true)
    }
    /// Move a stored object after persistence changes; `set.change_subset(&object)?`.
    pub fn change_subset(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
    ) -> Result<(), Failure> {
        let Some(current) = self.current_subset(object) else {
            return Ok(());
        };
        if current == appropriate_object_subset(object)? {
            return Ok(());
        }
        self.remove(object)?;
        self.append(object)?;
        Ok(())
    }
    pub(super) fn current_subset(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
    ) -> Option<SelectMgrBvhSubset> {
        SelectMgrBvhSubset::ALL.into_iter().find(|subset| {
            self.subsets[*subset as usize]
                .read()
                .objects
                .contains(object)
        })
    }
    /// Test membership across all subsets; `set.contains(&object)`.
    pub fn contains(&self, object: &Handle<dyn SelectMgrSelectableObject>) -> bool {
        self.current_subset(object).is_some()
    }
    /// Test whether all subsets are empty; `set.is_empty()`.
    pub fn is_empty(&self) -> bool {
        SelectMgrBvhSubset::ALL
            .into_iter()
            .all(|subset| self.is_subset_empty(subset))
    }
    /// Test one subset; `set.is_subset_empty(SelectMgrBvhSubset::ThreeD)`.
    pub fn is_subset_empty(&self, subset: SelectMgrBvhSubset) -> bool {
        self.subsets[subset as usize].read().objects.is_empty()
    }
    /// Resolve a zero-based BVH object index; `set.get_object_by_id(subset, 0)?`.
    pub fn get_object_by_id(
        &self,
        subset: SelectMgrBvhSubset,
        index: i32,
    ) -> Result<Handle<dyn SelectMgrSelectableObject>, Failure> {
        let one_based = index.checked_add(1).ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!("object index={index}; expected representable one-based index"),
            )
        })?;
        self.subsets[subset as usize]
            .read()
            .objects
            .find_key_int(one_based)
            .cloned()
    }
    /// Retain the current tree without updating it; `let retained = set.bvh(subset)`.
    pub fn bvh(&self, subset: SelectMgrBvhSubset) -> BvhTreeHandle<BvhVec3d> {
        self.subsets[subset as usize].read().tree.clone()
    }
    /// Require updates for every subset; `set.mark_dirty()`.
    pub fn mark_dirty(&self) {
        for subset in &self.subsets {
            subset.write().dirty = true;
        }
    }
}
fn appropriate_object_subset(
    object: &Handle<dyn SelectMgrSelectableObject>,
) -> Result<SelectMgrBvhSubset, Failure> {
    let object = object.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "selectable object=null; expected nonnull selectable handle",
        )
    })?;
    let persistence = object.transform_persistence();
    if persistence.is_null() {
        return Ok(unpersisted_object_subset(object));
    }
    Ok(
        match (
            (persistence.mode().0 & Graphic3dTransModeFlags::TMF_2D.0) != 0,
            persistence.is_ortho_pers(),
        ) {
            (true, true) => SelectMgrBvhSubset::OrthoTwoDPersistent,
            (true, false) => SelectMgrBvhSubset::TwoDPersistent,
            (false, true) => SelectMgrBvhSubset::OrthoThreeDPersistent,
            (false, false) => SelectMgrBvhSubset::ThreeDPersistent,
        },
    )
}
fn unpersisted_object_subset(object: &dyn SelectMgrSelectableObject) -> SelectMgrBvhSubset {
    let persistent = object.presentations().iter().any(|presentation| {
        let structure = presentation.cstructure();
        structure
            .get()
            .is_some_and(|s| s.has_group_transform_persistence() || s.has_group_flipping())
    });
    if persistent {
        SelectMgrBvhSubset::ThreeDPersistent
    } else {
        SelectMgrBvhSubset::ThreeD
    }
}
