//! Twenty-six native selectable parts share the presentation's virtual geometry factories.
use super::*;
use crate::{
    prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, select3d::*, v3d::V3dTypeOfOrientation,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;
/// Append real point-sensitive parts, leaving unsupported modes untouched; `ais_view_cube_compute_selection(&cube,&selection,0)?`.
pub fn ais_view_cube_compute_selection<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    if mode != 0 {
        return Ok(());
    }
    for orientation in V3dTypeOfOrientation::ALL {
        append_cube_sensitive_part(cube, selection, orientation)?;
    }
    Ok(())
}
fn append_cube_sensitive_part<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    selection: &Handle<SelectMgrSelection>,
    orientation: V3dTypeOfOrientation,
) -> Result<(), Failure> {
    let triangles = cube_part_selection_triangles(cube, orientation)?;
    if triangles.is_null() {
        return Ok(());
    }
    let owner =
        AisViewCubeOwner::with_source(&cube.presentable_base().this().cast(), orientation, 5);
    let sensitive = Handle::new(AisViewCubeSensitive::new(&owner.cast(), &triangles)?);
    let factor = if ais_view_cube_is_box_corner(orientation) {
        8
    } else if ais_view_cube_is_box_edge(orientation) {
        4
    } else {
        2
    };
    sensitive.set_sensitivity_factor(factor)?;
    selection.add(&sensitive.cast())
}
pub(super) fn cube_part_selection_triangles<C: AisViewCubePresentationSource + ?Sized>(
    cube: &C,
    orientation: V3dTypeOfOrientation,
) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
    let mut counts = AisViewCubeTriangleCounts::default();
    cube.create_box_part_triangles(&Handle::default(), &mut counts, orientation)?;
    if counts.vertices <= 0 {
        return Ok(Handle::default());
    }
    let triangles = Handle::new(Graphic3dArrayOfTriangles::new(
        counts.vertices,
        counts.triangles * 3,
        0,
    )?);
    cube.create_box_part_triangles(
        &triangles,
        &mut AisViewCubeTriangleCounts::default(),
        orientation,
    )?;
    Ok(triangles)
}
impl SelectMgrSelectableObject for AisViewCube {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.interaction.presentation.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_view_cube_compute_selection(self, selection, mode)
    }
    fn global_sel_owner(&self) -> Handle<dyn SelectMgrEntityOwnerSource> {
        Handle::default()
    }
    fn is_auto_hilight(&self) -> bool {
        false
    }
    fn clear_selected(&self) {
        // OCCT 3d097a0 explicitly ignores global selection: only detected-part highlighting exists.
    }
    fn hilight_selected(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        _owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
    ) -> Result<(), Failure> {
        // Native HilightSelected deliberately does not start animation; owners intercept clicks first.
        Ok(())
    }
    fn hilight_owner_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<(), Failure> {
        super::view_cube_highlight::ais_view_cube_hilight_owner_with_color(
            self, manager, style, owner,
        )
    }
}
impl PrsMgrPresentableObject for AisViewCube {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.interaction.presentation.interactive.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_view_cube_compute(self, prs, mode)
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0
    }
    crate::ais_presentable_overrides!();
}
