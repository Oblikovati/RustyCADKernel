// Rust translation of the EvalDN checks, frame reads and analytic DumpJson members of the ModelingData/TKG3d/GeomEval curves at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! What the GeomEval curves share: the `EvalDN` order check of every curve, the
//! `gp_XYZ` frame the analytic curves read from their `gp_Ax2` (`anO`, `aXD`, `aYD`,
//! `aZD`), and the analytic curves' `DumpJson` shape (class name, Geom_Curve base,
//! `&myPosition`, then the numeric parameters in order).
use crate::geom::dump_curve;
use crate::geom::errors;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field};
use rustycad_tkmath::gp::{Axis2, Xyz};
use std::io;

/// The location and the three directions of a `gp_Ax2` as `gp_XYZ` values;
/// `AxisFrame::new(Axis2::default()).x_dir`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AxisFrame {
    pub(crate) origin: Xyz,
    pub(crate) x_dir: Xyz,
    pub(crate) y_dir: Xyz,
    pub(crate) z_dir: Xyz,
}
impl AxisFrame {
    /// `Location()`, `XDirection()`, `YDirection()` and `Direction()` of `position`;
    /// `AxisFrame::new(axis)`.
    pub(crate) fn new(position: Axis2) -> Self {
        Self {
            origin: position.location().xyz(),
            x_dir: position.x_direction().xyz(),
            y_dir: position.y_direction().xyz(),
            z_dir: position.direction().xyz(),
        }
    }
}

/// Standard_RangeError of `EvalDN` below order 1, as every GeomEval curve raises it;
/// `check_derivative_order("GeomEval_SineWaveCurve::EvalDN", 0)` is an error.
pub(crate) fn check_derivative_order(member: &str, n: i32) -> Result<(), Failure> {
    if n < 1 {
        return Err(errors::range(&format!(
            "{member}: N = {n}; expected N >= 1"
        )));
    }
    Ok(())
}

/// Write `className`, the Geom_Curve base, `"Position"` and each `(name, value)` field;
/// `dump_analytic_curve(&mut s, -1, "GeomEval_SineWaveCurve", axis, &[("Amplitude", 2.0)])?`.
pub(crate) fn dump_analytic_curve(
    stream: &mut DumpStream,
    depth: i32,
    class_name: &str,
    position: Axis2,
    fields: &[(&str, f64)],
) -> io::Result<()> {
    dump_class_name(stream, class_name)?;
    dump_base(stream, depth, dump_curve)?;
    dump_field(stream, depth, "Position", |nested, _| {
        position.dump_json(nested)
    })?;
    for &(name, value) in fields {
        dump::dump_real(stream, name, value)?;
    }
    Ok(())
}
