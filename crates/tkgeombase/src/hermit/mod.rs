// Rust translation of ModelingData/TKGeomBase/Hermit/Hermit.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hermit: the 1D reparametrization a(u) of a rational B-spline such that
//! `a(u) D(u)` equals 1 with a null derivative at both ends (for C1 concatenation).
//!
//! The native file repeats every helper for `Geom_BSplineCurve` and
//! `Geom2d_BSplineCurve`; the port shares them through [`HermitSpline`].
mod poly_test;

use poly_test::{PolyKnots, poly_test};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkmath::bsplclib::{self, CurveSpan};
use rustycad_tkmath::gp::Point2;

/// The B-spline members Hermit reads (both `Geom_BSplineCurve` and
/// `Geom2d_BSplineCurve` provide them); `curve.hermit_nb_knots()`.
pub trait HermitSpline {
    /// `Knots()`.
    fn hermit_knots(&self) -> Array1<'static, f64>;
    /// `WeightsArray()` (unit weights for a non-rational curve).
    fn hermit_weights(&self) -> Array1<'static, f64>;
    /// `Multiplicities()`.
    fn hermit_multiplicities(&self) -> Array1<'static, i32>;
    /// `Degree()`.
    fn hermit_degree(&self) -> i32;
    /// `IsPeriodic()`.
    fn hermit_is_periodic(&self) -> bool;
    /// `FirstUKnotIndex()` and `LastUKnotIndex()`.
    fn hermit_knot_range(&self) -> Result<(i32, i32), Failure>;
    /// `LocateU(U, 0.0, I1, I2)`.
    fn hermit_locate_u(&self, u: f64) -> Result<(i32, i32), Failure>;
    /// `NbKnots()`.
    fn hermit_nb_knots(&self) -> i32;
    /// `Knot(Index)`.
    fn hermit_knot(&self, index: i32) -> Result<f64, Failure>;
}

macro_rules! hermit_spline {
    ($curve:ty) => {
        impl HermitSpline for $curve {
            fn hermit_knots(&self) -> Array1<'static, f64> {
                self.knots()
            }
            fn hermit_weights(&self) -> Array1<'static, f64> {
                self.weights_array(|weights| weights.clone())
            }
            fn hermit_multiplicities(&self) -> Array1<'static, i32> {
                self.multiplicities()
            }
            fn hermit_degree(&self) -> i32 {
                self.degree()
            }
            fn hermit_is_periodic(&self) -> bool {
                self.is_periodic()
            }
            fn hermit_knot_range(&self) -> Result<(i32, i32), Failure> {
                Ok((self.first_u_knot_index()?, self.last_u_knot_index()?))
            }
            fn hermit_locate_u(&self, u: f64) -> Result<(i32, i32), Failure> {
                self.locate_u(u, 0.0, false)
            }
            fn hermit_nb_knots(&self) -> i32 {
                self.nb_knots()
            }
            fn hermit_knot(&self, index: i32) -> Result<f64, Failure> {
                self.knot(index)
            }
        }
    };
}
hermit_spline!(GeomBSplineCurve);
hermit_spline!(Geom2dBSplineCurve);

// HermiteCoeff: 1/w(0), -w'(0)/w(0)^2, -w'(1)/w(1)^2, 1/w(1) of the weight function
// w(u) = sum N_i(u) w_i on the knots mapped onto [0, 1].
fn hermite_coeff<S: HermitSpline>(bs: &S) -> Result<[f64; 4], Failure> {
    let mut knots: Array1<'static, f64> = bs.hermit_knots();
    let weights: Array1<'static, f64> = bs.hermit_weights();
    let mults: Array1<'static, i32> = bs.hermit_multiplicities();
    bsplclib::reparametrize(0.0, 1.0, &mut knots)?;
    let (first, last) = bs.hermit_knot_range()?;
    let span = |u: f64, index: i32| CurveSpan {
        u,
        index,
        degree: bs.hermit_degree(),
        periodic: bs.hermit_is_periodic(),
        knots: &knots,
        mults: Some(&mults),
    };
    let (denom0, deriv0) = bsplclib::d1_scalar(&span(0.0, first), &weights, None)?;
    let (denom1, deriv1) = bsplclib::d1_scalar(&span(1.0, last - 1), &weights, None)?;
    Ok([
        1.0 / denom0,
        -deriv0 / (denom0 * denom0),
        -deriv1 / (denom1 * denom1),
        1.0 / denom1,
    ])
}

