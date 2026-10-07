//! `Graphic3d_TextureParams`: the sampling and coordinate-generation parameters of a texture.
use super::{
    Graphic3dLevelOfTextureAnisotropy, Graphic3dTextureUnit, Graphic3dTypeOfTextureFilter,
    Graphic3dTypeOfTextureMode,
};
use rustycad_tkernel::ncollection::{Vec2, Vec4};
use rustycad_tkernel::standard::{SharedState, StandardTransient};

/// The fields of `Graphic3d_TextureParams`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct TextureParamsFields {
    gen_plane_s: Vec4<f32>,
    gen_plane_t: Vec4<f32>,
    scale: Vec2<f32>,
    translation: Vec2<f32>,
    sampler_revision: u32,
    texture_unit: Graphic3dTextureUnit,
    filter: Graphic3dTypeOfTextureFilter,
    aniso_level: Graphic3dLevelOfTextureAnisotropy,
    gen_mode: Graphic3dTypeOfTextureMode,
    base_level: i32,
    max_level: i32,
    rot_angle: f32,
    to_modulate: bool,
    to_repeat: bool,
}

/// Texture parameters; the sampler revision counts the changes of the sampler state.
///
/// Example: `let params = Graphic3dTextureParams::default(); params.set_repeat(true);`.
pub struct Graphic3dTextureParams {
    transient: StandardTransient,
    fields: SharedState<TextureParamsFields>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dTextureParams,
    "Graphic3d_TextureParams",
    StandardTransient,
    transient
);

