// Rust translation of ModelingData/TKG2d/Adaptor2d/Adaptor2d_OffsetCurve.cxx (IsClosed, GetType and the curve-kind queries) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Closure, curve kind and kind-specific queries of `Adaptor2d_OffsetCurve`.
//!
//! An offset of a line is a line and an offset of a circle a circle; every other
//! offset is a `GeomAbs_OffsetCurve`, and the basis-specific queries answer only
//! for a null offset.
use super::curve2d::Adaptor2dCurve2d;
use super::offset_curve::{Adaptor2dOffsetCurve, OffsetState};
use crate::geom2d::errors::no_such_object;
use crate::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{
    Axis22d, Circle2, Direction2, Ellipse2, Hyperbola2, Line2, Parabola2, Vector2,
};

fn no_such(member: &str) -> Failure {
    no_such_object(&format!(
        "Adaptor2d_OffsetCurve::{member}; expected a basis of the matching type and, \
         except for Line and Circle, a null offset"
    ))
}

/// Closure (`IsClosed`): a nonzero offset of a closed curve is closed when the end
/// tangents are parallel and not opposite; `is_closed(&state)?`.
pub(super) fn is_closed(state: &OffsetState) -> Result<bool, Failure> {
    let basis: &dyn Adaptor2dCurve2d = state.basis()?;
    if state.offset == 0.0 {
        return basis.is_closed();
    }
    if basis.continuity()? == Shape::C0 || !basis.is_closed()? {
        return Ok(false);
    }
    let (_, start) = basis.d1(basis.first_parameter()?)?;
    let (_, end) = basis.d1(basis.last_parameter()?)?;
    Ok(start.is_parallel(end, precision::angular())?
        && !(start.is_opposite(end, precision::angular())?))
}

/// The curve kind (`GetType`); `get_type(&state)?`.
pub(super) fn get_type(state: &OffsetState) -> Result<CurveType, Failure> {
    let basis: CurveType = state.basis()?.get_type()?;
    if state.offset == 0.0 {
        return Ok(basis);
    }
    Ok(match basis {
        CurveType::Line => CurveType::Line,
        CurveType::Circle => CurveType::Circle,
        _ => CurveType::OffsetCurve,
    })
}

/// The line through the offset point at parameter 0 along its tangent (`Line`);
/// `line(&adaptor)?`.
pub(super) fn line(adaptor: &Adaptor2dOffsetCurve) -> Result<Line2, Failure> {
    if adaptor.get_type()? != CurveType::Line {
        return Err(no_such("Line"));
    }
    let (point, tangent): (_, Vector2) = adaptor.d1(0.0)?;
    Ok(Line2::new(point, Direction2::from_vector(tangent)?))
}

/// The offset circle: the radius grows by the offset for a direct frame and shrinks
/// for an indirect one; a negative result flips the X direction (`Circle`);
/// `circle(&state)?`.
pub(super) fn circle(state: &OffsetState) -> Result<Circle2, Failure> {
    if get_type(state)? != CurveType::Circle {
        return Err(no_such("Circle"));
    }
    let basis: Circle2 = state.basis()?.circle()?;
    if state.offset == 0.0 {
        return Ok(basis);
    }
    let mut axes: Axis22d = basis.position();
    let (xd, yd) = (axes.x_direction(), axes.y_direction());
    let crossed: f64 = xd.x() * yd.y() - xd.y() * yd.x();
    let sign: f64 = if crossed > 0.0 { 1.0 } else { -1.0 };
    let radius: f64 = basis.radius() + sign * state.offset;
    if radius > 0.0 {
        return Circle2::new(axes, radius);
    }
    if radius < 0.0 {
        axes.set_x_direction(axes.x_direction().reversed())?;
        return Circle2::new(axes, -radius);
    }
    // Circle of null radius.
    Err(no_such("Circle"))
}

