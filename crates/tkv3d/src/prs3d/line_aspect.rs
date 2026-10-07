//! `Prs3d_LineAspect` and its subclass `Prs3d_IsoAspect`. The iso aspect embeds the line
//! aspect as its parent, so a `Handle<Prs3dIsoAspect>` casts to `Handle<Prs3dLineAspect>`
//! (geom-design.md section 2); neither class has another virtual member than `DumpJson`.
use super::basic_aspect::{Prs3dBasicAspect, dump_graphic_aspects};
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::graphic3d::{Graphic3dAspectLine3d, Graphic3dAspectsMembers as _};
use rustycad_tkservice::json_dump::class_begin;
use std::io;

/// The line attributes of a presentation.
///
/// Example: `let aspect = Prs3dLineAspect::new(red, AspectTypeOfLine::Solid, 1.0)?; aspect.set_width(2.0)?;`.
pub struct Prs3dLineAspect {
    transient: StandardTransient,
    aspect: SharedState<Handle<Graphic3dAspectLine3d>>,
}
rustycad_tkernel::impl_transient!(Prs3dLineAspect, "Prs3d_LineAspect", dyn Prs3dBasicAspect, transient,
    interfaces: [dyn Prs3dBasicAspect]);

impl Prs3dLineAspect {
    /// A new line aspect of a color, type and width; a width <= 0 raises `Standard_OutOfRange`.
    /// Example: `Prs3dLineAspect::new(red, AspectTypeOfLine::Dash, 1.0)?`.
    pub fn new(color: Color, kind: AspectTypeOfLine, width: f64) -> Result<Self, Failure> {
        let aspect = Graphic3dAspectLine3d::new(color, kind, width)?;
        Ok(Self::from_aspect(Handle::new(aspect)))
    }
    /// Wraps an existing Graphic3d aspect; `Prs3dLineAspect::from_aspect(aspect)`.
    pub fn from_aspect(aspect: Handle<Graphic3dAspectLine3d>) -> Self {
        Self {
            transient: StandardTransient::default(),
            aspect: SharedState::new(aspect),
        }
    }
    /// Sets the color of the Graphic3d aspect; `aspect.set_color(red)`.
    pub fn set_color(&self, color: Color) {
        if let Some(aspect) = self.aspect().get() {
            aspect.set_interior_color(color);
        }
    }
    /// Sets the line type; `aspect.set_type_of_line(AspectTypeOfLine::Dot)`.
    pub fn set_type_of_line(&self, kind: AspectTypeOfLine) {
        if let Some(aspect) = self.aspect().get() {
            aspect.set_line_kind(kind);
        }
    }
    /// Sets the width (narrowed to float); a width <= 0 raises `Standard_OutOfRange`.
    /// Example: `aspect.set_width(2.0)?`.
    pub fn set_width(&self, width: f64) -> Result<(), Failure> {
        match self.aspect().get() {
            Some(aspect) => aspect.set_width_f64(width),
            None => Ok(()),
        }
    }
    /// The Graphic3d aspect; `aspect.aspect()`.
    pub fn aspect(&self) -> Handle<Graphic3dAspectLine3d> {
        self.aspect.read().clone()
    }
    /// Replaces the Graphic3d aspect; `aspect.set_aspect(other)`.
    pub fn set_aspect(&self, aspect: Handle<Graphic3dAspectLine3d>) {
        *self.aspect.write() = aspect;
    }
}

/// The `DumpJson` of a line aspect under a class name.
fn line_aspect_dump(
    line: &Prs3dLineAspect,
    class_name: &str,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, class_name)?;
    dump_graphic_aspects(stream, depth, "myAspect.get()", &line.aspect())
}

impl Prs3dBasicAspect for Prs3dLineAspect {
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        line_aspect_dump(self, "Prs3d_LineAspect", stream, depth)
    }
}

/// The line attributes of the U or V isoparameters, with their number.
///
/// Example: `let iso = Prs3dIsoAspect::new(gray, AspectTypeOfLine::Solid, 1.0, 1)?; iso.set_number(5);`.
pub struct Prs3dIsoAspect {
    line: Prs3dLineAspect,
    number: SharedState<i32>,
}
rustycad_tkernel::impl_transient!(Prs3dIsoAspect, "Prs3d_IsoAspect", Prs3dLineAspect, line,
    interfaces: [dyn Prs3dBasicAspect]);

impl Prs3dIsoAspect {
    /// A new iso aspect; a width <= 0 raises `Standard_OutOfRange`.
    /// Example: `Prs3dIsoAspect::new(gray, AspectTypeOfLine::Solid, 1.0, 10)?`.
    pub fn new(
        color: Color,
        kind: AspectTypeOfLine,
        width: f64,
        number: i32,
    ) -> Result<Self, Failure> {
        Ok(Self {
            line: Prs3dLineAspect::new(color, kind, width)?,
            number: SharedState::new(number),
        })
    }
    /// The line aspect part; `iso.line_aspect().aspect()`.
    pub fn line_aspect(&self) -> &Prs3dLineAspect {
        &self.line
    }
    /// Sets the number of isoparameters; `iso.set_number(4)`.
    pub fn set_number(&self, number: i32) {
        *self.number.write() = number;
    }
    /// The number of isoparameters; `iso.number()`.
    pub fn number(&self) -> i32 {
        *self.number.read()
    }
}

impl std::ops::Deref for Prs3dIsoAspect {
    type Target = Prs3dLineAspect;
    /// The inherited members of `Prs3d_LineAspect`.
    fn deref(&self) -> &Prs3dLineAspect {
        &self.line
    }
}

impl Prs3dBasicAspect for Prs3dIsoAspect {
    /// The inherited `Prs3d_LineAspect::DumpJson`, whose class name is the native
    /// `get_type_name()` of the line aspect (the class does not override it).
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        line_aspect_dump(&self.line, "Prs3d_LineAspect", stream, depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::quantity::NamedColor;
    use rustycad_tkernel::standard::RuntimeTyped;

    #[test]
    fn setters_reach_the_graphic_aspect() {
        let aspect =
            Prs3dLineAspect::new(Color::named(NamedColor::RED), AspectTypeOfLine::Solid, 1.0)
                .unwrap();
        aspect.set_color(Color::named(NamedColor::GREEN));
        aspect.set_type_of_line(AspectTypeOfLine::Dash);
        aspect.set_width(3.0).unwrap();
        let graphic = aspect.aspect();
        assert_eq!(graphic.interior_color(), Color::named(NamedColor::GREEN));
        assert_eq!(graphic.line_kind(), AspectTypeOfLine::Dash);
        assert_eq!(graphic.width(), 3.0);
        assert!(aspect.set_width(0.0).is_err());
    }

    #[test]
    fn iso_aspect_is_a_line_aspect() {
        let iso = Handle::new(
            Prs3dIsoAspect::new(
                Color::named(NamedColor::GRAY75),
                AspectTypeOfLine::Solid,
                1.0,
                3,
            )
            .unwrap(),
        );
        iso.set_number(7);
        assert_eq!(iso.number(), 7);
        let line: Handle<Prs3dLineAspect> = iso.cast();
        assert!(!line.is_null());
        assert_eq!(
            line.dynamic_type().unwrap().name(),
            Prs3dIsoAspect::TYPE_NAME
        );
        let mut stream = DumpStream::default();
        iso.dump_json(&mut stream, 0).unwrap();
        assert_eq!(stream.text(), "\"className\": \"Prs3d_LineAspect\"");
    }
}
