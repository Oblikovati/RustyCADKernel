//! `Prs3d_TextAspect`: the text attributes of a presentation.
use super::basic_aspect::{Prs3dBasicAspect, dump_graphic_aspects};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::{AspectTypeOfDisplayText, AspectTypeOfStyleText};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectText3d, Graphic3dAspectsMembers as _, Graphic3dAspectsTextMembers as _,
    Graphic3dHorizontalTextAlignment, Graphic3dTextPath, Graphic3dVerticalTextAlignment,
};
use rustycad_tkservice::json_dump::{class_begin, field_integer, field_real};
use std::io;

/// `Font_NOF_SERIF`, the font of the default text aspect.
const FONT_NOF_SERIF: &str = "serif";

/// The fields of a text aspect besides the Graphic3d aspect.
#[derive(Clone, Copy)]
struct Prs3dTextLayout {
    height: f64,
    horizontal: Graphic3dHorizontalTextAlignment,
    vertical: Graphic3dVerticalTextAlignment,
    orientation: Graphic3dTextPath,
}

impl Default for Prs3dTextLayout {
    /// Height 16, left, bottom, rightwards.
    fn default() -> Self {
        Self {
            height: 16.0,
            horizontal: Graphic3dHorizontalTextAlignment::Left,
            vertical: Graphic3dVerticalTextAlignment::Bottom,
            orientation: Graphic3dTextPath::Right,
        }
    }
}

/// The text attributes of a presentation.
///
/// Example: `let aspect = Prs3dTextAspect::new(); aspect.set_height(12.0);`.
pub struct Prs3dTextAspect {
    transient: StandardTransient,
    text_aspect: SharedState<Handle<Graphic3dAspectText3d>>,
    layout: SharedState<Prs3dTextLayout>,
}
rustycad_tkernel::impl_transient!(Prs3dTextAspect, "Prs3d_TextAspect", dyn Prs3dBasicAspect, transient,
    interfaces: [dyn Prs3dBasicAspect]);

impl Default for Prs3dTextAspect {
    fn default() -> Self {
        Self::new()
    }
}

