//! Packed material records translated from OpenGl_Material.hxx at OCCT 3d097a0.
/// Phong coefficients in native vec4 order; `RenderMaterialCommon::default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMaterialCommon {
    /// Diffuse RGB coefficients and opacity.
    pub diffuse: [f32; 4],
    /// Emissive RGB coefficients and reserved component.
    pub emission: [f32; 4],
    /// Specular RGB coefficients and Phong exponent.
    pub specular_shininess: [f32; 4],
    /// Ambient RGB coefficients and reserved component.
    pub ambient: [f32; 4],
}
impl Default for RenderMaterialCommon {
    fn default() -> Self {
        Self {
            diffuse: [1.; 4],
            emission: [1.; 4],
            specular_shininess: [1., 1., 1., 0.],
            ambient: [1.; 4],
        }
    }
}
impl RenderMaterialCommon {
    /// Read the unnormalized Phong exponent; `material.shine()`.
    pub fn shine(&self) -> f32 {
        self.specular_shininess[3]
    }
    /// Mutate the exponent directly, as in OCCT; `*material.change_shine() = 32.`.
    pub fn change_shine(&mut self) -> &mut f32 {
        &mut self.specular_shininess[3]
    }
    /// Highlight RGB without changing alpha, emission or gloss; `material.set_color([1.,0.,0.])`.
    pub fn set_color(&mut self, color: [f32; 3]) {
        self.ambient[..3].copy_from_slice(&color.map(|channel| channel * 0.25));
        self.diffuse[..3].copy_from_slice(&color);
    }
    pub(crate) fn packed(&self) -> [[f32; 4]; 4] {
        [
            self.diffuse,
            self.emission,
            self.specular_shininess,
            self.ambient,
        ]
    }
}
/// Metallic-roughness coefficients in native vec4 order; `RenderMaterialPbr::default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMaterialPbr {
    /// PBR base RGB and opacity.
    pub base_color: [f32; 4],
    /// PBR emission RGB and index of refraction.
    pub emission_ior: [f32; 4],
    /// Reserved, normalized roughness, metallicity, reserved.
    pub params: [f32; 4],
}
impl Default for RenderMaterialPbr {
    fn default() -> Self {
        Self {
            base_color: [1.; 4],
            emission_ior: [1.; 4],
            params: [1.; 4],
        }
    }
}
impl RenderMaterialPbr {
    /// Read metallicity from native channel b; `material.metallic()`.
    pub fn metallic(&self) -> f32 {
        self.params[2]
    }
    /// Mutate metallicity without validation, as in OCCT; `*material.change_metallic() = 0.5`.
    pub fn change_metallic(&mut self) -> &mut f32 {
        &mut self.params[2]
    }
    /// Read normalized roughness from native channel g; `material.roughness()`.
    pub fn roughness(&self) -> f32 {
        self.params[1]
    }
    /// Mutate normalized roughness directly; `*material.change_roughness() = 0.2`.
    pub fn change_roughness(&mut self) -> &mut f32 {
        &mut self.params[1]
    }
    /// Highlight RGB while preserving opacity; `material.set_color([1.,0.,0.])`.
    pub fn set_color(&mut self, color: [f32; 3]) {
        self.base_color[..3].copy_from_slice(&color);
    }
    pub(crate) fn packed(&self) -> [[f32; 4]; 3] {
        [self.base_color, self.emission_ior, self.params]
    }
}
