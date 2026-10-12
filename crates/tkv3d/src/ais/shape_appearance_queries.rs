//! Native AIS_Shape queries read actual shading state and the current facing model, not drawer scalars.
use super::AisShapeSource;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::quantity::Color;
use rustycad_tkservice::graphic3d::{Graphic3dMaterialAspect, Graphic3dNameOfMaterial};
/// Copy current native shading color into the caller's output; `ais_shape_color_into(&object,&mut color)`.
pub fn ais_shape_color_into<S: AisShapeSource + ?Sized>(object: &S, output: &mut Color) {
    let drawer = object.attributes();
    let shading = drawer.shading_aspect();
    if shading.is_null() {
        return;
    }
    *output = shading.color(object.presentable_base().read().current_facing_model);
}
/// Read the material name for the current facing model; `ais_shape_material(&object)`.
pub fn ais_shape_material<S: AisShapeSource + ?Sized>(object: &S) -> Graphic3dNameOfMaterial {
    let shading = object.attributes().shading_aspect();
    if shading.is_null() {
        return Graphic3dMaterialAspect::default().name();
    }
    shading.material(object.presentable_base().read().current_facing_model).name()
}
/// Read actual face transparency including tiny values and NaN; `ais_shape_transparency(&object)`.
pub fn ais_shape_transparency<S: AisShapeSource + ?Sized>(object: &S) -> f64 {
    let shading = object.attributes().shading_aspect();
    if shading.is_null() {
        return 0.0;
    }
    shading.transparency(object.presentable_base().read().current_facing_model)
}
pub(super) fn shape_color_value<S: AisShapeSource + ?Sized>(object: &S) -> Color {
    // Preserve native default-constructed output when its optional shading slot is null.
    let mut color = Color::default();
    ais_shape_color_into(object, &mut color);
    color
}