impl Default for Graphic3dTextureParams {
    /// Unit BaseColor, nearest filter, no anisotropy, manual coordinates, levels 0..1000.
    fn default() -> Self {
        let fields = TextureParamsFields {
            gen_plane_s: Vec4::new(0.0, 0.0, 0.0, 0.0),
            gen_plane_t: Vec4::new(0.0, 0.0, 0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            translation: Vec2::new(0.0, 0.0),
            sampler_revision: 0,
            texture_unit: Graphic3dTextureUnit::BASE_COLOR,
            filter: Graphic3dTypeOfTextureFilter::Nearest,
            aniso_level: Graphic3dLevelOfTextureAnisotropy::Off,
            gen_mode: Graphic3dTypeOfTextureMode::Manual,
            base_level: 0,
            max_level: 1000,
            rot_angle: 0.0,
            to_modulate: false,
            to_repeat: false,
        };
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
}

/// Unit, modulation and sampler state.
impl Graphic3dTextureParams {
    /// The default texture unit; `params.texture_unit()`.
    pub fn texture_unit(&self) -> Graphic3dTextureUnit {
        self.fields.read().texture_unit
    }
    /// Sets the texture unit; `params.set_texture_unit(Graphic3dTextureUnit::Unit1)`.
    pub fn set_texture_unit(&self, unit: Graphic3dTextureUnit) {
        self.fields.write().texture_unit = unit;
    }
    /// Whether the texture modulates the material color; `params.is_modulate()`.
    pub fn is_modulate(&self) -> bool {
        self.fields.read().to_modulate
    }
    /// Sets the modulation flag; `params.set_modulate(true)`.
    pub fn set_modulate(&self, to_modulate: bool) {
        self.fields.write().to_modulate = to_modulate;
    }
    /// Whether coordinates repeat outside [0, 1]; `params.is_repeat()`.
    pub fn is_repeat(&self) -> bool {
        self.fields.read().to_repeat
    }
    /// Sets the repeat flag; a change increments the sampler revision. `params.set_repeat(true)`.
    pub fn set_repeat(&self, to_repeat: bool) {
        let mut fields = self.fields.write();
        if fields.to_repeat != to_repeat {
            fields.to_repeat = to_repeat;
            fields.sampler_revision = fields.sampler_revision.wrapping_add(1);
        }
    }
    /// The texture filter; `params.filter()`.
    pub fn filter(&self) -> Graphic3dTypeOfTextureFilter {
        self.fields.read().filter
    }
    /// Sets the filter; a change increments the sampler revision.
    /// `params.set_filter(Graphic3dTypeOfTextureFilter::Trilinear)`.
    pub fn set_filter(&self, filter: Graphic3dTypeOfTextureFilter) {
        let mut fields = self.fields.write();
        if fields.filter != filter {
            fields.filter = filter;
            fields.sampler_revision = fields.sampler_revision.wrapping_add(1);
        }
    }
    /// The anisotropy level; `params.aniso_filter()`.
    pub fn aniso_filter(&self) -> Graphic3dLevelOfTextureAnisotropy {
        self.fields.read().aniso_level
    }
    /// Sets the anisotropy level; a change increments the sampler revision.
    /// `params.set_aniso_filter(Graphic3dLevelOfTextureAnisotropy::Fast)`.
    pub fn set_aniso_filter(&self, level: Graphic3dLevelOfTextureAnisotropy) {
        let mut fields = self.fields.write();
        if fields.aniso_level != level {
            fields.aniso_level = level;
            fields.sampler_revision = fields.sampler_revision.wrapping_add(1);
        }
    }
    /// The mipmap base level; `params.base_level()`.
    pub fn base_level(&self) -> i32 {
        self.fields.read().base_level
    }
    /// The mipmap maximum level; `params.max_level()`.
    pub fn max_level(&self) -> i32 {
        self.fields.read().max_level
    }
    /// Sets the mipmap range from two levels in any order; `params.set_levels_range(4, 0)`.
    pub fn set_levels_range(&self, first: i32, second: i32) {
        let mut fields = self.fields.write();
        fields.max_level = if first > second { first } else { second };
        fields.base_level = if first > second { second } else { first };
    }
    /// The sampler-state modification counter; `params.sampler_revision()`.
    pub fn sampler_revision(&self) -> u32 {
        self.fields.read().sampler_revision
    }
}

/// Coordinate transformation and generation.
impl Graphic3dTextureParams {
    /// The coordinate rotation in degrees; `params.rotation()`.
    pub fn rotation(&self) -> f32 {
        self.fields.read().rot_angle
    }
    /// Sets the rotation in degrees; `params.set_rotation(45.0)`.
    pub fn set_rotation(&self, angle_degrees: f32) {
        self.fields.write().rot_angle = angle_degrees;
    }
    /// The coordinate scale; `params.scale()`.
    pub fn scale(&self) -> Vec2<f32> {
        self.fields.read().scale
    }
    /// Sets the coordinate scale; `params.set_scale(Vec2::new(2.0, 2.0))`.
    pub fn set_scale(&self, scale: Vec2<f32>) {
        self.fields.write().scale = scale;
    }
    /// The coordinate translation; `params.translation()`.
    pub fn translation(&self) -> Vec2<f32> {
        self.fields.read().translation
    }
    /// Sets the coordinate translation; `params.set_translation(Vec2::new(0.5, 0.0))`.
    pub fn set_translation(&self, translation: Vec2<f32>) {
        self.fields.write().translation = translation;
    }
    /// The coordinate generation mode; `params.gen_mode()`.
    pub fn gen_mode(&self) -> Graphic3dTypeOfTextureMode {
        self.fields.read().gen_mode
    }
    /// The generation plane S; `params.gen_plane_s()`.
    pub fn gen_plane_s(&self) -> Vec4<f32> {
        self.fields.read().gen_plane_s
    }
    /// The generation plane T; `params.gen_plane_t()`.
    pub fn gen_plane_t(&self) -> Vec4<f32> {
        self.fields.read().gen_plane_t
    }
    /// Sets the generation mode and planes; `params.set_gen_mode(mode, plane_s, plane_t)`.
    pub fn set_gen_mode(
        &self,
        mode: Graphic3dTypeOfTextureMode,
        plane_s: Vec4<f32>,
        plane_t: Vec4<f32>,
    ) {
        let mut fields = self.fields.write();
        (fields.gen_mode, fields.gen_plane_s, fields.gen_plane_t) = (mode, plane_s, plane_t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampler_revision_counts_sampler_changes_only() {
        let params = Graphic3dTextureParams::default();
        params.set_repeat(true);
        params.set_repeat(true);
        params.set_filter(Graphic3dTypeOfTextureFilter::Trilinear);
        params.set_aniso_filter(Graphic3dLevelOfTextureAnisotropy::Off);
        params.set_modulate(true);
        params.set_rotation(30.0);
        assert_eq!(params.sampler_revision(), 2);
        assert!(params.is_modulate() && params.is_repeat());
    }

    #[test]
    fn levels_are_ordered() {
        let params = Graphic3dTextureParams::default();
        assert_eq!((params.base_level(), params.max_level()), (0, 1000));
        params.set_levels_range(5, 2);
        assert_eq!((params.base_level(), params.max_level()), (2, 5));
    }

    #[test]
    fn generation_mode_keeps_both_planes() {
        let params = Graphic3dTextureParams::default();
        assert_eq!(params.gen_mode(), Graphic3dTypeOfTextureMode::Manual);
        params.set_gen_mode(
            Graphic3dTypeOfTextureMode::Object,
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
        );
        assert_eq!(params.gen_plane_t().y, 1.0);
        assert_eq!(params.scale(), Vec2::new(1.0, 1.0));
    }
}
