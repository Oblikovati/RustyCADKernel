// Rust translation of CyCyAnalyticalIntersect of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The analytic intersections of two cylinders: a point, lines or two ellipses.
use super::super::super::gline::IntPatchGLine;
use super::super::super::line::IntPatchLineTransitions;
use super::super::super::point::IntPatchPoint;
use super::super::plane::{Lines, Points, push, tangent_point};
use super::super::transitions::{by_sign_tol, dot_cross};
use crate::int_surf::{IntSurfQuadric, IntSurfSituation};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{QuadQuadGeo, ResultType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Cylinder3, Direction3, Ellipse3, Line3, Point3, Vector3};
use std::f64::consts::PI;

type Quads<'a> = (&'a IntSurfQuadric, &'a IntSurfQuadric);

/// `CyCyAnalyticalIntersect(Quad1, Quad2, theInter, Tol, Empty, Same, Multpoint, slin,
/// spnt)`: the analytic result of two cylinders; false for a circle or no geometric
/// solution (the walking is used then), Standard_Failure for a parabola or hyperbola.
#[allow(clippy::too_many_arguments)]
pub(in super::super) fn cycy_analytical_intersect(
    quads: Quads<'_>,
    inter: &QuadQuadGeo,
    tol: f64,
    empty: &mut bool,
    same: &mut bool,
    multpoint: &mut bool,
    slin: &mut Lines,
    spnt: &mut Points,
) -> Result<bool, Failure> {
    let (cy1, cy2): (Cylinder3, Cylinder3) = (quads.0.cylinder()?, quads.1.cylinder()?);
    let typint: ResultType = inter.type_inter()?;
    let nb_sol: i32 = inter.nb_solutions()?;
    *empty = false;
    *same = false;
    match typint {
        ResultType::Empty => *empty = true,
        ResultType::Same => *same = true,
        ResultType::Point => spnt.push(tangent_point(inter.point(1)?, quads, tol, true)),
        ResultType::Line if nb_sol == 1 => {
            push(slin, tangency_line(&inter.line(1)?, (&cy1, &cy2), quads)?)
        }
        ResultType::Line => {
            for i in 1..=nb_sol {
                let linsol: Line3 = inter.line(i)?;
                let lsd = Vector3::from(linsol.direction());
                // Theoretically, qwe = +/- 1.0.
                let qwe: f64 = dot_cross(lsd, quads.0, quads.1, linsol.location())?;
                push(
                    slin,
                    IntPatchGLine::from_line(&linsol, false, by_sign_tol(qwe, 0.00000001))?,
                );
            }
        }
        ResultType::Ellipse => {
            *multpoint = true;
            two_ellipses(inter, quads, tol, slin)?;
        }
        ResultType::Parabola | ResultType::Hyperbola => {
            return Err(Failure::with_message(
                "Standard_Failure",
                &format!(
                    "IntCyCy(): Wrong intersection type! ({typint:?}; expected a point, lines or ellipses)"
                ),
            ));
        }
        // A circle is useful with trimmed surfaces (tangent by their bases).
        _ => return Ok(false),
    }
    Ok(true)
}

/// The situations of the tangency line of two cylinders from the radius vectors toward
/// their axes; for opposite centers of curvature, the normal and radius vector of the
/// first cylinder give the situation of the second one.
fn tangency_line(
    linsol: &Line3,
    (cy1, cy2): (&Cylinder3, &Cylinder3),
    quads: Quads<'_>,
) -> Result<IntPatchGLine, Failure> {
    use IntSurfSituation::{Inside, Outside};
    let ptref: Point3 = linsol.location();
    // Radius vectors.
    let crb1 = Direction3::from_vector(Vector3::between(ptref, cy1.location()))?;
    let crb2 = Direction3::from_vector(Vector3::between(ptref, cy2.location()))?;
    // Outer normals.
    let norm1: Vector3 = quads.0.normale_at(ptref)?;
    let norm2: Vector3 = quads.1.normale_at(ptref)?;
    let pos1: bool = norm1.dot(Vector3::from(crb1)) > 0.0;
    let pos2: bool = norm2.dot(Vector3::from(crb2)) > 0.0;
    let pick = |c: bool, yes, no| if c { yes } else { no };
    let (situcyl1, situcyl2) = if crb1.dot(crb2) < 0.0 {
        (pick(pos2, Inside, Outside), pick(pos1, Inside, Outside))
    } else if cy1.radius() < cy2.radius() {
        (pick(pos2, Outside, Inside), pick(pos1, Inside, Outside))
    } else {
        (pick(pos2, Inside, Outside), pick(pos1, Outside, Inside))
    };
    IntPatchGLine::from_line(
        linsol,
        true,
        IntPatchLineTransitions::Touch(situcyl1, situcyl2),
    )
}

