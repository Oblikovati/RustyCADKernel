//! Native shading reset preserves material ownership, facing model and distinct interior colors.
use super::{
    shape::AisShapeSource,
    shape_unset_appearance::{bind_shape_default_aspect, required_shape_drawer_link},
};
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrPresentableObjectUpdates};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::aspect::AspectTypeOfFacingModel;
use rustycad_tkservice::graphic3d::*;

/// Native UnsetMaterial deliberately keeps HasMaterial true; `ais_shape_unset_material(&object)?`.
pub fn ais_shape_unset_material<S: AisShapeSource + ?Sized>(object: &S) -> Result<(), Failure> {
    if !object.has_material() {
        return Ok(());
    }
    let drawer = object.attributes();
    if !drawer.has_own_shading_aspect() {
        return Ok(());
    }
    if retain_shape_own_shading(object, false) {
        restore_shape_linked_material(object)?;
        object.synchronize_aspects();
        return Ok(());
    }
    let mut replacements = Graphic3dAspectsReplaceMap::default();
    reset_shape_shading_slot(object, &mut replacements)?;
    object.replace_aspects(&replacements);
    Ok(())
}
pub(super) fn reset_shape_color_shading<S: AisShapeSource + ?Sized>(
    object: &S,
    replacements: &mut Graphic3dAspectsReplaceMap,
) -> Result<(), Failure> {
    if !object.attributes().has_own_shading_aspect() {
        return Ok(());
    }
    if retain_shape_own_shading(object, true) {
        return restore_shape_unset_color_material(object);
    }
    reset_shape_shading_slot(object, replacements)
}
fn retain_shape_own_shading<S: AisShapeSource + ?Sized>(object: &S, retain_material: bool) -> bool {
    let textured = object
        .attributes()
        .shading_aspect()
        .aspect()
        .to_map_texture();
    object.has_color()
        || (retain_material && object.has_material())
        || object.is_transparent()
        || textured
}
fn restore_shape_linked_material<S: AisShapeSource + ?Sized>(object: &S) -> Result<(), Failure> {
    let drawer = object.attributes();
    let facing = object.presentable_base().read().current_facing_model;
    if let Some(linked) = drawer.link().get() {
        drawer
            .shading_aspect()
            .set_material(&linked.shading_aspect().material(facing), facing);
    }
    if object.has_color() {
        drawer.shading_aspect().set_color(drawer.color(), facing)?;
        drawer
            .shading_aspect()
            .set_transparency(f64::from(drawer.transparency()), facing)?;
    }
    Ok(())
}
pub(super) fn reset_shape_shading_slot<S: AisShapeSource + ?Sized>(
    object: &S,
    replacements: &mut Graphic3dAspectsReplaceMap,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    let linked = required_shape_drawer_link(&drawer)?;
    bind_shape_default_aspect(
        replacements,
        drawer.shading_aspect().aspect().cast(),
        linked.shading_aspect().aspect().cast(),
    );
    drawer.fields.write().shading_aspect = Handle::default();
    Ok(())
}
fn restore_shape_unset_color_material<S: AisShapeSource + ?Sized>(
    object: &S,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    let facing = object.presentable_base().read().current_facing_model;
    let material = restored_shape_color_material(object)?;
    let [front, back] = shape_default_interior_colors(object);
    drawer.shading_aspect().set_material(&material, facing);
    drawer.shading_aspect().aspect().set_interior_color(front);
    drawer
        .shading_aspect()
        .aspect()
        .set_back_interior_color(back);
    Ok(())
}
fn restored_shape_color_material<S: AisShapeSource + ?Sized>(
    object: &S,
) -> Result<Graphic3dMaterialAspect, Failure> {
    let defaults = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Brass);
    let drawer = object.attributes();
    let facing = object.presentable_base().read().current_facing_model;
    let source = if object.has_material() {
        drawer.clone()
    } else {
        drawer.link()
    };
    let mut material = source.get().map_or(defaults.clone(), |source| {
        source.shading_aspect().material(facing)
    });
    apply_shape_color_reset_flags(object, &mut material, &defaults)?;
    Ok(material)
}
fn shape_default_interior_colors<S: AisShapeSource + ?Sized>(object: &S) -> [Color; 2] {
    let linked = object.attributes().link();
    let Some(linked) = linked.get() else {
        return [Color::named(NamedColor::CYAN1); 2];
    };
    let aspect = linked.shading_aspect().aspect();
    [aspect.interior_color(), aspect.back_interior_color()]
}

fn apply_shape_color_reset_flags<S: AisShapeSource + ?Sized>(
    object: &S,
    material: &mut Graphic3dMaterialAspect,
    defaults: &Graphic3dMaterialAspect,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    let facing = object.presentable_base().read().current_facing_model;
    if object.has_material() {
        let color = linked_shape_reset_color(&drawer, facing, defaults.ambient_color());
        material.set_color(color)?;
    }
    if object.is_transparent() {
        material.set_transparency(drawer.shading_aspect().transparency(facing) as f32)?;
    }
    Ok(())
}
fn linked_shape_reset_color(
    drawer: &Handle<Prs3dDrawer>,
    facing: AspectTypeOfFacingModel,
    fallback: Color,
) -> Color {
    drawer
        .link()
        .get()
        .map_or(fallback, |linked| linked.shading_aspect().color(facing))
}
