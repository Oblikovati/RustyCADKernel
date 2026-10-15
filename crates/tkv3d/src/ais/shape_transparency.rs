//! Native AIS_Shape transparency uses the current facing model and preserves aspect ownership.
use super::{shape::AisShapeSource, shape_appearance::refresh_shape_aspect_ownership};
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrPresentableObjectUpdates};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsRenderMembers, Graphic3dAspectsReplaceMap,
    Graphic3dPresentationAttributesSource,
};

/// Apply transparency before publishing the drawer scalar; `ais_shape_set_transparency(&shape,0.5)?`.
pub fn ais_shape_set_transparency<S: AisShapeSource + ?Sized>(
    object: &S,
    value: f64,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    let recompute = !drawer.has_own_shading_aspect();
    set_shape_drawer_transparency(object, &drawer, value)?;
    drawer.set_transparency(value as f32);
    refresh_shape_aspect_ownership(object, recompute);
    Ok(())
}

/// Release shading only when color, material and texture do not retain it; `ais_shape_unset_transparency(&shape)?`.
pub fn ais_shape_unset_transparency<S: AisShapeSource + ?Sized>(object: &S) -> Result<(), Failure> {
    let drawer = object.attributes();
    drawer.set_transparency(0.0);
    if !drawer.has_own_shading_aspect() {
        return Ok(());
    }
    if retain_shape_transparency_aspect(object) {
        let facing = object.presentable_base().read().current_facing_model;
        drawer.shading_aspect().set_transparency(0.0, facing)?;
        object.synchronize_aspects();
        return Ok(());
    }
    let mut replacements = Graphic3dAspectsReplaceMap::default();
    super::shape_unset_shading::reset_shape_shading_slot(object, &mut replacements)?;
    object.replace_aspects(&replacements);
    Ok(())
}

fn retain_shape_transparency_aspect<S: AisShapeSource + ?Sized>(object: &S) -> bool {
    let textured = object
        .attributes()
        .shading_aspect()
        .aspect()
        .to_map_texture();
    object.has_color() || object.has_material() || textured
}

/// `AIS_Shape::setTransparency` (protected): the transparency of the drawer shading aspect;
/// `set_shape_drawer_transparency(&shape, &drawer, 0.5)?`.
pub fn set_shape_drawer_transparency<S: AisShapeSource + ?Sized>(
    object: &S,
    drawer: &Handle<crate::prs3d::Prs3dDrawer>,
    value: f64,
) -> Result<(), Failure> {
    drawer.setup_own_shading_aspect(&Handle::default())?;
    let facing = object.presentable_base().read().current_facing_model;
    drawer.shading_aspect().set_transparency(value, facing)?;
    Ok(())
}
