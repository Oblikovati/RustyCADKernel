//! The non-virtual members of `SelectMgr_SelectableObject`.
use super::selectable_object::select_mgr_selection_of;
use super::{
    SelectMgrEntityOwnerSource, SelectMgrSelectableObject, SelectMgrSelection,
    SelectMgrStateOfSelection, SelectMgrTypeOfBvhUpdate, SelectMgrTypeOfUpdate,
};
use crate::prs_mgr::{PrsMgrPresentableObjectMembers as _, PrsMgrPresentationManager};
use crate::select3d::Select3DSensitiveEntityMembers as _;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkservice::graphic3d::{
    Graphic3dStructure, Graphic3dStructureMembers as _, Graphic3dStructureSource,
};

/// The assembly owner of the selectable parent, null without parent or owner.
fn parent_assembly_owner<O: SelectMgrSelectableObject + ?Sized>(
    object: &O,
) -> Handle<dyn SelectMgrEntityOwnerSource> {
    let parent: Handle<dyn SelectMgrSelectableObject> = object.parent().cast();
    parent
        .get()
        .map(|p| p.get_assembly_owner())
        .unwrap_or_default()
}

/// A presentation of the object for highlighting, in the manager's structure manager, with the
/// object persistence, clipping and transformation (`GetHilightPresentation`).
fn new_highlight_structure<O: SelectMgrSelectableObject + ?Sized>(
    object: &O,
    manager: &PrsMgrPresentationManager,
) -> Result<Handle<dyn Graphic3dStructureSource>, Failure> {
    let prs = Graphic3dStructure::new(&manager.structure_manager(), &Handle::default())?;
    prs.set_transform_persistence(&object.transform_persistence());
    prs.set_clip_planes(&object.clip_planes());
    prs.set_transformation(&object.transformation_geom());
    Ok(prs.cast())
}

