// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/LProp/LProp_CurveUtils.hxx (algorithms)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LProp_CurveUtils`: the algorithms of the curve local property classes.
//!
//! The native helpers take the property object and references to its own fields;
//! the port passes the object (a [`CurveProps`]) and reads the fields it needs after
//! the object has upgraded them, through [`CurveProps::view`]. The in/out fields the
//! helpers write (status, significant order, stored curvature) are passed as
//! `&mut` copies that the property class stores back, which is equivalent because
//! the object calls in between never touch them.
use super::curve_access::{CurveAccess, PropsVector};
use super::{Status, not_defined};
use rustycad_tkernel::standard::Failure;

// ComputeTangent: the chord sampling step, as a fraction of the parameter range,
// and its lower bound (the native THE_DIVISION_FACTOR and THE_MIN_STEP).
const THE_DIVISION_FACTOR: f64 = 1.0e-3;
const THE_MIN_STEP: f64 = 1.0e-7;

/// The fields of a curve property object that the helpers read after it has
/// upgraded its derivatives; `props.view().significant_order`.
#[derive(Clone, Copy, Debug)]
pub struct CurvePropsView<V: PropsVector> {
    /// The current parameter (`myU`).
    pub u: f64,
    /// The current point (`myPnt`).
    pub point: V::Point,
    /// The first three derivatives (`myDerivArr`).
    pub derivatives: [V; 3],
    /// Order of the first non-null derivative (`mySignificantFirstDerivativeOrder`).
    pub significant_order: i32,
    /// The last computed curvature (`myCurvature`).
    pub curvature: f64,
}

/// The members of a curve property object that the helpers call back (the
/// native `Props` template parameter); `CLPropsBase` implements it;
/// `CurveProps::is_tangent_defined(&mut props)?`.
pub trait CurveProps<V: PropsVector> {
    /// The curve type.
    type Curve;
    /// The evaluated curve (`myCurve`).
    fn props_curve(&self) -> &Self::Curve;
    /// The fields read after an upgrade.
    fn view(&self) -> CurvePropsView<V>;
    /// `D1()`: the first derivative, computed if needed.
    fn d1(&mut self) -> Result<V, Failure>;
    /// `D2()`: the second derivative, computed if needed.
    fn d2(&mut self) -> Result<V, Failure>;
    /// `D3()`: the third derivative, computed if needed.
    fn d3(&mut self) -> Result<V, Failure>;
    /// `IsTangentDefined()`.
    fn is_tangent_defined(&mut self) -> Result<bool, Failure>;
    /// `Curvature()`.
    fn curvature(&mut self) -> Result<f64, Failure>;
}

/// `EvalDerivatives`: the point and the derivatives up to `order` (0 to 3) at `u`;
/// any other order evaluates nothing, as the native switch;
/// `eval_derivatives::<DirectAccess, _, _>(&curve, 0.5, 2, &mut p, &mut d)?`.
pub fn eval_derivatives<A: CurveAccess<C, V>, C, V: PropsVector>(
    curve: &C,
    u: f64,
    order: i32,
    pnt: &mut V::Point,
    deriv: &mut [V; 3],
) -> Result<(), Failure> {
    match order {
        0 => *pnt = A::d0(curve, u)?,
        1 => (*pnt, deriv[0]) = A::d1(curve, u)?,
        2 => (*pnt, deriv[0], deriv[1]) = A::d2(curve, u)?,
        3 => (*pnt, deriv[0], deriv[1], deriv[2]) = A::d3(curve, u)?,
        _ => {}
    }
    Ok(())
}

/// `ComputeTangent`: the direction of the first significant derivative; when its
/// order is above 1 the sign follows the chord to a point `Delta` away (toward the
/// interior near the first parameter);
/// `compute_tangent::<DirectAccess, _, _>(&curve, u, &d, &p, 2)?`.
pub fn compute_tangent<A: CurveAccess<C, V>, C, V: PropsVector>(
    curve: &C,
    u: f64,
    deriv: &[V; 3],
    _ref_pnt: &V::Point,
    sig_order: i32,
) -> Result<V::Direction, Failure> {
    if sig_order == 1 {
        return deriv[0].to_direction();
    }
    let mut v: V = deriv[significant_index(sig_order)?];
    let other_u: f64 = chord_other_parameter::<A, C, V>(curve, u)?;
    let p1: V::Point = A::d0(curve, u.min(other_u))?;
    let p2: V::Point = A::d0(curve, u.max(other_u))?;
    if v.dot(V::between(p1, p2)) < 0.0 {
        v = -v;
    }
    v.to_direction()
}

// The native array read `theDerivArr[theSigOrder - 1]`; the property classes only
// pass orders 1 to 3, other orders would read outside the array.
fn significant_index(sig_order: i32) -> Result<usize, Failure> {
    if !(1..=3).contains(&sig_order) {
        return Err(Failure::out_of_range(&format!(
            "LProp_CurveUtils::ComputeTangent: significant order {sig_order}; expected in [1, 3]"
        )));
    }
    Ok(sig_order as usize - 1)
}

