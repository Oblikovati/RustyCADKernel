// Rust translation of the static NormalD0, NormalD1 and NormalD2 of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Darboux.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The unit normal of a surface and its first two derivatives, with the treatment
//! of singular points through the derivatives of `D1U ^ D1V` (CSLib).
use super::sweep_failures::sweep_failure;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::cslib::{self, NormalStatus, ParametricPosition};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Direction3, Vector3};

/// `MagTol` of the three functions.
const MAG_TOL: f64 = 0.000000001;

fn undefined_value(u: f64, v: f64) -> Failure {
    sweep_failure(
        "Geom_UndefinedValue",
        &format!("GeomFill_Darboux: no surface normal at ({u}, {v}); expected a defined normal"),
    )
}

/// A `(0..n, 0..n)` table of null vectors (the default `gp_Vec`).
fn table(n: i32) -> Result<Array2<'static, Vector3>, Failure> {
    Array2::with_bounds(0, n, 0, n).map_err(|error| {
        sweep_failure(
            "Standard_RangeError",
            &format!("NCollection_Array2(0, {n}): {error}"),
        )
    })
}

fn set(tab: &mut Array2<'_, Vector3>, i: i32, j: i32, value: Vector3) -> Result<(), Failure> {
    tab.set_value(i, j, value).map_err(|error| {
        sweep_failure(
            "Standard_OutOfRange",
            &format!("DerSurf({i}, {j}): {error}"),
        )
    })
}

fn position(surf: &dyn Adaptor3dSurface, u: f64, v: f64) -> Result<ParametricPosition, Failure> {
    Ok(ParametricPosition {
        u,
        v,
        u_min: surf.first_u_parameter()?,
        u_max: surf.last_u_parameter()?,
        v_min: surf.first_v_parameter()?,
        v_max: surf.last_v_parameter()?,
    })
}

/// `DerNUV(i, j) = CSLib::DNNUV(i, j, DerSurf)` for `i, j` in `0..=n`.
fn normal_derivatives(
    n: i32,
    der_surf: &Array2<'_, Vector3>,
) -> Result<Array2<'static, Vector3>, Failure> {
    let mut der_nuv = table(n)?;
    for i in 0..=n {
        for j in 0..=n {
            set(&mut der_nuv, i, j, cslib::dnnuv(i, j, der_surf)?)?;
        }
    }
    Ok(der_nuv)
}

/// The final `CSLib::Normal(MaxOrder, DerNUV, ...)`: the normal and its orders, or
/// Geom_UndefinedValue.
fn singular_normal(
    surf: &dyn Adaptor3dSurface,
    max_order: i32,
    der_nuv: &Array2<'_, Vector3>,
    (u, v): (f64, f64),
) -> Result<(Direction3, i32, i32), Failure> {
    let result = cslib::normal_at_singularity(max_order, der_nuv, MAG_TOL, position(surf, u, v)?)?;
    match result.result.normal {
        Some(normal) if result.result.status == NormalStatus::Defined => {
            Ok((normal, result.order_u, result.order_v))
        }
        _ => Err(undefined_value(u, v)),
    }
}

/// `NormalD0(U, V, Surf, Normal, OrderU, OrderV)`: the unit normal (the orders
/// are not read by the callers).
pub(crate) fn normal_d0(
    u: f64,
    v: f64,
    surf: &dyn Adaptor3dSurface,
) -> Result<Direction3, Failure> {
    let (uc, vc) = (surf.u_continuity()?, surf.v_continuity()?);
    let cont = if uc < vc { uc } else { vc };
    let d1 = surf.d1(u, v)?;
    let first = cslib::normal_with_magnitude(d1.d1u, d1.d1v, MAG_TOL)?;
    if first.status == NormalStatus::Defined {
        return first.normal.ok_or_else(|| undefined_value(u, v));
    }
    if cont == Shape::C0 || cont == Shape::C1 {
        return Err(undefined_value(u, v));
    }
    let max_order = 3;
    let mut der_surf = table(max_order + 1)?;
    for i in 1..=max_order + 1 {
        set(&mut der_surf, i, 0, surf.dn(u, v, i, 0)?)?;
    }
    for i in 0..=max_order + 1 {
        for j in 1..=max_order + 1 {
            set(&mut der_surf, i, j, surf.dn(u, v, i, j)?)?;
        }
    }
    let der_nuv = normal_derivatives(max_order, &der_surf)?;
    Ok(singular_normal(surf, max_order, &der_nuv, (u, v))?.0)
}

