// Rust translation of FoundationClasses/TKMath/gp/gp_Parab2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Parab2d: a planar parabola `Y² = 4·F·X` whose X axis is the mirror axis.
use super::super::placement::scaled_length;
use super::placement::axis22d_curve_placement;
use super::{Axis2d, Axis22d, Direction2, Failure, Point2, Transform2, Vector2, construction};
/// A planar parabola; `Parabola2::new(frame,2.0)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parabola2 {
    position: Axis22d,
    focal_length: f64,
}
impl Default for Parabola2 {
    /// Native default: RealLast focal length.
    fn default() -> Self {
        Self {
            position: Axis22d::default(),
            focal_length: f64::MAX,
        }
    }
}
axis22d_curve_placement!(Parabola2);
impl Parabola2 {
    /// Requires a non-negative focal length; `Parabola2::new(frame,2.0)?`.
    pub fn new(position: Axis22d, focal_length: f64) -> Result<Self, Failure> {
        if focal_length < 0.0 {
            return Err(construction(&format!(
                "gp_Parab2d() - focal length should be >= 0, got {focal_length}"
            )));
        }
        Ok(Self {
            position,
            focal_length,
        })
    }
    /// Uses the axis as mirror (X) axis with the given sense; `Parabola2::from_mirror_axis(axis,2.0,true)?`.
    pub fn from_mirror_axis(
        mirror_axis: Axis2d,
        focal_length: f64,
        sense: bool,
    ) -> Result<Self, Failure> {
        Self::new(Axis22d::from_axis(mirror_axis, sense)?, focal_length)
    }
    /// Builds the parabola from directrix and focus; a focus on the directrix yields a zero focal
    /// length whose X axis is the directrix turned by ∓90°; `Parabola2::from_directrix(d,f,true)?`.
    pub fn from_directrix(directrix: Axis2d, focus: Point2, sense: bool) -> Result<Self, Failure> {
        let (origin, along) = (directrix.location(), directrix.direction());
        let toward_focus = Vector2::between(origin, focus);
        let foot = Point2::from(origin.xy() + along.xy() * toward_focus.dot(Vector2::from(along)));
        let apex = Point2::from((foot.xy() + focus.xy()) * 0.5);
        let focal_length = 0.5 * foot.distance(focus);
        let x = if focal_length > 0.0 {
            Direction2::from_xy(focus.xy() - foot.xy())?
        } else {
            let turn = if sense {
                -std::f64::consts::FRAC_PI_2
            } else {
                std::f64::consts::FRAC_PI_2
            };
            directrix.rotated(origin, turn).direction()
        };
        Ok(Self {
            position: Axis22d::with_directions(apex, x, along)?,
            focal_length,
        })
    }
    /// Requires a non-negative focal length; `parabola.set_focal(1.0)?`.
    pub fn set_focal(&mut self, focal_length: f64) -> Result<(), Failure> {
        if focal_length < 0.0 {
            return Err(construction(&format!(
                "gp_Parab2d::SetFocal() - focal length should be >= 0, got {focal_length}"
            )));
        }
        self.focal_length = focal_length;
        Ok(())
    }
    /// Replaces apex and mirror axis, keeping the sense (gp_Parab2d::SetMirrorAxis);
    /// `parabola.set_mirror_axis(axis)?`.
    pub fn set_mirror_axis(&mut self, axis: Axis2d) -> Result<(), Failure> {
        self.position.set_x_axis(axis)
    }
    /// Returns the mirror (X) axis (gp_Parab2d::MirrorAxis); `parabola.axis_of_symmetry()`.
    pub fn axis_of_symmetry(&self) -> Axis2d {
        self.position.x_axis()
    }
    /// Returns the apex-to-focus distance; `parabola.focal()`.
    pub const fn focal(&self) -> f64 {
        self.focal_length
    }
    /// Returns `2·F`; `parabola.parameter()`.
    pub fn parameter(&self) -> f64 {
        2.0 * self.focal_length
    }
    /// Focus on the positive X side; `parabola.focus()`.
    pub fn focus(&self) -> Point2 {
        let (apex, x) = (self.location(), self.position.x_direction());
        Point2::new(
            apex.x() + self.focal_length * x.x(),
            apex.y() + self.focal_length * x.y(),
        )
    }
    /// Directrix parallel to Y on the negative X side; `parabola.directrix()`.
    pub fn directrix(&self) -> Axis2d {
        let (apex, x) = (self.location(), self.position.x_direction());
        let point = Point2::new(
            apex.x() - self.focal_length * x.x(),
            apex.y() - self.focal_length * x.y(),
        );
        Axis2d::new(point, self.position.y_direction())
    }
    /// Returns `[A, B, C, D, E, F]` in the global system; `parabola.coefficients()`.
    pub fn coefficients(&self) -> [f64; 6] {
        let p = 2.0 * self.focal_length;
        let transform = Transform2::transformation_to(self.position.x_axis());
        let first = [1, 2, 3].map(|column| transform.coefficient(1, column));
        let second = [1, 2, 3].map(|column| transform.coefficient(2, column));
        [
            second[0] * second[0],
            second[1] * second[1],
            second[0] * second[1],
            (second[0] * second[2]) - (p * first[0]),
            (second[1] * second[2]) - (p * first[1]),
            (second[2] * second[2]) - (2.0 * p * first[2]),
        ]
    }
    /// Scales focal length and frame; `parabola.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point2, factor: f64) -> Self {
        self.scale(center, factor);
        self
    }
    /// Scales in place; `parabola.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point2, factor: f64) {
        self.focal_length = scaled_length(self.focal_length, factor);
        self.position.scale(center, factor);
    }
    /// Transforms focal length by the scale factor and the frame; `parabola.transformed(transform)`.
    pub fn transformed(mut self, transform: Transform2) -> Self {
        self.transform(transform);
        self
    }
    /// Transforms in place; `parabola.transform(transform)`.
    pub fn transform(&mut self, transform: Transform2) {
        self.focal_length = scaled_length(self.focal_length, transform.scale_factor());
        self.position.transform(transform);
    }
}
