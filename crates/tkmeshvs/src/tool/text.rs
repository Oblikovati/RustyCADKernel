//! Font bytes follow native ToCString termination, without requiring UTF-8.
use super::{
    MeshVSTool,
    values::{DrawerValues, native_enum, required_value},
};
use crate::{MeshVSDrawer, MeshVSDrawerAttribute as Key, MeshVSDrawerSource};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, HandleReference},
    tcollection::AsciiString,
};
use rustycad_tkservice::{
    aspect::{AspectTypeOfDisplayText, AspectTypeOfStyleText},
    font::FontFontAspect,
    graphic3d::{Graphic3dAspectText3d, Graphic3dAspectsTextMembers},
};
struct TextValues {
    color: Color,
    expansion: f64,
    space: f64,
    font: AsciiString,
    style: i32,
    display: i32,
    font_aspect: i32,
}
impl MeshVSTool {
    /// Native text factory including bold default; `Self::create_aspect_text3d(&drawer,true)?`.
    pub fn create_aspect_text3d(
        drawer: &dyn HandleReference<dyn MeshVSDrawerSource>,
        use_defaults: bool,
    ) -> Result<Handle<Graphic3dAspectText3d>, Failure> {
        if drawer.is_null() {
            return Ok(Handle::default());
        }
        let Some(values) = text_values(&DrawerValues::new(drawer, use_defaults))? else {
            return Ok(Handle::default());
        };
        Ok(Handle::new(make_text(values)?))
    }
}
fn text_values(values: &DrawerValues<'_>) -> Result<Option<TextValues>, Failure> {
    let color = required_value!(values.read(
        Key::TextColor,
        Color::named(NamedColor::YELLOW),
        MeshVSDrawer::get_color
    ));
    let expansion =
        required_value!(values.read(Key::TextExpansionFactor, 1., MeshVSDrawer::get_double));
    let space = required_value!(values.read(Key::TextSpace, 0., MeshVSDrawer::get_double));
    let font = required_value!(values.read(
        Key::TextFont,
        AsciiString::new("Courier"),
        MeshVSDrawer::get_ascii_string
    ));
    text_style_values(values, text_parameters(color, expansion, space, font))
}
fn text_style_values(
    values: &DrawerValues<'_>,
    mut text: TextValues,
) -> Result<Option<TextValues>, Failure> {
    text.style = required_value!(values.read(Key::TextStyle, 0, MeshVSDrawer::get_integer));
    text.display = required_value!(values.read(Key::TextDisplayType, 0, MeshVSDrawer::get_integer));
    text.font_aspect =
        required_value!(values.read(Key::TextFontAspect, 1, MeshVSDrawer::get_integer));
    Ok(Some(text))
}
fn make_text(values: TextValues) -> Result<Graphic3dAspectText3d, Failure> {
    let style = text_style(values.style)?;
    let display = text_display(values.display)?;
    let font_aspect = font_aspect(values.font_aspect)?;
    let text = Graphic3dAspectText3d::new(
        values.color,
        None,
        values.expansion,
        values.space,
        style,
        display,
    );
    text.set_font(&terminated_font(&values.font));
    text.set_text_font_aspect(font_aspect);
    Ok(text)
}
fn terminated_font(font: &AsciiString) -> AsciiString {
    let bytes = font.as_bytes();
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    AsciiString::from_view(&bytes[..end])
}

fn text_parameters(color: Color, expansion: f64, space: f64, font: AsciiString) -> TextValues {
    TextValues {
        color,
        expansion,
        space,
        font,
        style: 0,
        display: 0,
        font_aspect: 1,
    }
}
fn text_style(value: i32) -> Result<AspectTypeOfStyleText, Failure> {
    native_enum(
        value,
        AspectTypeOfStyleText::from_native_value(value),
        "Aspect_TypeOfStyleText",
    )
}
fn text_display(value: i32) -> Result<AspectTypeOfDisplayText, Failure> {
    native_enum(
        value,
        AspectTypeOfDisplayText::from_native_value(value),
        "Aspect_TypeOfDisplayText",
    )
}
fn font_aspect(value: i32) -> Result<FontFontAspect, Failure> {
    native_enum(
        value,
        FontFontAspect::from_native_value(value),
        "Font_FontAspect",
    )
}