// The parameter of the chord end: `u - Delta`, or `u + Delta` within Delta of the
// first parameter (the last parameter is read first, as natively).
fn chord_other_parameter<A: CurveAccess<C, V>, C, V: PropsVector>(
    curve: &C,
    u: f64,
) -> Result<f64, Failure> {
    let u_supremum: f64 = A::last_parameter(curve)?;
    let u_infimum: f64 = A::first_parameter(curve)?;
    Ok(chord_other(u, u_infimum, chord_step(u_infimum, u_supremum)))
}

/// The other end of the sign-test chord from `u` on a range starting at
/// `infimum`; `chord_other(0.0, 0.0, 1e-3) == 1e-3`.
pub(crate) fn chord_other(u: f64, infimum: f64, delta: f64) -> f64 {
    if u - infimum < delta {
        return u + delta;
    }
    u - delta
}

/// The chord step of the sign test on `[first, last]`: 1e-3 of the range, at
/// least 1e-7, and 1e-7 for an unbounded range (RealFirst()/RealLast() bounds);
/// `chord_step(0.0, 1.0) == 1e-3`.
pub(crate) fn chord_step(first: f64, last: f64) -> f64 {
    let du: f64 = if last >= f64::MAX || first <= -f64::MAX {
        0.0
    } else {
        last - first
    };
    (du * THE_DIVISION_FACTOR).max(THE_MIN_STEP)
}

/// `ComputeCurvature`: `|D1 ^ D2| / |D1|^3`, or 0 when D2 is null or D1 and D2
/// are collinear within `tol_sq`; `compute_curvature(d1, d2, 1e-14)`.
pub fn compute_curvature<V: PropsVector>(d1: V, d2: V, tol_sq: f64) -> f64 {
    let dd1: f64 = d1.square_magnitude();
    let dd2: f64 = d2.square_magnitude();
    if dd2 <= tol_sq {
        return 0.0;
    }
    let n: f64 = d1.cross_square_magnitude(d2);
    let t: f64 = n / dd1 / dd2;
    if t <= tol_sq {
        return 0.0;
    }
    n.sqrt() / dd1 / dd1.sqrt()
}

/// `ComputeNormal`: the direction of `D2 (D1.D1) - D1 (D1.D2)`, Standard_ConstructionError
/// when it is null; `compute_normal(d1, d2)?`.
pub fn compute_normal<V: PropsVector>(d1: V, d2: V) -> Result<V::Direction, Failure> {
    normal_vector(d1, d2).to_direction()
}

// The vector triple product of the normal, in the native operation order.
fn normal_vector<V: PropsVector>(d1: V, d2: V) -> V {
    d2 * d1.dot(d1) - d1 * d1.dot(d2)
}

/// `ComputeCentreOfCurvature`: the point plus the unit normal divided by the
/// curvature; `compute_centre_of_curvature(p, d1, d2, 0.2)?`.
pub fn compute_centre_of_curvature<V: PropsVector>(
    pnt: V::Point,
    d1: V,
    d2: V,
    curvature: f64,
) -> Result<V::Point, Failure> {
    let mut norm: V = normal_vector(d1, d2);
    norm.normalize()?;
    norm.divide(curvature);
    Ok(V::translate(pnt, norm))
}

/// `SetParameter`: store `u`, evaluate the derivatives up to `der_order` and reset
/// the tangent status; `set_parameter::<DirectAccess, _, _>(&c, u, &mut su, 2, &mut p, &mut d, &mut st)?`.
pub fn set_parameter<A: CurveAccess<C, V>, C, V: PropsVector>(
    curve: &C,
    u: f64,
    stored_u: &mut f64,
    der_order: i32,
    pnt: &mut V::Point,
    deriv: &mut [V; 3],
    tan_status: &mut Status,
) -> Result<(), Failure> {
    *stored_u = u;
    eval_derivatives::<A, C, V>(curve, u, der_order, pnt, deriv)?;
    *tan_status = Status::Undecided;
    Ok(())
}

/// `EnsureDeriv`: raise the derivative order to `required` (re-evaluating every
/// derivative) and return derivative `required`;
/// `ensure_deriv::<DirectAccess, _, _>(&c, u, &mut order, 2, &mut p, &mut d)?`.
pub fn ensure_deriv<A: CurveAccess<C, V>, C, V: PropsVector>(
    curve: &C,
    u: f64,
    der_order: &mut i32,
    required: i32,
    pnt: &mut V::Point,
    deriv: &mut [V; 3],
) -> Result<V, Failure> {
    if *der_order < required {
        *der_order = required;
        eval_derivatives::<A, C, V>(curve, u, *der_order, pnt, deriv)?;
    }
    Ok(deriv[required as usize - 1])
}

