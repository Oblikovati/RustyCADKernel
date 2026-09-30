// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_FuncConeLSDist,
// GeomConvert_FuncCylinderLSDist and GeomConvert_FuncSphereLSDist (.hxx/.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The least-squares distance functions of a point cloud to a cone, a cylinder and
//! a sphere, minimized by the canonical surface recognizers.
//!
//! The native functions index the variables and the gradient with operator(), so
//! an index outside their bounds (or null points) raises. The trait callbacks of
//! `math` return a bool, so such a failure is recorded, the callback returns false,
//! and `take_failure` gives it back after the solver.
use crate::failure_capture::FailureCapture;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Xyz};
use rustycad_tkmath::math::{MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
use std::sync::RwLockReadGuard;

/// The shared point cloud of the functions (`occ::handle<NCollection_HArray1<gp_XYZ>>`).
pub type HArrayOfXyz = Handle<HArray1<Xyz>>;

// The array behind `points`, Standard_NullObject when the handle is null.
fn cloud<'h>(
    points: &'h HArrayOfXyz,
    member: &str,
) -> Result<RwLockReadGuard<'h, Array1<'static, Xyz>>, Failure> {
    match points.get() {
        Some(array) => Ok(array.array1()),
        None => Err(failure(
            "Standard_NullObject",
            &format!("{member}: null points; expected an array of gp_XYZ"),
        )),
    }
}

// X(1) .. X(N) with the native bound checks.
fn variables<const N: usize>(x: &Vector<'_>) -> Result<[f64; N], Failure> {
    let mut values: [f64; N] = [0.0; N];
    for (index, value) in (1..).zip(values.iter_mut()) {
        *value = *x.value(index)?;
    }
    Ok(values)
}

// G(index) *= factor, with the native bound check.
fn scale_component(g: &mut Vector<'_>, index: i32, factor: f64) -> Result<(), Failure> {
    *g.change_value(index)? *= factor;
    Ok(())
}

// G(1..4) += terms, with the native bound checks.
fn accumulate(g: &mut Vector<'_>, terms: [f64; 4]) -> Result<(), Failure> {
    for (index, term) in (1..).zip(terms) {
        *g.change_value(index)? += term;
    }
    Ok(())
}

/// `GeomConvert_FuncConeLSDist`: the sum of the square distances of the points to
/// the cone of apex frame (X1, X2, X3; direction), semi-angle X4 and radius X5.
///
/// `let f = FuncConeLSDist::new(&points, Direction3::default());`
#[derive(Default)]
pub struct FuncConeLSDist {
    my_points: HArrayOfXyz,
    my_dir: Direction3,
    capture: FailureCapture,
}

impl FuncConeLSDist {
    /// The function of `points` and axis direction `dir`;
    /// `FuncConeLSDist::new(&points, dir)`.
    pub fn new(points: &HArrayOfXyz, dir: Direction3) -> Self {
        Self {
            my_points: points.clone(),
            my_dir: dir,
            capture: FailureCapture::default(),
        }
    }

    /// `SetPoints(thePoints)`; `f.set_points(&points)`.
    pub fn set_points(&mut self, points: &HArrayOfXyz) {
        self.my_points = points.clone();
    }

    /// `SetDir(theDir)`; `f.set_dir(dir)`.
    pub fn set_dir(&mut self, dir: Direction3) {
        self.my_dir = dir;
    }

    /// The failure a callback raised natively, if any; `f.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    fn try_value(&self, x: &Vector<'_>) -> Result<f64, Failure> {
        let [x1, x2, x3, semi_angle, radius] = variables::<5>(x)?;
        let pos = Axis3::from_normal(Point3::new(x1, x2, x3), self.my_dir)?;
        let mut f: f64 = 0.0;
        for point in cloud(&self.my_points, "GeomConvert_FuncConeLSDist::Value")?.iter() {
            let pi = Point3::from(*point);
            let (u, v): (f64, f64) = elslib::cone_parameters(pos, radius, semi_angle, pi);
            let pp: Point3 = elslib::cone_d0(u, v, pos, radius, semi_angle);
            f += pi.square_distance(&pp);
        }
        Ok(f)
    }
}

