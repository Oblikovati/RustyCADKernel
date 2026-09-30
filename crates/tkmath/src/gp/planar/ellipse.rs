// Rust translation of FoundationClasses/TKMath/gp/gp_Elips2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Elips2d: a planar ellipse whose major axis is the X axis of its frame.
use super::super::placement::{absolute_factor, scaled_length};
use super::placement::{axis22d_curve_axes, axis22d_curve_placement, central_conic_coefficients};
use super::{Axis2d, Axis22d, Failure, Point2, Transform2, construction};
/// A planar ellipse; `Ellipse2::new(frame,3.0,2.0)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ellipse2 {
    position: Axis22d,
    major_radius: f64,
    minor_radius: f64,
}
impl Default for Ellipse2 {
    /// Native default: RealLast major radius and RealSmall (DBL_MIN) minor radius.
    fn default() -> Self {
        Self {
            position: Axis22d::default(),
            major_radius: f64::MAX,
            minor_radius: f64::MIN_POSITIVE,
        }
    }
}
axis22d_curve_placement!(Ellipse2);
axis22d_curve_axes!(Ellipse2);
impl Ellipse2 {
    /// Requires `0 <= minor <= major`; `Ellipse2::new(frame,3.0,2.0)?`.
    pub fn new(position: Axis22d, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if minor_radius < 0.0 || major_radius < minor_radius {
            return Err(construction(&format!(
                "gp_Elips2d() - invalid construction parameters: major {major_radius}, minor {minor_radius}; expected 0 <= minor <= major"
            )));
        }
        Ok(Self {
            position,
            major_radius,
            minor_radius,
        })
    }
    /// Uses the axis as major axis with the given sense; `Ellipse2::from_x_axis(axis,3.0,2.0,true)?`.
    pub fn from_x_axis(
        major_axis: Axis2d,
        major_radius: f64,
        minor_radius: f64,
        sense: bool,
    ) -> Result<Self, Failure> {
        Self::new(
            Axis22d::from_axis(major_axis, sense)?,
            major_radius,
            minor_radius,
        )
    }
    /// Requires the major radius to stay at least the minor one; `ellipse.set_major_radius(4.0)?`.
    pub fn set_major_radius(&mut self, major_radius: f64) -> Result<(), Failure> {
        if major_radius < self.minor_radius {
            return Err(construction(&format!(
                "gp_Elips2d::SetMajorRadius() - major radius should be greater or equal to minor radius: major {major_radius}, minor {}",
                self.minor_radius
            )));
        }
        self.major_radius = major_radius;
        Ok(())
    }
    /// Requires `0 <= minor <= major`; `ellipse.set_minor_radius(1.0)?`.
    pub fn set_minor_radius(&mut self, minor_radius: f64) -> Result<(), Failure> {
        if minor_radius < 0.0 || self.major_radius < minor_radius {
            return Err(construction(&format!(
                "gp_Elips2d::SetMinorRadius() - minor radius should be a positive number lesser or equal to major radius: minor {minor_radius}, major {}",
                self.major_radius
            )));
        }
        self.minor_radius = minor_radius;
        Ok(())
    }
    /// Returns the major radius; `ellipse.major_radius()`.
    pub const fn major_radius(&self) -> f64 {
        self.major_radius
    }
    /// Returns the minor radius; `ellipse.minor_radius()`.
    pub const fn minor_radius(&self) -> f64 {
        self.minor_radius
    }
    /// Computes `PI * major * minor`; `ellipse.area()`.
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.major_radius * self.minor_radius
    }
    /// Returns `[A, B, C, D, E, F]` in the global system, all zero for a point ellipse;
    /// `ellipse.coefficients()`.
    pub fn coefficients(&self) -> [f64; 6] {
        central_conic_coefficients(
            self.position.x_axis(),
            self.major_radius,
            self.minor_radius,
            1.0,
        )
    }
    /// Returns zero for a zero major radius, otherwise `sqrt(a²-b²)/a`; `ellipse.eccentricity()`.
    pub fn eccentricity(&self) -> f64 {
        if self.major_radius == 0.0 {
            return 0.0;
        }
        self.focal_half() / self.major_radius
    }
    fn focal_half(&self) -> f64 {
        (self.major_radius * self.major_radius - self.minor_radius * self.minor_radius).sqrt()
    }
    /// Distance between the foci; `ellipse.focal()`.
    pub fn focal(&self) -> f64 {
        2.0 * self.focal_half()
    }
    /// Focus on the positive X side; `ellipse.focus1()`.
    pub fn focus1(&self) -> Point2 {
        self.focus(self.focal_half())
    }
    /// Focus on the negative X side; `ellipse.focus2()`.
    pub fn focus2(&self) -> Point2 {
        self.focus(-self.focal_half())
    }
    fn focus(&self, offset: f64) -> Point2 {
        let (origin, x) = (self.location(), self.position.x_direction());
        Point2::new(origin.x() + offset * x.x(), origin.y() + offset * x.y())
    }
    /// Semi-latus rectum `b²/a`, or zero for a zero major radius; `ellipse.parameter()`.
    pub fn parameter(&self) -> f64 {
        if self.major_radius == 0.0 {
            return 0.0;
        }
        (self.minor_radius * self.minor_radius) / self.major_radius
    }
    /// Directrix on the positive X side; a circle has none; `ellipse.directrix1()?`.
    pub fn directrix1(&self) -> Result<Axis2d, Failure> {
        self.directrix(1.0, "gp_Elips2d::Directrix1() - zero eccentricity")
    }
    /// Directrix on the negative X side; `ellipse.directrix2()?`.
    pub fn directrix2(&self) -> Result<Axis2d, Failure> {
        self.directrix(-1.0, "gp_Elips2d::Directrix2() - zero eccentricity")
    }
    fn directrix(&self, side: f64, message: &str) -> Result<Axis2d, Failure> {
        let eccentricity = self.eccentricity();
        if eccentricity <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "{message}: eccentricity {eccentricity}"
            )));
        }
        let origin = self.position.x_direction().xy() * ((side * self.major_radius) / eccentricity)
            + self.location().xy();
        Ok(Axis2d::new(
            Point2::from(origin),
            self.position.y_direction(),
        ))
    }
    /// Scales radii and frame; `ellipse.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point2, factor: f64) -> Self {
        self.scale(center, factor);
        self
    }
    /// Scales in place; `ellipse.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point2, factor: f64) {
        self.major_radius = scaled_length(self.major_radius, factor);
        self.minor_radius = scaled_length(self.minor_radius, factor);
        self.position.scale(center, factor);
    }
    /// Multiplies then negates each radius, so zero radii may become `-0.0`;
    /// `ellipse.transformed(transform)`.
    pub fn transformed(mut self, transform: Transform2) -> Self {
        let factor = transform.scale_factor();
        self.major_radius = scaled_length(self.major_radius, factor);
        self.minor_radius = scaled_length(self.minor_radius, factor);
        self.position.transform(transform);
        self
    }
    /// Multiplies by `|scale|` (unlike `transformed`, so zero radii stay `+0.0`);
    /// `ellipse.transform(transform)`.
    pub fn transform(&mut self, transform: Transform2) {
        let factor = absolute_factor(transform.scale_factor());
        self.major_radius *= factor;
        self.minor_radius *= factor;
        self.position.transform(transform);
    }
}
