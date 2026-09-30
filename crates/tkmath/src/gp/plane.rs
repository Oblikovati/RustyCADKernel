// Rust translation of FoundationClasses/TKMath/gp/gp_Pln.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Pln plane position and oriented equation.
use super::{Axis1, Axis3, Direction3, Failure, Point3, axis2::perpendicular_candidate};
mod metrics;
mod transforms;
/// A plane retaining all three frame directions; `Plane3::new(origin,normal)?`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Plane3 {
    position: Axis3,
}
impl Plane3 {
    /// Copies the complete frame, including handedness; `Plane3::from_position(frame)`.
    pub const fn from_position(position: Axis3) -> Self {
        Self { position }
    }
    /// gp_Pln(P, V): X is the native auxiliary direction of the smallest normal component;
    /// `Plane3::new(origin,normal)?`.
    pub fn new(origin: Point3, normal: Direction3) -> Result<Self, Failure> {
        let [x, y, z] = *normal.coordinates();
        let [a, b, c] = perpendicular_candidate(x, y, z);
        let position = Axis3::new(origin, normal, Direction3::new(a, b, c)?)?;
        Ok(Self::from_position(position))
    }
    /// gp_Pln(A, B, C, D): the unnormalized coefficients choose the X direction and the axis
    /// that carries the origin in the same branch; `Plane3::from_equation(0.0,0.0,1.0,-5.0)?`.
    pub fn from_equation(a: f64, b: f64, c: f64, d: f64) -> Result<Self, Failure> {
        let origin = equation_origin(a, b, c, d);
        let [x, y, z] = perpendicular_candidate(a, b, c);
        let position = Axis3::new(origin, Direction3::new(a, b, c)?, Direction3::new(x, y, z)?)?;
        Ok(Self::from_position(position))
    }
    /// Returns the stored coordinate frame; `plane.position()`.
    pub const fn position(&self) -> Axis3 {
        self.position
    }
    /// Returns the plane origin; `plane.location()`.
    pub const fn location(&self) -> Point3 {
        self.position.location()
    }
    /// Returns the origin and stored normal; `plane.axis()`.
    pub fn axis(&self) -> Axis1 {
        self.position.axis()
    }
    /// Returns the X axis through the origin; `plane.x_axis()`.
    pub fn x_axis(&self) -> Axis1 {
        Axis1::new(self.location(), self.position.x_direction())
    }
    /// Returns the Y axis through the origin; `plane.y_axis()`.
    pub fn y_axis(&self) -> Axis1 {
        Axis1::new(self.location(), self.position.y_direction())
    }
    /// Replaces the normal axis through gp_Ax3::SetAxis; `plane.set_axis(axis)?`.
    pub fn set_axis(&mut self, axis: Axis1) -> Result<(), Failure> {
        self.position.set_axis(axis)
    }
    /// Moves the origin; `plane.set_location(point)`.
    pub fn set_location(&mut self, location: Point3) {
        self.position.set_location(location);
    }
    /// Replaces the whole frame; `plane.set_position(frame)`.
    pub fn set_position(&mut self, position: Axis3) {
        self.position = position;
    }
    /// Reverses the U (X) direction, changing handedness; `plane.u_reverse()`.
    pub fn u_reverse(&mut self) {
        self.position.x_reverse();
    }
    /// Reverses the V (Y) direction, changing handedness; `plane.v_reverse()`.
    pub fn v_reverse(&mut self) {
        self.position.y_reverse();
    }
    /// Tests right-handedness of the frame; `plane.direct()?`.
    pub fn direct(&self) -> Result<bool, Failure> {
        self.position.direct()
    }
    /// Signs the equation by frame handedness; `plane.coefficients()?`.
    pub fn coefficients(&self) -> Result<[f64; 4], Failure> {
        let mut normal = self.position.direction();
        if !self.position.direct()? {
            normal.reverse();
        }
        let [a, b, c] = *normal.coordinates();
        let origin = self.location();
        Ok([a, b, c, -(a * origin.x() + b * origin.y() + c * origin.z())])
    }
}
impl From<Axis3> for Plane3 {
    fn from(position: Axis3) -> Self {
        Self::from_position(position)
    }
}
/// The origin gp_Pln(A,B,C,D) pairs with each auxiliary-direction branch.
fn equation_origin(a: f64, b: f64, c: f64, d: f64) -> Point3 {
    let (abs_a, abs_b, abs_c) = (a.abs(), b.abs(), c.abs());
    let on_x = || Point3::new(-d / a, 0.0, 0.0);
    let on_y = || Point3::new(0.0, -d / b, 0.0);
    let on_z = || Point3::new(0.0, 0.0, -d / c);
    if abs_b <= abs_a && abs_b <= abs_c {
        return if abs_a > abs_c { on_x() } else { on_z() };
    }
    if abs_a <= abs_b && abs_a <= abs_c {
        return if abs_b > abs_c { on_y() } else { on_z() };
    }
    if abs_a > abs_b { on_x() } else { on_y() }
}
