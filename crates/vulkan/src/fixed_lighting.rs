//! OCCT3d097a0 GL lighting evaluates encoded materials and eye-space vertices.
use crate::{RenderError, RenderLighting, RenderShading, error::rejected};
use rustycad_tkernel::{ncollection::Mat4, quantity::ColorRgba};

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
        self.normal = Mat4::from_data(self.world)
            .inverted()
            .map_err(|error| {
                rejected(
                    (self.world, error),
                    "an invertible fixed lighting model-view",
                )
            })?
            .transposed()
            .data();
        self.eye = [0., 0., 1., 0.];
        encode_fixed_material_coefficients(self);
        Ok(())
    }
    pub(crate) fn is_fixed_lighting(&self) -> bool {
        self.fixed.is_some()
    }
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
    #[test]
    fn flat_smooth_scale_and_viewer_rules_are_native() {
        assert_eq!(
            fixed_shading_algorithm(RenderShading::PhongFacet),
            FixedLightingInterpolation::Flat
        );
        assert_eq!(
            fixed_shading_algorithm(RenderShading::Phong),
            FixedLightingInterpolation::Smooth
        );
        let mut scale = crate::identity_clip();
        scale[0] = 2.;
        let mut state = RenderLighting::new(
            vec![[0., 0., 2.]],
            crate::RenderMaterial::default(),
            scale,
            crate::identity_clip(),
            true,
            RenderShading::Phong,
        )
        .unwrap();
        state.normalize_fixed = true;
        state.material.common[0].diffuse = [0.25, 0.5, 0.75, 0.5];
        state.prepare_fixed_lighting().unwrap();
        assert!(state.is_fixed_lighting() && state.normalize_fixed);
        assert_eq!(state.eye, [0., 0., 1., 0.]);
        assert_eq!(state.material.common[0].diffuse[3], 0.5);
        assert!(state.material.common[0].diffuse[0] > 0.5);
    }
}
