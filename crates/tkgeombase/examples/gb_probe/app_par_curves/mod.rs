//! The Rust twin of tools/oracle/tkgeombase_probe_appparcurves*.cxx.
mod containers;
mod fit;
mod line;
pub use line::{multi_point, read_line};

pub use containers::{bernstein_case, multicurve_case, multipoint_case, read_points, spline_case};
pub use fit::{
    bspfunction_case, bspgradient_case, function_case, gradient_case, ls_case, read_constraints,
    resol_case,
};

use super::common as p;
pub use super::fem_tool::matrix;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_par_curves::{MultiCurve, MultiCurveEval, MultiPoint};
use rustycad_tkmath::math::IntegerVector;

pub fn ivector(v: &IntegerVector<'_>) {
    p::int(v.lower());
    p::int(v.upper());
    (v.lower()..=v.upper()).for_each(|i| p::int(v[i]));
}

pub fn dump(text: &str) {
    print!("{} ", text.replace('\n', "|"));
}

pub fn point(m: &MultiPoint) -> Result<(), Failure> {
    p::int(m.nb_points());
    p::int(m.nb_points2d());
    for i in 1..=m.nb_points() {
        p::pnt(m.point(i)?);
    }
    for i in m.nb_points() + 1..=m.nb_points() + m.nb_points2d() {
        p::pnt2d(m.point2d(i)?);
    }
    Ok(())
}

pub fn curve(c: &MultiCurve) -> Result<(), Failure> {
    p::int(c.nb_poles());
    p::int(c.nb_curves()?);
    for j in 1..=c.nb_poles() {
        point(&c.value(j)?)?;
    }
    Ok(())
}