/// `IsTangentDefined`: search the first derivative longer than `lin_tol` up to
/// order `min(cn, 3)`, upgrading the derivatives through the object;
/// `is_tangent_defined(&mut props, 4, 1e-7, &mut order, &mut status)?`.
pub fn is_tangent_defined<V: PropsVector, P: CurveProps<V>>(
    props: &mut P,
    cn: i32,
    lin_tol: f64,
    sig_order: &mut i32,
    tan_status: &mut Status,
) -> Result<bool, Failure> {
    if *tan_status == Status::Undefined {
        return Ok(false);
    }
    if *tan_status >= Status::Defined {
        return Ok(true);
    }
    let tol_sq: f64 = lin_tol * lin_tol;
    for order in 1..=4 {
        let Some(v) = significant_candidate(props, cn, order)? else {
            *tan_status = Status::Undefined;
            return Ok(false);
        };
        if v.square_magnitude() > tol_sq {
            *sig_order = order;
            *tan_status = Status::Defined;
            return Ok(true);
        }
    }
    Ok(false)
}

// The derivative of order `order`, or None when the continuity `cn` is below it or
// the order is 4 (both end the native search as undefined).
fn significant_candidate<V: PropsVector, P: CurveProps<V>>(
    props: &mut P,
    cn: i32,
    order: i32,
) -> Result<Option<V>, Failure> {
    if cn < order {
        return Ok(None);
    }
    match order {
        1 => props.d1().map(Some),
        2 => props.d2().map(Some),
        3 => props.d3().map(Some),
        _ => Ok(None),
    }
}

/// `Tangent`: LProp_NotDefined when the tangent is not defined, else the
/// tangent direction; `tangent::<DirectAccess, _, _>(&mut props)?`.
pub fn tangent<A: CurveAccess<P::Curve, V>, P: CurveProps<V>, V: PropsVector>(
    props: &mut P,
) -> Result<V::Direction, Failure> {
    if !props.is_tangent_defined()? {
        return Err(not_defined(
            "CLProps::Tangent: no derivative up to order 3 is longer than the resolution",
        ));
    }
    let view: CurvePropsView<V> = props.view();
    compute_tangent::<A, P::Curve, V>(
        props.props_curve(),
        view.u,
        &view.derivatives,
        &view.point,
        view.significant_order,
    )
}

/// `Curvature`: LProp_NotDefined without tangent, RealLast() when the first
/// significant derivative is above order 1 (without storing it), else the stored
/// curvature; `curvature(&mut props, 1e-7, &mut stored)?`.
pub fn curvature<V: PropsVector, P: CurveProps<V>>(
    props: &mut P,
    lin_tol: f64,
    stored_curvature: &mut f64,
) -> Result<f64, Failure> {
    if !props.is_tangent_defined()? {
        return Err(not_defined("CLProps::Curvature()"));
    }
    let view: CurvePropsView<V> = props.view();
    if view.significant_order > 1 {
        return Ok(f64::MAX);
    }
    *stored_curvature =
        compute_curvature(view.derivatives[0], view.derivatives[1], lin_tol * lin_tol);
    Ok(*stored_curvature)
}

/// `Normal`: LProp_NotDefined when the curvature is null (within `lin_tol`) or
/// RealLast(), else the normal direction; `normal(&mut props, 1e-7)?`.
pub fn normal<V: PropsVector, P: CurveProps<V>>(
    props: &mut P,
    lin_tol: f64,
) -> Result<V::Direction, Failure> {
    let curvature: f64 = props.curvature()?;
    if curvature == f64::MAX || curvature.abs() <= lin_tol {
        return Err(not_defined(
            "CLProps::Normal(): Curvature is null or infinity",
        ));
    }
    let view: CurvePropsView<V> = props.view();
    compute_normal(view.derivatives[0], view.derivatives[1])
}

/// `CentreOfCurvature`: LProp_NotDefined when the curvature is null (within
/// `lin_tol`), else the centre from the stored curvature;
/// `centre_of_curvature(&mut props, 1e-7)?`.
///
/// A RealLast() curvature passes the test and the centre uses the stored value of
/// an earlier call (0 initially), as natively.
pub fn centre_of_curvature<V: PropsVector, P: CurveProps<V>>(
    props: &mut P,
    lin_tol: f64,
) -> Result<V::Point, Failure> {
    if props.curvature()?.abs() <= lin_tol {
        return Err(not_defined(
            "CLProps::CentreOfCurvature: curvature is null; expected |curvature| > resolution",
        ));
    }
    let view: CurvePropsView<V> = props.view();
    compute_centre_of_curvature(
        view.point,
        view.derivatives[0],
        view.derivatives[1],
        view.curvature,
    )
}
