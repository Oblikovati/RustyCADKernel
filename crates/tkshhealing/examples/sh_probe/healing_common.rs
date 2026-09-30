//! Printers shared by the ShapeAnalysis, ShapeConstruct and ShapeFix probe operations, the
//! Rust twin of tools/oracle/tkshhealing_probe_healing_common.cxx.
use super::brep_lib_common::type_name;
use super::common::{flag, pnt, pnt2d, real, show};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::{Point2, Point3, Vector2};
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;

/// `sh_flags`: DONE1..DONE8 then FAIL1..FAIL8 of a tool's status.
pub fn flags(status: impl Fn(ShapeExtendStatus) -> bool) {
    for k in (1..=8).chain(10..=17) {
        flag(status(ShapeExtendStatus::ALL[k]));
    }
}

/// `tp_curve_points`.
pub fn curve_points<P: Copy>(
    value: impl Fn(f64) -> Result<P, Failure>,
    f: f64,
    l: f64,
    print: fn(P),
) {
    let (finf, linf) = (precision::is_infinite(f), precision::is_infinite(l));
    if !finf {
        show(value(f), print);
    }
    if !linf {
        show(value(l), print);
    }
    if !finf && !linf {
        show(value(0.5 * (f + l)), print);
    }
}

/// `sh_curve3d`.
pub fn curve3d(c: &Handle<dyn GeomCurve>, f: f64, l: f64) {
    type_name(c);
    let Some(curve) = c.get() else { return };
    real(f);
    real(l);
    curve_points::<Point3>(|u| curve.value(u), f, l, pnt);
}

/// `sh_curve2d`.
pub fn curve2d(c: &Handle<dyn Geom2dCurve>, f: f64, l: f64) {
    type_name(c);
    let Some(curve) = c.get() else { return };
    real(f);
    real(l);
    curve_points::<Point2>(|u| curve.value(u), f, l, pnt2d);
}

/// `sh_vec2d`.
pub fn vec2d(v: Vector2) {
    real(v.x());
    real(v.y());
}
