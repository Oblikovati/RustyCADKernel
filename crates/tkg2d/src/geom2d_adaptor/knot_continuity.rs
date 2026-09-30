// Rust translation of ModelingData/TKG2d/Geom2dAdaptor/Geom2dAdaptor_Curve.cxx and ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Curve.cxx (LocalContinuity and the interval continuity switches that both adaptors repeat) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The continuity steps that Geom2dAdaptor_Curve and GeomAdaptor_Curve run word for
//! word on the knots of a B-spline and on the basis of an offset curve.
use crate::geom2d::errors::{array_bounds, domain, no_such_object};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::Shape;

/// The knots, multiplicities, degree and periodicity of one B-spline parameter
/// direction; `KnotSpans { knots: &k, mults: &m, degree: 3, periodic: false }`.
pub struct KnotSpans<'a> {
    /// The distinct knots, one-based.
    pub knots: &'a Array1<'a, f64>,
    /// The knot multiplicities, one-based.
    pub mults: &'a Array1<'a, i32>,
    /// The degree.
    pub degree: i32,
    /// Whether the direction is periodic.
    pub periodic: bool,
}

/// The knot index of `u` and the parameter brought into the knot range
/// (`BSplCLib::LocateParameter` over `1..=NbKnots`); `locate_knot(&spans, 0.5)?`.
pub fn locate_knot(spans: &KnotSpans<'_>, u: f64) -> Result<(i32, f64), Failure> {
    let (mut index, mut new_u): (i32, f64) = (0, 0.0);
    let count: i32 = spans.knots.len() as i32;
    bsplclib::locate_parameter(
        spans.degree,
        spans.knots,
        spans.mults,
        u,
        spans.periodic,
        1,
        count,
        &mut index,
        &mut new_u,
    )?;
    Ok((index, new_u))
}

/// The native loop `TM(Index1 + 1)`, then `TM(Index1 + 1..=Index2)`: the highest
/// multiplicity of the knots after `index1` up to `index2`;
/// `max_multiplicity(&mults, 1, 3)?`.
pub fn max_multiplicity(mults: &Array1<'_, i32>, index1: i32, index2: i32) -> Result<i32, Failure> {
    let mut max: i32 = *mults.value(index1 + 1).map_err(array_bounds)?;
    for i in index1 + 1..=index2 {
        max = max.max(*mults.value(i).map_err(array_bounds)?);
    }
    Ok(max)
}

/// `LocalContinuity` of the curve adaptors: `C(Degree - m)` with `m` the highest
/// multiplicity of the knots strictly inside `[u1, u2]`, CN between two consecutive
/// knots of a non-periodic curve; `local_continuity(&spans, 0.0, 1.0)?`.
pub fn local_continuity(spans: &KnotSpans<'_>, u1: f64, u2: f64) -> Result<Shape, Failure> {
    let count: i32 = spans.knots.len() as i32;
    let (mut index1, new_first) = locate_knot(spans, u1)?;
    let (mut index2, new_last) = locate_knot(spans, u2)?;
    let next_knot: f64 = *spans.knots.value(index1 + 1).map_err(array_bounds)?;
    if (new_first - next_knot).abs() < precision::p_confusion() && index1 < count {
        index1 += 1;
    }
    let last_knot: f64 = *spans.knots.value(index2).map_err(array_bounds)?;
    if (new_last - last_knot).abs() < precision::p_confusion() {
        index2 -= 1;
    }
    // Beware of periodic curves.
    if spans.periodic && index1 == count {
        index1 = 1;
    }
    let mult_max: i32 = if index2 - index1 <= 0 && !spans.periodic {
        100 // CN between two consecutive knots.
    } else {
        spans.degree - max_multiplicity(spans.mults, index1, index2)?
    };
    Ok(match mult_max {
        i32::MIN..=0 => Shape::C0,
        1 => Shape::C1,
        2 => Shape::C2,
        3 => Shape::C3,
        _ => Shape::CN,
    })
}

/// `Continuity` of an offset curve from its basis continuity: one order is lost,
/// G1 and G2 stay, and a C0 basis raises Standard_NoSuchObject naming `class`;
/// `offset_continuity(Shape::C2, "GeomAdaptor_Curve")? == Shape::C1`.
pub fn offset_continuity(basis: Shape, class: &str) -> Result<Shape, Failure> {
    Ok(match basis {
        Shape::CN => Shape::CN,
        Shape::C3 => Shape::C2,
        Shape::C2 => Shape::C1,
        Shape::C1 => Shape::C0,
        Shape::G1 => Shape::G1,
        Shape::G2 => Shape::G2,
        Shape::C0 => return Err(no_such_object(&format!("{class}::Continuity"))),
    })
}

/// The basis continuity of an offset's intervals, one order higher; G1 and G2 have
/// none (both adaptors' messages name GeomAdaptor_Curve);
/// `offset_basis_shape(Shape::C1)? == Shape::C2`.
pub fn offset_basis_shape(shape: Shape) -> Result<Shape, Failure> {
    Ok(match shape {
        Shape::G1 | Shape::G2 => return Err(domain("GeomAdaptor_Curve::NbIntervals")),
        Shape::C0 => Shape::C1,
        Shape::C1 => Shape::C2,
        Shape::C2 => Shape::C3,
        Shape::C3 | Shape::CN => Shape::CN,
    })
}

/// The `BSplCLib::Intervals` continuity order of `shape` (the degree for CN);
/// Standard_DomainError naming `class::member()` for C0, G1 and G2;
/// `interval_order(Shape::C2, 3, "GeomAdaptor_Curve", "NbIntervals")? == 2`.
pub fn interval_order(
    shape: Shape,
    degree: i32,
    class: &str,
    member: &str,
) -> Result<i32, Failure> {
    Ok(match shape {
        Shape::C1 => 1,
        Shape::C2 => 2,
        Shape::C3 => 3,
        Shape::CN => degree,
        _ => return Err(domain(&format!("{class}::{member}()"))),
    })
}
