// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_Point.hxx and
// IntPolyh_Point.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPolyh_Point`: a point of a surface with its 3D and parametric coordinates.
use super::array::IntPolyhDumpItem;
use super::dump::c_exp;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::Point3;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

/// `IntPolyh_Point`: `(x, y, z)` on the surface at `(u, v)`, the part-of-common flag
/// (the out codes of `CommonBox`) and the degenerated flag.
///
/// ```
/// use rustycad_tkgeomalgo::int_polyh::IntPolyhPoint;
/// let p = IntPolyhPoint::new(1.0, 2.0, 3.0, 0.1, 0.2);
/// assert_eq!(p.square_modulus(), 14.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntPolyhPoint {
    x: f64,
    y: f64,
    z: f64,
    u: f64,
    v: f64,
    poc: i32,
    degenerated: bool,
}

impl Default for IntPolyhPoint {
    /// The origin at `(0, 0)`, part of the common box, not degenerated.
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.0)
    }
}

impl IntPolyhPoint {
    /// The constructor from coordinates; part of the common box, not degenerated.
    /// `IntPolyhPoint::new(1.0, 2.0, 3.0, 0.1, 0.2).u() == 0.1`.
    pub const fn new(x: f64, y: f64, z: f64, u: f64, v: f64) -> Self {
        Self {
            x,
            y,
            z,
            u,
            v,
            poc: 1,
            degenerated: false,
        }
    }

    /// `X()`. `p.x()`.
    pub const fn x(&self) -> f64 {
        self.x
    }
    /// `Y()`. `p.y()`.
    pub const fn y(&self) -> f64 {
        self.y
    }
    /// `Z()`. `p.z()`.
    pub const fn z(&self) -> f64 {
        self.z
    }
    /// `U()`. `p.u()`.
    pub const fn u(&self) -> f64 {
        self.u
    }
    /// `V()`. `p.v()`.
    pub const fn v(&self) -> f64 {
        self.v
    }
    /// `PartOfCommon()`: 0 inside the common box, else its out codes. `p.part_of_common()`.
    pub const fn part_of_common(&self) -> i32 {
        self.poc
    }

    /// `Set(x, y, z, u, v, II = 1)`; keeps the degenerated flag. `p.set(1.0, 2.0, 3.0, 0.0, 0.0, 1)`.
    pub fn set(&mut self, x: f64, y: f64, z: f64, u: f64, v: f64, ii: i32) {
        (self.x, self.y, self.z, self.u, self.v, self.poc) = (x, y, z, u, v, ii);
    }
    /// `SetX(x)`. `p.set_x(1.0)`.
    pub fn set_x(&mut self, x: f64) {
        self.x = x;
    }
    /// `SetY(y)`. `p.set_y(1.0)`.
    pub fn set_y(&mut self, y: f64) {
        self.y = y;
    }
    /// `SetZ(z)`. `p.set_z(1.0)`.
    pub fn set_z(&mut self, z: f64) {
        self.z = z;
    }
    /// `SetU(u)`. `p.set_u(1.0)`.
    pub fn set_u(&mut self, u: f64) {
        self.u = u;
    }
    /// `SetV(v)`. `p.set_v(1.0)`.
    pub fn set_v(&mut self, v: f64) {
        self.v = v;
    }
    /// `SetPartOfCommon(ii)`. `p.set_part_of_common(0)`.
    pub fn set_part_of_common(&mut self, ii: i32) {
        self.poc = ii;
    }
    /// `SetDegenerated(theFlag)`. `p.set_degenerated(true)`.
    pub fn set_degenerated(&mut self, flag: bool) {
        self.degenerated = flag;
    }
    /// `Degenerated()`. `p.degenerated()`.
    pub const fn degenerated(&self) -> bool {
        self.degenerated
    }

    /// `Middle(MySurface, P1, P2)`: the surface point at the middle of the parameters of
    /// `p1` and `p2`; the flags are kept. `p.middle(&surface, &a, &b)?`.
    pub fn middle(
        &mut self,
        surface: &dyn Adaptor3dSurface,
        p1: &IntPolyhPoint,
        p2: &IntPolyhPoint,
    ) -> Result<(), Failure> {
        self.u = (p1.u() + p2.u()) * 0.5;
        self.v = (p1.v() + p2.v()) * 0.5;
        let pt_xyz: Point3 = surface.value(self.u, self.v)?;
        (self.x, self.y, self.z) = (pt_xyz.x(), pt_xyz.y(), pt_xyz.z());
        Ok(())
    }