impl Prs3dTextAspect {
    /// The default: yellow serif text of height 16; `Prs3dTextAspect::new()`.
    pub fn new() -> Self {
        let aspect = Graphic3dAspectText3d::new(
            Color::named(NamedColor::YELLOW),
            Some(FONT_NOF_SERIF),
            1.0,
            0.0,
            AspectTypeOfStyleText::Normal,
            AspectTypeOfDisplayText::Normal,
        );
        Self::from_aspect(Handle::new(aspect))
    }
    /// Wraps an existing Graphic3d text aspect; `Prs3dTextAspect::from_aspect(aspect)`.
    pub fn from_aspect(aspect: Handle<Graphic3dAspectText3d>) -> Self {
        Self {
            transient: StandardTransient::default(),
            text_aspect: SharedState::new(aspect),
            layout: SharedState::new(Prs3dTextLayout::default()),
        }
    }
    /// Sets the text color; `aspect.set_color(red)`.
    pub fn set_color(&self, color: Color) {
        if let Some(text) = self.aspect().get() {
            text.set_interior_color(color);
        }
    }
    /// Sets the font; `aspect.set_font("Arial")`.
    pub fn set_font(&self, font: &str) {
        if let Some(text) = self.aspect().get() {
            text.set_font(&AsciiString::from(font));
        }
    }
    /// Sets the height; `aspect.set_height(12.0)`.
    pub fn set_height(&self, height: f64) {
        self.layout.write().height = height;
    }
    /// Sets the angle of the Graphic3d aspect (narrowed to float); `aspect.set_angle(30.0)`.
    pub fn set_angle(&self, angle: f64) {
        if let Some(text) = self.aspect().get() {
            text.set_text_angle_f64(angle);
        }
    }
    /// The height; `aspect.height()`.
    pub fn height(&self) -> f64 {
        self.layout.read().height
    }
    /// The angle of the Graphic3d aspect; `aspect.angle()`.
    pub fn angle(&self) -> f64 {
        self.aspect()
            .get()
            .map_or(0.0, |text| f64::from(text.text_angle()))
    }
    /// Sets the horizontal justification; `aspect.set_horizontal_justification(Graphic3dHorizontalTextAlignment::Center)`.
    pub fn set_horizontal_justification(&self, justification: Graphic3dHorizontalTextAlignment) {
        self.layout.write().horizontal = justification;
    }
    /// Sets the vertical justification; `aspect.set_vertical_justification(Graphic3dVerticalTextAlignment::Top)`.
    pub fn set_vertical_justification(&self, justification: Graphic3dVerticalTextAlignment) {
        self.layout.write().vertical = justification;
    }
    /// Sets the orientation; `aspect.set_orientation(Graphic3dTextPath::Up)`.
    pub fn set_orientation(&self, orientation: Graphic3dTextPath) {
        self.layout.write().orientation = orientation;
    }
    /// The horizontal justification; `aspect.horizontal_justification()`.
    pub fn horizontal_justification(&self) -> Graphic3dHorizontalTextAlignment {
        self.layout.read().horizontal
    }
    /// The vertical justification; `aspect.vertical_justification()`.
    pub fn vertical_justification(&self) -> Graphic3dVerticalTextAlignment {
        self.layout.read().vertical
    }
    /// The orientation; `aspect.orientation()`.
    pub fn orientation(&self) -> Graphic3dTextPath {
        self.layout.read().orientation
    }
    /// The Graphic3d aspect; `aspect.aspect()`.
    pub fn aspect(&self) -> Handle<Graphic3dAspectText3d> {
        self.text_aspect.read().clone()
    }
    /// Replaces the Graphic3d aspect; `aspect.set_aspect(other)`.
    pub fn set_aspect(&self, aspect: Handle<Graphic3dAspectText3d>) {
        *self.text_aspect.write() = aspect;
    }
}

impl Prs3dBasicAspect for Prs3dTextAspect {
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Prs3d_TextAspect")?;
        dump_graphic_aspects(stream, depth, "myTextAspect.get()", &self.aspect())?;
        let layout = *self.layout.read();
        field_real(stream, "myHeight", layout.height)?;
        field_integer(
            stream,
            "myHorizontalJustification",
            layout.horizontal as i32,
        )?;
        field_integer(stream, "myVerticalJustification", layout.vertical as i32)?;
        field_integer(stream, "myOrientation", layout.orientation as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_setters() {
        let aspect = Prs3dTextAspect::new();
        assert_eq!(aspect.height(), 16.0);
        assert_eq!(aspect.aspect().font().as_bytes(), b"serif");
        aspect.set_font("Arial");
        aspect.set_angle(30.0);
        aspect.set_height(10.0);
        aspect.set_horizontal_justification(Graphic3dHorizontalTextAlignment::Center);
        aspect.set_vertical_justification(Graphic3dVerticalTextAlignment::Top);
        aspect.set_orientation(Graphic3dTextPath::Up);
        assert_eq!(aspect.aspect().font().as_bytes(), b"Arial");
        assert_eq!(aspect.angle(), 30.0);
        assert_eq!(
            aspect.horizontal_justification(),
            Graphic3dHorizontalTextAlignment::Center
        );
        assert_eq!(
            aspect.vertical_justification(),
            Graphic3dVerticalTextAlignment::Top
        );
        assert_eq!(aspect.orientation(), Graphic3dTextPath::Up);
        let mut stream = DumpStream::default();
        aspect.dump_json(&mut stream, 0).unwrap();
        assert!(stream.text().ends_with("\"Height\": 10, \"HorizontalJustification\": 1, \"VerticalJustification\": 2, \"Orientation\": 0"));
    }
}
