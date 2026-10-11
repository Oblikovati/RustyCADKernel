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
    world: [f32; 16],
    normal: [f32; 16],
    eye: [f32; 4],
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
                self.shading as u32,
                self.lights.source_count() as u32,
                u32::from(self.vertex_colors),
                u32::from(self.front_vertex_colors_only),
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
    if material.common.iter().any(|face| face.diffuse[3] != 1.) {
        return Err(rejected(
            material.common,
            "opaque front/back diffuse alpha equal to 1",
        ));
    }
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
impl PrimitiveDraw {
    /// Attach opaque surface or nodal-marker lighting with one normal per vertex; `draw.with_lighting(state)?`.
    pub fn with_lighting(mut self, lighting: RenderLighting) -> Result<Self, RenderError> {
        if lighting.normals.len() != self.vertices.len()
            || (self.topology() != crate::PrimitiveTopology::Triangles
                && self.point_style.is_none())
            || (self.point_style.is_some() && lighting.shading == RenderShading::PhongFacet)
            || self.textured_style.is_some()
            || self.capping_pass.is_mask()
        {
            return Err(rejected(
                (
                    lighting.normals.len(),
                    self.vertices.len(),
                    self.topology(),
                    self.textured_style.is_some(),
                    self.capping_pass,
                ),
                "one normal per untextured triangle or Gouraud/Phong marker vertex outside stencil masks",
            ));
        }
        self.lighting = Some(lighting);
        Ok(self)
    }
    /// Inspect a retained lighting snapshot; `draw.lighting()`.
    pub fn lighting(&self) -> Option<&RenderLighting> {
        self.lighting.as_ref()
    }
}

#[cfg(test)]
#[path = "lighting_tests.rs"]
mod tests;