// The cubic Bezier poles (0, Herm0), (0, Herm0 + Herm1/3), (0, Herm3 - Herm2/3),
// (0, Herm3) of the Hermite polynomial, as a Geom2d B-spline on [0, 1].
fn hermite_curve(herm: [f64; 4]) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let ordinates: [f64; 4] = [
        herm[0],
        herm[0] + herm[1] / 3.0,
        herm[3] - herm[2] / 3.0,
        herm[3],
    ];
    let mut poles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    for (i, y) in ordinates.iter().enumerate() {
        poles[i as i32 + 1] = Point2::new(0.0, *y);
    }
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    (knots[1], knots[2]) = (0.0, 1.0);
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    (mults[1], mults[2]) = (4, 4);
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &poles, &knots, &mults, 3, false,
    )?))
}

// The Hermite coefficients of a modified a(u): the end ordinates and slopes of its
// first and last pole pairs.
fn curve_coeff(curve: &Geom2dBSplineCurve) -> Result<[f64; 4], Failure> {
    let n: i32 = curve.nb_poles();
    let (p1, p2) = (curve.pole(1)?.y(), curve.pole(2)?.y());
    let (pn1, pn) = (curve.pole(n - 1)?.y(), curve.pole(n)?.y());
    Ok([p1, 3.0 * (p2 - p1), 3.0 * (pn - pn1), pn])
}

// InsertKnots: none when U4 = 0, one when U4 = U5, two otherwise (U5 = 1 skipped).
fn insert_knots(curve: &Geom2dBSplineCurve, u4: f64, u5: f64) -> Result<(), Failure> {
    if u4 != 0.0 {
        curve.insert_knot(u4, 1, 0.0)?;
    }
    if (u5 != 1.0) && (u5 != u4) {
        curve.insert_knot(u5, 1, 0.0)?;
    }
    Ok(())
}

// MovePoles: raise the unconstrained poles (3 to NbPoles - 2) to the first pole level.
fn move_poles(curve: &Geom2dBSplineCurve) -> Result<(), Failure> {
    for i in 3..=(curve.nb_poles() - 2) {
        let x: f64 = curve.pole(i)?.x();
        curve.set_pole(i, Point2::new(x, curve.pole(1)?.y()))?;
    }
    Ok(())
}

/// The shared result of the positivity and tolerance searches: the initial
/// Hermite polynomial, the modified one (BS2) and the inserted knots.
struct Search {
    herm: [f64; 4],
    bs2: Handle<Geom2dBSplineCurve>,
    upos: PolyKnots,
    utol: PolyKnots,
    uint1: f64,
    uint2: f64,
}

// The common body of Solution and Solutionbis, up to the knot tolerance check.
fn search<S: HermitSpline>(bs: &S, tol_poles: f64, tol_knots: f64) -> Result<Search, Failure> {
    let herm: [f64; 4] = hermite_coeff(bs)?;
    let bs2 = hermite_curve(herm)?;
    let mut zeroboucle: i32 = 0;
    let conf: f64 = precision::confusion();
    let upos: PolyKnots = poly_test(herm, bs, &mut zeroboucle, conf, (1.0, 0.0))?;
    insert_knots(&bs2, upos.u4, upos.u5)?;
    let (ux, uy): (f64, f64) = positivity_bounds(&upos);
    let mut boucle: i32 = 1;
    let mut utol = poly_test(curve_coeff(&bs2)?, bs, &mut boucle, tol_poles, (ux, uy))?;
    insert_knots(&bs2, utol.u4, utol.u5)?;
    let (mut uint1, mut uint2): (f64, f64) = (0.0, 1.0);
    if boucle == 2 {
        let coeff: [f64; 4] = curve_coeff(&bs2)?;
        let start: f64 = if utol.u4 == 0.0 {
            uint2 = utol.u5;
            uint2
        } else {
            uint1 = utol.u4;
            uint1
        };
        utol = poly_test(coeff, bs, &mut boucle, tol_poles, (start, 0.0))?;
        insert_knots(&bs2, utol.u4, utol.u5)?;
    }
    let knot2: f64 = bs2.knot(2)?;
    let knot_before_last: f64 = bs2.knot(bs2.nb_knots() - 1)?;
    if (knot2 < tol_knots) || (knot_before_last > (1.0 - tol_knots)) {
        return Err(Failure::with_type(
            "Standard_DimensionError",
            "Hermit Impossible Tolerance",
            "",
        ));
    }
    Ok(Search {
        herm,
        bs2,
        upos,
        utol,
        uint1,
        uint2,
    })
}

