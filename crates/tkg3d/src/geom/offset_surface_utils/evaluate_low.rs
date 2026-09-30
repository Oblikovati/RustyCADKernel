// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetSurfaceUtils.pxx (EvaluateD0, EvaluateD1) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Offset point and first derivatives, with the singular-point loop: osculating
//! surface, higher-order normal, derivative replacement, then a shifted point.
use super::derivatives::{DerivativeOrders, compute_derivatives, entry};
use super::{
    BasisDomain, OffsetOutcome, OsculatingInfo, OsculatingQuery, THE_D1_MAGNITUDE_TOL,
    calculate_d0, compute_dnormal_u, compute_dnormal_v, is_infinite_coord, offset_point, put,
    query_osculating, replace_derivative, shift_point, zero_table,
};
use crate::geom::{BasisSurfaceEvaluator, GeomSurface, GeomSurfaceD1};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::cslib::{self, NormalStatus, ParametricPosition, SingularNormal};
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

/// The position record of CSLib::Normal at `(u, v)` in `domain`.
pub(super) fn position(uv: (f64, f64), domain: BasisDomain) -> ParametricPosition {
    let (u_min, u_max, v_min, v_max) = domain.bounds;
    ParametricPosition {
        u: uv.0,
        v: uv.1,
        u_min,
        u_max,
        v_min,
        v_max,
    }
}

/// CSLib::Normal(MaxOrder, DerNUV, THE_D1_MAGNITUDE_TOL, U, V, bounds...).
pub(super) fn singular_normal(
    max_order: i32,
    der_nuv: &Array2<'_, Vector3>,
    uv: (f64, f64),
    domain: BasisDomain,
) -> Result<SingularNormal, Failure> {
    cslib::normal_at_singularity(
        max_order,
        der_nuv,
        THE_D1_MAGNITUDE_TOL,
        position(uv, domain),
    )
}

/// ComputeDerivatives through the osculating surface when there is one, through
/// the basis alone otherwise (the two native call sites of every evaluator).
pub(super) fn derivatives_with<B: BasisSurfaceEvaluator + ?Sized>(
    orders: DerivativeOrders,
    uv: (f64, f64),
    basis: &B,
    oscillating: (&OsculatingInfo, &Handle<dyn GeomSurface>),
    tables: (&mut Array2<'_, Vector3>, &mut Array2<'_, Vector3>),
) -> Result<bool, Failure> {
    let (info, surface) = oscillating;
    let (der_nuv, der_surf) = tables;
    match surface.get().filter(|_| info.has_osculating()) {
        Some(osc) => compute_derivatives(
            orders,
            uv,
            basis,
            (info.along_u, info.along_v),
            Some(osc),
            der_nuv,
            der_surf,
        ),
        None => compute_derivatives(orders, uv, basis, (false, false), None, der_nuv, der_surf),
    }
}

/// Offset point from the basis D1 at `(u, v)` (`EvaluateD0`); `None` when no
/// normal can be found; `evaluate_d0((u, v), &*basis, 2.0, None)?`.
pub fn evaluate_d0<B: BasisSurfaceEvaluator + ?Sized>(
    uv: (f64, f64),
    basis: &B,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
) -> Result<Option<Point3>, Failure> {
    let d1: GeomSurfaceD1 = basis.basis_d1(uv.0, uv.1)?;
    Ok(evaluate_d0_from(uv, basis, offset, osc, d1)?.into_option())
}

/// `EvaluateD0` with the basis point and first derivatives already evaluated; a
/// failure leaves the basis point of the last pass in the output;
/// `evaluate_d0_from((u, v), &*basis, 2.0, None, d1)?.into_option()`.
pub fn evaluate_d0_from<B: BasisSurfaceEvaluator + ?Sized>(
    start: (f64, f64),
    basis: &B,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
    first: GeomSurfaceD1,
) -> Result<OffsetOutcome<Point3>, Failure> {
    let domain = BasisDomain::of(basis)?;
    let mut uv: (f64, f64) = start;
    let mut current: GeomSurfaceD1 = first;
    let mut is_first_iteration: bool = true;
    loop {
        if !is_first_iteration {
            current = basis.basis_d1(uv.0, uv.1)?;
        }
        is_first_iteration = false;
        let (mut value, d1u, d1v) = (current.point, current.d1u, current.d1v);
        if is_infinite_coord(d1u) || is_infinite_coord(d1v) {
            return Ok(OffsetOutcome::failure(current.point));
        }
        if calculate_d0(&mut value, d1u, d1v, offset, 1.0)? {
            return Ok(OffsetOutcome::success(value));
        }
        if let Some(value) = singular_d0(uv, basis, offset, osc, (value, d1u, d1v), domain)? {
            return Ok(OffsetOutcome::success(value));
        }
        if !shift_point(start, &mut uv, domain, d1u, d1v) {
            return Ok(OffsetOutcome::failure(current.point));
        }
    }
}

