// Rust port of DataExchange/TKXCAF/XCAFPrs/XCAFPrs_Style.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFPrs_Style`: the presentation style of a shape (surface and curve colors, a
//! visualization material and the visibility).
use crate::xcaf_doc::XCAFDocVisMaterial;
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkernel::quantity::{Color, ColorRgba, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, hash_bytes};
use rustycad_tkservice::image::ImageTextureSource;
use rustycad_tkservice::json_dump::{class_begin, field_color, field_dumped, field_integer};
use std::io;

/// `XCAFPrs_Style`.
///
/// ```
/// use rustycad_tkernel::quantity::{Color, NamedColor};
/// use rustycad_tkxcaf::xcaf_prs::XCAFPrsStyle;
/// let mut style = XCAFPrsStyle::new();
/// assert!(style.is_empty());
/// style.set_color_surf(Color::named(NamedColor::RED));
/// assert!(style.is_set_color_surf() && !style.is_empty());
/// ```
#[derive(Clone)]
pub struct XCAFPrsStyle {
    material: Handle<XCAFDocVisMaterial>,
    color_surf: ColorRgba,
    color_curv: Color,
    has_color_surf: bool,
    has_color_curv: bool,
    is_visible: bool,
}

impl Default for XCAFPrsStyle {
    fn default() -> Self {
        Self {
            material: Handle::default(),
            color_surf: ColorRgba::from_rgb(Color::named(NamedColor::YELLOW), 1.0),
            color_curv: Color::named(NamedColor::YELLOW),
            has_color_surf: false,
            has_color_curv: false,
            is_visible: true,
        }
    }
}

impl XCAFPrsStyle {
    /// `XCAFPrs_Style()`: visible, without colors nor material.
    pub fn new() -> Self {
        Self::default()
    }
    /// `IsEmpty`: no color, no material and visible.
    pub fn is_empty(&self) -> bool {
        !self.has_color_surf && !self.has_color_curv && self.material.is_null() && self.is_visible
    }
    /// `Material`.
    pub fn material(&self) -> &Handle<XCAFDocVisMaterial> {
        &self.material
    }
    /// `SetMaterial(theMaterial)`.
    pub fn set_material(&mut self, material: &Handle<XCAFDocVisMaterial>) {
        self.material = material.clone();
    }
    /// `IsSetColorSurf`.
    pub fn is_set_color_surf(&self) -> bool {
        self.has_color_surf
    }
    /// `GetColorSurf`: the RGB part of the surface color.
    pub fn get_color_surf(&self) -> Color {
        *self.color_surf.rgb()
    }
    /// `SetColorSurf(const Quantity_Color&)`: an opaque surface color.
    pub fn set_color_surf(&mut self, color: Color) {
        self.set_color_surf_rgba(ColorRgba::from_rgb(color, 1.0));
    }
    /// `GetColorSurfRGBA`.
    pub fn get_color_surf_rgba(&self) -> ColorRgba {
        self.color_surf
    }
    /// `SetColorSurf(const Quantity_ColorRGBA&)`.
    pub fn set_color_surf_rgba(&mut self, color: ColorRgba) {
        self.color_surf = color;
        self.has_color_surf = true;
    }
    /// `UnSetColorSurf`: back to an opaque yellow, unset.
    pub fn unset_color_surf(&mut self) {
        self.has_color_surf = false;
        self.color_surf = ColorRgba::from_rgb(Color::named(NamedColor::YELLOW), 1.0);
    }
    /// `IsSetColorCurv`.
    pub fn is_set_color_curv(&self) -> bool {
        self.has_color_curv
    }
    /// `GetColorCurv`.
    pub fn get_color_curv(&self) -> Color {
        self.color_curv
    }
    /// `SetColorCurv(col)`.
    pub fn set_color_curv(&mut self, color: Color) {
        self.color_curv = color;
        self.has_color_curv = true;
    }
    /// `UnSetColorCurv`: back to yellow, unset.
    pub fn unset_color_curv(&mut self) {
        self.has_color_curv = false;
        self.color_curv = Color::named(NamedColor::YELLOW);
    }
    /// `SetVisibility(theVisibility)`.
    pub fn set_visibility(&mut self, visible: bool) {
        self.is_visible = visible;
    }
    /// `IsVisible`.
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
}

/// Texture, equality, hashing and dump.
impl XCAFPrsStyle {
    /// `BaseColorTexture`: the base color texture of the PBR definition, else the diffuse
    /// texture of the common one, else null.
    pub fn base_color_texture(&self) -> Handle<dyn ImageTextureSource> {
        let Some(material) = self.material.get() else {
            return Handle::default();
        };
        let pbr = material.pbr_material();
        if material.has_pbr_material() && !pbr.base_color_texture.is_null() {
            return pbr.base_color_texture;
        }
        let common = material.common_material();
        if material.has_common_material() && !common.diffuse_texture.is_null() {
            return common.diffuse_texture;
        }
        Handle::default()
    }
    /// `IsEqual(theOther)` (and `operator==`): both invisible, or the same flags, the same
    /// material object and equal set colors.
    pub fn is_equal(&self, other: &Self) -> bool {
        if self.is_visible != other.is_visible {
            return false;
        }
        if !self.is_visible {
            return true;
        }
        self.has_color_surf == other.has_color_surf
            && self.has_color_curv == other.has_color_curv
            && self.material.object_address() == other.material.object_address()
            && (!self.has_color_surf || self.color_surf.is_equal(other.color_surf))
            && (!self.has_color_curv || self.color_curv.is_equal(other.color_curv))
    }
    /// `std::hash<XCAFPrs_Style>`: 1 when invisible, else the hash of the set colors and the
    /// material address (0 without any).
    pub fn hash_code(&self) -> usize {
        if !self.is_visible {
            return 1;
        }
        let mut parts = Vec::new();
        if self.has_color_surf {
            parts.push(self.color_surf.occt_hash());
        }
        if self.has_color_curv {
            parts.push(self.color_curv.occt_hash());
        }
        if !self.material.is_null() {
            parts.push(self.material.object_address() as usize);
        }
        if parts.is_empty() {
            return 0;
        }
        let bytes: Vec<u8> = parts.iter().flat_map(|part| part.to_ne_bytes()).collect();
        hash_bytes(&bytes)
    }
    /// `DumpJson`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFPrs_Style")?;
        field_dumped(stream, depth, "&myColorSurf", &|s, _| {
            self.color_surf.dump_json(s)
        })?;
        field_color(stream, depth, "&myColorCurv", self.color_curv)?;
        field_integer(stream, "myHasColorSurf", i32::from(self.has_color_surf))?;
        field_integer(stream, "myHasColorCurv", i32::from(self.has_color_curv))?;
        field_integer(stream, "myIsVisible", i32::from(self.is_visible))
    }
}

impl PartialEq for XCAFPrsStyle {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
