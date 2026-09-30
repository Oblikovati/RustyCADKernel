// Rust translation of ModelingData/TKGeomBase/GeomLProp/GeomLProp_SurfaceUtils.hxx
// (algorithms) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LProp_SurfaceUtils`: the algorithms of the surface local property classes.
//!
//! As in `LProp_CurveUtils`, the helpers take the property object (a
//! [`SurfaceProps`]) and read its fields after the calls that upgrade them
//! ([`SurfaceProps::view`]); the fields they write are `&mut` copies stored back
//! by the caller. The native helpers that take a field *by value* read it before
//! those calls; the port passes that earlier value explicitly (see
//! [`tangent_u`], [`require_curvature`] and [`is_umbilic`]).
use super::surface_access::SurfaceAccess;
pub use super::surface_curvatures::{SurfaceCurvatures, compute_surf_curvatures};
use crate::lprop::curve_utils::{chord_other, chord_step};
use crate::lprop::{Status, not_defined};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::cslib;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

/// The point and the partial derivatives up to order 2 of a surface property
/// object (`myPnt`, `myD1u`, `myD1v`, `myD2u`, `myD2v`, `myDuv`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfaceJet {
    /// The point.
    pub point: Point3,
    /// First derivative in U.
    pub d1u: Vector3,
    /// First derivative in V.
    pub d1v: Vector3,
    /// Second derivative in U.
    pub d2u: Vector3,
    /// Second derivative in V.
    pub d2v: Vector3,
    /// Mixed second derivative.
    pub duv: Vector3,
}

/// The four lazily computed statuses of a surface property object.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceStatuses {
    /// `myUTangentStatus`.
    pub u_tangent: Status,
    /// `myVTangentStatus`.
    pub v_tangent: Status,
    /// `myNormalStatus`.
    pub normal: Status,
    /// `myCurvatureStatus`.
    pub curvature: Status,
}

/// The fields of a surface property object read after an upgrade;
/// `props.view().jet.d1u`.
#[derive(Clone, Copy, Debug)]
pub struct SurfacePropsView {
    /// The current U parameter.
    pub u: f64,
    /// The current V parameter.
    pub v: f64,
    /// The derivative order already evaluated (`myDerOrder`).
    pub der_order: i32,
    /// The point and the derivatives.
    pub jet: SurfaceJet,
    /// The stored normal (`myNormal`).
    pub normal: Direction3,
    /// The stored curvatures and their directions.
    pub curvatures: SurfaceCurvatures,
}

/// The members of a surface property object that the helpers call back (the
/// native `Props` template parameter); `SLPropsBase` implements it.
pub trait SurfaceProps {
    /// The surface type.
    type Surface;
    /// The evaluated surface (`mySurf`).
    fn props_surface(&self) -> &Self::Surface;
    /// The fields read after an upgrade.
    fn view(&self) -> SurfacePropsView;
    /// `D1U()`.
    fn d1u(&mut self) -> Result<Vector3, Failure>;
    /// `D1V()`.
    fn d1v(&mut self) -> Result<Vector3, Failure>;
    /// `D2U()`.
    fn d2u(&mut self) -> Result<Vector3, Failure>;
    /// `D2V()`.
    fn d2v(&mut self) -> Result<Vector3, Failure>;
    /// `IsTangentUDefined()`.
    fn is_tangent_u_defined(&mut self) -> Result<bool, Failure>;
    /// `IsTangentVDefined()`.
    fn is_tangent_v_defined(&mut self) -> Result<bool, Failure>;
    /// `IsNormalDefined()`.
    fn is_normal_defined(&mut self) -> Result<bool, Failure>;
    /// `IsCurvatureDefined()`.
    fn is_curvature_defined(&mut self) -> Result<bool, Failure>;
}

/// `EvalSurfDerivatives`: the point and the derivatives up to `order` (0 to 2) at
/// `(u, v)`; any other order evaluates nothing;
/// `eval_surf_derivatives::<DirectAccess, _>(&s, u, v, 2, &mut jet)?`.
pub fn eval_surf_derivatives<A: SurfaceAccess<S>, S>(
    surface: &S,
    u: f64,
    v: f64,
    order: i32,
    jet: &mut SurfaceJet,
) -> Result<(), Failure> {
    match order {
        0 => jet.point = A::d0(surface, u, v)?,
        1 => {
            let d1 = A::d1(surface, u, v)?;
            (jet.point, jet.d1u, jet.d1v) = (d1.point, d1.d1u, d1.d1v);
        }
        2 => {
            let d2 = A::d2(surface, u, v)?;
            (jet.point, jet.d1u, jet.d1v) = (d2.point, d2.d1u, d2.d1v);
            (jet.d2u, jet.d2v, jet.duv) = (d2.d2u, d2.d2v, d2.d2uv);
        }
        _ => {}
    }
    Ok(())
}

