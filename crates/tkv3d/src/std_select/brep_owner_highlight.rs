//! Native decomposition highlight creates and updates a genuine StdSelect_Shape child.
use super::brep_owner_dispatch::brep_owner_effective_mode;
use super::{StdSelectBRepOwnerMembers, StdSelectBRepOwnerSource, StdSelectShape};
use crate::{prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_UNKNOWN, Graphic3dPresentationAttributesMembers,
    Graphic3dPresentationAttributesSource,
};
pub(super) fn highlight_brep_owner<S: StdSelectBRepOwnerSource + ?Sized>(
    owner: &S,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
    mode: i32,
) -> Result<(), Failure> {
    if !owner.has_selectable() {
        return Ok(());
    }
    let mode = brep_owner_effective_mode(owner, mode);
    let selectable = owner.selectable();
    let layer = if style.z_layer() != GRAPHIC3D_ZLAYER_ID_UNKNOWN {
        style.z_layer()
    } else {
        selectable.z_layer()
    };
    if !owner.comes_from_decomposition() {
        return manager.color(&selectable.cast(), style, mode, &Handle::default(), layer);
    }
    let child = prepare_brep_highlight_child(owner, style);
    child.set_z_layer(selectable.z_layer());
    child.set_transform_persistence(&selectable.transform_persistence());
    child.set_local_transformation(&owner.location().transformation())?;
    update_brep_child_attributes(&child.attributes(), style);
    manager.color(&child.cast(), style, mode, &selectable.cast(), layer)
}
fn prepare_brep_highlight_child<S: StdSelectBRepOwnerSource + ?Sized>(
    owner: &S,
    style: &Handle<Prs3dDrawer>,
) -> Handle<dyn super::StdSelectShapeSource> {
    let child = owner.brep_owner_base().presentation.read().clone();
    if !child.is_null() && child.to_be_updated(true) {
        owner.brep_owner_base().presentation.write().nullify();
    }
    let child = owner.brep_owner_base().presentation.read().clone();
    if !child.is_null() {
        return child;
    }
    let shape = owner.shape_ref().clone_shape();
    let child = StdSelectShape::new(&shape, style).into_cast();
    *owner.brep_owner_base().presentation.write() = child;
    owner.brep_owner_base().presentation.read().clone()
}
fn update_brep_child_attributes(drawer: &Handle<Prs3dDrawer>, style: &Handle<Prs3dDrawer>) {
    drawer.set_link(style);
    drawer.set_color(style.color());
    drawer.set_transparency(style.transparency());
    drawer.set_basic_fill_area_aspect(style.basic_fill_area_aspect());
}
