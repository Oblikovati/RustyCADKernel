// Rust translation of the frame code shared by ModelingData/TKG3d/GeomGridEval/GeomGridEval_{Circle,Ellipse,Hyperbola,Parabola}.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pre-extracted conic frame (`aCX, aCY, aCZ, aXX ... aYZ`) with the native
//! operation order of each formula.
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

/// Center and axis coordinates that the conic evaluators pre-extract.
#[derive(Clone, Copy)]
pub(super) struct GridFrame3 {
    center: [f64; 3],
    x: [f64; 3],
    y: [f64; 3],
}

// The three coordinates of a direction.
fn coordinates(d: Direction3) -> [f64; 3] {
    [d.x(), d.y(), d.z()]
}

impl GridFrame3 {
    /// Frame of a center and its X and Y directions.
    pub(super) fn new(center: Point3, x_dir: Direction3, y_dir: Direction3) -> Self {
        Self {
            center: [center.x(), center.y(), center.z()],
            x: coordinates(x_dir),
            y: coordinates(y_dir),
        }
    }
    // `a * aXk + b * aYk` for coordinate k.
    fn term(&self, k: usize, a: f64, b: f64) -> f64 {
        a * self.x[k] + b * self.y[k]
    }
    /// `a * X + b * Y`, evaluated as `(a * aXX + b * aYX, ...)`.
    pub(super) fn combined(&self, a: f64, b: f64) -> Vector3 {
        Vector3::new(self.term(0, a, b), self.term(1, a, b), self.term(2, a, b))
    }
    /// `a * X`, as `(a * aXX, a * aXY, a * aXZ)`.
    pub(super) fn scaled_x(&self, a: f64) -> Vector3 {
        Vector3::new(a * self.x[0], a * self.x[1], a * self.x[2])
    }
    /// `C + a * X + b * Y`, evaluated left to right as `aCX + a * aXX + b * aYX`.
    pub(super) fn offset_point(&self, a: f64, b: f64) -> Point3 {
        let c = self.center;
        Point3::new(
            c[0] + a * self.x[0] + b * self.y[0],
            c[1] + a * self.x[1] + b * self.y[1],
            c[2] + a * self.x[2] + b * self.y[2],
        )
    }
    /// `C + v`, for the circle, whose formula scales the whole combination by `R`.
    pub(super) fn translated(&self, v: Vector3) -> Point3 {
        let c = self.center;
        Point3::new(c[0] + v.x(), c[1] + v.y(), c[2] + v.z())
    }
}

/// A frame with major and minor radii: the ellipse and hyperbola terms
/// `C + MajR * a * X + MinR * b * Y` and `MajR * a * X + MinR * b * Y`.
#[derive(Clone, Copy)]
pub(super) struct RadiiFrame3 {
    pub(super) frame: GridFrame3,
    pub(super) major: f64,
    pub(super) minor: f64,
}

impl RadiiFrame3 {
    /// `aCX + aMajR * a * aXX + aMinR * b * aYX` (and Y, Z).
    pub(super) fn point(&self, a: f64, b: f64) -> Point3 {
        self.frame.offset_point(self.major * a, self.minor * b)
    }
    /// `aMajR * a * aXX + aMinR * b * aYX` (and Y, Z); a negated coefficient gives the
    /// same bits as the native `-aMajR * sinU` spelling.
    pub(super) fn vector(&self, a: f64, b: f64) -> Vector3 {
        self.frame.combined(self.major * a, self.minor * b)
    }
}