/// `FindSurfTangentOrder`: the order (1 or 2) of the first derivative longer than
/// `sqrt(tol_sq)`, bounded by the continuity `cn`; `order` keeps the last order
/// tried when none is found, as natively;
/// `find_surf_tangent_order(d1, d2, 4, 1e-14, &mut order, &mut status)`.
pub fn find_surf_tangent_order(
    d1: Vector3,
    d2: Vector3,
    cn: i32,
    tol_sq: f64,
    order: &mut i32,
    status: &mut Status,
) -> bool {
    let derivs: [Vector3; 2] = [d1, d2];
    *order = 0;
    while *order < 3 {
        *order += 1;
        if cn < *order {
            *status = Status::Undefined;
            return false;
        }
        if *order <= 2 && derivs[*order as usize - 1].square_magnitude() > tol_sq {
            *status = Status::Defined;
            return true;
        }
    }
    *status = Status::Undefined;
    false
}

/// `ComputeSurfTangent`: the first derivative's direction when `sig_order` is 1,
/// else the second derivative oriented along the chord of the iso line;
/// `compute_surf_tangent::<DirectAccess, _>(&s, u, v, d1u, d2u, 1, true)?`.
pub fn compute_surf_tangent<A: SurfaceAccess<S>, S>(
    surface: &S,
    u: f64,
    v: f64,
    first_deriv: Vector3,
    sec_deriv: Vector3,
    sig_order: i32,
    is_u: bool,
) -> Result<Direction3, Failure> {
    if sig_order == 1 {
        return Direction3::from_vector(first_deriv);
    }
    let (u_infimum, v_infimum, u_supremum, v_supremum) = A::bounds(surface)?;
    let (p1, p2): (Point3, Point3) = if is_u {
        let other: f64 = chord_other(u, u_infimum, chord_step(u_infimum, u_supremum));
        (
            A::d0(surface, u.min(other), v)?,
            A::d0(surface, u.max(other), v)?,
        )
    } else {
        let other: f64 = chord_other(v, v_infimum, chord_step(v_infimum, v_supremum));
        (
            A::d0(surface, u, v.min(other))?,
            A::d0(surface, u, v.max(other))?,
        )
    };
    let mut aux: Vector3 = sec_deriv;
    if aux.dot(Vector3::between(p1, p2)) < 0.0 {
        aux = -aux;
    }
    Direction3::from_vector(aux)
}

/// `ComputeSurfNormal`: `CSLib::Normal(D1U, D1V, LinTol)`; writes `normal` and
/// returns true when the status is CSLib_Done;
/// `compute_surf_normal(d1u, d1v, 1e-7, &mut normal)?`.
pub fn compute_surf_normal(
    d1u: Vector3,
    d1v: Vector3,
    lin_tol: f64,
    normal: &mut Direction3,
) -> Result<bool, Failure> {
    let result = cslib::normal(d1u, d1v, lin_tol)?;
    if let Some(direction) = result.normal {
        *normal = direction;
    }
    Ok(result.status == cslib::DerivativeStatus::Done)
}

/// `SetParameters`: store `(u, v)`, evaluate the derivatives up to `der_order`
/// and reset the four statuses;
/// `set_parameters::<DirectAccess, _>(&s, (u, v), &mut uv, 2, &mut jet, &mut st)?`.
pub fn set_parameters<A: SurfaceAccess<S>, S>(
    surface: &S,
    uv: (f64, f64),
    stored_uv: &mut (f64, f64),
    der_order: i32,
    jet: &mut SurfaceJet,
    statuses: &mut SurfaceStatuses,
) -> Result<(), Failure> {
    *stored_uv = uv;
    eval_surf_derivatives::<A, S>(surface, uv.0, uv.1, der_order, jet)?;
    *statuses = SurfaceStatuses::default();
    Ok(())
}

/// `EnsureSurfDeriv`: raise the derivative order to `required` (re-evaluating
/// every derivative) and return the field `result` selects;
/// `ensure_surf_deriv::<DirectAccess, _>(&s, (u, v), &mut order, 2, &mut jet, |j| j.d2u)?`.
pub fn ensure_surf_deriv<A: SurfaceAccess<S>, S>(
    surface: &S,
    uv: (f64, f64),
    der_order: &mut i32,
    required: i32,
    jet: &mut SurfaceJet,
    result: fn(&SurfaceJet) -> Vector3,
) -> Result<Vector3, Failure> {
    if *der_order < required {
        *der_order = required;
        eval_surf_derivatives::<A, S>(surface, uv.0, uv.1, *der_order, jet)?;
    }
    Ok(result(jet))
}

