// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Circuit.cxx (IsSharpCorner, InitOpen, InsertCorner and the static CrossProd) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corners of the lines: a point item is inserted at each sharp corner (a
//! salient break for the side of the computation) and at the ends of open lines.
use super::{CircuitState, trimmed_item};
use crate::mat2d::{Mat2dLine, seq};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dOffsetCurve};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg2d::geom2d::{
    Geom2dBoundedCurve, Geom2dCartesianPoint, Geom2dCurve, Geom2dGeometry, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Direction2, Point2};

/// `CrossProd(Geom1, Geom2, DotProd)`: the cross and dot products of the tangent
/// directions at the end of `geom1` and at the start of `geom2`, both curves.
pub(super) fn cross_prod(
    geom1: &Handle<dyn Geom2dGeometry>,
    geom2: &Handle<dyn Geom2dGeometry>,
) -> Result<(f64, f64), Failure> {
    let curve1: Handle<Geom2dTrimmedCurve> = trimmed_item(geom1)?;
    let dir1 = Direction2::from_vector(curve1.dn(curve1.last_parameter(), 1)?)?;
    let curve2: Handle<Geom2dTrimmedCurve> = trimmed_item(geom2)?;
    let dir2 = Direction2::from_vector(curve2.dn(curve2.first_parameter(), 1)?)?;
    Ok((dir1.crossed(dir2), dir1.dot(dir2)))
}

// A point item at `p`.
fn point_item(p: Point2) -> Handle<dyn Geom2dGeometry> {
    Handle::new(Geom2dCartesianPoint::from_pnt2d(p)).cast()
}

// The tangent directions `nb_test * DU` inside the end of `c1` and the start of `c2`:
// their cross and dot products.
fn inner_products(
    c1: &Geom2dTrimmedCurve,
    c2: &Geom2dTrimmedCurve,
    nb_test: i32,
) -> Result<(f64, f64), Failure> {
    let du: f64 = precision::confusion();
    let u1: f64 = c1.last_parameter() - f64::from(nb_test) * du;
    let u2: f64 = c2.first_parameter() + f64::from(nb_test) * du;
    let dir1 = Direction2::from_vector(c1.dn(u1, 1)?)?;
    let dir2 = Direction2::from_vector(c2.dn(u2, 1)?)?;
    Ok((dir1.crossed(dir2), dir1.dot(dir2)))
}

impl CircuitState {
    /// `IsSharpCorner(Geom1, Geom2, Direction)`: true when the common point of the two
    /// curves is a salient break for the side `direction` (KHROMOV, Oct 2002: the
    /// angular tolerance is 1e-8, as in InitOpen).
    pub(super) fn is_sharp_corner(
        &self,
        geom1: &Handle<dyn Geom2dGeometry>,
        geom2: &Handle<dyn Geom2dGeometry>,
        direction: f64,
    ) -> Result<bool, Failure> {
        let (pro_vec, dot_prod) = cross_prod(geom1, geom2)?;
        let c1: Handle<Geom2dTrimmedCurve> = geom1.cast();
        let c2: Handle<Geom2dTrimmedCurve> = geom2.cast();
        match self.join_type {
            JoinType::Arc => sharp_for_arc(&c1, &c2, (pro_vec, dot_prod), direction),
            JoinType::Intersection => {
                sharp_for_intersection(&c1, &c2, (pro_vec, dot_prod), direction)
            }
            JoinType::Tangent => Ok(false),
        }
    }

    /// `InitOpen(Line)`: the ends of an open line become point items, and a point is
    /// inserted at each break between two of its curves.
    pub(super) fn init_open(&self, line: &mut Mat2dLine) -> Result<(), Failure> {
        let first = trimmed_item(&seq::first(line)?)?;
        seq::insert_before(line, 1, point_item(first.start_point()?))?;
        let last = trimmed_item(&seq::last(line)?)?;
        line.append(point_item(last.end_point()?));
        let mut i: i32 = 2;
        while i <= line.length() - 2 {
            let (cross, dot_prod) = cross_prod(&seq::at(line, i)?, &seq::at(line, i + 1)?)?;
            if cross.abs() > 1.0e-8 || dot_prod < 0. {
                let curve = trimmed_item(&seq::at(line, i)?)?;
                seq::insert_after(line, i, point_item(curve.end_point()?))?;
                i += 1;
            }
            i += 1;
        }
        Ok(())
    }

