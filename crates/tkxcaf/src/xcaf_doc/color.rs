// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_Color.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_Color`: the RGBA color of a color label.
use super::support::{downcast, find_or_add, live};
use rustycad_tkernel::quantity::{Color, ColorRgba, ColorSpace, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState, StandardTransient};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::json::dump_attribute_json_base;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable,
};
use rustycad_tkservice::json_dump::{class_begin, field_dumped};
use std::io;

/// `XCAFDoc_Color::GetID()`.
const COLOR_ID: Guid = guid_from_text("efd212f0-6dfd-11d4-b9c8-0060b0ee281b");

/// `XCAFDoc_Color`.
///
/// ```
/// use rustycad_tkernel::quantity::NamedColor;
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tkxcaf::xcaf_doc::XCAFDocColor;
/// let data = TdfData::new();
/// let color = XCAFDocColor::set_named(&data.root(), NamedColor::RED)?;
/// assert_eq!(color.get_noc(), NamedColor::RED);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct XCAFDocColor {
    transient: StandardTransient,
    base: TdfAttributeBase,
    color: SharedState<ColorRgba>,
}
rustycad_tkernel::impl_transient!(XCAFDocColor, "XCAFDoc_Color", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl XCAFDocColor {
    /// `XCAFDoc_Color()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        COLOR_ID
    }
    /// The color attribute of the label, added when absent.
    fn find_or_add(label: &TdfLabel) -> Result<Handle<XCAFDocColor>, Failure> {
        find_or_add(label, &COLOR_ID, Self::new)
    }
    /// `Set(L, C)` with a `Quantity_Color`.
    pub fn set(label: &TdfLabel, color: Color) -> Result<Handle<XCAFDocColor>, Failure> {
        let attribute = Self::find_or_add(label)?;
        live(&attribute, "XCAFDoc_Color::Set")?.set_color(color)?;
        Ok(attribute)
    }
    /// `Set(L, C)` with a `Quantity_ColorRGBA`.
    pub fn set_rgba(label: &TdfLabel, color: ColorRgba) -> Result<Handle<XCAFDocColor>, Failure> {
        let attribute = Self::find_or_add(label)?;
        live(&attribute, "XCAFDoc_Color::Set")?.set_color_rgba(color)?;
        Ok(attribute)
    }
    /// `Set(L, C)` with a `Quantity_NameOfColor`.
    pub fn set_named(label: &TdfLabel, color: NamedColor) -> Result<Handle<XCAFDocColor>, Failure> {
        let attribute = Self::find_or_add(label)?;
        live(&attribute, "XCAFDoc_Color::Set")?.set_noc(color)?;
        Ok(attribute)
    }
    /// `Set(L, R, G, B, alpha)`.
    pub fn set_values(
        label: &TdfLabel,
        rgb: [f64; 3],
        alpha: f64,
    ) -> Result<Handle<XCAFDocColor>, Failure> {
        let attribute = Self::find_or_add(label)?;
        live(&attribute, "XCAFDoc_Color::Set")?.set_rgb_alpha(rgb, alpha)?;
        Ok(attribute)
    }
    /// `Set(C)` with a `Quantity_Color`: backed up; the alpha stays.
    pub fn set_color(&self, color: Color) -> Result<(), Failure> {
        self.backup()?;
        self.color.write().set_rgb(color);
        Ok(())
    }
    /// `Set(C)` with a `Quantity_ColorRGBA`: backed up.
    pub fn set_color_rgba(&self, color: ColorRgba) -> Result<(), Failure> {
        self.backup()?;
        *self.color.write() = color;
        Ok(())
    }
    /// `Set(C)` with a `Quantity_NameOfColor`: backed up; the alpha stays.
    pub fn set_noc(&self, color: NamedColor) -> Result<(), Failure> {
        self.backup()?;
        self.color.write().set_rgb(Color::named(color));
        Ok(())
    }
    /// `Set(R, G, B, alpha)`: backed up; the RGB values and the alpha (as a float).
    pub fn set_rgb_alpha(&self, [red, green, blue]: [f64; 3], alpha: f64) -> Result<(), Failure> {
        self.backup()?;
        let mut rgb = Color::default();
        rgb.set_values(red, green, blue, ColorSpace::Rgb)?;
        let mut color = self.color.write();
        color.set_rgb(rgb);
        color.set_alpha(alpha as f32);
        Ok(())
    }
    /// `GetColor`.
    pub fn get_color(&self) -> Color {
        *self.color.read().rgb()
    }
    /// `GetColorRGBA`.
    pub fn get_color_rgba(&self) -> ColorRgba {
        *self.color.read()
    }
    /// `GetNOC`: the name nearest to the color.
    pub fn get_noc(&self) -> NamedColor {
        self.get_color().name()
    }
    /// `GetRGB(R, G, B)`.
    pub fn get_rgb(&self) -> [f64; 3] {
        self.get_color().values(ColorSpace::Rgb)
    }
    /// `GetAlpha`.
    pub fn get_alpha(&self) -> f32 {
        self.color.read().alpha()
    }
}

impl TdfAttribute for XCAFDocColor {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        COLOR_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = downcast::<XCAFDocColor>(with, "XCAFDoc_Color::Restore")?;
        *self.color.write() = live(&other, "XCAFDoc_Color::Restore")?.get_color_rgba();
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = downcast::<XCAFDocColor>(into, "XCAFDoc_Color::Paste")?;
        live(&target, "XCAFDoc_Color::Paste")?.set_color_rgba(self.get_color_rgba())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_Color")?;
        dump_attribute_json_base(self, stream, depth)?;
        let color = self.get_color_rgba();
        field_dumped(stream, depth, "&myColor", &|nested, _| {
            color.dump_json(nested)
        })
    }
}