/// `IsTangentUDefined`: the tangent search on D1U and D2U;
/// `is_tangent_u_defined(&mut props, 4, 1e-7, &mut order, &mut status)?`.
///
/// The native call `FindSurfTangentOrder(D1U(), D2U(), ...)` evaluates its
/// arguments right to left (GCC): D2U() first, which upgrades every derivative.
pub fn is_tangent_u_defined<P: SurfaceProps>(
    props: &mut P,
    cn: i32,
    lin_tol: f64,
    sig_order: &mut i32,
    tan_status: &mut Status,
) -> Result<bool, Failure> {
    if let Some(known) = known_status(*tan_status) {
        return Ok(known);
    }
    props.d2u()?;
    props.d1u()?;
    let jet: SurfaceJet = props.view().jet;
    let tol_sq: f64 = lin_tol * lin_tol;
    Ok(find_surf_tangent_order(
        jet.d1u, jet.d2u, cn, tol_sq, sig_order, tan_status,
    ))
}

/// `IsTangentVDefined`: the tangent search on D1V and D2V (D2V() evaluated first,
/// as for U); `is_tangent_v_defined(&mut props, 4, 1e-7, &mut order, &mut status)?`.
pub fn is_tangent_v_defined<P: SurfaceProps>(
    props: &mut P,
    cn: i32,
    lin_tol: f64,
    sig_order: &mut i32,
    tan_status: &mut Status,
) -> Result<bool, Failure> {
    if let Some(known) = known_status(*tan_status) {
        return Ok(known);
    }
    props.d2v()?;
    props.d1v()?;
    let jet: SurfaceJet = props.view().jet;
    let tol_sq: f64 = lin_tol * lin_tol;
    Ok(find_surf_tangent_order(
        jet.d1v, jet.d2v, cn, tol_sq, sig_order, tan_status,
    ))
}

// The answer of a status that is already decided: false when undefined, true when
// defined or computed, None when undecided.
fn known_status(status: Status) -> Option<bool> {
    match status {
        Status::Undecided => None,
        Status::Undefined => Some(false),
        Status::Defined | Status::Computed => Some(true),
    }
}

/// `TangentU`: LProp_NotDefined without U tangent, else the U tangent direction.
///
/// `sig_order_at_call` is the significant order *before* the tangent test: the
/// native helper takes it by value, so on the first call of a fresh object (order
/// 0) the direction comes from the chord-oriented D2U, which raises
/// Standard_ConstructionError when D2U is null (a plane);
/// `tangent_u::<DirectAccess, _>(&mut props, 1)?`.
pub fn tangent_u<A: SurfaceAccess<P::Surface>, P: SurfaceProps>(
    props: &mut P,
    sig_order_at_call: i32,
) -> Result<Direction3, Failure> {
    if !props.is_tangent_u_defined()? {
        return Err(not_defined(
            "SLProps::TangentU: the U tangent is not defined",
        ));
    }
    let view: SurfacePropsView = props.view();
    let (d1u, d2u) = (view.jet.d1u, view.jet.d2u);
    let surface = props.props_surface();
    compute_surf_tangent::<A, P::Surface>(
        surface,
        view.u,
        view.v,
        d1u,
        d2u,
        sig_order_at_call,
        true,
    )
}

/// `TangentV`: LProp_NotDefined without V tangent, else the V tangent direction,
/// with the same by-value significant order as [`tangent_u`];
/// `tangent_v::<DirectAccess, _>(&mut props, 1)?`.
pub fn tangent_v<A: SurfaceAccess<P::Surface>, P: SurfaceProps>(
    props: &mut P,
    sig_order_at_call: i32,
) -> Result<Direction3, Failure> {
    if !props.is_tangent_v_defined()? {
        return Err(not_defined(
            "SLProps::TangentV: the V tangent is not defined",
        ));
    }
    let view: SurfacePropsView = props.view();
    let (d1v, d2v) = (view.jet.d1v, view.jet.d2v);
    let surface = props.props_surface();
    compute_surf_tangent::<A, P::Surface>(
        surface,
        view.u,
        view.v,
        d1v,
        d2v,
        sig_order_at_call,
        false,
    )
}

