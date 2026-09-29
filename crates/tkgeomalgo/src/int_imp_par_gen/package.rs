// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of IntImpParGen.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static members of the class `IntImpParGen`: positions and transitions of an
//! intersection point.
use crate::int_res2d::{Domain, Position, Situation, Transition, TypeTrans};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};

// `TOLERANCE_ANGULAIRE` of IntImpParGen.cxx.
const TOLERANCE_ANGULAIRE: f64 = 0.00000001;
// `DERIVEE_PREMIERE_NULLE` of IntImpParGen.cxx: the square magnitude (or, in the
// second DetermineTransition, the magnitude) below which a tangent is null.
const DERIVEE_PREMIERE_NULLE: f64 = 0.000000000001;

/// `IntImpParGen::NormalizeOnDomain(Par1, Dom1)`: on a closed domain, shifts the
/// parameter by whole periods towards the domain while the shifted value stays on
/// the far side of the other bound; `normalize_on_domain(7.0, &domain)?`.
pub fn normalize_on_domain(param: f64, domain: &Domain) -> Result<f64, Failure> {
    let mut mod_param: f64 = param;
    if domain.is_closed() {
        let (t, mut periode) = domain.equivalent_parameters();
        periode -= t;
        while mod_param < domain.first_parameter()?
            && mod_param + periode < domain.last_parameter()?
        {
            mod_param += periode;
        }
        while mod_param > domain.last_parameter()?
            && mod_param - periode > domain.first_parameter()?
        {
            mod_param -= periode;
        }
    }
    Ok(mod_param)
}

/// `IntImpParGen::DeterminePosition(Pos1, Dom1, P1, Tol)`: Head when the point is
/// within the first tolerance of the first point, End when within the last one (the
/// nearer parameter wins when both match), Middle otherwise. The fourth native
/// argument is named `Tol` in the header but is the parameter of the point;
/// `determine_position(&domain, point, 0.5)?`.
pub fn determine_position(domain: &Domain, pnt1: Point2, param1: f64) -> Result<Position, Failure> {
    let mut pos1: Position = Position::Middle;
    if domain.has_first_point()
        && pnt1.distance(domain.first_point()?) <= domain.first_tolerance()?
    {
        pos1 = Position::Head;
    }
    let at_last: bool = domain.has_last_point()
        && pnt1.distance(domain.last_point()?) <= domain.last_tolerance()?;
    // A point at both ends is at the end of the nearer parameter.
    if at_last
        && (pos1 != Position::Head
            || (param1 - domain.last_parameter()?).abs()
                < (param1 - domain.first_parameter()?).abs())
    {
        pos1 = Position::End;
    }
    Ok(pos1)
}

/// The TOUCH classification shared by the DetermineTransition variants: the
/// curvatures `val1` and `val2` along the common normal decide the sides, and a
/// difference within `tol` leaves them unknown.
pub(crate) fn set_touch_transitions(
    (pos1, t1): (Position, &mut Transition),
    (pos2, t2): (Position, &mut Transition),
    (val1, val2): (f64, f64),
    opos: bool,
    tol: f64,
) {
    if (val1 - val2).abs() <= tol {
        t1.set_touch_value(true, pos1, Situation::Unknown, opos);
        t2.set_touch_value(true, pos2, Situation::Unknown, opos);
    } else if val2 > val1 {
        t2.set_touch_value(true, pos2, Situation::Inside, opos);
        let side: Situation = if opos {
            Situation::Inside
        } else {
            Situation::Outside
        };
        t1.set_touch_value(true, pos1, side, opos);
    } else {
        t2.set_touch_value(true, pos2, Situation::Outside, opos);
        let side: Situation = if opos {
            Situation::Outside
        } else {
            Situation::Inside
        };
        t1.set_touch_value(true, pos1, side, opos);
    }
}

/// The IN/OUT classification shared by the DetermineTransition variants: a
/// negative cross product of the tangents makes the first curve enter.
pub(crate) fn set_crossing_transitions(
    (pos1, t1): (Position, &mut Transition),
    (pos2, t2): (Position, &mut Transition),
    sgn: f64,
) {
    if sgn < 0.0 {
        t1.set_value(false, pos1, TypeTrans::In);
        t2.set_value(false, pos2, TypeTrans::Out);
    } else {
        t1.set_value(false, pos1, TypeTrans::Out);
        t2.set_value(false, pos2, TypeTrans::In);
    }
}

