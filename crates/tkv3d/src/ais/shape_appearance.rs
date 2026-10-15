//! Native AIS_Shape appearance bodies used by TexturedShape; group replacement preserves identity.
use super::shape::AisShapeSource;
use crate::{
    prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrPresentableObjectUpdates},
    prs3d::Prs3dDrawer,
};
use rustycad_tkernel::{
    quantity::Color,
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::*;

/// Native parent color body, including all drawer aspects; `ais_shape_set_color(&object,color)?`.
pub fn ais_shape_set_color<S: AisShapeSource + ?Sized>(
    object: &S,
    color: Color,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    let recompute = set_shape_drawer_color(object, &drawer, color)?;
    drawer.set_color(color);
    object.presentable_base().fields.write().has_own_color = true;
    refresh_shape_aspect_ownership(object, recompute);
    object.recompute_computed();
    Ok(())
}
pub(super) fn set_shape_drawer_color<S: AisShapeSource + ?Sized>(
    object: &S,
    drawer: &Handle<Prs3dDrawer>,
    color: Color,
) -> Result<bool, Failure> {
    // Native calls all three setup operations even when an earlier one creates new state.
    let mut recompute = drawer.setup_own_shading_aspect(&Handle::default())?;
    recompute |= drawer.set_own_line_aspects(&Handle::default())?;
    recompute |= drawer.setup_own_point_aspect(&Handle::default())?;
    drawer
        .shading_aspect()
        .set_color(color, object.presentable_base().read().current_facing_model)?;
    drawer.point_aspect().set_color(color);
    for line in shape_colored_line_aspects(drawer) {
        line.set_color(color);
    }
    Ok(recompute)
}
pub(super) fn shape_colored_line_aspects(
    drawer: &Prs3dDrawer,
) -> [Handle<crate::prs3d::Prs3dLineAspect>; 6] {
    [
        drawer.line_aspect(),
        drawer.wire_aspect(),
        drawer.free_boundary_aspect(),
        drawer.un_free_boundary_aspect(),
        drawer.seen_line_aspect(),
        drawer.face_boundary_aspect(),
    ]
}
/// Native parent material assignment keeps existing explicit color/transparency; `ais_shape_set_material(&object,&material)?`.
pub fn ais_shape_set_material<S: AisShapeSource + ?Sized>(
    object: &S,
    material: &Graphic3dMaterialAspect,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    let recompute = !drawer.has_own_shading_aspect();
    set_shape_drawer_material(object, &drawer, material, (object.has_color(), object.is_transparent()))?;
    object.presentable_base().fields.write().has_own_material = true;
    refresh_shape_aspect_ownership(object, recompute);
    Ok(())
}
pub(super) fn set_shape_drawer_material<S: AisShapeSource + ?Sized>(
    object: &S,
    drawer: &Handle<Prs3dDrawer>,
    material: &Graphic3dMaterialAspect,
    keep: (bool, bool),
) -> Result<(), Failure> {
    drawer.setup_own_shading_aspect(&Handle::default())?;
    let facing = object.presentable_base().read().current_facing_model;
    let shading = drawer.shading_aspect();
    let color = shading.color(facing);
    let transparency = shading.transparency(facing);
    shading.set_material(material, facing);
    if keep.0 {
        shading.set_color(color, facing)?;
    }
    if keep.1 {
        shading.set_transparency(transparency, facing)?;
    }
    Ok(())
}
pub(super) fn refresh_shape_aspect_ownership<S: AisShapeSource + ?Sized>(
    object: &S,
    recompute: bool,
) {
    if !recompute || !object.attributes().has_link() {
        object.synchronize_aspects();
        return;
    }
    ais_shape_replace_with_new_own_aspects(object);
}
/// `AIS_Shape::replaceWithNewOwnAspects` (protected): the presentation aspects of the linked
/// drawer replaced by the own ones that differ; `ais_shape_replace_with_new_own_aspects(&shape)`.
pub fn ais_shape_replace_with_new_own_aspects<S: AisShapeSource + ?Sized>(object: &S) {
    let drawer = object.attributes();
    let linked = drawer.link();
    let mut replacements = Graphic3dAspectsReplaceMap::default();
    for (old, new) in shape_replaceable_aspects(&linked)
        .into_iter()
        .zip(shape_replaceable_aspects(&drawer))
    {
        if old != new {
            replacements.insert(old.object_address() as usize, (old, new));
        }
    }
    object.replace_aspects(&replacements);
}
pub(super) fn shape_replaceable_aspects(
    drawer: &Handle<Prs3dDrawer>,
) -> [Handle<dyn Graphic3dAspectsSource>; 8] {
    [
        drawer.shading_aspect().aspect().cast(),
        drawer.line_aspect().aspect().cast(),
        drawer.wire_aspect().aspect().cast(),
        drawer.free_boundary_aspect().aspect().cast(),
        drawer.un_free_boundary_aspect().aspect().cast(),
        drawer.seen_line_aspect().aspect().cast(),
        drawer.face_boundary_aspect().aspect().cast(),
        drawer.point_aspect().aspect().cast(),
    ]
}
