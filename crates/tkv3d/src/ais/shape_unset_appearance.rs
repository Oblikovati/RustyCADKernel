//! Native parent unset semantics, including the retained hasOwnMaterial flag at OCCT 3d097a0.
use super::shape::AisShapeSource;
use crate::{
    prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrPresentableObjectUpdates},
    prs3d::Prs3dDrawer,
};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, exceptions::null_object},
};
use rustycad_tkservice::graphic3d::*;

/// Native UnsetColor does not refresh a textured copied aspect; `ais_shape_unset_color(&object)?`.
pub fn ais_shape_unset_color<S: AisShapeSource + ?Sized>(object: &S) -> Result<(), Failure> {
    if !object.has_color() {
        return Ok(());
    }
    object.presentable_base().fields.write().has_own_color = false;
    let drawer = object.attributes();
    let color = drawer
        .link()
        .get()
        .map_or(Color::named(NamedColor::WHITE), Prs3dDrawer::color);
    drawer.set_color(color);
    let mut replacements = Graphic3dAspectsReplaceMap::default();
    reset_shape_line_colors(object, &mut replacements)?;
    super::shape_unset_shading::reset_shape_color_shading(object, &mut replacements)?;
    reset_shape_point_aspect(object, &mut replacements)?;
    finish_shape_color_reset(object, &replacements);
    Ok(())
}
fn finish_shape_color_reset<S: AisShapeSource + ?Sized>(
    object: &S,
    replacements: &Graphic3dAspectsReplaceMap,
) {
    object.replace_aspects(replacements);
    object.synchronize_aspects();
    object.recompute_computed();
}
fn reset_shape_line_colors<S: AisShapeSource + ?Sized>(
    object: &S,
    replacements: &mut Graphic3dAspectsReplaceMap,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    if object.has_width() {
        restore_shape_width_colors(&drawer);
        return Ok(());
    }
    let linked = required_shape_drawer_link(&drawer)?;
    let old = super::shape_appearance::shape_colored_line_aspects(&drawer);
    let defaults = super::shape_appearance::shape_colored_line_aspects(&linked);
    for (old, new) in old.into_iter().zip(defaults) {
        bind_shape_default_aspect(replacements, old.aspect().cast(), new.aspect().cast());
    }
    clear_shape_line_slots(&drawer);
    Ok(())
}
pub(super) fn clear_shape_line_slots(drawer: &Handle<Prs3dDrawer>) {
    let mut fields = drawer.fields.write();
    fields.line_aspect = Handle::default();
    fields.wire_aspect = Handle::default();
    fields.free_boundary_aspect = Handle::default();
    fields.un_free_boundary_aspect = Handle::default();
    fields.seen_line_aspect = Handle::default();
    fields.face_boundary_aspect = Handle::default();
}
fn restore_shape_width_colors(drawer: &Handle<Prs3dDrawer>) {
    let linked = drawer.link();
    for (index, line) in super::shape_appearance::shape_colored_line_aspects(drawer)
        .into_iter()
        .enumerate()
    {
        let color = linked
            .get()
            .map_or(Color::named(SHAPE_DEFAULT_LINE_COLORS[index]), |_| {
                super::shape_appearance::shape_colored_line_aspects(&linked)[index]
                    .aspect()
                    .interior_color()
            });
        line.set_color(color);
    }
}
fn reset_shape_point_aspect<S: AisShapeSource + ?Sized>(
    object: &S,
    replacements: &mut Graphic3dAspectsReplaceMap,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    if !drawer.has_own_point_aspect() {
        return Ok(());
    }
    let linked = required_shape_drawer_link(&drawer)?;
    bind_shape_default_aspect(
        replacements,
        drawer.point_aspect().aspect().cast(),
        linked.point_aspect().aspect().cast(),
    );
    drawer.fields.write().point_aspect = Handle::default();
    Ok(())
}
pub(super) fn required_shape_drawer_link(
    drawer: &Handle<Prs3dDrawer>,
) -> Result<Handle<Prs3dDrawer>, Failure> {
    let linked = drawer.link();
    if linked.is_null() {
        return Err(null_object(
            "AIS_Shape aspect replacement: null drawer link; expected a linked default drawer (native null dereference)",
        ));
    }
    Ok(linked)
}
pub(super) fn bind_shape_default_aspect(
    replacements: &mut Graphic3dAspectsReplaceMap,
    old: Handle<dyn Graphic3dAspectsSource>,
    default: Handle<dyn Graphic3dAspectsSource>,
) {
    if old != default {
        replacements.insert(old.object_address() as usize, (old, default));
    }
}

const SHAPE_DEFAULT_LINE_COLORS: [NamedColor; 6] = [
    NamedColor::YELLOW,
    NamedColor::RED,
    NamedColor::GREEN,
    NamedColor::YELLOW,
    NamedColor::YELLOW,
    NamedColor::BLACK,
];
