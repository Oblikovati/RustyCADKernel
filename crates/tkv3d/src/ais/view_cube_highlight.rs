//! Immediate detected-part highlights retain persistence, affinity, layer and live style.
use super::{AisViewCubeOwner, AisViewCubeSource};
use crate::{prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;
/// Submit real part geometry only during immediate highlighting; `ais_view_cube_hilight_owner_with_color(&cube,&manager,&style,&owner)?`.
pub fn ais_view_cube_hilight_owner_with_color<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<(), Failure> {
    if owner.is_null() || !manager.is_immediate_mode_on() {
        return Ok(());
    }
    let owner = owner.cast::<AisViewCubeOwner>();
    let Some(owner) = owner.get() else {
        return Ok(());
    };
    let prs = cube.get_hilight_presentation(manager)?;
    prepare_cube_highlight(cube, &prs, style);
    let group = prs.new_group()?;
    group.set_group_primitives_aspect(&style.shading_aspect().aspect().cast());
    let triangles =
        super::view_cube_selection::cube_part_selection_triangles(cube, owner.main_orientation())?;
    if !triangles.is_null() {
        group.add_primitive_array(&triangles.cast(), true);
    }
    if manager.is_immediate_mode_on() {
        manager.add_to_immediate_list(&prs);
    }
    Ok(())
}
fn prepare_cube_highlight<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    prs: &Handle<dyn Graphic3dStructureSource>,
    style: &Prs3dDrawer,
) {
    prs.clear(true);
    prs.cstructure()
        .cstructure_base()
        .fields
        .write()
        .view_affinity = cube.view_affinity();
    prs.set_transform_persistence(&cube.transform_persistence());
    let layer = if style.z_layer() != GRAPHIC3D_ZLAYER_ID_UNKNOWN {
        style.z_layer()
    } else {
        cube.attributes().z_layer()
    };
    prs.set_z_layer(layer);
}