/// The two curves of a DetermineTransition call: position, tangent (replaced by the
/// normal when null, as the native in-out argument) and normal (second derivative).
pub struct TransitionSide<'a> {
    /// The position of the point on the curve.
    pub pos: Position,
    /// The first derivative; the null-tangent variants overwrite it with `norm`.
    pub tan: &'a mut Vector2,
    /// The second derivative.
    pub norm: Vector2,
    /// The transition that receives the result.
    pub trans: &'a mut Transition,
}

/// `IntImpParGen::DetermineTransition(Pos1, Tan1, Norm1, Trans1, Pos2, Tan2, Norm2,
/// Trans2, Tol)`: a null tangent (square magnitude at most 1e-12) is replaced by the
/// normal and a null normal too makes the transitions undecided; parallel tangents
/// (within 1e-8 of the product of the magnitudes) give TOUCH, decided by the
/// normals; the tolerance argument is unused;
/// `determine_transition(side1, side2, 1e-7)`.
pub fn determine_transition(side1: TransitionSide<'_>, side2: TransitionSide<'_>, _tol: f64) {
    side1.trans.set_position(side1.pos);
    side2.trans.set_position(side2.pos);
    let courbure1: bool = !replace_null_tangent(side1.tan, side1.norm);
    let courbure2: bool = !replace_null_tangent(side2.tan, side2.norm);
    let decide: bool = side1.tan.square_magnitude() > DERIVEE_PREMIERE_NULLE
        && side2.tan.square_magnitude() > DERIVEE_PREMIERE_NULLE;
    if !decide {
        side1.trans.set_undecided_value(side1.pos);
        side2.trans.set_undecided_value(side2.pos);
        return;
    }
    let (tan1, tan2) = (*side1.tan, *side2.tan);
    let sgn: f64 = tan1.crossed(tan2);
    let norm: f64 = tan1.magnitude() * tan2.magnitude();
    let (s1, s2) = ((side1.pos, side1.trans), (side2.pos, side2.trans));
    // Tested as the native `<=` so that a NaN cross product is a crossing.
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
    set_touch_transitions(s1, s2, (val1, val2), opos, TOLERANCE_ANGULAIRE);
}

// Replaces a null tangent (square magnitude at most 1e-12) by the normal; true when
// it was null.
fn replace_null_tangent(tan: &mut Vector2, norm: Vector2) -> bool {
    if tan.square_magnitude() <= DERIVEE_PREMIERE_NULLE {
        *tan = norm;
        return true;
    }
    false
}

/// `IntImpParGen::DetermineTransition(Pos1, Tan1, Trans1, Pos2, Tan2, Trans2, Tol)`:
/// the IN/OUT transitions of two crossing curves; false (the transitions keep their
/// type, with the new positions) when a tangent is null (magnitude at most 1e-12) or
/// the tangents are parallel. The tolerance argument is unused;
/// `determine_crossing_transition((pos1, tan1, &mut t1), (pos2, tan2, &mut t2), 1e-7)`.
pub fn determine_crossing_transition(
    (pos1, tan1, t1): (Position, Vector2, &mut Transition),
    (pos2, tan2, t2): (Position, Vector2, &mut Transition),
    _tol: f64,
) -> bool {
    t1.set_position(pos1);
    t2.set_position(pos2);
    let tan1_magnitude: f64 = tan1.magnitude();
    if tan1_magnitude <= DERIVEE_PREMIERE_NULLE {
        return false;
    }
    let tan2_magnitude: f64 = tan2.magnitude();
    if tan2_magnitude <= DERIVEE_PREMIERE_NULLE {
        return false;
    }
    let sgn: f64 = tan1.crossed(tan2);
    if sgn.abs() <= TOLERANCE_ANGULAIRE * (tan1_magnitude * tan2_magnitude) {
        return false;
    }
    set_crossing_transitions((pos1, t1), (pos2, t2), sgn);
    true
}
