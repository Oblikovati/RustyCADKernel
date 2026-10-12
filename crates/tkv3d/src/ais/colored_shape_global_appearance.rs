//! Native global appearance visits borrowed custom values before invoking the inherited shape body.
use super::shape_appearance::set_shape_drawer_material;
use super::{
    AisColoredDrawer, AisColoredShapeSource,
    colored_shape_expansion_fill::visit_colored_map_entries,
};
use rustycad_tkernel::{
    quantity::Color,
    standard::{Failure, Handle, HandleFieldRef, HandleReference},
};
use rustycad_tkservice::graphic3d::*;

/// Apply global color only to custom slots without explicit color; `ais_colored_shape_set_color(&object,color)?`.
pub fn ais_colored_shape_set_color<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    color: Color,
) -> Result<(), Failure> {
    visit_custom_drawers(object, |drawer, _| {
        propagate_colored_color(object, drawer, color)
    })?;
    super::ais_shape_set_color(object, color)
}
/// Apply global width before publishing the parent scalar; `ais_colored_shape_set_width(&object,2.0)?`.
pub fn ais_colored_shape_set_width<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    width: f64,
) -> Result<(), Failure> {
    visit_custom_drawers(object, |drawer, _| propagate_colored_width(drawer, width))?;
    super::ais_shape_set_width(object, width)
}
/// Apply global transparency to non-overridden custom shading; `ais_colored_shape_set_transparency(&object,0.5)?`.
pub fn ais_colored_shape_set_transparency<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    value: f64,
) -> Result<(), Failure> {
    visit_custom_drawers(object, |drawer, _| {
        propagate_colored_transparency(object, drawer, value)
    })?;
    super::ais_shape_set_transparency(object, value)
}
/// Preserve explicit custom color and transparency during material propagation; `ais_colored_shape_set_material(&object,&material)?`.
pub fn ais_colored_shape_set_material<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    material: &Graphic3dMaterialAspect,
) -> Result<(), Failure> {
    visit_custom_drawers(object, |drawer, slot| {
        propagate_colored_material(object, drawer, slot, material)
    })?;
    super::ais_shape_set_material(object, material)
}
fn visit_custom_drawers<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    mut apply: impl FnMut(
        &AisColoredDrawer,
        &HandleFieldRef<'_, Option<Handle<AisColoredDrawer>>, AisColoredDrawer>,
    ) -> Result<(), Failure>,
) -> Result<(), Failure> {
    visit_colored_map_entries(&object.colored_shape_base().shape_colors, |_, slot| {
        slot.with_object(|drawer| {
            apply(
                drawer.expect("custom drawer=None; expected a non-null native map value"),
                slot,
            )
        })
    })
}
fn propagate_colored_color<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    drawer: &AisColoredDrawer,
    color: Color,
) -> Result<(), Failure> {
    if drawer.has_own_color() {
        return Ok(());
    }
    if drawer.has_own_shading_aspect() {
        drawer
            .shading_aspect()
            .set_color(color, object.presentable_base().read().current_facing_model)?;
    }
    for line in owned_colored_lines(drawer).into_iter().flatten() {
        line.set_color(color);
    }
    Ok(())
}
fn propagate_colored_width(drawer: &AisColoredDrawer, width: f64) -> Result<(), Failure> {
    if drawer.has_own_width() {
        return Ok(());
    }
    for line in owned_colored_lines(drawer).into_iter().flatten() {
        line.set_width(width)?;
    }
    Ok(())
}
fn propagate_colored_transparency<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    drawer: &AisColoredDrawer,
    value: f64,
) -> Result<(), Failure> {
    if drawer.has_own_transparency() || !drawer.has_own_shading_aspect() {
        return Ok(());
    }
    let facing = object.presentable_base().read().current_facing_model;
    drawer.shading_aspect().set_transparency(value, facing)
}
fn propagate_colored_material<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    drawer: &AisColoredDrawer,
    slot: &impl HandleReference<AisColoredDrawer>,
    material: &Graphic3dMaterialAspect,
) -> Result<(), Failure> {
    if drawer.has_own_material() || !drawer.has_own_shading_aspect() {
        return Ok(());
    }
    let keep = (drawer.has_own_color(), drawer.has_own_transparency());
    // The temporary native handle binds only at the protected setMaterial call, not during traversal.
    let parent = slot.clone_handle().cast();
    set_shape_drawer_material(object, &parent, material, keep)
}
fn owned_colored_lines(
    drawer: &AisColoredDrawer,
) -> [Option<rustycad_tkernel::standard::Handle<crate::prs3d::Prs3dLineAspect>>; 3] {
    [
        drawer.has_own_line_aspect().then(|| drawer.line_aspect()),
        drawer.has_own_wire_aspect().then(|| drawer.wire_aspect()),
        drawer
            .has_own_face_boundary_aspect()
            .then(|| drawer.face_boundary_aspect()),
    ]
}
