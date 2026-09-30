// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector (Bisector.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Bisector: the bisecting line (the locus of the points equidistant
//! from two elements) between two points, lines, circles or general curves.
//!
//! The analytic bisectors of points, lines and circles are [`BisecAna`]; the bisector
//! of a curve and a point is [`BisecPC`] and the bisector of two curves [`BisecCC`].
//! All three implement the trait [`BisectorCurve`] (`Bisector_Curve`, a
//! `Geom2d_Curve`), as docs/topalgo-design.md section 3 fixes. [`Bisec`] chooses and
//! trims the right one, and [`BisectorInter`] intersects two of them.
mod bisec;
mod bisec_ana;
mod bisec_cc;
mod bisec_pc;
mod curve;
mod eval_failure;
mod function_h;
mod function_inter;
mod inter;
mod intervals;
mod point_on_bis;
mod poly_bis;

pub use bisec::Bisec;
pub use bisec_ana::BisecAna;
pub use bisec_cc::BisecCC;
pub use bisec_pc::BisecPC;
pub use curve::BisectorCurve;
pub use function_h::BisectorFunctionH;
pub use function_inter::BisectorFunctionInter;
pub use inter::BisectorInter;
pub use point_on_bis::PointOnBis;
pub use poly_bis::PolyBis;

use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::{Point2, Vector2};

/// `Bisector::IsConvex`: true when the curve turns to the side `sign` (or is flat) at
/// the middle of its parameter range, `sign * (D1 ^ D2) < 1e-5`.
/// ```
/// use rustycad_tktopalgo::bisector::is_convex;
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
/// let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis, 1.0, true)?).cast();
/// assert!(is_convex(&circle, -1.0)? && !is_convex(&circle, 1.0)?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub fn is_convex(cu: &Handle<dyn Geom2dCurve>, sign: f64) -> Result<bool, Failure> {
    let u1: f64 = (cu.last_parameter() + cu.first_parameter()) / 2.;
    let (_, v1, v2): (Point2, Vector2, Vector2) = cu.d2(u1)?;
    let tol: f64 = 1.0e-5;
    Ok(sign * v1.crossed(v2) < tol)
}
