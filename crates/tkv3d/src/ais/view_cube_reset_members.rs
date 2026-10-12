//! Reset mutates retained drawers and preserves custom labels, axes radii, position and animation.
use super::{AisViewCubeGeometryMembers, AisViewCubeSource};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::standard::{Failure, Handle};
/// Restore geometry/style defaults without resetting navigation; `cube.reset_styles()?`.
pub trait AisViewCubeResetMembers: AisViewCubeSource {
    /// Reset selected style/size fields, retaining Y-up/labels/radii/persistence; `cube.reset_styles()?`.
    fn reset_styles(&self) -> Result<(), Failure> {
        // Native ResetStyles calls these virtual hooks, including application subclasses.
        self.unset_attributes();
        self.unset_hilight_attributes();
        reset_cube_geometry_defaults(self);
        self.set_size(70.0, true);
        Ok(())
    }
}
impl<T: AisViewCubeSource + ?Sized> AisViewCubeResetMembers for T {}
/// Reset native attributes in the existing drawer and mark stale; `ais_view_cube_unset_attributes(&cube)?`.
pub fn ais_view_cube_unset_attributes<C: AisViewCubeSource + ?Sized>(
    cube: &C,
) -> Result<(), Failure> {
    super::ais_view_cube_default_attributes(cube)?;
    cube.set_to_update_all();
    Ok(())
}
/// Preserve dynamic drawer identity, replace its shading and clear ordinary highlight; `ais_view_cube_unset_hilight_attributes(&cube)?`.
pub fn ais_view_cube_unset_hilight_attributes<C: AisViewCubeSource + ?Sized>(
    cube: &C,
) -> Result<(), Failure> {
    cube.presentable_base().fields.write().hilight_drawer = Handle::default();
    ais_view_cube_default_highlight_attributes(cube)?;
    cube.set_to_update_all();
    Ok(())
}
/// Reset cyan/unlit dynamic shading inside the existing drawer; `ais_view_cube_default_highlight_attributes(&cube)?`.
pub fn ais_view_cube_default_highlight_attributes<C: AisViewCubeSource + ?Sized>(
    cube: &C,
) -> Result<(), Failure> {
    super::view_cube_interaction::configure_cube_dynamic_drawer(&cube.dynamic_hilight_attributes())
}
fn reset_cube_geometry_defaults<C: AisViewCubeSource + ?Sized>(cube: &C) {
    let mut settings = cube.view_cube_presentation_base().settings.write();
    let g = &mut settings.geometry;
    g.edge_min_size = 2.0;
    g.corner_min_size = 2.0;
    g.edge_gap = 0.0;
    g.round_radius = 0.0;
    g.facet_extension = 1.0;
    g.axes_padding = 1.0;
    settings.draw_axes = true;
    settings.draw_edges = true;
    settings.draw_corners = true;
}
