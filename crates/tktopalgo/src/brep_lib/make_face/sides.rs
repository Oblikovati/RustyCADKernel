// Rust translation of the sides of BRepLib_MakeFace::Init(S, UMin, UMax, VMin, VMax,
// TolDegen) (BRepLib_MakeFace.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sides and corners of the parameter box of the bounded `BRepLib_MakeFace::Init`.
use crate::brep_lib::degenerated_curve::degenerated_tolerance;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, SurfaceBounds};
use rustycad_tkmath::gp::{Direction2, Point2, StandardDirection2};

/// One side of the parameter box (UMin, UMax, VMin or VMax): whether it is infinite,
/// its iso curve, whether that curve is degenerated, its tolerance, its p-curve line.
pub(super) struct Side {
    pub(super) infinite: bool,
    pub(super) curve: Handle<dyn GeomCurve>,
    pub(super) degenerated: bool,
    pub(super) tol: f64,
    pub(super) line: Handle<dyn Geom2dCurve>,
}

/// The four sides, in the order UMin, UMax, VMin, VMax.
pub(super) const UMIN: usize = 0;
pub(super) const UMAX: usize = 1;
pub(super) const VMIN: usize = 2;
pub(super) const VMAX: usize = 3;

/// The iso curve side of the box at `param` (U iso for the U sides), infinite sides
/// without curve; the degeneration is decided within `max_tol`.
fn side(
    s: &dyn GeomSurface,
    u_side: bool,
    param: f64,
    infinite: bool,
    max_tol: f64,
) -> Result<Side, Failure> {
    let (origin, direction) = if u_side {
        (
            Point2::new(param, 0.),
            Direction2::from_standard(StandardDirection2::Y),
        )
    } else {
        (
            Point2::new(0., param),
            Direction2::from_standard(StandardDirection2::X),
        )
    };
    let mut side = Side {
        infinite,
        curve: Handle::default(),
        degenerated: false,
        tol: precision::confusion(),
        line: Handle::default(),
    };
    if infinite {
        return Ok(side);
    }
    side.curve = if u_side {
        s.u_iso(param)?
    } else {
        s.v_iso(param)?
    };
    if let Some(tol) = degenerated_tolerance(&side.curve, max_tol)? {
        (side.degenerated, side.tol) = (true, tol);
    }
    side.line = Handle::new(Geom2dLine::from_point(origin, direction)).cast();
    Ok(side)
}

/// The four sides of the box, "compute infinite flags" and "compute 3d curves and
/// degenerate flag".
pub(super) fn sides(
    s: &dyn GeomSurface,
    (umin, umax, vmin, vmax): SurfaceBounds,
    max_tol: f64,
) -> Result<[Side; 4], Failure> {
    Ok([
        side(
            s,
            true,
            umin,
            precision::is_negative_infinite(umin),
            max_tol,
        )?,
        side(
            s,
            true,
            umax,
            precision::is_positive_infinite(umax),
            max_tol,
        )?,
        side(
            s,
            false,
            vmin,
            precision::is_negative_infinite(vmin),
            max_tol,
        )?,
        side(
            s,
            false,
            vmax,
            precision::is_positive_infinite(vmax),
            max_tol,
        )?,
    ])
}

/// The closed flags: a closed direction whose bounds are the natural ones.
pub(super) fn closed_flags(
    s: &dyn GeomSurface,
    natural: SurfaceBounds,
    bounds: SurfaceBounds,
) -> Result<(bool, bool), Failure> {
    let epsilon: f64 = precision::p_confusion();
    let (nu1, nu2, nv1, nv2) = natural;
    let (umin, umax, vmin, vmax) = bounds;
    let uclosed: bool =
        s.is_u_closed()? && (umin - nu1).abs() < epsilon && (umax - nu2).abs() < epsilon;
    let vclosed: bool =
        s.is_v_closed()? && (vmin - nv1).abs() < epsilon && (vmax - nv2).abs() < epsilon;
    Ok((uclosed, vclosed))
}

/// "compute vertices": V00, V10, V11, V01 at the finite corners, then shared by the
/// closed directions and the degenerated sides, in the native order.
pub(super) fn corners(
    s: &dyn GeomSurface,
    sides: &[Side; 4],
    bounds: SurfaceBounds,
    closed: (bool, bool),
) -> Result<[Vertex; 4], Failure> {
    let (umin, umax, vmin, vmax) = bounds;
    let [mut v00, mut v10, mut v11, mut v01] = [
        corner(s, sides, (UMIN, VMIN), (umin, vmin))?,
        corner(s, sides, (UMAX, VMIN), (umax, vmin))?,
        corner(s, sides, (UMAX, VMAX), (umax, vmax))?,
        corner(s, sides, (UMIN, VMAX), (umin, vmax))?,
    ];
    if closed.0 {
        (v10, v11) = (v00.clone(), v01.clone());
    }
    if closed.1 {
        (v01, v11) = (v00.clone(), v10.clone());
    }
    if sides[UMIN].degenerated {
        v00 = v01.clone();
    }
    if sides[UMAX].degenerated {
        v10 = v11.clone();
    }
    if sides[VMIN].degenerated {
        v00 = v10.clone();
    }
    if sides[VMAX].degenerated {
        v01 = v11.clone();
    }
    Ok([v00, v10, v11, v01])
}

/// The vertex at a corner of two finite sides (null otherwise), with the larger of
/// their tolerances.
fn corner(
    s: &dyn GeomSurface,
    sides: &[Side; 4],
    (iu, iv): (usize, usize),
    (u, v): (f64, f64),
) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    if sides[iu].infinite || sides[iv].infinite {
        return Ok(vertex);
    }
    let tolerance: f64 = std_max(sides[iu].tol, sides[iv].tol);
    BRepBuilder::default().make_vertex_at(&mut vertex, s.value(u, v)?, tolerance)?;
    Ok(vertex)
}
