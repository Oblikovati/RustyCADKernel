// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_Projector.cxx (Project and Shoot) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projections of HLRAlgo_Projector: points, points with a derivative, and the
//! sight lines through the view plane.
use super::projector::Projector;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Line3, Point2, Point3, Vector2, Vector3};

/// The two spellings of `sqrt(2)/2` of the native standard views (type 0 uses the
/// lower one in `Project(P, X, Y, Z)`).
// FRAC_1_SQRT_2 is the native literal 0.7071067811865476, bit for bit.
const HALF_SQRT2_HIGH: f64 = std::f64::consts::FRAC_1_SQRT_2;
const HALF_SQRT2_LOW: f64 = 0.7071067811865475;

impl Projector {
    /// Project(P, Pout): the projection of `p` in the view plane. For the standard view
    /// of type 0 the native member computes the coordinates but never stores them, so
    /// `pout` keeps its value; `projector.project(p, &mut projected)`.
    pub fn project(&self, p: Point3, pout: &mut Point2) {
        match self.kind() {
            0 => {}
            1 => pout.set_coordinates(p.x(), p.y()),
            2 => pout.set_coordinates(p.x(), p.z()),
            3 => {
                let xmy05 = (p.x() - p.y()) * 0.5;
                let z07 = p.z() * HALF_SQRT2_HIGH;
                pout.set_coordinates(HALF_SQRT2_HIGH * (p.x() + p.y()), -xmy05 + z07);
            }
            _ => *pout = self.transformed_projection(p),
        }
    }

    /// The general projection: the transformed point, divided by the perspective
    /// ratio when the projection is a perspective.
    fn transformed_projection(&self, p: Point3) -> Point2 {
        let mut p2 = p;
        self.transform_point(&mut p2);
        if self.perspective() {
            let r = self.perspective_ratio(p2.z());
            return Point2::new(p2.x() / r, p2.y() / r);
        }
        Point2::new(p2.x(), p2.y())
    }

    /// Project(P, X, Y, Z): the view coordinates of `p` and its depth `z`; in
    /// perspective `x` and `y` are divided by the perspective ratio;
    /// `let (x, y, z) = projector.project_xyz(p);`.
    pub fn project_xyz(&self, p: Point3) -> (f64, f64, f64) {
        match self.kind() {
            0 => {
                let x07 = p.x() * HALF_SQRT2_LOW;
                let (y05, z05) = (p.y() * 0.5, p.z() * 0.5);
                (
                    x07 - y05 + z05,
                    x07 + y05 - z05,
                    HALF_SQRT2_LOW * (p.y() + p.z()),
                )
            }
            1 => (p.x(), p.y(), p.z()),
            2 => (p.x(), p.z(), -p.y()),
            3 => {
                let xmy05 = (p.x() - p.y()) * 0.5;
                let z07 = p.z() * HALF_SQRT2_HIGH;
                (HALF_SQRT2_HIGH * (p.x() + p.y()), -xmy05 + z07, xmy05 + z07)
            }
            _ => self.transformed_xyz(p),
        }
    }

    fn transformed_xyz(&self, p: Point3) -> (f64, f64, f64) {
        let mut p2 = p;
        self.transform_point(&mut p2);
        let (x, y, z) = (p2.x(), p2.y(), p2.z());
        if !self.perspective() {
            return (x, y, z);
        }
        let r = self.perspective_ratio(z);
        (x / r, y / r, z)
    }

    /// Project(P, D1, Pout, D1out): the projection of `p` and of the derivative `d1`
    /// there (with the perspective derivative formula); the standard views are not
    /// used; `let (p2, d2) = projector.project_d1(p, d);`.
    pub fn project_d1(&self, p: Point3, d1: Vector3) -> (Point2, Vector2) {
        let mut pp = p;
        pp.transform(self.transformation());
        let mut dd1 = d1;
        dd1.transform(self.transformation());
        if !self.perspective() {
            return (Point2::new(pp.x(), pp.y()), Vector2::new(dd1.x(), dd1.y()));
        }
        let focus = self.inner_focus();
        let r = self.perspective_ratio(pp.z());
        let scale = focus * r * r;
        let point = Point2::new(pp.x() / r, pp.y() / r);
        let derivative = Vector2::new(
            dd1.x() / r + pp.x() * dd1.z() / scale,
            dd1.y() / r + pp.y() * dd1.z() / scale,
        );
        (point, derivative)
    }

    /// Shoot(X, Y): the sight line through the view point `(x, y)` in the model: from
    /// the eye `(0, 0, focus)` towards `(x, y, -focus)` in perspective (a null
    /// direction raises Standard_ConstructionError), along -Z otherwise;
    /// `projector.shoot(1.0, 2.0)?`.
    pub fn shoot(&self, x: f64, y: f64) -> Result<Line3, Failure> {
        let line = if self.perspective() {
            let focus = self.inner_focus();
            Line3::new(Point3::new(0.0, 0.0, focus), Direction3::new(x, y, -focus)?)
        } else {
            Line3::new(Point3::new(x, y, 0.0), Direction3::new(0.0, 0.0, -1.0)?)
        };
        Ok(line.transformed(self.inverted_transformation()))
    }
}
