// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Surface.cxx (LocalContinuity, U/VContinuity, NbU/VIntervals, U/VIntervals, IsU/VClosed, U/VPeriod) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Continuity, continuity intervals, closure and periods of `GeomAdaptor_Surface`.
//!
//! The U and V members of the native class are mirror images; here one function
//! takes the direction. A B-spline is `C(Degree - m)` over the adaptor range, and its
//! intervals are those of the iso curve at the first knot of the other direction;
//! a swept surface has the intervals of a B-spline basis curve; an offset surface
//! those of its basis one order higher.
use super::curve::GeomAdaptorCurve;
use super::surface::GeomAdaptorSurface;
use super::surface_query::basis_surface_adaptor;
use super::surface_state::{SurfaceData, SurfaceState};
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::errors::{array_bounds, domain, no_such_object};
use crate::geom::{GeomBSplineSurface, GeomCurve, GeomSurface, GeomSweptSurface};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_adaptor::knot_continuity::{KnotSpans, locate_knot, max_multiplicity};
use rustycad_tkmath::geom_abs::{CurveType, Shape};

/// A parametric direction of the surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Direction {
    U,
    V,
}
impl Direction {
    fn name(self) -> &'static str {
        match self {
            Direction::U => "U",
            Direction::V => "V",
        }
    }
    // The adaptor range along this direction.
    fn range(self, state: &SurfaceState) -> (f64, f64) {
        match self {
            Direction::U => (state.u_first, state.u_last),
            Direction::V => (state.v_first, state.v_last),
        }
    }
}

// The file-local LocalContinuity of the surface: unlike GeomAdaptor_Curve's, the
// first index moves past a knot at the first bound without a bound check, and a
// single span is CN even on a periodic surface.
fn local_continuity(
    degree: i32,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    (first, last): (f64, f64),
    periodic: bool,
) -> Result<Shape, Failure> {
    let count: i32 = knots.len() as i32;
    let spans = KnotSpans {
        knots,
        mults,
        degree,
        periodic,
    };
    let (mut index1, new_first) = locate_knot(&spans, first)?;
    let (mut index2, new_last) = locate_knot(&spans, last)?;
    let eps_knot: f64 = precision::p_confusion();
    if (new_first - *knots.value(index1 + 1).map_err(array_bounds)?).abs() < eps_knot {
        index1 += 1;
    }
    if (new_last - *knots.value(index2).map_err(array_bounds)?).abs() < eps_knot {
        index2 -= 1;
    }
    // Handle periodic curves.
    if periodic && index1 == count {
        index1 = 1;
    }
    if index2 == index1 {
        return Ok(Shape::CN);
    }
    let mult_max: i32 = max_multiplicity(mults, index1, index2)?;
    Ok(match degree - mult_max {
        i32::MIN..=0 => Shape::C0,
        1 => Shape::C1,
        2 => Shape::C2,
        3 => Shape::C3,
        _ => Shape::CN,
    })
}

/// `UContinuity` / `VContinuity`; Standard_NoSuchObject for an unknown kind;
/// `continuity(&state, Direction::U)?`.
pub(super) fn continuity(state: &SurfaceState, direction: Direction) -> Result<Shape, Failure> {
    match (&state.data, direction) {
        (SurfaceData::BSpline(data), _) => bspline_continuity(&data.surface, state, direction),
        (SurfaceData::Offset(_), _) => offset_continuity(state, direction),
        (SurfaceData::Extrusion(data), Direction::U) => data.curve()?.continuity(),
        (SurfaceData::Revolution(data), Direction::V) => data.curve()?.continuity(),
        (SurfaceData::None, _) => Err(no_such_object(&format!(
            "GeomAdaptor_Surface::{}Continuity",
            direction.name()
        ))),
        _ => Ok(Shape::CN),
    }
}

fn bspline_continuity(
    surface: &GeomBSplineSurface,
    state: &SurfaceState,
    direction: Direction,
) -> Result<Shape, Failure> {
    let geom: &dyn GeomSurface = state.surface()?;
    match direction {
        Direction::U => local_continuity(
            surface.u_degree(),
            &surface.u_knots(),
            &surface.u_multiplicities(),
            direction.range(state),
            geom.is_u_periodic(),
        ),
        Direction::V => local_continuity(
            surface.v_degree(),
            &surface.v_knots(),
            &surface.v_multiplicities(),
            direction.range(state),
            geom.is_v_periodic(),
        ),
    }
}

// The continuity of a new adaptor on the offset basis (BasisSurface()), one order lower.
fn offset_continuity(state: &SurfaceState, direction: Direction) -> Result<Shape, Failure> {
    let basis: GeomAdaptorSurface = basis_surface_adaptor(state)?;
    let basis_shape: Shape = match direction {
        Direction::U => basis.u_continuity()?,
        Direction::V => basis.v_continuity()?,
    };
    Ok(match basis_shape {
        Shape::CN | Shape::C3 => Shape::CN,
        Shape::G2 | Shape::C2 => Shape::C1,
        Shape::G1 | Shape::C1 | Shape::C0 => Shape::C0,
    })
}

// The basis continuity of an offset's intervals; G1 and G2 have none (the native
// message names GeomAdaptor_Curve).
fn offset_basis_shape(shape: Shape, member: &str) -> Result<Shape, Failure> {
    Ok(match shape {
        Shape::G1 | Shape::G2 => {
            return Err(domain(&format!(
                "GeomAdaptor_Curve::{member}: {shape:?} has no intervals; expected C0..CN"
            )));
        }
        Shape::C0 => Shape::C1,
        Shape::C1 => Shape::C2,
        Shape::C2 => Shape::C3,
        Shape::C3 | Shape::CN => Shape::CN,
    })
}