// Whether the kind-specific basis query applies: the basis has `kind` and the offset is null.
fn basis_is(state: &OffsetState, kinds: &[CurveType]) -> Result<bool, Failure> {
    let basis: CurveType = state.basis()?.get_type()?;
    Ok(kinds.contains(&basis) && state.offset == 0.0)
}

/// The basis ellipse of a null offset (`Ellipse`); `ellipse(&state)?`.
pub(super) fn ellipse(state: &OffsetState) -> Result<Ellipse2, Failure> {
    if !basis_is(state, &[CurveType::Ellipse])? {
        return Err(no_such("Ellipse"));
    }
    state.basis()?.ellipse()
}

/// The basis hyperbola of a null offset (`Hyperbola`); `hyperbola(&state)?`.
pub(super) fn hyperbola(state: &OffsetState) -> Result<Hyperbola2, Failure> {
    if !basis_is(state, &[CurveType::Hyperbola])? {
        return Err(no_such("Hyperbola"));
    }
    state.basis()?.hyperbola()
}

/// The basis parabola of a null offset (`Parabola`); `parabola(&state)?`.
pub(super) fn parabola(state: &OffsetState) -> Result<Parabola2, Failure> {
    if !basis_is(state, &[CurveType::Parabola])? {
        return Err(no_such("Parabola"));
    }
    state.basis()?.parabola()
}

const POLE_CURVES: [CurveType; 2] = [CurveType::BezierCurve, CurveType::BSplineCurve];

/// The basis degree of a null offset of a Bezier or B-spline curve (`Degree`);
/// `degree(&state)?`.
pub(super) fn degree(state: &OffsetState) -> Result<i32, Failure> {
    if !basis_is(state, &POLE_CURVES)? {
        return Err(no_such("Degree"));
    }
    state.basis()?.degree()
}

/// The basis pole count of a null offset of a Bezier or B-spline curve (`NbPoles`);
/// `nb_poles(&state)?`.
pub(super) fn nb_poles(state: &OffsetState) -> Result<i32, Failure> {
    if !basis_is(state, &POLE_CURVES)? {
        return Err(no_such("NbPoles"));
    }
    state.basis()?.nb_poles()
}

/// The basis knot count of a null offset (`NbKnots`); `nb_knots(&state)?`.
pub(super) fn nb_knots(state: &OffsetState) -> Result<i32, Failure> {
    if state.offset != 0.0 {
        return Err(no_such("NbKnots"));
    }
    state.basis()?.nb_knots()
}

/// The basis Bezier curve of a null offset (`Bezier`); `bezier(&state)?`.
pub(super) fn bezier(state: &OffsetState) -> Result<Handle<Geom2dBezierCurve>, Failure> {
    if state.offset != 0.0 || get_type(state)? != CurveType::BezierCurve {
        return Err(no_such_object(
            "Adaptor2d_OffsetCurve::Bezier() - wrong curve type",
        ));
    }
    state.basis()?.bezier()
}

/// The basis B-spline curve of a null offset (`BSpline`); `bspline(&state)?`.
pub(super) fn bspline(state: &OffsetState) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    if state.offset != 0.0 || get_type(state)? != CurveType::BSplineCurve {
        return Err(no_such_object(
            "Adaptor2d_OffsetCurve::BSpline() - wrong curve type",
        ));
    }
    state.basis()?.bspline()
}

/// Samples of the basis: at least 20, `3 + NbPoles` for a Bezier basis and
/// `NbKnots * Degree` for a B-spline one, at most 300 (`NbSamples`, the file-local
/// `nbPoints`); `nb_samples(&state)?`.
pub(super) fn nb_samples(state: &OffsetState) -> Result<i32, Failure> {
    let basis: &dyn Adaptor2dCurve2d = state.basis()?;
    let mut count: i32 = 20;
    if basis.get_type()? == CurveType::BezierCurve {
        count = count.max(3 + basis.nb_poles()?);
    } else if basis.get_type()? == CurveType::BSplineCurve {
        count = count.max(basis.nb_knots()? * basis.degree()?);
    }
    Ok(count.min(300))
}
