//! OCCT3d097a0 GL lighting evaluates encoded materials and eye-space vertices.
use crate::{RenderError, RenderLighting, RenderShading, error::rejected};
use rustycad_tkernel::{ncollection::Mat4, quantity::ColorRgba};
use rustycad_tkservice::graphic3d::{Graphic3dAspectsMembers, Graphic3dAspectsRenderMembers};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub(crate) enum FixedLightingInterpolation {
    Smooth = 3,
    Flat = 4,
}

impl RenderLighting {
    pub(crate) fn prepare_fixed_lighting(&mut self) -> Result<(), RenderError> {
        self.fixed = Some(fixed_shading_algorithm(self.shading));
        self.world = (Mat4::from_data(self.view) * Mat4::from_data(self.world)).data();
        self.normal = fixed_normal_matrix(self.world)?;
        self.eye = [0., 0., 1., 0.];
        restore_fixed_material_opacity(self);
        encode_fixed_material_coefficients(self);
        Ok(())
    }
    pub(crate) fn is_fixed_lighting(&self) -> bool {
        self.fixed.is_some()
    }
}
pub(crate) fn snapshot_fixed_material_opacity(
    lighting: &mut RenderLighting,
    aspect: &dyn rustycad_tkservice::graphic3d::Graphic3dAspectsSource,
) {
    let front = aspect.front_material().alpha();
    let back = if aspect.distinguish() {
        aspect.back_material().alpha()
    } else {
        front
    };
    // Native Material::init retains intrinsic diffuse alpha independently of pass selection.
    lighting.fixed_source_opacity = Some([front, back]);
}
fn fixed_normal_matrix(world: [f32; 16]) -> Result<[f32; 16], RenderError> {
    Ok(Mat4::from_data(world)
        .inverted()
        .map_err(|error| rejected((world, error), "an invertible fixed lighting model-view"))?
        .transposed()
        .data())
}
fn restore_fixed_material_opacity(lighting: &mut RenderLighting) {
    let Some(opacity) = lighting.fixed_source_opacity else {
        return;
    };
    lighting.material.common[0].diffuse[3] = opacity[0];
    lighting.material.common[1].diffuse[3] = opacity[1];
}
fn fixed_shading_algorithm(shading: RenderShading) -> FixedLightingInterpolation {
    if shading == RenderShading::PhongFacet {
        return FixedLightingInterpolation::Flat;
    }
    FixedLightingInterpolation::Smooth
}
fn encode_fixed_material_coefficients(lighting: &mut RenderLighting) {
    for face in &mut lighting.material.common {
        face.diffuse = ColorRgba::linear_to_srgb_vec4(face.diffuse);
        face.ambient = ColorRgba::linear_to_srgb_vec4(face.ambient);
        face.emission = ColorRgba::linear_to_srgb_vec4(face.emission);
        face.specular_shininess = ColorRgba::linear_to_srgb_vec4(face.specular_shininess);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scaled_fixed_state() -> RenderLighting {
        let mut scale = crate::identity_clip();
        scale[0] = 2.;
        RenderLighting::new(
            vec![[0., 0., 2.]],
            crate::RenderMaterial::default(),
            scale,
            crate::identity_clip(),
            true,
            RenderShading::Phong,
        )
        .unwrap()
    }
    #[test]
    fn facet_is_flat_and_phong_is_smooth_in_the_fixed_family() {
        assert_eq!(
            fixed_shading_algorithm(RenderShading::PhongFacet),
            FixedLightingInterpolation::Flat
        );
        assert_eq!(
            fixed_shading_algorithm(RenderShading::Phong),
            FixedLightingInterpolation::Smooth
        );
    }
    #[test]
    fn scale_normalization_and_infinite_viewer_preserve_material_alpha() {
        let mut state = scaled_fixed_state();
        state.normalize_fixed = true;
        state.material.common[0].diffuse = [0.25, 0.5, 0.75, 0.5];
        state.prepare_fixed_lighting().unwrap();
        assert!(state.is_fixed_lighting() && state.normalize_fixed);
        assert_eq!(state.eye, [0., 0., 1., 0.]);
        assert_eq!(state.material.common[0].diffuse[3], 0.5);
        assert!(state.material.common[0].diffuse[0] > 0.5);
    }
    #[test]
    fn fixed_material_opacity_survives_opaque_pass_selection_and_face_distinction() {
        use rustycad_tkservice::graphic3d::*;
        let aspect = Graphic3dAspects::default();
        let mut front = aspect.front_material();
        front.set_transparency(0.5).unwrap();
        aspect.set_front_material(front);
        aspect.set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.9);
        let mut lighting = scaled_fixed_state();
        snapshot_fixed_material_opacity(&mut lighting, &aspect);
        assert_eq!(lighting.fixed_source_opacity, Some([0.5, 0.5]));
        lighting.prepare_fixed_lighting().unwrap();
        assert_eq!(lighting.material.common[0].diffuse[3], 0.5);
        assert_eq!(lighting.material.common[1].diffuse[3], 0.5);
    }
}
