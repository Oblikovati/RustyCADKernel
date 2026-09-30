// Rust translation of Recadre, Tol3d, CoincideOnArc, VerifyTgline, GetLinePoint2d and
// FindParameter of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_RstInt.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static helpers of `IntPatch_RstInt`.
use super::super::IntPatchIType;
use super::super::h_inter_tool::IntPatchHInterTool;
use super::super::line::IntPatchLine;
use super::super::point_line::IntPatchPointLine;
use super::super::rline::IntPatchRLine;
use super::super::wline::IntPatchWLine;
use crate::int_start::{TheArc, TheTopolTool, TheVertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::f64::consts::PI;

/// The line of the vertex search: a walking or a restriction line.
pub(super) enum PointLineRef {
    Walking(Handle<IntPatchWLine>),
    Restriction(Handle<IntPatchRLine>),
}

impl PointLineRef {
    /// The line as a point line.
    pub(super) fn line(&self) -> &dyn IntPatchPointLine {
        match self {
            Self::Walking(w) => w.get().expect("walking line"),
            Self::Restriction(r) => r.get().expect("restriction line"),
        }
    }
}

/// `U` brought within 1.5 PI of the reference `up` by turns of 2 PI.
fn near(mut u: f64, up: f64) -> f64 {
    while u < (up - 1.5 * PI) {
        u += PI + PI;
    }
    while u > (up + 1.5 * PI) {
        u -= PI + PI;
    }
    u
}

fn is_periodic_u(t: SurfaceType) -> bool {
    matches!(
        t,
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere | SurfaceType::Torus
    )
}

/// `Recadre(typeS1, typeS2, wlin, Param, U1, V1, U2, V2)`: the periodic parameters near
/// those of the point `param` (clamped to the line) of the walking line. The V2 loop of
/// a torus compares with V1 of the point first, as natively.
pub(super) fn recadre(
    (type_s1, type_s2): (SurfaceType, SurfaceType),
    wlin: &IntPatchWLine,
    param: i32,
    [u1, v1, u2, v2]: [f64; 4],
) -> Result<[f64; 4], Failure> {
    let nbpnts: i32 = wlin.nb_pnts()?;
    let param: i32 = if param < 1 {
        1
    } else if param > nbpnts {
        nbpnts
    } else {
        param
    };
    let (u1p, v1p, u2p, v2p) = wlin.point(param)?.parameters();
    let mut out = [u1, v1, u2, v2];
    if is_periodic_u(type_s1) {
        out[0] = near(out[0], u1p);
    }
    if type_s1 == SurfaceType::Torus {
        out[1] = near(out[1], v1p);
    }
    if is_periodic_u(type_s2) {
        out[2] = near(out[2], u2p);
    }
    if type_s2 == SurfaceType::Torus {
        let mut v2 = out[3];
        while v2 < (v1p - 1.5 * PI) {
            v2 += PI + PI;
        }
        while v2 > (v2p + 1.5 * PI) {
            v2 -= PI + PI;
        }
        out[3] = v2;
    }
    Ok(out)
}

/// `Tol3d(vtx, Domain, tolDef)`: the 3d tolerance of the vertex, or the default one
/// (at least Precision::Confusion) on a domain without 3d data.
pub(super) fn tol3d_vertex(
    vtx: &TheVertex,
    domain: &(dyn TheTopolTool + 'static),
    tol_def: f64,
) -> Result<f64, Failure> {
    if domain.has_3d() {
        return domain.tol_3d_vertex(vtx);
    }
    Ok(if tol_def < precision::confusion() {
        precision::confusion()
    } else {
        tol_def
    })
}

/// `Tol3d(arc, Domain, tolDef)`.
pub(super) fn tol3d_arc(
    arc: &TheArc,
    domain: &(dyn TheTopolTool + 'static),
    tol_def: f64,
) -> Result<f64, Failure> {
    if domain.has_3d() {
        return domain.tol_3d_arc(arc);
    }
    Ok(if tol_def < precision::confusion() {
        precision::confusion()
    } else {
        tol_def
    })
}

/// `CoincideOnArc(Ptsommet, A, Surf, Toler, Domain, Vtx)`: the vertex of the arc nearest
/// to the point within the tolerances (the last of equal ones), written into `vtx`.
pub(super) fn coincide_on_arc(
    ptsommet: Point3,
    a: &TheArc,
    surf: &Adaptor3dSurfaceHandle,
    toler: f64,
    domain: &(dyn TheTopolTool + 'static),
    vtx: &mut TheVertex,
) -> Result<bool, Failure> {
    let mut distmin: f64 = f64::MAX;
    let tolarc: f64 = std_max(toler, tol3d_arc(a, domain, 0.0)?);
    domain.initialize_arc(a)?;
    domain.init_vertex_iterator()?;
    while domain.more_vertex() {
        let vtx1: TheVertex = domain.vertex()?;
        let prm: f64 = IntPatchHInterTool::parameter(&vtx1, a)?;
        let p2d: Point2 = a.value(prm)?;
        let point: Point3 = surf.value(p2d.x(), p2d.y())?;
        let dist: f64 = point.distance(&ptsommet);
        let tol: f64 = std_max(tolarc, tol3d_vertex(&vtx1, domain, 0.0)?);
        if dist <= tol && dist <= distmin {
            // The best coincidence.
            distmin = dist;
            *vtx = vtx1;
        }
        domain.next_vertex()?;
    }
    Ok(distmin < f64::MAX)
}

/// Whether the three coordinates of `t` are under `tol`.
fn is_small(t: Vector3, tol: f64) -> bool {
    t.x().abs() < tol && t.y().abs() < tol && t.z().abs() < tol
}

/// `VerifyTgline(wlin, param, Tol, Tgl)`: a null tangent becomes the chord to the
/// nearest point of the line not within `tol` (to avoid very close walking points),
/// toward the farther end first.
pub(super) fn verify_tgline(
    wlin: &IntPatchWLine,
    param: i32,
    tol: f64,
    tgl: &mut Vector3,
) -> Result<(), Failure> {
    if !is_small(*tgl, tol) {
        return Ok(());
    }
    let nbpt: i32 = wlin.nb_pnts()?;
    let mut forward: bool = (nbpt - param) >= (param - 1);
    for _ in 0..2 {
        let found = if forward {
            chord(wlin, param, (param + 1..=nbpt).collect(), tol, true)?
        } else {
            chord(wlin, param, (1..param).rev().collect(), tol, false)?
        };
        if let Some(t) = found {
            *tgl = t;
            return Ok(());
        }
        forward = !forward;
    }
    Ok(())
}

/// The first chord of the point `param` to the points `others` that is not small.
fn chord(
    wlin: &IntPatchWLine,
    param: i32,
    others: Vec<i32>,
    tol: f64,
    forward: bool,
) -> Result<Option<Vector3>, Failure> {
    let p: Point3 = wlin.point(param)?.value();
    for i in others {
        let q: Point3 = wlin.point(i)?.value();
        let t = if forward {
            Vector3::between(p, q)
        } else {
            Vector3::between(q, p)
        };
        if !is_small(t, tol) {
            return Ok(Some(t));
        }
    }
    Ok(None)
}

/// `GetLinePoint2d(L, param, OnFirst, U, V)`: the parameters on one surface of the
/// point of the line of (real) index `param`, interpolated between its neighbours.
pub(super) fn get_line_point_2d(
    l: &PointLineRef,
    param: f64,
    on_first: bool,
) -> Result<(f64, f64), Failure> {
    let line: &dyn IntPatchPointLine = l.line();
    let nbptlin: i32 = line.nb_pnts()?;
    let mut par: f64 = param.trunc();
    let mut irang: i32 = par as i32;
    if irang == nbptlin {
        irang -= 1;
        par = 1.0;
    } else {
        par = (param - par).abs();
    }
    let (us1, vs1) = line.point(irang)?.parameters_on_surface(on_first);
    let (us2, vs2) = line.point(irang + 1)?.parameters_on_surface(on_first);
    Ok(((1.0 - par) * us1 + par * us2, (1.0 - par) * vs1 + par * vs2))
}

/// The inputs of `FindParameter`.
pub(super) struct FindInput<'a> {
    pub(super) other_surf: &'a Adaptor3dSurfaceHandle,
    pub(super) tol: f64,
    pub(super) ptsom: Point3,
    pub(super) ptsom2d: Point2,
    pub(super) param_approche: i32,
    pub(super) on_first: bool,
}

/// `FindParameter(L, OtherSurf, Tol, Ptsom, Ptsom2d, Param, Tgl, ParamApproche,
/// OnFirst)`: the parameter of the point on the line (the arc parameter of the
/// projection on a restriction line of the other surface; the interpolated index of the
/// crossing, or the nearest index, on a walking line, searched near `ParamApproche`,
/// then toward the nearer end, then the farther one) and the tangent there.
pub(super) fn find_parameter(
    l: &PointLineRef,
    input: &FindInput<'_>,
    param: &mut f64,
    tgl: &mut Vector3,
) -> Result<bool, Failure> {
    *tgl = Vector3::new(0.0, 0.0, 0.0);
    match l {
        PointLineRef::Restriction(rlin) => find_on_restriction(rlin, input, param, tgl),
        PointLineRef::Walking(wlin) => find_on_walking(wlin, input, param, tgl),
    }
}

fn find_on_restriction(
    rlin: &IntPatchRLine,
    input: &FindInput<'_>,
    param: &mut f64,
    tgl: &mut Vector3,
) -> Result<bool, Failure> {
    let arc: TheArc = if !input.on_first && rlin.is_arc_on_s1() {
        rlin.arc_on_s1()
    } else if input.on_first && rlin.is_arc_on_s2() {
        rlin.arc_on_s2()
    } else {
        return Ok(false);
    };
    // The native result of Project is not tested: the parameter is kept when the
    // projection fails.
    if let Some((prm, _)) = IntPatchHInterTool::project(&arc, input.ptsom2d)? {
        *param = prm;
    }
    let (p2d, d2d): (Point2, Vector2) = arc.d1(*param)?;
    let d1 = input.other_surf.d1(p2d.x(), p2d.y())?;
    if d1.point.square_distance(&input.ptsom) > input.tol * input.tol {
        return Ok(false);
    }
    tgl.set_linear_form(d2d.x(), d1.d1u, d2d.y(), d1.d1v);
    Ok(true)
}

/// The search ranges of a walking line: near the approximate index, then toward the
/// nearer end, then the farther one.
fn search_ranges(nbpt: i32, param_approche: i32) -> [(i32, i32); 3] {
    let mut inf: i32 = 1;
    let mut sup: i32 = nbpt;
    if (param_approche - 2) > inf {
        inf = param_approche - 2;
    }
    if (param_approche + 2) < sup {
        sup = param_approche + 2;
    }
    if inf - 1 < nbpt - sup {
        return [(inf, sup), (1, inf), (sup, nbpt)];
    }
    [(inf, sup), (sup, nbpt), (1, inf)]
}

fn find_on_walking(
    wlin: &IntPatchWLine,
    input: &FindInput<'_>,
    param: &mut f64,
    tgl: &mut Vector3,
) -> Result<bool, Failure> {
    let tol2: f64 = input.tol * input.tol;
    let nbpt: i32 = wlin.nb_pnts()?;
    for (inf, sup) in search_ranges(nbpt, input.param_approche) {
        let mut p1: Point3 = wlin.point(inf)?.value();
        let mut v1 = Vector3::between(input.ptsom, p1);
        let mut norm1: f64 = v1.square_magnitude();
        let mut normmin: f64 = tol2;
        let mut ibest: i32 = 0;
        if norm1 <= normmin {
            normmin = norm1;
            ibest = inf;
        }
        for i in inf + 1..=sup {
            let p2: Point3 = wlin.point(i)?.value();
            let v2 = Vector3::between(input.ptsom, p2);
            let norm2: f64 = v2.square_magnitude();
            if v1.dot(v2) < 0.0 {
                *param = f64::from(i - 1) + 1.0 / (1.0 + (norm2 / norm1).sqrt());
                *tgl = Vector3::between(p1, p2);
                return Ok(true);
            } else if norm2 < normmin {
                normmin = norm2;
                ibest = i;
            }
            (v1, p1, norm1) = (v2, p2, norm2);
        }
        if ibest != 0 {
            *param = f64::from(ibest);
            return Ok(true);
        }
    }
    Ok(false)
}

/// The Standard_DomainError of a line that is neither walking nor restriction.
pub(super) fn not_point_line(typ: IntPatchIType) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!(
            "IntPatch_RstInt::PutVertexOnLine() - a line of type {typ:?}; expected a walking or restriction line"
        ),
    )
}

/// The walking or restriction line of the handle.
pub(super) fn point_line_of(l: &Handle<dyn IntPatchLine>) -> Result<PointLineRef, Failure> {
    match l.arc_type() {
        IntPatchIType::Walking => Ok(PointLineRef::Walking(l.cast())),
        IntPatchIType::Restriction => Ok(PointLineRef::Restriction(l.cast())),
        typ => Err(not_point_line(typ)),
    }
}