/// The non-virtual members of a selectable object.
pub trait SelectMgrSelectableObjectMembers: SelectMgrSelectableObject {
    /// `RecomputePrimitives()`: recomputes every selection mode; `object.recompute_primitives()?`.
    fn recompute_primitives(&self) -> Result<(), Failure> {
        for selection in self.selections().iter().filter_map(Handle::get) {
            self.recompute_primitives_of_mode(selection.mode())?;
        }
        Ok(())
    }
    /// `RecomputePrimitives(mode)`: clears and computes the selection of the mode again (or a
    /// new one), with the assembly owner of a selectable parent for mode 0.
    /// Example: `object.recompute_primitives_of_mode(0)?`.
    fn recompute_primitives_of_mode(&self, mode: i32) -> Result<(), Failure> {
        let existing = select_mgr_selection_of(self, mode);
        let selection = if existing.is_null() {
            Handle::new(SelectMgrSelection::new(mode))
        } else {
            existing.clone()
        };
        if let Some(value) = existing.get() {
            value.clear();
        }
        self.compute_selection(&selection, mode)?;
        if mode == 0 {
            let owner = parent_assembly_owner(self);
            if !owner.is_null() {
                self.set_assembly_owner(&owner, mode);
            }
        }
        if let Some(value) = selection.get() {
            value.set_update_status(SelectMgrTypeOfUpdate::Partial);
            value.update_bvh_status(if existing.is_null() {
                SelectMgrTypeOfBvhUpdate::Add
            } else {
                SelectMgrTypeOfBvhUpdate::Renew
            });
        }
        if existing.is_null() {
            self.selectable_base()
                .fields
                .write()
                .selections
                .push(selection);
        }
        Ok(())
    }
    /// `AddSelection`: computes an empty selection, then replaces the selection of the mode.
    /// Example: `object.add_selection(&selection, 0)?`.
    fn add_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        let Some(value) = selection.get() else {
            return Ok(());
        };
        if value.is_empty() {
            self.compute_selection(selection, mode)?;
            value.set_update_status(SelectMgrTypeOfUpdate::Partial);
            value.update_bvh_status(SelectMgrTypeOfBvhUpdate::Add);
        }
        let is_replaced = {
            let mut fields = self.selectable_base().fields.write();
            let index = fields
                .selections
                .iter()
                .position(|s| s.get().is_some_and(|v| v.mode() == mode));
            if let Some(index) = index {
                fields.selections.remove(index);
            }
            fields.selections.push(selection.clone());
            index.is_some()
        };
        if is_replaced {
            value.update_bvh_status(SelectMgrTypeOfBvhUpdate::Renew);
        }
        if mode == 0 {
            let owner = parent_assembly_owner(self);
            if !owner.is_null() {
                self.set_assembly_owner(&owner, mode);
            }
        }
        Ok(())
    }
    /// `ClearSelections`: clears every selection, flagged for BVH removal (and a full update).
    /// Example: `object.clear_selections(false)`.
    fn clear_selections(&self, to_update: bool) {
        for selection in self.selections().iter().filter_map(Handle::get) {
            selection.clear();
            selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::Remove);
            if to_update {
                selection.set_update_status(SelectMgrTypeOfUpdate::Full);
            }
        }
    }
    /// `Selection(mode)`; `object.selection(0)`.
    fn selection(&self, mode: i32) -> Handle<SelectMgrSelection> {
        select_mgr_selection_of(self, mode)
    }
    /// `HasSelection(mode)`; `object.has_selection(0)`.
    fn has_selection(&self, mode: i32) -> bool {
        !self.selection(mode).is_null()
    }
    /// `Selections`; `object.selections()`.
    fn selections(&self) -> Vec<Handle<SelectMgrSelection>> {
        self.selectable_base().read().selections
    }
    /// `GetHilightPresentation`: the highlight presentation, created on demand.
    /// Example: `object.get_hilight_presentation(&manager)?`.
    fn get_hilight_presentation(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
    ) -> Result<Handle<dyn Graphic3dStructureSource>, Failure> {
        let current = self.selectable_base().read().hilight_prs;
        let Some(manager) = manager.get().filter(|_| current.is_null()) else {
            return Ok(current);
        };
        let prs = new_highlight_structure(self, manager)?;
        self.selectable_base().fields.write().hilight_prs = prs.clone();
        Ok(prs)
    }
    /// `GetSelectPresentation`: the selection presentation, created on demand.
    /// Example: `object.get_select_presentation(&manager)?`.
    fn get_select_presentation(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
    ) -> Result<Handle<dyn Graphic3dStructureSource>, Failure> {
        let current = self.selectable_base().read().selection_prs;
        let Some(manager) = manager.get().filter(|_| current.is_null()) else {
            return Ok(current);
        };
        let prs = new_highlight_structure(self, manager)?;
        self.selectable_base().fields.write().selection_prs = prs.clone();
        Ok(prs)
    }
    /// `UpdateSelection(mode)`: `updateSelection`; `object.update_selection(-1)`.
    fn update_selection(&self, mode: i32) {
        self.update_selection_internal(mode);
    }
    /// `SetAssemblyOwner`: the owner of every sensitive of the mode (all modes for -1).
    /// Example: `object.set_assembly_owner(&owner, -1)`.
    fn set_assembly_owner(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>, mode: i32) {
        for selection in self.selections().iter().filter_map(Handle::get) {
            if mode != -1 && selection.mode() != mode {
                continue;
            }
            for entity in selection.entities().iter().filter_map(Handle::get) {
                if let Some(sensitive) = entity.base_sensitive().get() {
                    sensitive.set(owner);
                }
            }
            if mode != -1 {
                return;
            }
        }
    }
    /// `BndBoxOfSelected`: the box of the sensitives of the activated selections whose owner is
    /// in the set, transformed by the object. Example: `object.bnd_box_of_selected(&owners)`.
    fn bnd_box_of_selected(
        &self,
        owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
    ) -> BoundingBox3 {
        let mut bounds = BoundingBox3::default();
        if owners.is_empty() {
            return bounds;
        }
        for selection in self.selections().iter().filter_map(Handle::get) {
            if selection.get_selection_state() != SelectMgrStateOfSelection::Activated {
                continue;
            }
            for sensitive in selection
                .entities()
                .iter()
                .filter_map(Handle::get)
                .map(|e| e.base_sensitive())
            {
                let Some(value) = sensitive.get() else {
                    continue;
                };
                if owners
                    .iter()
                    .any(|o| o.object_address() == value.owner_id().object_address())
                {
                    let entity_box = value.bounding_box();
                    let (min, max) = (entity_box.corner_min(), entity_box.corner_max());
                    bounds.update_bounds([min.x, min.y, min.z], [max.x, max.y, max.z]);
                }
            }
        }
        if !bounds.is_void() && self.has_transformation() {
            bounds = bounds.transformed(self.transformation());
        }
        bounds
    }
    /// `GlobalSelectionMode`; `object.global_selection_mode()`.
    fn global_selection_mode(&self) -> i32 {
        self.selectable_base().read().global_sel_mode
    }
    /// `setGlobalSelMode` (protected): a negative mode becomes 0; `object.set_global_sel_mode(1)`.
    fn set_global_sel_mode(&self, mode: i32) {
        self.selectable_base().fields.write().global_sel_mode = mode.max(0);
    }
}
impl<T: SelectMgrSelectableObject + ?Sized> SelectMgrSelectableObjectMembers for T {}