// One singular-point attempt of EvaluateD0: the offset point, or None to shift.
fn singular_d0<B: BasisSurfaceEvaluator + ?Sized>(
    uv: (f64, f64),
    basis: &B,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
    point: (Point3, Vector3, Vector3),
    domain: BasisDomain,
) -> Result<Option<Point3>, Failure> {
    const MAX_ORDER: i32 = 3;
    let (mut value, d1u, d1v) = point;
    let (info, surface) = query_osculating(osc, uv.0, uv.1)?;
    let mut der_nuv = zero_table(MAX_ORDER, MAX_ORDER)?;
    let mut der_surf = zero_table(MAX_ORDER + 1, MAX_ORDER + 1)?;
    put(&mut der_surf, 1, 0, d1u)?;
    put(&mut der_surf, 0, 1, d1v)?;
    let orders = DerivativeOrders {
        max_order: MAX_ORDER,
        min_order: 1,
        nu: 0,
        nv: 0,
    };
    if !derivatives_with(
        orders,
        uv,
        basis,
        (&info, &surface),
        (&mut der_nuv, &mut der_surf),
    )? {
        return Ok(None);
    }
    let found = singular_normal(MAX_ORDER, &der_nuv, uv, domain)?;
    let (mut status, mut normal) = (found.result.status, found.result.normal);
    if status == NormalStatus::InfinityOfSolutions {
        let (mut new_du, mut new_dv) = (d1u, d1v);
        let tol2: f64 = THE_D1_MAGNITUDE_TOL * THE_D1_MAGNITUDE_TOL;
        if replace_derivative(uv, domain.bounds, [&mut new_du, &mut new_dv], tol2, basis)? {
            let retried = cslib::normal_with_magnitude(new_du, new_dv, THE_D1_MAGNITUDE_TOL)?;
            (status, normal) = (retried.status, retried.normal.or(normal));
        }
    }
    match (status, normal) {
        (NormalStatus::Defined, Some(normal)) => {
            offset_point(&mut value, offset * info.sign(), Vector3::from(normal));
            Ok(Some(value))
        }
        _ => Ok(None),
    }
}

/// Offset point and first derivatives from the basis D2 at `(u, v)`
/// (`EvaluateD1`); `evaluate_d1((u, v), &*basis, 2.0, None)?`.
pub fn evaluate_d1<B: BasisSurfaceEvaluator + ?Sized>(
    uv: (f64, f64),
    basis: &B,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
) -> Result<Option<GeomSurfaceD1>, Failure> {
    let d2 = basis.basis_d2(uv.0, uv.1)?;
    let first = GeomSurfaceD1 {
        point: d2.point,
        d1u: d2.d1u,
        d1v: d2.d1v,
    };
    let second: [Vector3; 3] = [d2.d2u, d2.d2v, d2.d2uv];
    Ok(evaluate_d1_from(uv, basis, offset, osc, (first, second))?.into_option())
}

/// `EvaluateD1` with the basis derivatives up to order 2 already evaluated; a failure
/// leaves the basis point and first derivatives of the last pass in the outputs;
/// `evaluate_d1_from((u, v), &*basis, 2.0, None, (d1, [d2u, d2v, d2uv]))?.into_option()`.
pub fn evaluate_d1_from<B: BasisSurfaceEvaluator + ?Sized>(
    start: (f64, f64),
    basis: &B,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
    first: (GeomSurfaceD1, [Vector3; 3]),
) -> Result<OffsetOutcome<GeomSurfaceD1>, Failure> {
    let domain = BasisDomain::of(basis)?;
    let mut uv: (f64, f64) = start;
    let (mut current, mut d2) = first;
    let mut is_first_iteration: bool = true;
    loop {
        if !is_first_iteration {
            let r = basis.basis_d2(uv.0, uv.1)?;
            current = GeomSurfaceD1 {
                point: r.point,
                d1u: r.d1u,
                d1v: r.d1v,
            };
            d2 = [r.d2u, r.d2v, r.d2uv];
        }
        is_first_iteration = false;
        if is_infinite_coord(current.d1u) || is_infinite_coord(current.d1v) {
            return Ok(OffsetOutcome::failure(current));
        }
        if let Some(result) = attempt_d1(uv, basis, offset, osc, (current, d2), domain)? {
            return Ok(OffsetOutcome::success(result));
        }
        if !shift_point(start, &mut uv, domain, current.d1u, current.d1v) {
            return Ok(OffsetOutcome::failure(current));
        }
    }
}

