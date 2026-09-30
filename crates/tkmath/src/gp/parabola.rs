// Rust translation of FoundationClasses/TKMath/gp/gp_Parab.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Parab: a parabola `Y² = 4·F·X` in its right-handed frame.
use super::placement::{axis2_curve_placement, scaled_length};
use super::{Axis1, Axis2, Failure, Line3, Point3, Transform3, construction};
/// A located parabola; `Parabola3::new(frame,2.0)?`.
#[derive(Clone, Copy, Debug)]
pub struct Parabola3 {
    position: Axis2,
    focal_length: f64,
}
impl Default for Parabola3 {
    /// Native default: RealLast focal length.
    fn default() -> Self {
        Self {
            position: Axis2::default(),
            focal_length: f64::MAX,
        }
    }
}
axis2_curve_placement!(Parabola3);
impl Parabola3 {
    /// Requires a non-negative focal length; `Parabola3::new(frame,2.0)?`.
    pub fn new(position: Axis2, focal_length: f64) -> Result<Self, Failure> {
        if focal_length < 0.0 {
            return Err(construction(&format!(
                "gp_Parab() - focal length should be >= 0, got {focal_length}"
            )));
        }
        Ok(Self {
            position,
            focal_length,
        })
    }
    /// Builds the parabola from its directrix and focus; fails when the focus lies on the
    /// directrix; `Parabola3::from_directrix(directrix,focus)?`.
    pub fn from_directrix(directrix: Axis1, focus: Point3) -> Result<Self, Failure> {
        let line = Line3::from_axis(directrix);
        let focal_length = line.distance(focus) / 2.0;
        let x = line.normal(focus)?.direction();
        let normal = x.crossed(line.direction())?;
        let apex = Point3::new(
            focus.x() - focal_length * x.x(),
            focus.y() - focal_length * x.y(),
            focus.z() - focal_length * x.z(),
        );
        Ok(Self {
            position: Axis2::with_x_direction(apex, normal, x)?,
            focal_length,
        })
    }
    /// Requires a non-negative focal length; `parabola.set_focal(1.0)?`.
    pub fn set_focal(&mut self, focal_length: f64) -> Result<(), Failure> {
        if focal_length < 0.0 {
            return Err(construction(&format!(
                "gp_Parab::SetFocal() - focal length should be >= 0, got {focal_length}"
            )));
        }
        self.focal_length = focal_length;
        Ok(())
    }
    /// Moves the apex only; `parabola.set_location(point)`.
    pub fn set_location(&mut self, location: Point3) {
        self.position.set_location(location);
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
    pub fn focus(&self) -> Point3 {
        let (apex, x) = (self.location(), self.position.x_direction());
        Point3::new(
            apex.x() + self.focal_length * x.x(),
            apex.y() + self.focal_length * x.y(),
            apex.z() + self.focal_length * x.z(),
        )
    }
    /// Directrix parallel to Y on the negative X side; `parabola.directrix()`.
    pub fn directrix(&self) -> Axis1 {
        let (apex, x) = (self.location(), self.position.x_direction());
        let point = Point3::new(
            apex.x() - self.focal_length * x.x(),
            apex.y() - self.focal_length * x.y(),
            apex.z() - self.focal_length * x.z(),
        );
        Axis1::new(point, self.position.y_direction())
    }
    /// Scales focal length and frame; `parabola.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point3, factor: f64) -> Self {
        self.scale(center, factor);
        self
    }
    /// Scales in place; `parabola.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        self.focal_length = scaled_length(self.focal_length, factor);
        self.position.scale(center, factor);
    }
    /// Transforms focal length by the scale factor and the frame; `parabola.transformed(transform)?`.
    pub fn transformed(mut self, transform: Transform3) -> Result<Self, Failure> {
        self.transform(transform)?;
        Ok(self)
    }
    /// Transforms in place; `parabola.transform(transform)?`.
    pub fn transform(&mut self, transform: Transform3) -> Result<(), Failure> {
        self.focal_length = scaled_length(self.focal_length, transform.scale_factor());
        self.position.transform(transform)
    }
}
