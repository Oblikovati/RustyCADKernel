// Rust translation of BRepFill_LocationLaw::TransformInG0Law, TransformInCompatibleLaw,
// DeleteTransform, NbHoles, IsG1 and the static Norm and ToG0
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_LocationLaw.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The continuity of a location law at the vertices of its path, and the placements
//! that make consecutive laws continue each other.
use super::BRepFillLocationLawBase;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_fill::{GeomFillLocationLaw, LocationD0, LocationD1};
use rustycad_tkmath::gp::{Axis1, Direction3, Matrix3, Point2, Point3, Vector2, Vector3, Xyz};

/// `Norm(M)`: the largest sum of the absolute values of a row.
fn norm(m: &Matrix3) -> Result<f64, Failure> {
    let mut norme: f64 = 0.;
    for i in 1..=3 {
        let coord: Xyz = m.row(i)?;
        let r: f64 = coord.x().abs() + coord.y().abs() + coord.z().abs();
        if i == 1 || r > norme {
            norme = r;
        }
    }
    Ok(norme)
}

/// `ToG0(M1, M2, T)`: `T` such that `T.M2 = M1` (`T = M2^-1 * M1`).
pub(super) fn to_g0(m1: &Matrix3, m2: &Matrix3) -> Result<Matrix3, Failure> {
    let mut t: Matrix3 = m2.inverted()?;
    t *= *m1;
    Ok(t)
}

/// The placement of `law` at `param` written over `placement` (unchanged when the law
/// fails, as natively).
fn d0_at(
    law: &dyn GeomFillLocationLaw,
    param: f64,
    placement: &mut LocationD0,
) -> Result<(), Failure> {
    law.d0(param, placement)?;
    Ok(())
}

/// The placements at the junction of the laws `previous` and `next`: the end of the
/// first (on the domain end carried over in `last`) and the start of the second.
pub(super) fn junction(
    base: &BRepFillLocationLawBase,
    previous: i32,
    next: i32,
    last: &mut f64,
    m1: &mut LocationD0,
    m2: &mut LocationD0,
) -> Result<(), Failure> {
    d0_at(base.law_at(previous)?, *last, m1)?;
    let (first, new_last) = base.law_at(next)?.get_domain()?;
    *last = new_last;
    d0_at(base.law_at(next)?, first, m2)
}

/// `TransformInG0Law()`.
pub(super) fn transform_in_g0_law(base: &BRepFillLocationLawBase) -> Result<(), Failure> {
    let n: i32 = base.count()?;
    let (_, mut last) = base.law_at(1)?.get_domain()?;
    let (mut m1, mut m2) = (LocationD0::default(), LocationD0::default());
    for ipath in 2..=n {
        junction(base, ipath - 1, ipath, &mut last, &mut m1, &mut m2)?;
        base.law_at(ipath)?.set_trsf(&to_g0(&m1.m, &m2.m)?)?;
    }
    // Is the law periodical? The placements are computed and not used, as natively.
    if base.path.closed()? {
        junction(base, n, 1, &mut last, &mut m1, &mut m2)?;
    }
    Ok(())
}

/// `TransformInCompatibleLaw(AngularTolerance)`: the G0 placement when the tangents
/// agree, else the rotation about the tangent that brings the normal of the next law
/// (turned by the tangent change) onto the normal of the previous one.
pub(super) fn transform_in_compatible_law(
    base: &BRepFillLocationLawBase,
    tol_angular: f64,
) -> Result<(), Failure> {
    let (_, mut last) = base.law_at(1)?.get_domain()?;
    let (mut m1, mut m2) = (LocationD0::default(), LocationD0::default());
    let mut trsf = Matrix3::default();
    for ipath in 2..=base.count()? {
        junction(base, ipath - 1, ipath, &mut last, &mut m1, &mut m2)?;
        let t1 = Vector3::from(m1.m.column(3)?);
        let t2 = Vector3::from(m2.m.column(3)?);
        let n1 = Vector3::from(m1.m.column(1)?);
        let mut n2 = Vector3::from(m2.m.column(1)?);
        if t1.is_parallel(t2, tol_angular)? && !t1.is_opposite(t2, tol_angular)? {
            // Correction G0.
            trsf = to_g0(&m1.m, &m2.m)?;
        } else {
            let cross: Vector3 = t1.crossed(t2);
            let alpha: f64 = t2.angle_with_ref(t1, cross)?;
            let axe = Axis1::new(Point3::new(0., 0., 0.), Direction3::from_xyz(*cross.xyz())?);
            n2.rotate(axe, alpha)?;
            let angle: f64 = n2.angle_with_ref(n1, t1)?;
            trsf.set_rotation(Xyz::new(0., 0., 1.), angle)?;
        }
        base.law_at(ipath)?.set_trsf(&trsf)?;
    }
    Ok(())
}

/// `DeleteTransform()`.
pub(super) fn delete_transform(base: &BRepFillLocationLawBase) -> Result<(), Failure> {
    let mut id = Matrix3::default();
    id.set_identity();
    for ii in 1..=base.edges.len() as i32 {
        base.law_at(ii)?.set_trsf(&id)?;
    }
    *base.disc.write() = None;
    Ok(())
}