    /// `InsertCorner(Line)`: a point item is inserted at each sharp corner of a closed
    /// line (between its last and first items too).
    pub(super) fn insert_corner(&self, line: &mut Mat2dLine) -> Result<(), Failure> {
        let mut i: i32 = 1;
        while i <= line.length() {
            let isuiv: i32 = if i == line.length() { 1 } else { i + 1 };
            if self.is_sharp_corner(&seq::at(line, i)?, &seq::at(line, isuiv)?, self.direction)? {
                let curve = trimmed_item(&seq::at(line, isuiv)?)?;
                seq::insert_after(line, i, point_item(curve.start_point()?))?;
                i += 1;
            }
            i += 1;
        }
        Ok(())
    }
}

// The GeomAbs_Arc test: up to 10 tangents inside the curves decide a salient or a
// re-entrant break; a flat corner is not sharp; a turn back is salient when the
// parallels of the two curves on the side of the computation do not meet.
fn sharp_for_arc(
    c1: &Handle<Geom2dTrimmedCurve>,
    c2: &Handle<Geom2dTrimmedCurve>,
    (mut pro_vec, mut dot_prod): (f64, f64),
    direction: f64,
) -> Result<bool, Failure> {
    let tol_ang: f64 = 1.0e-8;
    let mut nb_test: i32 = 1;
    while nb_test <= 10 {
        if pro_vec * direction < -tol_ang {
            return Ok(true); // Salient.
        }
        if pro_vec * direction > tol_ang {
            return Ok(false); // Re-entrant.
        }
        if dot_prod > 0. {
            return Ok(false); // Flat.
        }
        (pro_vec, dot_prod) = inner_products(c1, c2, nb_test)?;
        nb_test += 1;
    }
    turn_back_is_salient(c1, c2, direction)
}

// A turn back: the parallels of the halves of the curves at the corner, at a tenth
// of the distance from the corner to the middle of the curves, on the side of the
// computation.
fn turn_back_is_salient(
    c1: &Handle<Geom2dTrimmedCurve>,
    c2: &Handle<Geom2dTrimmedCurve>,
    direction: f64,
) -> Result<bool, Failure> {
    let tol: f64 = precision::confusion();
    let mil_c1: f64 = (c1.last_parameter() + c1.first_parameter()) * 0.5;
    let mil_c2: f64 = (c2.last_parameter() + c2.first_parameter()) * 0.5;
    let p: Point2 = c1.value(c1.last_parameter())?;
    let p1: Point2 = c1.value(mil_c1)?;
    let p2: Point2 = c2.value(mil_c2)?;
    let mut d: f64 = std_min(p1.distance(p), p2.distance(p));
    d /= 10.;
    if direction < 0. {
        d = -d;
    }
    let hc1: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_curve(
        &c1.cast::<dyn Geom2dCurve>(),
    )?)
    .cast();
    let hc2: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_curve(
        &c2.cast::<dyn Geom2dCurve>(),
    )?)
    .cast();
    let oc1 = Adaptor2dOffsetCurve::with_bounds(&hc1, d, mil_c1, c1.last_parameter());
    let oc2 = Adaptor2dOffsetCurve::with_bounds(&hc2, d, c2.first_parameter(), mil_c2);
    let mut intersect = GInter::new();
    intersect.perform_between(&oc1, &oc2, tol, tol)?;
    Ok(!intersect.is_done() || intersect.is_empty()?)
}

// The GeomAbs_Intersection test: only a flat turn back can be salient, decided by
// up to 10 tangents inside the curves.
fn sharp_for_intersection(
    c1: &Handle<Geom2dTrimmedCurve>,
    c2: &Handle<Geom2dTrimmedCurve>,
    (pro_vec, dot_prod): (f64, f64),
    direction: f64,
) -> Result<bool, Failure> {
    let tol_ang: f64 = 1.0e-8;
    if !(pro_vec.abs() <= tol_ang && dot_prod < 0.) {
        return Ok(false);
    }
    for nb_test in 1..=10 {
        let (pro_vec, _) = inner_products(c1, c2, nb_test)?;
        if pro_vec * direction < -tol_ang {
            return Ok(true); // Salient.
        }
        if pro_vec * direction > tol_ang {
            return Ok(false); // Re-entrant.
        }
    }
    Ok(false)
}
