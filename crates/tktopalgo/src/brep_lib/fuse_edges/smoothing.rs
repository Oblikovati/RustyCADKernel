// Rust translation of BCSmoothing and MakeClosedCurve (BRepLib_FuseEdges.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A closed fused B-spline made periodic, its origin moved to the chain vertex, and
//! its continuity restored when the move made it C0.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::extrema::LocateExtPC;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;
use std::cmp::Ordering::Greater;

/// The static `BCSmoothing`: up to 5 passes removing the inner knots of multiplicity
/// above `degree - cont` (within `tol`) and inserting knots in the spans around those
/// that stay; after 5 passes, a last run at the tolerance 1e10.
fn bc_smoothing(c: &GeomBSplineCurve, cont: i32, tol: f64) -> Result<(), Failure> {
    const NB_ITER: i32 = 5;
    let mut iter: i32 = 1;
    while iter <= NB_ITER {
        let m: i32 = c.degree() - cont;
        if m < 1 {
            return Ok(());
        }
        let (mults, knots) = (c.multiplicities(), c.knots());
        let mut kept: Vec<i32> = Vec::new();
        for i in 2..c.nb_knots() {
            if mults[i] > m && !c.remove_knot(i, m, tol)? {
                kept.push(i);
            }
        }
        if kept.is_empty() {
            return Ok(());
        }
        for knot in knots_to_insert(&kept, |i| knots[i]) {
            c.insert_knot(knot, m, 1.0e-3, true)?;
        }
        iter += 1;
    }
    bc_smoothing(c, cont, 1.0e10)
}

/// "Prepare knots for inserting": the middles of the spans around each kept knot, each
/// more than 1e-3 away from the previous one (starting at the first knot).
fn knots_to_insert(kept: &[i32], knot: impl Fn(i32) -> f64) -> Vec<f64> {
    let mut inserted: Vec<f64> = Vec::new();
    let mut last_knot: f64 = knot(1);
    for &index in kept {
        for k in [
            0.5 * (knot(index) + knot(index - 1)),
            0.5 * (knot(index + 1) + knot(index)),
        ] {
            if (k - last_knot).abs() > 1.0e-3 {
                inserted.push(k);
                last_knot = k;
            }
        }
    }
    inserted
}

/// The static `MakeClosedCurve`: the B-spline made periodic with its origin at the
/// first knot on `pf` (or the projection of `pf` from the nearest knot), segmented to
/// its period and smoothed when that made it C0; `f` and `l` become its range.
pub(super) fn make_closed_curve(
    c: &mut Handle<dyn GeomCurve>,
    pf: Point3,
    f: &mut f64,
    l: &mut f64,
) -> Result<(), Failure> {
    let bc: Handle<GeomBSplineCurve> = c.cast::<GeomBSplineCurve>();
    let b = bc.get().expect("a B-spline curve");
    let cont = b.continuity();
    // Find new origin
    b.set_periodic()?;
    let porig: f64 = new_origin(&bc, pf, (*f, *l))?;
    b.set_origin_parameter(porig, precision::p_confusion())?;
    let (first, last) = (b.first_parameter(), b.last_parameter());
    b.segment(first, last, precision::p_confusion())?;
    if cont > Continuity::C0 && b.continuity() == Continuity::C0 {
        bc_smoothing(b, 1, precision::confusion())?;
    }
    *c = bc.cast();
    (*f, *l) = (b.first_parameter(), b.last_parameter());
    Ok(())
}

/// The origin: the first knot within Precision::Confusion() of `pf`, else the
/// projection of `pf` located from the nearest knot (Standard_ConstructionError when it
/// fails).
fn new_origin(
    bc: &Handle<GeomBSplineCurve>,
    pf: Point3,
    (f, l): (f64, f64),
) -> Result<f64, Failure> {
    let b = bc.get().expect("a B-spline curve");
    let eps: f64 = precision::confusion() * precision::confusion();
    let (mut porig, mut dmin, mut pmin) = (2. * l, 1.0e100, f);
    for k in b.first_u_knot_index()?..=b.last_u_knot_index()? {
        let knot: f64 = b.knot(k)?;
        let d: f64 = pf.square_distance(&b.value(knot)?);
        if d.partial_cmp(&eps) != Some(Greater) {
            porig = knot;
            break;
        }
        if d < dmin {
            (pmin, dmin) = (knot, d);
        }
    }
    if porig.partial_cmp(&l) != Some(Greater) {
        return Ok(porig);
    }
    // try to project
    let adaptor = GeomAdaptorCurve::with_curve(&bc.clone().cast())?;
    let projection = LocateExtPC::with_curve(pf, &adaptor, pmin, precision::p_confusion())?;
    if !projection.is_done() {
        return Err(Failure::with_message(
            "Standard_ConstructionError",
            "FuseEdges : Projection failed for closed curve",
        ));
    }
    Ok(projection.point()?.parameter())
}
