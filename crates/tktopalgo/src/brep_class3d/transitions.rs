// Rust translation of the static GetTransi, GetNormalOnFaceBound and Trans of
// BRepClass3d_SClassifier.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The transition of a line that passes through an edge between two faces, and the
//! state a transition gives.
use super::sclassifier::face_normal;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_curve_surface::TransitionOnCurve;
use rustycad_tkmath::gp::{Direction3, Line3};

/// `GetTransi(f1, f2, e, param, L, trans)`: 1 with the transition of the line through
/// the edge (parameter `param`) between the faces, 0 to skip the edge (the line runs
/// between the two normal sides), -1 for a probably faulty line (no normal, or the line
/// orthogonal to a normal).
pub(super) fn get_transi(
    f1: &Face,
    f2: &Face,
    e: &Edge,
    param: f64,
    line: &Line3,
    trans: &mut TransitionOnCurve,
) -> Result<i32, Failure> {
    let Some(nf1) = get_normal_on_face_bound(e, f1, param)? else {
        return Ok(-1);
    };
    let Some(nf2) = get_normal_on_face_bound(e, f2, param)? else {
        return Ok(-1);
    };
    let ldir = line.direction();
    let angular = precision::angular();
    if ldir.dot(nf1).abs() < angular || ldir.dot(nf2).abs() < angular {
        return Ok(-1); // line is orthogonal to normal(s)
    }
    if nf1.is_parallel(nf2, angular) {
        let ang_d = nf1.dot(ldir);
        if ang_d.abs() < angular {
            return Ok(-1);
        }
        *trans = if ang_d > 0.0 {
            TransitionOnCurve::Out
        } else {
            TransitionOnCurve::In
        };
        return Ok(1);
    }
    let n = nf1.crossed(nf2)?;
    // proj LDir on the plane defined by nf1/nf2 directions
    let proj_l = Direction3::from_xyz(n.xyz().crossed(ldir.xyz()).crossed(n.xyz()))?;
    let (f_ad, s_ad) = (nf1.dot(proj_l), nf2.dot(proj_l));
    if f_ad < -angular && s_ad < -angular {
        *trans = TransitionOnCurve::In;
    } else if f_ad > angular && s_ad > angular {
        *trans = TransitionOnCurve::Out;
    } else {
        return Ok(0);
    }
    Ok(1)
}

/// `GetNormalOnFaceBound(E, F, param, OutDir)`: the face normal at the pcurve point
/// of parameter `param`; `None` without a pcurve, outside its range or without a normal.
fn get_normal_on_face_bound(
    edge: &Edge,
    face: &Face,
    param: f64,
) -> Result<Option<Direction3>, Failure> {
    let on_face = BRepTool::curve_on_face(edge, face)?;
    let Some(pcurve) = on_face.pcurve.get() else {
        return Ok(None);
    };
    if param < on_face.first || param > on_face.last {
        return Ok(None);
    }
    let p2d = pcurve.d0(param)?;
    face_normal(face, p2d.x(), p2d.y())
}

/// `Trans(parmin, tran, state)`: the transition reversed for a negative parameter;
/// leaving the solid (Out) means IN (3), else OUT (4).
pub(super) fn trans(parmin: f64, tran: &mut TransitionOnCurve, state: &mut i32) {
    // if parmin is negative we should reverse transition
    if parmin < 0.0 {
        *tran = if *tran == TransitionOnCurve::Out {
            TransitionOnCurve::In
        } else {
            TransitionOnCurve::Out
        };
    }
    //-- The line is going from inside the solid to outside the solid.
    *state = if *tran == TransitionOnCurve::Out {
        3
    } else {
        4
    };
}