// One pass of the EvaluateD1 loop body: the regular formula, or the singular
// fallback; None to shift the point.
fn attempt_d1<B: BasisSurfaceEvaluator + ?Sized>(
    uv: (f64, f64),
    basis: &B,
    offset: f64,
    osc: Option<&dyn OsculatingQuery>,
    derivatives: (GeomSurfaceD1, [Vector3; 3]),
    domain: BasisDomain,
) -> Result<Option<GeomSurfaceD1>, Failure> {
    let (current, [d2u, d2v, d2uv]) = derivatives;
    let (d1u, d1v) = (current.d1u, current.d1v);
    let mut normal: Vector3 = super::bounded_cross_product(d1u, d1v);
    let mut info = OsculatingInfo::default();
    let mut surface: Handle<dyn GeomSurface> = Handle::default();
    let singular: bool = normal.square_magnitude() <= THE_D1_MAGNITUDE_TOL * THE_D1_MAGNITUDE_TOL;
    if singular {
        (info, surface) = query_osculating(osc, uv.0, uv.1)?;
    }
    let sign: f64 = info.sign();
    if !singular {
        normal.normalize()?;
        let mut value: Point3 = current.point;
        offset_point(&mut value, offset * sign, normal);
        let n1u: Vector3 = compute_dnormal_u(d1u, d1v, d2u, d2uv, normal);
        let n1v: Vector3 = compute_dnormal_v(d1u, d1v, d2v, d2uv, normal);
        let point = value;
        return Ok(Some(GeomSurfaceD1 {
            point,
            d1u: d1u + n1u * (offset * sign),
            d1v: d1v + n1v * (offset * sign),
        }));
    }
    singular_d1(
        uv,
        basis,
        (offset, sign),
        (&info, &surface),
        (current, [d2u, d2v, d2uv]),
        domain,
    )
}

// The singular branch of EvaluateD1, with the retry after a replaced derivative.
fn singular_d1<B: BasisSurfaceEvaluator + ?Sized>(
    uv: (f64, f64),
    basis: &B,
    offset_sign: (f64, f64),
    oscillating: (&OsculatingInfo, &Handle<dyn GeomSurface>),
    derivatives: (GeomSurfaceD1, [Vector3; 3]),
    domain: BasisDomain,
) -> Result<Option<GeomSurfaceD1>, Failure> {
    const MAX_ORDER: i32 = 3;
    let (offset, sign) = offset_sign;
    let (current, [d2u, d2v, d2uv]) = derivatives;
    let mut der_nuv = zero_table(MAX_ORDER + 1, MAX_ORDER + 1)?;
    let mut der_surf = zero_table(MAX_ORDER + 2, MAX_ORDER + 2)?;
    let initial = [
        (1, 0, current.d1u),
        (0, 1, current.d1v),
        (1, 1, d2uv),
        (2, 0, d2u),
        (0, 2, d2v),
    ];
    for (i, j, value) in initial {
        put(&mut der_surf, i, j, value)?;
    }
    let orders = DerivativeOrders {
        max_order: MAX_ORDER,
        min_order: 2,
        nu: 1,
        nv: 1,
    };
    if !derivatives_with(
        orders,
        uv,
        basis,
        oscillating,
        (&mut der_nuv, &mut der_surf),
    )? {
        return Ok(None);
    }
    let mut found: SingularNormal = singular_normal(MAX_ORDER, &der_nuv, uv, domain)?;
    if found.result.status == NormalStatus::InfinityOfSolutions {
        let (mut new_du, mut new_dv) = (current.d1u, current.d1v);
        let tol2: f64 = THE_D1_MAGNITUDE_TOL * THE_D1_MAGNITUDE_TOL;
        if replace_derivative(uv, domain.bounds, [&mut new_du, &mut new_dv], tol2, basis)? {
            put(&mut der_surf, 1, 0, new_du)?;
            put(&mut der_surf, 0, 1, new_dv)?;
            if !derivatives_with(
                orders,
                uv,
                basis,
                oscillating,
                (&mut der_nuv, &mut der_surf),
            )? {
                return Ok(None);
            }
            found = singular_normal(MAX_ORDER, &der_nuv, uv, domain)?;
        }
    }
    let Some(normal) = defined_normal(&found) else {
        return Ok(None);
    };
    let mut point: Point3 = current.point;
    offset_point(&mut point, offset * sign, Vector3::from(normal));
    let scale: f64 = offset * sign;
    let dn10: Vector3 = cslib::dn_normal(1, 0, &der_nuv, found.order_u, found.order_v)?;
    let dn01: Vector3 = cslib::dn_normal(0, 1, &der_nuv, found.order_u, found.order_v)?;
    let d1u: Vector3 = dn10 * scale + entry(&der_surf, 1, 0)?;
    let d1v: Vector3 = dn01 * scale + entry(&der_surf, 0, 1)?;
    Ok(Some(GeomSurfaceD1 { point, d1u, d1v }))
}

/// The normal of a CSLib result whose status is Defined.
pub(super) fn defined_normal(found: &SingularNormal) -> Option<Direction3> {
    (found.result.status == NormalStatus::Defined)
        .then_some(found.result.normal)
        .flatten()
}