/// `NbHoles(Tol)`: the vertices `2..=count + 1` (of the junction before them) where the
/// law is not G0, searched while none is recorded.
pub(super) fn nb_holes(base: &BRepFillLocationLawBase, tol: f64) -> Result<i32, Failure> {
    if base.disc.read().is_none() {
        let mut seq: Vec<i32> = Vec::new();
        for ii in 2..=base.count()? + 1 {
            if is_g1(base, ii - 1, tol, 1.0e-12)? == -1 {
                seq.push(ii);
            }
        }
        if !seq.is_empty() {
            *base.disc.write() = Some(seq);
        }
    }
    Ok(base.disc.read().as_ref().map_or(0, |d| d.len() as i32))
}

/// `D1(param)` of `law` over `d1`; whether it succeeded.
fn d1_at(law: &dyn GeomFillLocationLaw, param: f64, d1: &mut LocationD1) -> Result<bool, Failure> {
    let mut bid1 = one_based::<Point2>(1)?;
    let mut bid2 = one_based::<Vector2>(1)?;
    law.d1(param, d1, &mut bid1, &mut bid2)
}

/// `D0(param)` of `law` over the value and translation of `d1`.
fn d0_over(law: &dyn GeomFillLocationLaw, param: f64, d1: &mut LocationD1) -> Result<(), Failure> {
    let mut d0 = LocationD0 { m: d1.m, v: d1.v };
    law.d0(param, &mut d0)?;
    (d1.m, d1.v) = (d0.m, d0.v);
    Ok(())
}

/// The placements and derivatives at the end of law `i1` and the start of law `i2`
/// (D0 where D1 fails), and whether the derivatives are known. At the closing vertex
/// the D1 of the second law does not update the flag, as natively.
fn junction_d1(
    base: &BRepFillLocationLawBase,
    i1: i32,
    i2: i32,
    closing: bool,
) -> Result<(LocationD1, LocationD1, bool), Failure> {
    let (mut p1, mut p2) = (LocationD1::default(), LocationD1::default());
    let law1 = base.law_at(i1)?;
    let (_, last) = law1.get_domain()?;
    let mut ok_d1: bool = d1_at(law1, last, &mut p1)?;
    if !ok_d1 {
        d0_over(law1, last, &mut p1)?;
    }
    let law2 = base.law_at(i2)?;
    let (first, _) = law2.get_domain()?;
    if ok_d1 {
        let second: bool = d1_at(law2, first, &mut p2)?;
        if !closing {
            ok_d1 = second;
        }
    }
    if !ok_d1 {
        d0_over(law2, first, &mut p2)?;
    }
    Ok((p1, p2, ok_d1))
}

/// `IsG1(Index, SpatialTolerance, AngularTolerance)`.
pub(super) fn is_g1(
    base: &BRepFillLocationLawBase,
    index: i32,
    spatial_tolerance: f64,
    angular_tolerance: f64,
) -> Result<i32, Failure> {
    let n: i32 = base.count()?;
    let mut found: (LocationD1, LocationD1, bool, Edge) = (
        LocationD1::default(),
        LocationD1::default(),
        false,
        Edge::default(),
    );
    if index > 0 && index < n {
        let (p1, p2, ok) = junction_d1(base, index, index + 1, false)?;
        found = (p1, p2, ok, base.edge_at(index + 1)?);
    }
    if index == 0 || index == n {
        if !base.path.closed()? {
            return Ok(-1);
        }
        let (p1, p2, ok) = junction_d1(base, n, 1, true)?;
        found = (p1, p2, ok, base.edge_at(1)?);
    }
    let (p1, p2, ok_d1, edge) = found;
    let v: Vertex = if edge.orientation() == Orientation::Reversed {
        topexp::last_vertex(&edge, false)
    } else {
        topexp::first_vertex(&edge, false)
    };
    let tol_eps: f64 = spatial_tolerance + 2. * BRepTool::vertex_tolerance(&v)?;
    classify(
        &p1,
        &p2,
        ok_d1,
        (tol_eps, spatial_tolerance, angular_tolerance),
    )
}

/// The G0 and G1 tests of IsG1 on the two placements.
fn classify(
    p1: &LocationD1,
    p2: &LocationD1,
    ok_d1: bool,
    tols: (f64, f64, f64),
) -> Result<i32, Failure> {
    const EPS_NUL: f64 = 1.0e-12;
    let (tol_eps, spatial_tolerance, angular_tolerance) = tols;
    let mut is_g0: bool = true;
    if (p1.v - p2.v).magnitude() > tol_eps {
        is_g0 = false;
    }
    if norm(&(p1.m - p2.m))? > spatial_tolerance {
        is_g0 = false;
    }
    if !is_g0 {
        return Ok(-1);
    }
    if !ok_d1 {
        // No control of the derivative.
        return Ok(0);
    }
    let mut is_g1: bool = true;
    if p1.dv.magnitude() > EPS_NUL
        && p2.dv.magnitude() > EPS_NUL
        && p1.dv.angle(p2.dv)? > angular_tolerance
    {
        is_g1 = false;
    }
    // For the next, the tests are mostly empirical.
    let (norm1, norm2) = (norm(&p1.dm)?, norm(&p2.dm)?);
    // When both norms are null, it is good.
    if norm1 > EPS_NUL || norm2 > EPS_NUL {
        if norm1 > EPS_NUL && norm2 > EPS_NUL {
            // The normalized matrices are compared.
            if norm(&(p1.dm.divided(norm1)? - p2.dm.divided(norm2)?))? > angular_tolerance {
                is_g1 = false;
            }
        } else {
            // One null, the other is not.
            is_g1 = false;
        }
    }
    Ok(if is_g1 { 1 } else { 0 })
}