// Ux, Uy: the positivity knots in order, one of them when only one was inserted.
fn positivity_bounds(upos: &PolyKnots) -> (f64, f64) {
    if upos.u4 == 0.0 {
        return (upos.u5, upos.u5);
    }
    if upos.u5 != 1.0 {
        return (upos.u4.min(upos.u5), upos.u4.max(upos.u5));
    }
    (upos.u4, upos.u4)
}

impl Search {
    // No knot inserted on the right (U5 = 1 everywhere) / on the left (U4 = 0).
    fn right_untouched(&self) -> bool {
        (self.upos.u5 == 1.0) && (self.utol.u5 == 1.0) && (self.uint2 == 1.0)
    }
    fn left_untouched(&self) -> bool {
        (self.upos.u4 == 0.0) && (self.utol.u4 == 0.0) && (self.uint1 == 0.0)
    }
}

// Hermit::Solution for either curve kind.
fn solution<S: HermitSpline>(
    bs: &S,
    tol_poles: f64,
    tol_knots: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let found: Search = search(bs, tol_poles, tol_knots)?;
    let bs1 = hermite_curve(found.herm)?;
    let first_inner: f64 = found.bs2.knot(2)?;
    let last_inner: f64 = found.bs2.knot(found.bs2.nb_knots() - 1)?;
    if found.right_untouched() {
        insert_knots(&bs1, first_inner, 1.0)?;
    } else if found.left_untouched() {
        insert_knots(&bs1, last_inner, 1.0)?;
    } else {
        insert_knots(&bs1, last_inner, first_inner)?;
    }
    move_poles(&bs1)?;
    Ok(bs1)
}

/// `Hermit::Solution(BS, TolPoles = 1e-6, TolKnots = 1e-6)` for a 3D B-spline:
/// the spline a(u) to multiply `bs` by; Standard_DimensionError ("Hermit
/// Impossible Tolerance") when the poles or the knots cannot meet the tolerances;
/// `hermit::solution_3d(&curve, 1e-6, 1e-6)?`.
pub fn solution_3d(
    bs: &Handle<GeomBSplineCurve>,
    tol_poles: f64,
    tol_knots: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    solution(live(bs)?, tol_poles, tol_knots)
}

/// `Hermit::Solution(BS, TolPoles = 1e-6, TolKnots = 1e-6)` for a 2D B-spline;
/// `hermit::solution_2d(&curve, 1e-6, 1e-6)?`.
pub fn solution_2d(
    bs: &Handle<Geom2dBSplineCurve>,
    tol_poles: f64,
    tol_knots: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    solution(live(bs)?, tol_poles, tol_knots)
}

/// `Hermit::Solutionbis(BS, Knotmin, Knotmax, TolPoles = 1e-6, TolKnots = 1e-6)`:
/// the knots a(u) would get, written into `knot_min` (the second knot of the
/// modified polynomial) and `knot_max` (its last inner knot) only where the
/// native code writes them; `hermit::solutionbis(&curve, &mut kmin, &mut kmax, 1e-6, 1e-6)?`.
pub fn solutionbis(
    bs: &Handle<GeomBSplineCurve>,
    knot_min: &mut f64,
    knot_max: &mut f64,
    tol_poles: f64,
    tol_knots: f64,
) -> Result<(), Failure> {
    let found: Search = search(live(bs)?, tol_poles, tol_knots)?;
    let first_inner: f64 = found.bs2.knot(2)?;
    let last_inner: f64 = found.bs2.knot(found.bs2.nb_knots() - 1)?;
    if found.right_untouched() {
        *knot_min = first_inner;
    } else if found.left_untouched() {
        *knot_max = last_inner;
    } else {
        *knot_min = first_inner;
        *knot_max = last_inner;
    }
    Ok(())
}

// The curve of a handle; Standard_NullObject for a null one (a native null
// dereference).
fn live<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &Handle<T>,
) -> Result<&T, Failure> {
    crate::lprop::curve_access::live(handle)
}
