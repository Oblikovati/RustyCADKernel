// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_VisMaterialCommon.hxx and
// XCAFDoc_VisMaterialPBR.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two definitions of a visualization material: the common (Phong) one and the
//! metal-roughness (PBR) one.
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::quantity::{Color, ColorRgba, ColorSpace};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::image::ImageTextureSource;
use rustycad_tkservice::json_dump::{
    class_begin, field_color, field_dumped, field_integer, field_real,
};
use std::io;

/// An RGB color of constant components (always within range).
fn rgb(value: f64) -> Color {
    Color::new(value, value, value, ColorSpace::Rgb).expect("components within [0, 1]")
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED(stream, depth, texture.get())`: skipped for a null texture.
fn field_texture(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    texture: &Handle<dyn ImageTextureSource>,
) -> io::Result<()> {
    match texture.get() {
        Some(texture) => field_dumped(stream, depth, field, &|s, d| texture.dump_json(s, d)),
        None => Ok(()),
    }
}

/// The same texture object, or both null (the handle `==`).
fn same_texture(
    first: &Handle<dyn ImageTextureSource>,
    second: &Handle<dyn ImageTextureSource>,
) -> bool {
    first.object_address() == second.object_address()
}

/// `XCAFDoc_VisMaterialCommon`: the common (Phong) material definition.
#[derive(Clone)]
pub struct XCAFDocVisMaterialCommon {
    /// `DiffuseTexture`: the image of the diffuse color.
    pub diffuse_texture: Handle<dyn ImageTextureSource>,
    /// `AmbientColor`: 0.1 gray by default.
    pub ambient_color: Color,
    /// `DiffuseColor`: 0.8 gray by default.
    pub diffuse_color: Color,
    /// `SpecularColor`: 0.2 gray by default.
    pub specular_color: Color,
    /// `EmissiveColor`: black by default.
    pub emissive_color: Color,
    /// `Shininess`: 1 by default.
    pub shininess: f32,
    /// `Transparency`: within [0, 1], 0 (opaque) by default.
    pub transparency: f32,
    /// `IsDefined`: true by default.
    pub is_defined: bool,
}

impl Default for XCAFDocVisMaterialCommon {
    fn default() -> Self {
        Self {
            diffuse_texture: Handle::default(),
            ambient_color: rgb(0.1),
            diffuse_color: rgb(0.8),
            specular_color: rgb(0.2),
            emissive_color: rgb(0.0),
            shininess: 1.0,
            transparency: 0.0,
            is_defined: true,
        }
    }
}

impl XCAFDocVisMaterialCommon {
    /// `IsEqual`: both undefined, or the same texture and equal values.
    ///
    /// ```
    /// use rustycad_tkxcaf::xcaf_doc::XCAFDocVisMaterialCommon;
    /// let mut other = XCAFDocVisMaterialCommon::default();
    /// assert!(other.is_equal(&XCAFDocVisMaterialCommon::default()));
    /// other.shininess = 0.5;
    /// assert!(!other.is_equal(&XCAFDocVisMaterialCommon::default()));
    /// ```
    pub fn is_equal(&self, other: &Self) -> bool {
        if other.is_defined != self.is_defined {
            return false;
        }
        if !self.is_defined {
            return true;
        }
        same_texture(&other.diffuse_texture, &self.diffuse_texture)
            && other.ambient_color.is_equal(self.ambient_color)
            && other.diffuse_color.is_equal(self.diffuse_color)
            && other.specular_color.is_equal(self.specular_color)
            && other.emissive_color.is_equal(self.emissive_color)
            && other.shininess == self.shininess
            && other.transparency == self.transparency
    }
    /// `DumpJson`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_VisMaterialCommon")?;
        field_texture(stream, depth, "DiffuseTexture.get()", &self.diffuse_texture)?;
        field_color(stream, depth, "&AmbientColor", self.ambient_color)?;
        field_color(stream, depth, "&DiffuseColor", self.diffuse_color)?;
        field_color(stream, depth, "&SpecularColor", self.specular_color)?;
        field_color(stream, depth, "&EmissiveColor", self.emissive_color)?;
        field_real(stream, "Shininess", f64::from(self.shininess))?;
        field_real(stream, "Transparency", f64::from(self.transparency))?;
        field_integer(stream, "IsDefined", i32::from(self.is_defined))
    }
}

