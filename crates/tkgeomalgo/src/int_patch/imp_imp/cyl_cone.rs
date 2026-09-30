// Rust translation of IntCyCo and ExploreCurve of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cylinder against a cone.
use super::super::gline::IntPatchGLine;
use super::bounds::{CurveOptions, analytic_line, analytic_points};
use super::plane::{Lines, Points, push, tangent_point};
use super::transitions::{by_sign_tol, dot_cross};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{
    Curve as IntAnaCurve, IntQuadQuad, QuadQuadGeo, Quadric, ResultType,
};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Cone3, Cylinder3, Point3};

type Quads<'a> = (&'a IntSurfQuadric, &'a IntSurfQuadric);

/// `IntCyCo(Quad1, Quad2, Tol, Reversed, Empty, Multpoint, slin, spnt)`: the cylinder
/// and the cone: a tangent point (with the parameters of the first quadric twice, as
/// natively), two circles, or the analytic curves split at the apex.
#[allow(clippy::too_many_arguments)]
pub(super) fn int_cyco(
    quads: Quads<'_>,
    tol: f64,
    reversed: bool,
    empty: &mut bool,
    multpoint: &mut bool,
    slin: &mut Lines,
    spnt: &mut Points,
) -> Result<bool, Failure> {
    let (cy, co): (Cylinder3, Cone3) = if reversed {
        (quads.1.cylinder()?, quads.0.cone()?)
    } else {
        (quads.0.cylinder()?, quads.1.cone()?)
    };
    let mut inter = QuadQuadGeo::new();
    inter.perform_cylinder_cone(cy, co, tol)?;
    if !inter.is_done() {
        return Ok(false);
    }
    let typint: ResultType = inter.type_inter()?;
    *empty = false;
    match typint {
        ResultType::Empty => *empty = true,
        ResultType::Point => spnt.push(tangent_point(
            inter.point(1)?,
            (quads.0, quads.0),
            tol,
            true,
        )),
        ResultType::Circle => {
            for j in 1..=2 {
                let cirsol: Circle3 = inter.circle(j)?;
                let (ptref, tgt) = elclib::d1(0.0, &cirsol);
                let qwe: f64 = dot_cross(tgt, quads.0, quads.1, ptref)?;
                push(
                    slin,
                    IntPatchGLine::from_circle(&cirsol, false, by_sign_tol(qwe, 0.00000001)),
                );
            }
        }
        ResultType::NoGeometricSolution => {
            let anaint = IntQuadQuad::cylinder(cy, &Quadric::from_cone(co), tol)?;
            return split_curves(&anaint, &co, quads, tol, (empty, multpoint), (slin, spnt));
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// The points and the curves (each split at the apex of the cone) of the analytic
/// intersection.
fn split_curves(
    anaint: &IntQuadQuad,
    co: &Cone3,
    quads: Quads<'_>,
    tol: f64,
    (empty, multpoint): (&mut bool, &mut bool),
    (slin, spnt): (&mut Lines, &mut Points),
) -> Result<bool, Failure> {
    if !anaint.is_done() {
        return Ok(false);
    }
    if anaint.nb_pnt()? == 0 && anaint.nb_curve()? == 0 {
        *empty = true;
        return Ok(true);
    }
    analytic_points(anaint, quads, tol, spnt)?;
    let opts = CurveOptions {
        check_tangent: false,
        tol_trans: 0.00000001,
        open_ends: true,
        eval_on_fail: true,
    };
    let mut ends: (Point3, Point3) = (Point3::default(), Point3::default());
    for i in 1..=anaint.nb_curve()? {
        let mut pieces: Vec<IntAnaCurve> = Vec::new();
        explore_curve(co, anaint.curve(i)?, 10.0 * tol, &mut pieces)?;
        for curvsol in &pieces {
            analytic_line(curvsol, quads, &opts, (slin, multpoint), tol, &mut ends)?;
        }
    }
    Ok(true)
}

/// `ExploreCurve(theCo, theCrv, theTol, theLC)`: splits the curve at its parameters
/// where it passes through the apex of the cone (within `tol`); returns whether it was
/// split (otherwise the list holds the curve itself).
pub(super) fn explore_curve(
    co: &Cone3,
    crv: &IntAnaCurve,
    tol: f64,
    lc: &mut Vec<IntAnaCurve>,
) -> Result<bool, Failure> {
    let sq_tol: f64 = tol * tol;
    let papx: Point3 = co.apex();
    let (mut t1, t2) = crv.domain()?;
    lc.clear();
    let mut params: List<f64> = List::default();
    crv.find_parameter(papx, &mut params)?;
    if params.is_empty() {
        lc.push(*crv);
        return Ok(false);
    }
    for &p in params.iter() {
        let mut prm: f64 = p;
        if (prm - t1) < precision::p_confusion() {
            continue;
        }
        let is_last: bool = (t2 - prm) < precision::p_confusion();
        if is_last {
            prm = t2;
        }
        if crv.value(prm)?.square_distance(&papx) < sq_tol {
            let mut c1: IntAnaCurve = *crv;
            c1.set_domain(t1, prm)?;
            t1 = prm;
            lc.push(c1);
        }
        if is_last {
            break;
        }
    }
    if lc.is_empty() {
        lc.push(*crv);
        return Ok(false);
    }
    if (t2 - t1) > precision::p_confusion() {
        let mut c1: IntAnaCurve = *crv;
        c1.set_domain(t1, t2)?;
        lc.push(c1);
    }
    Ok(true)
}