impl MultipleVarFunction for FuncConeLSDist {
    fn variables(&self) -> i32 {
        5
    }

    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        let computed = self.try_value(x);
        let Some(value) = self.capture.keep(computed) else {
            return false;
        };
        *f = value;
        true
    }
}

/// `GeomConvert_FuncCylinderLSDist`: the sum over the points of (d^2 - R^2)^2,
/// d the distance to the axis through (X1, X2, X3) along the direction, R = X4.
///
/// Quirk: `Gradient` scales G(6) of a 4-variable gradient, which raises
/// Standard_OutOfRange natively after G is doubled; `Values` scales G(4).
/// `let f = FuncCylinderLSDist::new(&points, Direction3::default());`
#[derive(Default)]
pub struct FuncCylinderLSDist {
    my_points: HArrayOfXyz,
    my_dir: Direction3,
    capture: FailureCapture,
}

impl FuncCylinderLSDist {
    /// The function of `points` and axis direction `dir`;
    /// `FuncCylinderLSDist::new(&points, dir)`.
    pub fn new(points: &HArrayOfXyz, dir: Direction3) -> Self {
        Self {
            my_points: points.clone(),
            my_dir: dir,
            capture: FailureCapture::default(),
        }
    }

    /// `SetPoints(thePoints)`; `f.set_points(&points)`.
    pub fn set_points(&mut self, points: &HArrayOfXyz) {
        self.my_points = points.clone();
    }

    /// `SetDir(theDir)`; `f.set_dir(dir)`.
    pub fn set_dir(&mut self, dir: Direction3) {
        self.my_dir = dir;
    }

    /// The failure a callback raised natively, if any; `f.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    // F = sum d^2 and G = 2 sum d * dD2/dX (G(4) not yet scaled by -2R); F is reset
    // once the variables are read, as natively (Gradient passes a scratch F).
    fn sums(
        &self,
        x: &Vector<'_>,
        (f, g): (&mut f64, &mut Vector<'_>),
        member: &str,
    ) -> Result<(), Failure> {
        let [x1, x2, x3, radius] = variables::<4>(x)?;
        let (loc, r2): (Xyz, f64) = (Xyz::new(x1, x2, x3), radius * radius);
        let (dx, dy, dz): (f64, f64, f64) = (self.my_dir.x(), self.my_dir.y(), self.my_dir.z());
        let dir_xyz: Xyz = Xyz::new(dx, dy, dz);
        *f = 0.0;
        g.init(0.0);
        for point in cloud(&self.my_points, member)?.iter() {
            let v: Xyz = *point - loc;
            let d: f64 = v.cross_square_magnitude(dir_xyz) - r2;
            let (vx, vy, vz): (f64, f64, f64) = (v.x(), v.y(), v.z());
            let dx0: f64 = 2. * (vz * dx - vx * dz) * dz - 2. * (vx * dy - vy * dx) * dy;
            let dy0: f64 = -2. * (vy * dz - vz * dy) * dz + 2. * (vx * dy - vy * dx) * dx;
            let dz0: f64 = 2. * (vy * dz - vz * dy) * dy - 2. * (vz * dx - vx * dz) * dx;
            accumulate(g, [d * dx0, d * dy0, d * dz0, d])?;
            *f += d * d;
        }
        g.multiply(2.0);
        Ok(())
    }

    fn try_value(&self, x: &Vector<'_>) -> Result<f64, Failure> {
        let [x1, x2, x3, radius] = variables::<4>(x)?;
        let loc = Xyz::new(x1, x2, x3);
        let r2: f64 = radius * radius;
        let dir_xyz = Xyz::new(self.my_dir.x(), self.my_dir.y(), self.my_dir.z());
        let mut f: f64 = 0.0;
        for point in cloud(&self.my_points, "GeomConvert_FuncCylinderLSDist::Value")?.iter() {
            let d: f64 = (*point - loc).cross_square_magnitude(dir_xyz) - r2;
            f += d * d;
        }
        Ok(f)
    }
}

impl MultipleVarFunction for FuncCylinderLSDist {
    fn variables(&self) -> i32 {
        4
    }

    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        let computed = self.try_value(x);
        let Some(value) = self.capture.keep(computed) else {
            return false;
        };
        *f = value;
        true
    }
}