/// `IsNormalDefined`: the CSLib normal of the *current* D1U and D1V (never
/// upgraded here: with N = 0 they are the null defaults);
/// `is_normal_defined(d1u, d1v, 1e-7, &mut normal, &mut status)?`.
pub fn is_normal_defined(
    d1u: Vector3,
    d1v: Vector3,
    lin_tol: f64,
    normal: &mut Direction3,
    norm_status: &mut Status,
) -> Result<bool, Failure> {
    if let Some(known) = known_status(*norm_status) {
        return Ok(known);
    }
    if compute_surf_normal(d1u, d1v, lin_tol, normal)? {
        *norm_status = Status::Computed;
        return Ok(true);
    }
    *norm_status = Status::Undefined;
    Ok(false)
}

/// `Normal`: LProp_NotDefined without normal, else the stored normal;
/// `normal(&mut props)?`.
pub fn normal<P: SurfaceProps>(props: &mut P) -> Result<Direction3, Failure> {
    if !props.is_normal_defined()? {
        return Err(not_defined("SLProps::Normal: the normal is not defined"));
    }
    Ok(props.view().normal)
}

/// `IsCurvatureDefined`: continuity at least 2, normal and both tangents
/// defined, then the principal curvatures of the fundamental forms;
/// `is_curvature_defined(&mut props, 4, &mut curvatures, &mut status)?`.
pub fn is_curvature_defined<P: SurfaceProps>(
    props: &mut P,
    cn: i32,
    curvatures: &mut SurfaceCurvatures,
    curv_status: &mut Status,
) -> Result<bool, Failure> {
    if let Some(known) = known_status(*curv_status) {
        return Ok(known);
    }
    if cn < 2 || !props.is_normal_defined()? || !tangents_defined(props)? {
        *curv_status = Status::Undefined;
        return Ok(false);
    }
    if props.view().der_order < 2 {
        props.d2u()?;
    }
    let view: SurfacePropsView = props.view();
    let jet: SurfaceJet = view.jet;
    let defined = compute_surf_curvatures(
        [jet.d1u, jet.d1v, jet.d2u, jet.d2v, jet.duv],
        view.normal,
        curvatures,
    )?;
    *curv_status = if defined {
        Status::Computed
    } else {
        Status::Undefined
    };
    Ok(defined)
}

// `IsTangentUDefined() && IsTangentVDefined()`, short-circuited as natively.
fn tangents_defined<P: SurfaceProps>(props: &mut P) -> Result<bool, Failure> {
    Ok(props.is_tangent_u_defined()? && props.is_tangent_v_defined()?)
}

/// `RequireCurvature`: LProp_NotDefined without curvature, else `value_at_call`.
///
/// The native helper takes the curvature field by value, read *before* the
/// curvature is computed: the first query of a point returns the value of the
/// previous point (0 on a fresh object), the next queries the right one;
/// `require_curvature(&mut props, stored_max)?`.
pub fn require_curvature<P: SurfaceProps>(
    props: &mut P,
    value_at_call: f64,
) -> Result<f64, Failure> {
    if !props.is_curvature_defined()? {
        return Err(not_defined("SLProps: the curvature is not defined"));
    }
    Ok(value_at_call)
}

/// `IsUmbilic`: LProp_NotDefined without curvature, else whether the extreme
/// curvatures (read before the computation, as [`require_curvature`]) are equal
/// within `|Epsilon(max)|`; `is_umbilic(&mut props, max, min)?`.
pub fn is_umbilic<P: SurfaceProps>(
    props: &mut P,
    max_curv_at_call: f64,
    min_curv_at_call: f64,
) -> Result<bool, Failure> {
    if !props.is_curvature_defined()? {
        return Err(not_defined(
            "SLProps::IsUmbilic: the curvature is not defined",
        ));
    }
    Ok((max_curv_at_call - min_curv_at_call).abs() < epsilon(max_curv_at_call).abs())
}

/// `CurvatureDirections`: LProp_NotDefined without curvature, else the directions
/// of the maximum and minimum curvatures; `curvature_directions(&mut props)?`.
pub fn curvature_directions<P: SurfaceProps>(
    props: &mut P,
) -> Result<(Direction3, Direction3), Failure> {
    if !props.is_curvature_defined()? {
        return Err(not_defined(
            "SLProps::CurvatureDirections: the curvature is not defined",
        ));
    }
    let curvatures: SurfaceCurvatures = props.view().curvatures;
    Ok((curvatures.dir_max, curvatures.dir_min))
}
