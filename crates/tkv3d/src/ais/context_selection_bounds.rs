//! Native selected-owner bounds include presentation or activated sensitive geometry.
use super::{AisInteractiveContext, AisInteractiveObject, AisSelectedOwner};
use crate::{prs_mgr::PrsMgrPresentableObjectMembers as _, select_mgr::*, v3d::V3dView};
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Failure, Handle, exceptions::null_object},
};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkservice::graphic3d::Graphic3dCViewMembers as _;
type SelectedBoundsGroups = DataMap<Handle<dyn AisInteractiveObject>, Vec<AisSelectedOwner>>;
impl AisInteractiveContext {
    /// Aggregate selected geometry visible in an optional view; `context.bounding_box_of_selection(Some(&view))?`.
    pub fn bounding_box_of_selection(
        &self,
        view: Option<&V3dView>,
    ) -> Result<BoundingBox3, Failure> {
        let view_id = view.map_or(-1, |v| v.view().identification());
        let mut bounds = BoundingBox3::default();
        let mut groups = SelectedBoundsGroups::default();
        // Native Objects() iteration leaves the selection container cursor untouched.
        let mut owners = self.selected_walk();
        while let Some(owner) = owners.current() {
            accumulate_selected_owner(&owner, view_id, &mut bounds, &mut groups)?;
            owners.next();
        }
        for (object, owners) in groups.items() {
            bounds.add_box(&object.bnd_box_of_selected(owners));
        }
        Ok(bounds)
    }
    /// Fit the selected bounds with native margin/update arguments; `context.fit_selected(&view, 0.01, false)?`.
    pub fn fit_selected(&self, view: &V3dView, margin: f64, update: bool) -> Result<(), Failure> {
        let bounds = self.bounding_box_of_selection(Some(view))?;
        if bounds.is_void() {
            return Ok(());
        }
        view.fit_all_box(&bounds, margin, update)
    }
    /// Fit selection with native defaults; `context.fit_selected_default(&view)?`.
    pub fn fit_selected_default(&self, view: &V3dView) -> Result<(), Failure> {
        // Native convenience callers expect the selected camera to be refreshed immediately.
        self.fit_selected(view, 0.01, true)
    }
}
fn accumulate_selected_owner(
    owner: &AisSelectedOwner,
    view_id: i32,
    bounds: &mut BoundingBox3,
    groups: &mut SelectedBoundsGroups,
) -> Result<(), Failure> {
    let object = selected_bounds_object(owner)?;
    if object.is_infinite() || !selected_object_visible(&*object, view_id)? {
        return Ok(());
    }
    if *owner == object.global_sel_owner() {
        append_global_selected_bounds(bounds, &object);
    } else {
        append_selected_bounds_owner(groups, &object, owner);
    }
    Ok(())
}
fn selected_bounds_object(
    owner: &AisSelectedOwner,
) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
    let owner = owner
        .get()
        .ok_or_else(|| null_object("selected owner=null; expected SelectMgrEntityOwner"))?;
    let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
    if object.is_null() {
        return Err(null_object(
            "selected owner selectable=null/non-interactive; expected an AIS interactive object",
        ));
    }
    Ok(object)
}
fn append_global_selected_bounds(
    bounds: &mut BoundingBox3,
    object: &Handle<dyn AisInteractiveObject>,
) {
    let mut object_bounds = BoundingBox3::default();
    object.bounding_box(&mut object_bounds);
    bounds.add_box(&object_bounds);
}

fn selected_object_visible(
    object: &dyn AisInteractiveObject,
    view_id: i32,
) -> Result<bool, Failure> {
    if view_id == -1 {
        return Ok(true);
    }
    let affinity = object.view_affinity();
    let affinity = affinity.get().ok_or_else(|| {
        null_object(
            "selected object affinity=null; expected Graphic3dViewAffinity for a specific view",
        )
    })?;
    Ok(affinity.is_visible(view_id))
}
fn append_selected_bounds_owner(
    groups: &mut SelectedBoundsGroups,
    object: &Handle<dyn AisInteractiveObject>,
    owner: &AisSelectedOwner,
) {
    if !groups.is_bound(object) {
        groups.bind(object.clone(), Vec::new());
    }
    let owners = groups.change_find(object).expect("bound selected object");
    // Native IndexedMap groups sensitive owners once per selectable object.
    if !owners.contains(owner) {
        owners.push(owner.clone());
    }
}