impl MultipleVarFunctionWithGradient for FuncCylinderLSDist {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        let mut scratch: f64 = 0.0;
        let computed = self
            .sums(
                x,
                (&mut scratch, g),
                "GeomConvert_FuncCylinderLSDist::Gradient",
            )
            .and_then(|()| scale_component(g, 6, -2.0 * *x.value(4)?));
        self.capture.keep(computed).is_some()
    }

    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        let computed = self
            .sums(x, (f, g), "GeomConvert_FuncCylinderLSDist::Values")
            .and_then(|()| scale_component(g, 4, -2.0 * *x.value(4)?));
        self.capture.keep(computed).is_some()
    }
}

/// `GeomConvert_FuncSphereLSDist`: the sum over the points of (d^2 - R^2)^2, d
/// the distance to the center (X1, X2, X3), R = X4.
///
/// `let f = FuncSphereLSDist::new(&points);`
#[derive(Default)]
pub struct FuncSphereLSDist {
    my_points: HArrayOfXyz,
    capture: FailureCapture,
}

impl FuncSphereLSDist {
    /// The function of `points`; `FuncSphereLSDist::new(&points)`.
    pub fn new(points: &HArrayOfXyz) -> Self {
        Self {
            my_points: points.clone(),
            capture: FailureCapture::default(),
        }
    }

    /// `SetPoints(thePoints)`; `f.set_points(&points)`.
    pub fn set_points(&mut self, points: &HArrayOfXyz) {
        self.my_points = points.clone();
    }

    /// The failure a callback raised natively, if any; `f.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    // F = sum d^2 and G = -4 * sum d * (P - C) (and -4 R * sum d for G(4)); F is
    // reset after G, once the variables are read (Gradient passes a scratch F).
    fn sums(
        &self,
        x: &Vector<'_>,
        (f, g): (&mut f64, &mut Vector<'_>),
        member: &str,
    ) -> Result<(), Failure> {
        let [x1, x2, x3, radius] = variables::<4>(x)?;
        let loc = Xyz::new(x1, x2, x3);
        let r2: f64 = radius * radius;
        g.init(0.0);
        *f = 0.0;
        for point in cloud(&self.my_points, member)?.iter() {
            let d_loc: Xyz = *point - loc;
            let d: f64 = d_loc.square_modulus() - r2;
            accumulate(g, [d * d_loc.x(), d * d_loc.y(), d * d_loc.z(), d])?;
            *f += d * d;
        }
        g.multiply(-4.0);
        scale_component(g, 4, radius)
    }

    fn try_value(&self, x: &Vector<'_>) -> Result<f64, Failure> {
        let [x1, x2, x3, radius] = variables::<4>(x)?;
        let loc = Xyz::new(x1, x2, x3);
        let r2: f64 = radius * radius;
        let mut f: f64 = 0.0;
        for point in cloud(&self.my_points, "GeomConvert_FuncSphereLSDist::Value")?.iter() {
            let d: f64 = (*point - loc).square_modulus() - r2;
            f += d * d;
        }
        Ok(f)
    }
}

impl MultipleVarFunction for FuncSphereLSDist {
    fn variables(&self) -> i32 {
        4
    }

    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        let computed = self.try_value(x);
        let Some(value) = self.capture.keep(computed) else {
            return false;
        };
        *f = value;
        true
    }
}

impl MultipleVarFunctionWithGradient for FuncSphereLSDist {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        let mut scratch: f64 = 0.0;
        let computed = self.sums(
            x,
            (&mut scratch, g),
            "GeomConvert_FuncSphereLSDist::Gradient",
        );
        self.capture.keep(computed).is_some()
    }

    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        let computed = self.sums(x, (f, g), "GeomConvert_FuncSphereLSDist::Values");
        self.capture.keep(computed).is_some()
    }
}
