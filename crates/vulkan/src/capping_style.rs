//! Copy only the object properties selected by OpenGl_CappingPlaneResource::updateAspect.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
pub(crate) fn cap_aspect(
    plane: &Graphic3dClipPlane,
    group: &Handle<dyn Graphic3dAspectsSource>,
) -> Handle<dyn Graphic3dAspectsSource> {
    let aspect = Handle::new(Graphic3dAspects::default());
    *aspect.aspects_base().fields.write() =
        plane.capping_aspect().aspects_base().fields.read().clone();
    let Some(group) = group.get() else {
        return aspect.cast();
    };
    if plane.to_use_object_material() {
        aspect.set_front_material(group.front_material());
        aspect.set_interior_color(group.interior_color());
    }
    if plane.to_use_object_texture() {
        aspect.set_texture_set(group.texture_set());
        aspect.set_texture_map_on(group.to_map_texture());
    }
    if plane.to_use_object_shader() {
        aspect.set_shader_program(group.shader_program());
    }
    aspect.cast()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::quantity::ColorRgba;
    #[test]
    fn inheritance_preserves_cap_alpha_and_shading_while_copying_requested_properties() {
        let plane = Graphic3dClipPlane::default();
        plane
            .capping_aspect()
            .set_interior_color_rgba(ColorRgba::new(0., 1., 0., 0.5).unwrap());
        plane
            .capping_aspect()
            .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
        let group = Handle::new(Graphic3dAspects::default());
        group.set_interior_color_rgba(ColorRgba::new(1., 0., 0., 1.).unwrap());
        group.set_texture_map_on(true);
        let source = group.cast();
        assert!(!cap_aspect(&plane, &source).to_map_texture());
        plane.set_use_object_material(true);
        plane.set_use_object_texture(true);
        let cap = cap_aspect(&plane, &source);
        assert_eq!(cap.interior_color_rgba().alpha(), 0.5);
        assert_eq!(cap.interior_color().red(), 1.);
        assert_eq!(cap.shading_model(), Graphic3dTypeOfShadingModel::Unlit);
        assert!(cap.to_map_texture());
        assert_eq!(plane.capping_aspect().interior_color().green(), 1.);
    }
}
