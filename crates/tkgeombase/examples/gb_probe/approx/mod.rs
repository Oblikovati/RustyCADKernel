//! The Rust twin of tools/oracle/tkgeombase_probe_approx_line.cxx and
//! tkgeombase_probe_approx.cxx.
mod adaptors;
mod compute;
mod fit;
mod line;
mod sweep;

pub use adaptors::{
    adapted_curve, adapted_curve2d, adapted_surface, cos_case, curve2d_case, curve3d_case,
    curvilinear_case, curvlin_case, same_case,
};
pub use compute::{bspl_case, build_compute, compute_case, compute_results};
pub use fit::{fit_case, mcurves_case};
pub use sweep::sweep_case;

use super::app_par_curves::curve;
use super::common::{self as p, Reader};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_par_curves::{Constraint, MultiBSpCurve, MultiCurveEval};
use rustycad_tkgeombase::approx::{ComputeLineOptions, ParametrizationType};
use rustycad_tkmath::geom_abs::Shape;

/// `GeomAbs_Shape(code)`: the native enumeration order.
pub fn shape(code: i32) -> Shape {
    [
        Shape::C0,
        Shape::G1,
        Shape::C1,
        Shape::G2,
        Shape::C2,
        Shape::C3,
        Shape::CN,
    ][code as usize]
}

pub fn constraint(code: i32) -> Constraint {
    Constraint::from_code(code).expect("probe constraint")
}

/// `gb_ax_array`.
pub fn array(a: &Array1<'_, f64>) {
    p::int(a.lower());
    p::int(a.upper());
    a.iter().for_each(|&v| p::real(v));
}

/// `gb_ax_bspline`.
pub fn bspline(c: &MultiBSpCurve) -> Result<(), Failure> {
    curve(c)?;
    p::int(MultiCurveEval::degree(c)?);
    if c.nb_poles() == 0 {
        return Ok(());
    }
    array(&c.knots()?);
    c.multiplicities()?.iter().for_each(|&m| p::int(m));
    Ok(())
}

/// `gb_ax_read_options`.
pub fn read_options(r: &mut Reader<'_>) -> ComputeLineOptions {
    ComputeLineOptions {
        degree_min: r.integer(),
        degree_max: r.integer(),
        tolerance_3d: r.real(),
        tolerance_2d: r.real(),
        nb_iterations: r.integer(),
        cutting: r.flag(),
        parametrization: ParametrizationType::from_code(r.integer()).expect("probe type"),
        squares: r.flag(),
    }
}