// The curve whose intervals give the surface intervals in `direction`, if any: the
// iso at the first knot of the other direction of a B-spline, or the B-spline basis
// of the swept surface of that direction, each over the adaptor range.
fn interval_curve(
    state: &SurfaceState,
    direction: Direction,
) -> Result<Option<GeomAdaptorCurve>, Failure> {
    let (first, last) = direction.range(state);
    let curve: Handle<dyn GeomCurve> = match (&state.data, direction) {
        (SurfaceData::BSpline(data), Direction::U) => {
            let bspline: &GeomBSplineSurface = &data.surface;
            bspline.v_iso(bspline.v_knot(bspline.first_v_knot_index()?)?)?
        }
        (SurfaceData::BSpline(data), Direction::V) => {
            let bspline: &GeomBSplineSurface = &data.surface;
            bspline.u_iso(bspline.u_knot(bspline.first_u_knot_index()?)?)?
        }
        (SurfaceData::Extrusion(_), Direction::U) | (SurfaceData::Revolution(_), Direction::V) => {
            let basis = swept_basis(state)?;
            let adaptor = GeomAdaptorCurve::with_range(&basis, first, last)?;
            return Ok((adaptor.get_type()? == CurveType::BSplineCurve).then_some(adaptor));
        }
        _ => return Ok(None),
    };
    Ok(Some(GeomAdaptorCurve::with_range(&curve, first, last)?))
}

// The basis curve of the loaded Geom swept surface (natively a down cast of mySurface).
fn swept_basis(state: &SurfaceState) -> Result<Handle<dyn GeomCurve>, Failure> {
    let surface: Handle<dyn GeomSweptSurface> = state.surface.cast();
    match surface.get() {
        Some(swept) => Ok(swept.basis_curve()),
        None => Err(no_such_object(
            "GeomAdaptor_Surface: the loaded surface is not a Geom swept surface; expected one",
        )),
    }
}

/// `NbUIntervals` / `NbVIntervals`; `nb_intervals(&state, Direction::U, Shape::C2)?`.
pub(super) fn nb_intervals(
    state: &SurfaceState,
    direction: Direction,
    shape: Shape,
) -> Result<i32, Failure> {
    if let SurfaceData::Offset(data) = &state.data {
        let member = format!("Nb{}Intervals", direction.name());
        let basis_shape: Shape = offset_basis_shape(shape, &member)?;
        return match direction {
            Direction::U => data.basis_adaptor.nb_u_intervals(basis_shape),
            Direction::V => data.basis_adaptor.nb_v_intervals(basis_shape),
        };
    }
    match interval_curve(state, direction)? {
        Some(curve) => curve.nb_intervals(shape),
        None => Ok(1),
    }
}

/// `UIntervals` / `VIntervals`: the adaptor range when the direction has a single
/// interval; `intervals(&state, Direction::U, &mut out, Shape::C2)?`.
pub(super) fn intervals(
    state: &SurfaceState,
    direction: Direction,
    out: &mut Array1<'_, f64>,
    shape: Shape,
) -> Result<(), Failure> {
    if let SurfaceData::Offset(data) = &state.data {
        let member = format!("{}Intervals", direction.name());
        let basis_shape: Shape = offset_basis_shape(shape, &member)?;
        return match direction {
            Direction::U => data.basis_adaptor.u_intervals(out, basis_shape),
            Direction::V => data.basis_adaptor.v_intervals(out, basis_shape),
        };
    }
    if let Some(curve) = interval_curve(state, direction)? {
        return curve.intervals(out, shape);
    }
    let (first, last) = direction.range(state);
    let lower: i32 = out.lower();
    out.set_value(lower, first).map_err(array_bounds)?;
    out.set_value(lower + 1, last).map_err(array_bounds)
}

/// `IsUClosed` / `IsVClosed`: the surface is closed and the adaptor spans a period
/// (periodic surface) or the whole surface range; `is_closed(&state, Direction::U)?`.
pub(super) fn is_closed(state: &SurfaceState, direction: Direction) -> Result<bool, Failure> {
    let surface: &dyn GeomSurface = state.surface()?;
    let (u1, u2, v1, v2) = surface.bounds();
    let (closed, periodic, (s1, s2)) = match direction {
        Direction::U => (surface.is_u_closed()?, surface.is_u_periodic(), (u1, u2)),
        Direction::V => (surface.is_v_closed()?, surface.is_v_periodic(), (v1, v2)),
    };
    if !closed {
        return Ok(false);
    }
    let (first, last) = direction.range(state);
    if periodic {
        return Ok(((s1 - s2).abs() - (first - last).abs()).abs() < precision::p_confusion());
    }
    Ok((s1 - first).abs() < precision::p_confusion()
        && (s2 - last).abs() < precision::p_confusion())
}

/// `UPeriod` / `VPeriod`: Standard_NoSuchObject when not periodic, the surface
/// period otherwise; `period(&state, Direction::U)?`.
pub(super) fn period(state: &SurfaceState, direction: Direction) -> Result<f64, Failure> {
    let surface: &dyn GeomSurface = state.surface()?;
    let periodic: bool = match direction {
        Direction::U => surface.is_u_periodic(),
        Direction::V => surface.is_v_periodic(),
    };
    if !periodic {
        return Err(no_such_object(&format!(
            "GeomAdaptor_Surface::{}Period: the surface is not {}-periodic; expected Is{}Periodic()",
            direction.name(),
            direction.name(),
            direction.name()
        )));
    }
    match direction {
        Direction::U => surface.u_period(),
        Direction::V => surface.v_period(),
    }
}