/// `XCAFDoc_VisMaterialPBR`: the metal-roughness material definition.
#[derive(Clone)]
pub struct XCAFDocVisMaterialPbr {
    /// `BaseColorTexture`: RGB texture of the base color.
    pub base_color_texture: Handle<dyn ImageTextureSource>,
    /// `MetallicRoughnessTexture`: RG texture of the metalness and roughness.
    pub metallic_roughness_texture: Handle<dyn ImageTextureSource>,
    /// `EmissiveTexture`: RGB emission map.
    pub emissive_texture: Handle<dyn ImageTextureSource>,
    /// `OcclusionTexture`: R occlusion map.
    pub occlusion_texture: Handle<dyn ImageTextureSource>,
    /// `NormalTexture`: normal map.
    pub normal_texture: Handle<dyn ImageTextureSource>,
    /// `BaseColor`: opaque white by default.
    pub base_color: ColorRgba,
    /// `EmissiveFactor`: black by default.
    pub emissive_factor: Vec3<f32>,
    /// `Metallic`: within [0, 1], 1 by default.
    pub metallic: f32,
    /// `Roughness`: within [0, 1], 1 by default.
    pub roughness: f32,
    /// `RefractionIndex`: within [1, 3], 1.5 by default.
    pub refraction_index: f32,
    /// `IsDefined`: true by default.
    pub is_defined: bool,
}

impl Default for XCAFDocVisMaterialPbr {
    fn default() -> Self {
        Self {
            base_color_texture: Handle::default(),
            metallic_roughness_texture: Handle::default(),
            emissive_texture: Handle::default(),
            occlusion_texture: Handle::default(),
            normal_texture: Handle::default(),
            base_color: ColorRgba::from_rgb(rgb(1.0), 1.0),
            emissive_factor: Vec3::new(0.0, 0.0, 0.0),
            metallic: 1.0,
            roughness: 1.0,
            refraction_index: 1.5,
            is_defined: true,
        }
    }
}

impl XCAFDocVisMaterialPbr {
    /// `IsEqual`: both undefined, or the same textures and equal values.
    ///
    /// ```
    /// use rustycad_tkxcaf::xcaf_doc::XCAFDocVisMaterialPbr;
    /// let mut other = XCAFDocVisMaterialPbr::default();
    /// other.is_defined = false;
    /// assert!(!other.is_equal(&XCAFDocVisMaterialPbr::default()));
    /// ```
    pub fn is_equal(&self, other: &Self) -> bool {
        if other.is_defined != self.is_defined {
            return false;
        }
        if !self.is_defined {
            return true;
        }
        self.same_textures(other)
            && other.base_color.is_equal(self.base_color)
            && other.emissive_factor == self.emissive_factor
            && other.metallic == self.metallic
            && other.roughness == self.roughness
            && other.refraction_index == self.refraction_index
    }
    /// The five textures are the same objects.
    fn same_textures(&self, other: &Self) -> bool {
        same_texture(&other.base_color_texture, &self.base_color_texture)
            && same_texture(
                &other.metallic_roughness_texture,
                &self.metallic_roughness_texture,
            )
            && same_texture(&other.emissive_texture, &self.emissive_texture)
            && same_texture(&other.occlusion_texture, &self.occlusion_texture)
            && same_texture(&other.normal_texture, &self.normal_texture)
    }
    /// `DumpJson`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_VisMaterialPBR")?;
        field_texture(
            stream,
            depth,
            "BaseColorTexture.get()",
            &self.base_color_texture,
        )?;
        field_texture(
            stream,
            depth,
            "MetallicRoughnessTexture.get()",
            &self.metallic_roughness_texture,
        )?;
        field_texture(
            stream,
            depth,
            "EmissiveTexture.get()",
            &self.emissive_texture,
        )?;
        field_texture(
            stream,
            depth,
            "OcclusionTexture.get()",
            &self.occlusion_texture,
        )?;
        field_texture(stream, depth, "NormalTexture.get()", &self.normal_texture)?;
        field_dumped(stream, depth, "&BaseColor", &|s, _| {
            self.base_color.dump_json(s)
        })?;
        field_dumped(stream, depth, "&EmissiveFactor", &|s, _| {
            self.emissive_factor.dump_json(s)
        })?;
        field_real(stream, "Metallic", f64::from(self.metallic))?;
        field_real(stream, "Roughness", f64::from(self.roughness))?;
        field_real(stream, "RefractionIndex", f64::from(self.refraction_index))?;
        field_integer(stream, "IsDefined", i32::from(self.is_defined))
    }
}