    /// `Add(P1)` (also `operator+`): the sum of the coordinates, flags reset.
    /// `a.add(&b)`.
    #[allow(clippy::should_implement_trait)]
    pub fn add(&self, p1: &IntPolyhPoint) -> IntPolyhPoint {
        IntPolyhPoint::new(
            self.x + p1.x,
            self.y + p1.y,
            self.z + p1.z,
            self.u + p1.u,
            self.v + p1.v,
        )
    }

    /// `Sub(P1)` (also `operator-`). `a.sub(&b)`.
    #[allow(clippy::should_implement_trait)]
    pub fn sub(&self, p1: &IntPolyhPoint) -> IntPolyhPoint {
        IntPolyhPoint::new(
            self.x - p1.x,
            self.y - p1.y,
            self.z - p1.z,
            self.u - p1.u,
            self.v - p1.v,
        )
    }

    /// `Divide(RR)` (also `operator/`): the default point when `|RR|` is not above
    /// Precision::Computational (bug #18: no division by zero).
    /// `IntPolyhPoint::new(2.0, 4.0, 6.0, 0.2, 0.4).divide(2.0).x() == 1.0`.
    pub fn divide(&self, rr: f64) -> IntPolyhPoint {
        // `abs(RR) > Computational` natively: a NaN divisor gives the default point.
        let divisible: bool = rr.abs() > precision::computational();
        if !divisible {
            return IntPolyhPoint::default();
        }
        IntPolyhPoint::new(
            self.x / rr,
            self.y / rr,
            self.z / rr,
            self.u / rr,
            self.v / rr,
        )
    }

    /// `Multiplication(RR)` (also `operator*`). `a.multiplication(2.0)`.
    pub fn multiplication(&self, rr: f64) -> IntPolyhPoint {
        IntPolyhPoint::new(
            self.x * rr,
            self.y * rr,
            self.z * rr,
            self.u * rr,
            self.v * rr,
        )
    }

    /// `SquareModulus()`: of the 3D coordinates. `a.square_modulus()`.
    pub fn square_modulus(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// `SquareDistance(P2)`: in 3D. `a.square_distance(&b)`.
    pub fn square_distance(&self, p2: &IntPolyhPoint) -> f64 {
        (self.x - p2.x) * (self.x - p2.x)
            + (self.y - p2.y) * (self.y - p2.y)
            + (self.z - p2.z) * (self.z - p2.z)
    }

    /// `Dot(P2)`: of the 3D coordinates. `a.dot(&b)`.
    pub fn dot(&self, b: &IntPolyhPoint) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }

    /// `Cross(P1, P2)`: this point's 3D coordinates become `P1 ^ P2`. `n.cross(&a, &b)`.
    pub fn cross(&mut self, a: &IntPolyhPoint, b: &IntPolyhPoint) {
        self.x = a.y * b.z - a.z * b.y;
        self.y = a.z * b.x - a.x * b.z;
        self.z = a.x * b.y - a.y * b.x;
    }

    /// `Dump()`: the native `printf` text. `p.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(
            out,
            "\nPoint : x={}g y={}g z={}g u={}g v={}g",
            c_exp(self.x, 8, 3),
            c_exp(self.y, 8, 3),
            c_exp(self.z, 8, 3),
            c_exp(self.u, 8, 3),
            c_exp(self.v, 8, 3)
        )
    }

    /// `Dump(i)`: the native `printf` text numbered `i`. `p.dump_index(3, &mut text)?`.
    pub fn dump_index(&self, i: i32, out: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(
            out,
            "\nPoint({i:3}) : x={}g y={}g z={}g u={}g v={}g poc={:3}",
            c_exp(self.x, 8, 3),
            c_exp(self.y, 8, 3),
            c_exp(self.z, 8, 3),
            c_exp(self.u, 8, 3),
            c_exp(self.v, 8, 3),
            self.poc
        )
    }
}

impl IntPolyhDumpItem for IntPolyhPoint {
    fn dump_item(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        self.dump(out)
    }
}

impl Add for IntPolyhPoint {
    type Output = IntPolyhPoint;
    /// `operator+`.
    fn add(self, p1: IntPolyhPoint) -> IntPolyhPoint {
        IntPolyhPoint::add(&self, &p1)
    }
}

impl Sub for IntPolyhPoint {
    type Output = IntPolyhPoint;
    /// `operator-`.
    fn sub(self, p1: IntPolyhPoint) -> IntPolyhPoint {
        IntPolyhPoint::sub(&self, &p1)
    }
}

impl Div<f64> for IntPolyhPoint {
    type Output = IntPolyhPoint;
    /// `operator/`.
    fn div(self, rr: f64) -> IntPolyhPoint {
        self.divide(rr)
    }
}

impl Mul<f64> for IntPolyhPoint {
    type Output = IntPolyhPoint;
    /// `operator*`.
    fn mul(self, rr: f64) -> IntPolyhPoint {
        self.multiplication(rr)
    }
}
