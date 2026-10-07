// Rust translation of ChFiDS_CircSection.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A circular or linear section of a fillet with its parameter range.
use rustycad_tkmath::gp::{Circle3, Line3};

/// A section of a fillet: a circle and a line, both default, plus one shared parameter
/// range (`ChFiDS_CircSection`).
///
/// Setting the circle keeps the line and setting the line keeps the circle, as natively;
/// only the range is shared.
///
/// ```
/// use rustycad_tkfillet::chfi_ds::CircSection;
/// use rustycad_tkmath::gp::Line3;
/// let mut section = CircSection::new();
/// section.set_line(Line3::default(), 1.0, 2.0);
/// assert_eq!(section.get_line().1, 1.0);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct CircSection {
    circle: Circle3,
    line: Line3,
    first: f64,
    last: f64,
}

impl CircSection {
    /// The default section: default circle and line, range `[0, 0]`; `CircSection::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Set(gp_Circ, F, L)`: stores the circle and the range; `section.set_circle(c, 0.0, 1.0)`.
    pub fn set_circle(&mut self, circle: Circle3, first: f64, last: f64) {
        self.circle = circle;
        self.first = first;
        self.last = last;
    }

    /// `Set(gp_Lin, F, L)`: stores the line and the range; `section.set_line(l, 0.0, 1.0)`.
    pub fn set_line(&mut self, line: Line3, first: f64, last: f64) {
        self.line = line;
        self.first = first;
        self.last = last;
    }

    /// `Get(gp_Circ&, F&, L&)`: the circle and the range; `let (c, f, l) = section.get_circle();`.
    pub fn get_circle(&self) -> (Circle3, f64, f64) {
        (self.circle, self.first, self.last)
    }

    /// `Get(gp_Lin&, F&, L&)`: the line and the range; `let (l, f, last) = section.get_line();`.
    pub fn get_line(&self) -> (Line3, f64, f64) {
        (self.line, self.first, self.last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::gp::{Axis2, Direction3, Point3};

    #[test]
    fn setting_the_line_keeps_the_circle_and_shares_the_range() {
        let mut section = CircSection::new();
        let circle = Circle3::new(Axis2::default(), 2.0).unwrap();
        section.set_circle(circle, 0.5, 1.5);
        let line = Line3::new(Point3::new(1., 0., 0.), Direction3::default());
        section.set_line(line, 3.0, 4.0);
        let (kept_circle, first, last) = section.get_circle();
        assert_eq!((kept_circle.radius(), first, last), (2.0, 3.0, 4.0));
        assert_eq!(section.get_line(), (line, 3.0, 4.0));
    }

    #[test]
    fn default_section_has_a_zero_range() {
        let (_, first, last) = CircSection::new().get_circle();
        assert_eq!((first, last), (0.0, 0.0));
    }
}
