//! `Prs3d_ArrowAspect`: the angle, length and line attributes of arrows.
use super::basic_aspect::{Prs3dBasicAspect, dump_graphic_aspects};
use super::prs3d_invalid_angle;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::graphic3d::{Graphic3dAspectLine3d, Graphic3dAspectsMembers as _};
use rustycad_tkservice::json_dump::{class_begin, field_real};
use std::f64::consts::PI;
use std::io;

/// The arrow geometry fields.
#[derive(Clone, Copy)]
struct Prs3dArrowShape {
    angle: f64,
    length: f64,
    is_zoomable: bool,
}

/// The attributes of arrows.
///
/// Native quirk: the `(angle, length)` and `(aspect)` constructors leave `myIsZoomable`
/// uninitialized; the port uses `true`, the value of the default constructor.
/// Example: `let arrow = Prs3dArrowAspect::new()?; arrow.set_angle(0.2)?;`.
pub struct Prs3dArrowAspect {
    transient: StandardTransient,
    arrow_aspect: SharedState<Handle<Graphic3dAspectLine3d>>,
    shape: SharedState<Prs3dArrowShape>,
}
rustycad_tkernel::impl_transient!(Prs3dArrowAspect, "Prs3d_ArrowAspect", dyn Prs3dBasicAspect, transient,
    interfaces: [dyn Prs3dBasicAspect]);

/// The white solid line of the arrow constructors.
fn white_line() -> Result<Handle<Graphic3dAspectLine3d>, Failure> {
    let line = Graphic3dAspectLine3d::new(
        Color::named(NamedColor::WHITE),
        AspectTypeOfLine::Solid,
        1.0,
    )?;
    Ok(Handle::new(line))
}

impl Prs3dArrowAspect {
    /// The default: a white arrow of 10 degrees and length 1, zoomable; `Prs3dArrowAspect::new()?`.
    pub fn new() -> Result<Self, Failure> {
        Self::with_angle_and_length(PI / 180.0 * 10.0, 1.0)
    }
    /// A white arrow of an angle and length, unchecked; `Prs3dArrowAspect::with_angle_and_length(0.2, 5.0)?`.
    pub fn with_angle_and_length(angle: f64, length: f64) -> Result<Self, Failure> {
        Ok(Self::build(white_line()?, angle, length))
    }
    /// An arrow of 10 degrees and length 1 over an existing line aspect.
    /// Example: `Prs3dArrowAspect::from_aspect(line)`.
    pub fn from_aspect(aspect: Handle<Graphic3dAspectLine3d>) -> Self {
        Self::build(aspect, PI / 180.0 * 10.0, 1.0)
    }
    fn build(aspect: Handle<Graphic3dAspectLine3d>, angle: f64, length: f64) -> Self {
        Self {
            transient: StandardTransient::default(),
            arrow_aspect: SharedState::new(aspect),
            shape: SharedState::new(Prs3dArrowShape {
                angle,
                length,
                is_zoomable: true,
            }),
        }
    }
    /// Sets the angle; outside `(0, PI/2)` raises `Prs3d_InvalidAngle`. `arrow.set_angle(0.3)?`.
    pub fn set_angle(&self, angle: f64) -> Result<(), Failure> {
        if angle <= 0.0 || angle >= PI / 2.0 {
            return Err(prs3d_invalid_angle(&format!(
                "Prs3d_ArrowAspect::SetAngle() - angle out of range: {angle}; expected (0, PI/2)"
            )));
        }
        self.shape.write().angle = angle;
        Ok(())
    }
    /// The angle; `arrow.angle()`.
    pub fn angle(&self) -> f64 {
        self.shape.read().angle
    }
    /// Sets the length; `arrow.set_length(5.0)`.
    pub fn set_length(&self, length: f64) {
        self.shape.write().length = length;
    }
    /// The length; `arrow.length()`.
    pub fn length(&self) -> f64 {
        self.shape.read().length
    }
    /// Sets whether the arrow scales with the view; `arrow.set_zoomable(false)`.
    pub fn set_zoomable(&self, is_zoomable: bool) {
        self.shape.write().is_zoomable = is_zoomable;
    }
    /// Whether the arrow scales with the view; `arrow.is_zoomable()`.
    pub fn is_zoomable(&self) -> bool {
        self.shape.read().is_zoomable
    }
    /// Sets the color of the line aspect; `arrow.set_color(red)`.
    pub fn set_color(&self, color: Color) {
        if let Some(line) = self.aspect().get() {
            line.set_interior_color(color);
        }
    }
    /// The line aspect; `arrow.aspect()`.
    pub fn aspect(&self) -> Handle<Graphic3dAspectLine3d> {
        self.arrow_aspect.read().clone()
    }
    /// Replaces the line aspect; `arrow.set_aspect(line)`.
    pub fn set_aspect(&self, aspect: Handle<Graphic3dAspectLine3d>) {
        *self.arrow_aspect.write() = aspect;
    }
}

impl Prs3dBasicAspect for Prs3dArrowAspect {
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Prs3d_ArrowAspect")?;
        dump_graphic_aspects(stream, depth, "myArrowAspect.get()", &self.aspect())?;
        let shape = *self.shape.read();
        field_real(stream, "myAngle", shape.angle)?;
        field_real(stream, "myLength", shape.length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_range_is_checked() {
        let arrow = Prs3dArrowAspect::new().unwrap();
        assert_eq!(arrow.angle(), PI / 180.0 * 10.0);
        assert!(arrow.is_zoomable());
        let error = arrow.set_angle(PI / 2.0).unwrap_err();
        assert_eq!(error.exception_type(), "Prs3d_InvalidAngle");
        assert!(arrow.set_angle(0.0).is_err());
        arrow.set_angle(0.5).unwrap();
        arrow.set_length(3.0);
        arrow.set_zoomable(false);
        assert_eq!(
            (arrow.angle(), arrow.length(), arrow.is_zoomable()),
            (0.5, 3.0, false)
        );
    }
}