/// Fills `DerSurf(i, j)` and `DerSurf(j, i)` with `Surf->DN` for the orders above
/// `known` (the ones the D2 or D3 call did not give).
fn fill_high_orders(
    surf: &dyn Adaptor3dSurface,
    (u, v): (f64, f64),
    der_surf: &mut Array2<'_, Vector3>,
    last_i: i32,
    known: i32,
) -> Result<(), Failure> {
    for i in 0..=last_i {
        for j in i..=last_i + 1 {
            if i + j > known {
                set(der_surf, i, j, surf.dn(u, v, i, j)?)?;
                if i != j {
                    set(der_surf, j, i, surf.dn(u, v, j, i)?)?;
                }
            }
        }
    }
    Ok(())
}

/// `MaxOrder`: 0 when the first-derivative normal is defined, 3 otherwise.
fn max_order_of(d1u: Vector3, d1v: Vector3) -> Result<i32, Failure> {
    let first = cslib::normal_with_magnitude(d1u, d1v, MAG_TOL)?;
    Ok(if first.status == NormalStatus::Defined {
        0
    } else {
        3
    })
}

/// `NormalD1(U, V, Surf, Normal, D1UNormal, D1VNormal)`.
pub(crate) fn normal_d1(
    u: f64,
    v: f64,
    surf: &dyn Adaptor3dSurface,
) -> Result<(Direction3, Vector3, Vector3), Failure> {
    let d2 = surf.d2(u, v)?;
    let max_order = max_order_of(d2.d1u, d2.d1v)?;
    let mut der_surf = table(max_order + 2)?;
    set(&mut der_surf, 1, 0, d2.d1u)?;
    set(&mut der_surf, 0, 1, d2.d1v)?;
    set(&mut der_surf, 1, 1, d2.d2uv)?;
    set(&mut der_surf, 2, 0, d2.d2u)?;
    set(&mut der_surf, 0, 2, d2.d2v)?;
    fill_high_orders(surf, (u, v), &mut der_surf, max_order + 1, 2)?;
    let der_nuv = normal_derivatives(max_order + 1, &der_surf)?;
    let (normal, ou, ov) = singular_normal(surf, max_order, &der_nuv, (u, v))?;
    Ok((
        normal,
        cslib::dn_normal(1, 0, &der_nuv, ou, ov)?,
        cslib::dn_normal(0, 1, &der_nuv, ou, ov)?,
    ))
}

/// The normal and its derivatives of `NormalD2` (`D1UNormal`, `D1VNormal`,
/// `D2UNormal`, `D2VNormal`, `D2UVNormal`).
pub(crate) struct NormalDerivatives {
    pub normal: Direction3,
    pub d1u: Vector3,
    pub d1v: Vector3,
    pub d2u: Vector3,
    pub d2v: Vector3,
    pub d2uv: Vector3,
}

/// `NormalD2(U, V, Surf, Normal, D1UNormal, D1VNormal, D2UNormal, D2VNormal,
/// D2UVNormal)`.
pub(crate) fn normal_d2(
    u: f64,
    v: f64,
    surf: &dyn Adaptor3dSurface,
) -> Result<NormalDerivatives, Failure> {
    let d3 = surf.d3(u, v)?;
    let max_order = max_order_of(d3.d1u, d3.d1v)?;
    let mut der_surf = table(max_order + 3)?;
    for (i, j, value) in [
        (1, 0, d3.d1u),
        (0, 1, d3.d1v),
        (1, 1, d3.d2uv),
        (2, 0, d3.d2u),
        (0, 2, d3.d2v),
        (3, 0, d3.d3u),
        (2, 1, d3.d3uuv),
        (1, 2, d3.d3uvv),
        (0, 3, d3.d3v),
    ] {
        set(&mut der_surf, i, j, value)?;
    }
    fill_high_orders(surf, (u, v), &mut der_surf, max_order + 2, 3)?;
    let der_nuv = normal_derivatives(max_order + 2, &der_surf)?;
    let (normal, ou, ov) = singular_normal(surf, max_order, &der_nuv, (u, v))?;
    Ok(NormalDerivatives {
        normal,
        d1u: cslib::dn_normal(1, 0, &der_nuv, ou, ov)?,
        d1v: cslib::dn_normal(0, 1, &der_nuv, ou, ov)?,
        d2u: cslib::dn_normal(2, 0, &der_nuv, ou, ov)?,
        d2v: cslib::dn_normal(0, 2, &der_nuv, ou, ov)?,
        d2uv: cslib::dn_normal(1, 1, &der_nuv, ou, ov)?,
    })
}