/// A multiple vertex at a tangent point of the two ellipses.
fn multiple_vertex(p: Point3, quads: Quads<'_>, tol: f64) -> IntPatchPoint {
    let mut pt: IntPatchPoint = tangent_point(p, quads, tol, true);
    pt.set_multiple(true);
    pt
}

/// An ellipse line with its vertices at 0 and 2 PI and the two multiple points.
fn ellipse_line(
    elipsol: &Ellipse3,
    transitions: IntPatchLineTransitions,
    quads: Quads<'_>,
    tol: f64,
    (pmult1, pmult2): (&IntPatchPoint, &IntPatchPoint),
) -> Result<IntPatchGLine, Failure> {
    let glig = IntPatchGLine::from_ellipse(elipsol, false, transitions);
    let ap: Point3 = elclib::value(0.0, elipsol);
    let mut ip: IntPatchPoint = tangent_point(ap, quads, tol, false);
    ip.set_multiple(false);
    ip.set_parameter(0.0);
    glig.add_vertex(&ip)?;
    glig.set_first_point(1);
    ip.set_parameter(2.0 * PI);
    glig.add_vertex(&ip)?;
    glig.set_last_point(2);
    glig.add_vertex(pmult1)?;
    glig.add_vertex(pmult2)?;
    Ok(glig)
}

/// The two ellipses through the tangent points at PI/2 and 3PI/2 of the first one.
fn two_ellipses(
    inter: &QuadQuadGeo,
    quads: Quads<'_>,
    tol: f64,
    slin: &mut Lines,
) -> Result<(), Failure> {
    let elipsol: Ellipse3 = inter.ellipse(1)?;
    let pttang1: Point3 = elclib::value(0.5 * PI, &elipsol);
    let pttang2: Point3 = elclib::value(1.5 * PI, &elipsol);
    let mut pmult1: IntPatchPoint = multiple_vertex(pttang1, quads, tol);
    let mut pmult2: IntPatchPoint = multiple_vertex(pttang2, quads, tol);
    // The first ellipse: the transition computed at 0 gives Trans2, Trans1 (it should
    // be computed at PI).
    let (ptref, tgt) = elclib::d1(0.0, &elipsol);
    let qwe: f64 = dot_cross(tgt, quads.0, quads.1, ptref)?;
    let swapped = match by_sign_tol(qwe, 0.00000001) {
        IntPatchLineTransitions::Types(t1, t2) => IntPatchLineTransitions::Types(t2, t1),
        other => other,
    };
    pmult1.set_parameter(0.5 * PI);
    pmult2.set_parameter(1.5 * PI);
    push(
        slin,
        ellipse_line(&elipsol, swapped, quads, tol, (&pmult1, &pmult2))?,
    );
    // The second ellipse.
    let elipsol: Ellipse3 = inter.ellipse(2)?;
    let param1: f64 = elclib::parameter(&elipsol, pttang1)?;
    let param2: f64 = elclib::parameter(&elipsol, pttang2)?;
    let parampourtransition: f64 = if param1 < param2 {
        pmult1.set_parameter(0.5 * PI);
        pmult2.set_parameter(1.5 * PI);
        PI
    } else {
        pmult1.set_parameter(1.5 * PI);
        pmult2.set_parameter(0.5 * PI);
        0.0
    };
    // The transition is computed on a point of this line.
    let (ptref, tgt) = elclib::d1(parampourtransition, &elipsol);
    let qwe: f64 = dot_cross(tgt, quads.0, quads.1, ptref)?;
    push(
        slin,
        ellipse_line(
            &elipsol,
            by_sign_tol(qwe, 0.00000001),
            quads,
            tol,
            (&pmult1, &pmult2),
        )?,
    );
    Ok(())
}
