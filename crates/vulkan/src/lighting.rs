//! Native Phong-family shading state is immutable and independent of Vulkan handles.
use crate::{PrimitiveDraw, RenderError, RenderLightSet, RenderMaterial, error::rejected};
use rustycad_tkernel::ncollection::{Mat4, Vec4};
/// Legacy surface algorithms; `RenderShading::Phong` interpolates normals per fragment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum RenderShading {
    /// Derive a face normal from fragment position derivatives.
    PhongFacet,
    /// Illuminate vertices and interpolate front/back colors.
    Gouraud,
    /// Interpolate nodal normals and illuminate fragments.
    Phong,
}
/// One snapshot of geometry normals, transforms, materials and lights; `draw.with_lighting(state)?`.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderLighting {
    pub(crate) normals: Vec<[f32; 3]>,
    pub(crate) material: RenderMaterial,
    pub(crate) world: [f32; 16],
    pub(crate) view: [f32; 16],
    pub(crate) normalize_fixed: bool,
    pub(crate) fixed: Option<crate::fixed_lighting::FixedLightingInterpolation>,
    pub(crate) fixed_source_opacity: Option<[f32; 2]>,
    pub(crate) normal: [f32; 16],
    pub(crate) eye: [f32; 4],
    pub(crate) shading: RenderShading,
    pub(crate) lights: RenderLightSet,
    pub(crate) vertex_colors: bool,
    pub(crate) front_vertex_colors_only: bool,
}
impl RenderLighting {
    /// Snapshot world-space shading; `RenderLighting::new(normals, material, model, view, false, RenderShading::Phong)?`.
    pub fn new(
        normals: Vec<[f32; 3]>,
        material: RenderMaterial,
        world: [f32; 16],
        view: [f32; 16],
        perspective: bool,
        shading: RenderShading,
    ) -> Result<Self, RenderError> {
        validate_lighting_components(&normals, &material, &world, &view)?;
        let normal = inverse_normal_transform(world)?;
        let eye = lighting_eye(view, perspective)?;
        Ok(Self {
            normals,
            material,
            world,
            view,
            normalize_fixed: false,
            fixed: None,
            fixed_source_opacity: None,
            normal,
            eye,
            shading,
            lights: RenderLightSet::default(),
            vertex_colors: false,
            front_vertex_colors_only: false,
        })
    }
    /// Attach enabled lights without changing the geometry snapshot; `state.with_lights(lights)`.
    pub fn with_lights(mut self, lights: RenderLightSet) -> Self {
        self.lights = lights;
        self
    }
    /// Multiply lit coefficients by vertex colors, optionally on front faces only; `state.with_vertex_colors(true, false)`.
    pub fn with_vertex_colors(mut self, enabled: bool, front_only: bool) -> Self {
        self.vertex_colors = enabled;
        self.front_vertex_colors_only = front_only;
        self
    }
    /// Inspect nodal normals; `state.normals()`.
    pub fn normals(&self) -> &[[f32; 3]] {
        &self.normals
    }
    /// Inspect the resolved surface algorithm; `state.shading()`.
    pub fn shading(&self) -> RenderShading {
        self.shading
    }
    pub(crate) fn encode(&self) -> Vec<u8> {
        let mut bytes = self.encode_coefficients();
        bytes.extend(
            [
                self.fixed.map_or(self.shading as u32, |mode| mode as u32),
                self.lights.source_count() as u32,
                u32::from(self.vertex_colors),
                u32::from(self.front_vertex_colors_only)
                    + 2 * u32::from(self.is_fixed_lighting() && self.normalize_fixed),
            ]
            .into_iter()
            .flat_map(u32::to_ne_bytes),
        );
        for light in &self.lights.sources {
            bytes.extend(encode_light_source(light));
        }
        bytes
    }
    fn encode_coefficients(&self) -> Vec<u8> {
        self.world
            .into_iter()
            .chain(self.normal)
            .chain(self.eye)
            .chain(self.lights.ambient)
            .chain(self.material.packed_common().into_iter().flatten())
            .flat_map(f32::to_ne_bytes)
            .collect()
    }
}
fn inverse_normal_transform(world: [f32; 16]) -> Result<[f32; 16], RenderError> {
    Mat4::from_data(world)
        .inverted()
        .map(|inverse| inverse.transposed().data())
        .map_err(|error| rejected((world, error), "an invertible model matrix"))
}
fn lighting_eye(view: [f32; 16], perspective: bool) -> Result<[f32; 4], RenderError> {
    let inverse = Mat4::from_data(view)
        .inverted()
        .map_err(|error| rejected((view, error), "an invertible world-view matrix"))?;
    let vector = if perspective {
        Vec4::new(0., 0., 0., 1.)
    } else {
        Vec4::new(0., 0., 1., 0.)
    };
    let mut eye = (inverse * vector).data();
    eye[3] = f32::from(perspective);
    Ok(eye)
}
fn encode_light_source(light: &crate::light_snapshot::PackedRenderLight) -> Vec<u8> {
    let mut bytes: Vec<u8> = [light.kind, 0, 0, 0]
        .into_iter()
        .flat_map(u32::to_ne_bytes)
        .collect();
    bytes.extend(
        light
            .color
            .into_iter()
            .chain(light.position)
            .chain(light.direction)
            .chain(light.params)
            .flat_map(f32::to_ne_bytes),
    );
    bytes
}
fn validate_lighting_components(
    normals: &[[f32; 3]],
    material: &RenderMaterial,
    world: &[f32; 16],
    view: &[f32; 16],
) -> Result<(), RenderError> {
    validate_material_opacity(material)?;
    if normals
        .iter()
        .flatten()
        .chain(world)
        .chain(view)
        .chain(material.packed_common().iter().flatten())
        .any(|v| !v.is_finite())
    {
        return Err(rejected(
            (normals, material, world, view),
            "finite lighting normals, common coefficients and matrices",
        ));
    }
    Ok(())
}
fn validate_material_opacity(material: &RenderMaterial) -> Result<(), RenderError> {
    let opacity = material.common.map(|face| face.diffuse[3]);
    if opacity
        .iter()
        .any(|value| !value.is_finite() || !(0. ..=1.).contains(value))
    {
        return Err(rejected(
            opacity,
            "finite diffuse opacity in [0,1] for both material faces",
        ));
    }
    Ok(())
}
impl RenderLighting {
    pub(crate) fn uses_translucent_material(&self) -> bool {
        self.material.common.iter().any(|face| face.diffuse[3] < 1.)
    }
    pub(crate) fn validate_opaque_marker_lighting(&self) -> Result<(), RenderError> {
        if self.uses_translucent_material() {
            return Err(rejected(
                self.material.common.map(|face| face.diffuse[3]),
                "opaque marker diffuse opacity equal to 1",
            ));
        }
        Ok(())
    }
}
impl PrimitiveDraw {
    /// Attach surface or native ordered nodal line/opaque marker lighting with one normal per vertex; `draw.with_lighting(state)?`.
    pub fn with_lighting(mut self, lighting: RenderLighting) -> Result<Self, RenderError> {
        self.validate_lighting_topology(&lighting)?;
        if self.topology() == crate::PrimitiveTopology::Points {
            lighting.validate_opaque_marker_lighting()?;
        } else if self.topology() == crate::PrimitiveTopology::Triangles {
            self.surface_blend |= lighting.uses_translucent_material();
        }
        self.lighting = Some(lighting);
        self.unlit_back_color = None;
        Ok(self)
    }
    fn validate_lighting_topology(&self, lighting: &RenderLighting) -> Result<(), RenderError> {
        if self.rejects_lighting_topology(lighting) {
            return Err(rejected(
                (
                    lighting.normals.len(),
                    self.vertices.len(),
                    self.topology(),
                    self.textured_style.is_some(),
                    self.capping_pass,
                ),
                "one normal per ordinary triangle or Gouraud/Phong line/marker vertex",
            ));
        }
        Ok(())
    }
    fn rejects_lighting_topology(&self, lighting: &RenderLighting) -> bool {
        lighting.normals.len() != self.vertices.len()
            || (self.topology() != crate::PrimitiveTopology::Triangles
                && self.point_style.is_none()
                && self.line_style.is_none())
            || ((self.point_style.is_some() || self.line_style.is_some())
                && lighting.shading == RenderShading::PhongFacet)
            || self.textured_style.is_some()
            || self.capping_pass.is_mask()
    }
    /// Inspect a retained lighting snapshot; `draw.lighting()`.
    pub fn lighting(&self) -> Option<&RenderLighting> {
        self.lighting.as_ref()
    }
}

#[cfg(test)]
#[path = "lighting_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "lighting_alpha_tests.rs"]
mod alpha_tests;
