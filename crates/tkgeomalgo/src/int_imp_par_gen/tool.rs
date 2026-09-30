// Rust translation of IntImpParGen_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The free functions of IntImpParGen_Tool.hxx: older variants of the static
//! members of `IntImpParGen` that measure null vectors against gp::Resolution().
use super::package::{self, TransitionSide, set_crossing_transitions, set_touch_transitions};
use crate::int_res2d::{Domain, Position, Situation};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Point2, Vector2};

// `TOLERANCE_ANGULAIRE` of IntImpParGen_Tool.cxx.
const TOLERANCE_ANGULAIRE: f64 = 0.00000001;

/// `NormalizeOnDomain(Par, Dom)`: on a closed domain, adds periods while the
/// parameter is below the first bound, then subtracts one period when it is above
/// the last bound; `normalize_on_domain(-1.0, &domain)?`.
pub fn normalize_on_domain(param: f64, domain: &Domain) -> Result<f64, Failure> {
    let mut mod_param: f64 = param;
    if domain.is_closed() {
        let (t, mut periode) = domain.equivalent_parameters();
        periode -= t;
        if domain.has_first_point() {
            while mod_param < domain.first_parameter()? {
                mod_param += periode;
            }
        }
        if domain.has_last_point() && mod_param > domain.last_parameter()? {
            mod_param -= periode;
        }
    }
    Ok(mod_param)
}

/// `Determine_Position(Pos, Dom, P, Param)`: the same body as
/// `IntImpParGen::DeterminePosition`; `determine_position(&domain, point, 0.5)?`.
pub fn determine_position(domain: &Domain, pnt1: Point2, param1: f64) -> Result<Position, Failure> {
    package::determine_position(domain, pnt1, param1)
}

/// `Determine_Transition(Pos1, Tan1, Norm1, Trans1, Pos2, Tan2, Norm2, Trans2,
/// ToleranceAng)`: a tangent shorter than gp::Resolution() is replaced by the normal
/// (undecided when that is null too); parallel tangents give TOUCH, whose sides are
/// unknown when the curvatures differ by at most gp::Resolution(). Unlike
/// `IntImpParGen::DetermineTransition` it does not reset the positions first, and the
/// tolerance argument is unused; `determine_transition(side1, side2, 1e-7)`.
pub fn determine_transition(side1: TransitionSide<'_>, side2: TransitionSide<'_>, _tol: f64) {
    let resolution: f64 = gp::resolution();
    let courbure1: bool = !replace_short_tangent(side1.tan, side1.norm);
    let courbure2: bool = !replace_short_tangent(side2.tan, side2.norm);
    if side1.tan.magnitude() <= resolution || side2.tan.magnitude() <= resolution {
        side1.trans.set_undecided_value(side1.pos);
        side2.trans.set_undecided_value(side2.pos);
        return;
    }
    let (tan1, tan2) = (*side1.tan, *side2.tan);
    let sgn: f64 = tan1.crossed(tan2);
    let norm: f64 = tan1.magnitude() * tan2.magnitude();
    let (s1, s2) = ((side1.pos, side1.trans), (side2.pos, side2.trans));
    let touch: bool = sgn.abs() <= TOLERANCE_ANGULAIRE * norm;
    if !touch {
        set_crossing_transitions(s1, s2, sgn);
        return;
    }
    let opos: bool = tan1.dot(tan2) < 0.0;
    if !(courbure1 || courbure2) {
        s1.1.set_touch_value(true, s1.0, Situation::Unknown, opos);
        s2.1.set_touch_value(true, s2.0, Situation::Unknown, opos);
        return;
    }
    let normal: Vector2 = Vector2::new(-tan1.y(), tan1.x());
    let val1: f64 = if courbure1 {
        normal.dot(side1.norm)
    } else {
        0.0
    };
    let val2: f64 = if courbure2 {
        normal.dot(side2.norm)
    } else {
        0.0
    };
    set_touch_transitions(s1, s2, (val1, val2), opos, resolution);
}

// Replaces a tangent not longer than gp::Resolution() by the normal; true when it
// was replaced.
fn replace_short_tangent(tan: &mut Vector2, norm: Vector2) -> bool {
    if tan.magnitude() <= gp::resolution() {
        *tan = norm;
        return true;
    }
    false
}
