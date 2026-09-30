// Rust translation of IntPP, IntPCy and IntPSp of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The plane against a plane, a cylinder or a sphere.
use super::super::gline::IntPatchGLine;
use super::super::line::{IntPatchLine, IntPatchLineTransitions};
use super::super::point::IntPatchPoint;
use super::seam::{adjust_to_seam, adjust_to_sphere_seam};
use super::transitions::{by_sign, dot_cross};
use crate::int_surf::{IntSurfQuadric, IntSurfSituation};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana::{QuadQuadGeo, ResultType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Cylinder3, Ellipse3, Line3, Plane3, Point3, Sphere3, Vector3};

/// The lines and points of a pair solver.
pub(super) type Lines = Vec<Handle<dyn IntPatchLine>>;
/// The isolated points of a pair solver.
pub(super) type Points = Vec<IntPatchPoint>;

/// Appends a conic line.
pub(super) fn push(slin: &mut Lines, line: IntPatchGLine) {
    slin.push(Handle::new(line).cast());
}

/// The crossing line of the plane and the other quadric, oriented by the mixed product
/// at its location.
pub(super) fn crossing_line(
    lin: &Line3,
    (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
) -> Result<IntPatchGLine, Failure> {
    let orig: Point3 = lin.location();
    let discri: f64 = dot_cross(Vector3::from(lin.direction()), quad1, quad2, orig)?;
    IntPatchGLine::from_line(lin, false, by_sign(discri))
}

/// The crossing circle, oriented at its parameter 0.
pub(super) fn crossing_circle(
    c: &Circle3,
    (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
) -> Result<IntPatchGLine, Failure> {
    let (ptref, tgt) = elclib::d1(0.0, c);
    let x: f64 = dot_cross(tgt, quad1, quad2, ptref)?;
    Ok(IntPatchGLine::from_circle(c, false, by_sign(x)))
}

/// The crossing ellipse, oriented at its parameter 0.
pub(super) fn crossing_ellipse(
    e: &Ellipse3,
    (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
) -> Result<IntPatchGLine, Failure> {
    let (ptref, tgt) = elclib::d1(0.0, e);
    let x: f64 = dot_cross(tgt, quad1, quad2, ptref)?;
    Ok(IntPatchGLine::from_ellipse(e, false, by_sign(x)))
}

/// A tangent isolated point with its parameters on both quadrics.
pub(super) fn tangent_point(
    psol: Point3,
    (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
    tol: f64,
    tangent: bool,
) -> IntPatchPoint {
    let (u1, v1) = quad1.parameters(psol).unwrap_or((0.0, 0.0));
    let (u2, v2) = quad2.parameters(psol).unwrap_or((0.0, 0.0));
    let mut ptsol = IntPatchPoint::new();
    ptsol.set_value_tangent(psol, tol, tangent);
    ptsol.set_parameters(u1, v1, u2, v2);
    ptsol
}

/// `IntPP(Quad1, Quad2, Tolang, TolTang, Same, slin)`: the line of two planes, or
/// `same` for coincident planes; false when the intersection fails.
pub(super) fn int_pp(
    quads: (&IntSurfQuadric, &IntSurfQuadric),
    (tolang, tol_tang): (f64, f64),
    same: &mut bool,
    slin: &mut Lines,
) -> Result<bool, Failure> {
    let (pl1, pl2): (Plane3, Plane3) = (quads.0.plane(), quads.1.plane());
    let mut inter = QuadQuadGeo::new();
    inter.perform_planes(pl1, pl2, tolang, tol_tang)?;
    if !inter.is_done() {
        return Ok(false);
    }
    *same = false;
    match inter.type_inter()? {
        ResultType::Same => *same = true,
        ResultType::Empty => {}
        _ => push(slin, crossing_line(&inter.line(1)?, quads)?),
    }
    Ok(true)
}

/// The situations `(plane, cylinder)` of a tangency line from the normals at `orig`.
fn tangency_situations(
    normp: Vector3,
    normcyl: Vector3,
    test_curvature: Vector3,
) -> (IntSurfSituation, IntSurfSituation) {
    let same_side: bool = normp.dot(normcyl) > 0.0;
    if normp.dot(test_curvature) > 0.0 {
        let situp = if same_side {
            IntSurfSituation::Inside
        } else {
            IntSurfSituation::Outside
        };
        return (situp, IntSurfSituation::Outside);
    }
    let situp = if same_side {
        IntSurfSituation::Outside
    } else {
        IntSurfSituation::Inside
    };
    (situp, IntSurfSituation::Inside)
}

/// `IntPCy(Quad1, Quad2, Tolang, TolTang, Reversed, Empty, slin, H)`: the plane and the
/// cylinder of height `h` (0 when infinite): a tangency line, two lines, a circle
/// (on the cylinder seam) or an ellipse.
pub(super) fn int_pcy(
    quads: (&IntSurfQuadric, &IntSurfQuadric),
    (tolang, tol_tang): (f64, f64),
    reversed: bool,
    empty: &mut bool,
    slin: &mut Lines,
    h: f64,
) -> Result<bool, Failure> {
    let (pl, cy): (Plane3, Cylinder3) = if reversed {
        (quads.1.plane(), quads.0.cylinder()?)
    } else {
        (quads.0.plane(), quads.1.cylinder()?)
    };
    let mut inter = QuadQuadGeo::new();
    inter.perform_plane_cylinder(pl, cy, tolang, tol_tang, h)?;
    if !inter.is_done() {
        return Ok(false);
    }
    let typint: ResultType = inter.type_inter()?;
    let nb_sol: i32 = inter.nb_solutions()?;
    *empty = false;
    match typint {
        ResultType::Empty => *empty = true,
        ResultType::Line if nb_sol == 1 => {
            let linsol: Line3 = inter.line(1)?;
            push(slin, tangency_line(&linsol, &cy, quads, reversed)?);
        }
        ResultType::Line => {
            push(slin, crossing_line(&inter.line(1)?, quads)?);
            push(slin, crossing_line(&inter.line(2)?, quads)?);
        }
        ResultType::Circle => {
            let mut cirsol: Circle3 = inter.circle(1)?;
            adjust_to_seam(&cy.position(), &mut cirsol)?;
            push(slin, crossing_circle(&cirsol, quads)?);
        }
        ResultType::Ellipse => push(slin, crossing_ellipse(&inter.ellipse(1)?, quads)?),
        _ => return Ok(false), // must not happen
    }
    Ok(true)
}

/// The tangency line of the plane and the cylinder, with TOUCH transitions.
fn tangency_line(
    linsol: &Line3,
    cy: &Cylinder3,
    (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
    reversed: bool,
) -> Result<IntPatchGLine, Failure> {
    let orig: Point3 = linsol.location();
    let test_curvature = Vector3::between(orig, cy.location());
    let (normp, normcyl) = if reversed {
        (quad2.normale_at(orig)?, quad1.normale_at(orig)?)
    } else {
        (quad1.normale_at(orig)?, quad2.normale_at(orig)?)
    };
    let (situp, situcyl) = tangency_situations(normp, normcyl, test_curvature);
    let situations = if reversed {
        IntPatchLineTransitions::Touch(situcyl, situp)
    } else {
        IntPatchLineTransitions::Touch(situp, situcyl)
    };
    IntPatchGLine::from_line(linsol, true, situations)
}

/// `IntPSp(Quad1, Quad2, Tolang, TolTang, Reversed, Empty, slin, spnt)`: the plane and
/// the sphere: a tangent point or a circle (on the sphere seam when its axis is the
/// sphere one).
#[allow(clippy::too_many_arguments)]
pub(super) fn int_psp(
    quads: (&IntSurfQuadric, &IntSurfQuadric),
    (tolang, tol_tang): (f64, f64),
    reversed: bool,
    empty: &mut bool,
    slin: &mut Lines,
    spnt: &mut Points,
) -> Result<bool, Failure> {
    let (pl, sp): (Plane3, Sphere3) = if reversed {
        (quads.1.plane(), quads.0.sphere()?)
    } else {
        (quads.0.plane(), quads.1.sphere()?)
    };
    let mut inter = QuadQuadGeo::new();
    inter.perform_plane_sphere(pl, sp)?;
    if !inter.is_done() {
        return Ok(false);
    }
    let typint: ResultType = inter.type_inter()?;
    *empty = false;
    match typint {
        ResultType::Empty => *empty = true,
        ResultType::Point => spnt.push(tangent_point(inter.point(1)?, quads, tol_tang, true)),
        ResultType::Circle => {
            let mut cirsol: Circle3 = inter.circle(1)?;
            adjust_to_sphere_seam(&sp, &mut cirsol, tolang)?;
            push(slin, crossing_circle(&cirsol, quads)?);
        }
        _ => return Ok(false), // must not happen
    }
    Ok(true)
}
