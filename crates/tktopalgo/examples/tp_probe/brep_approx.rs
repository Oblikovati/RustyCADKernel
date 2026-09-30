//! BRepApprox probe operations, the Rust twin of tools/oracle/tktopalgo_probe_brepapprox.cxx
//! (docs/topalgo-design.md section 12): the walking lines of IntPatch_Intersection on two
//! faces read by `tp_face`, approximated by BRepApprox_Approx, and the approximation lines
//! of points and of curve poles.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real};
use super::face::tp_face;
use super::int_walk::pnt2s;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkgeomalgo::approx_int::ApproxIntRange;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchIntersection, IntPatchIntersectionOptions, IntPatchPointLine, IntPatchWLine,
};
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfQuadric};
use rustycad_tkgeombase::app_par_curves::{MultiBSpCurve, MultiCurveEval};
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_approx::{BRepApproxApprox, BRepApproxApproxLine};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;

/// `TpBraFace`: the face adaptor (restricted or not) and its tool.
pub struct BraFace {
    pub s: Handle<BRepAdaptorSurface>,
    pub d: BRepTopAdaptorTopolTool,
}

/// `tp_bra_face`: `face restr`.
pub fn bra_face(r: &mut Reader) -> Result<BraFace, Failure> {
    let f = tp_face(r)?;
    let restr = r.flag();
    let s = Handle::new(BRepAdaptorSurface::with_face(&f.face, restr)?);
    let d = BRepTopAdaptorTopolTool::with_surface(&s.clone().cast())?;
    Ok(BraFace { s, d })
}

/// `TpBraWalk`: the two faces and the walking lines of their intersection.
pub struct BraWalk {
    pub a: BraFace,
    pub b: BraFace,
    pub lines: Vec<Handle<IntPatchWLine>>,
}

/// `tp_bra_walk`: `face1 restr1 face2 restr2 tolarc toltang geomint`.
pub fn bra_walk(r: &mut Reader) -> Result<BraWalk, Failure> {
    let a = bra_face(r)?;
    let b = bra_face(r)?;
    let tols = (r.real(), r.real());
    let options = IntPatchIntersectionOptions {
        is_geom_int: r.flag(),
        is_req_to_keep_rline: false,
        is_req_to_post_wl_proc: true,
    };
    let (sa, sb): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
        (a.s.clone().cast(), b.s.clone().cast());
    let mut inter = IntPatchIntersection::new();
    inter.perform(&sa, &a.d, &sb, &b.d, tols, options)?;
    let mut lines = Vec::new();
    for i in 1..=inter.nb_lines()? {
        let l: Handle<IntPatchWLine> = inter.line(i)?.cast();
        if !l.is_null() {
            lines.push(l);
        }
    }
    int(lines.len() as i32);
    Ok(BraWalk { a, b, lines })
}

/// `tp_bra_line`: the approximation line of walking line `k`, null past the last one.
pub fn bra_line(w: &BraWalk, k: i32) -> Handle<BRepApproxApproxLine> {
    if k < 1 || k as usize > w.lines.len() {
        return Handle::default();
    }
    let curve: Handle<IntSurfLineOn2S> = w.lines[k as usize - 1].curve();
    Handle::new(BRepApproxApproxLine::from_line(&curve, false))
}

/// `tp_bra_range`: None (and -1 printed) for a null line.
pub fn bra_range(r: &mut Reader, l: &Handle<BRepApproxApproxLine>) -> Option<(i32, i32)> {
    let kind = r.integer();
    let (mut a, mut b) = (0, 0);
    if kind == 1 {
        a = r.integer();
        b = r.integer();
    }
    if l.is_null() {
        int(-1);
        return None;
    }
    if kind == 0 {
        a = 1;
        b = l.nb_pnts().expect("probe points");
    }
    Some((a, b))
}

fn one_based<T: Default + Clone>(n: i32) -> Array1<'static, T> {
    Array1::with_bounds(1, n).expect("probe array")
}

/// `tp_bra_mbsp`.
fn mbsp(m: &MultiBSpCurve) -> Result<(), Failure> {
    let np = m.nb_poles();
    int(np);
    if np == 0 {
        return Ok(());
    }
    attempt(|| MultiCurveEval::degree(m).map(int));
    attempt(|| {
        let k = m.knots()?;
        int(k.length());
        k.iter().copied().for_each(real);
        Ok(())
    });
    attempt(|| {
        m.multiplicities()?.iter().copied().for_each(int);
        Ok(())
    });
    let nc = m.nb_curves()?;
    int(nc);
    for c in 1..=nc {
        mbsp_curve(m, c, np)?;
    }
    Ok(())
}

/// The dimension and the poles of curve `c` of a multi-curve.
fn mbsp_curve(m: &MultiBSpCurve, c: i32, np: i32) -> Result<(), Failure> {
    let d = m.dimension(c)?;
    int(d);
    if d == 3 {
        let mut p: Array1<'static, Point3> = one_based(np);
        m.curve_3d(c, &mut p)?;
        p.iter().copied().for_each(super::common::pnt);
    } else {
        let mut p: Array1<'static, Point2> = one_based(np);
        m.curve_2d(c, &mut p)?;
        p.iter().copied().for_each(pnt2d);
    }
    Ok(())
}

pub fn partype(k: i32) -> ParametrizationType {
    match k {
        0 => ParametrizationType::ChordLength,
        1 => ParametrizationType::Centripetal,
        _ => ParametrizationType::IsoParametric,
    }
}

/// `tp_bra_settings`.
fn settings(r: &mut Reader, a: &mut BRepApproxApprox) {
    if !r.flag() {
        return;
    }
    let (t3, t2) = (r.real(), r.real());
    let (d0, d1, it, np) = (r.integer(), r.integer(), r.integer(), r.integer());
    let tg = r.flag();
    a.set_parameters(t3, t2, d0, d1, it, np, tg, partype(r.integer()));
}

/// `tp_bra_result`.
fn result(a: &BRepApproxApprox) {
    flag(a.is_done());
    real(a.tol_reached3d());
    real(a.tol_reached2d());
    int(a.nb_multi_curves());
    attempt(|| mbsp(a.value(1)));
}

fn range_of((x, u1, u2): (bool, bool, bool), (lo, hi): (i32, i32)) -> ApproxIntRange {
    ApproxIntRange {
        approx_xyz: x,
        approx_u1v1: u1,
        approx_u2v2: u2,
        indicemin: lo,
        indicemax: hi,
    }
}

/// One line of `tp_bra_approx_case`.
fn approx_line(
    r: &mut Reader,
    w: &BraWalk,
    a: &mut BRepApproxApprox,
    (spaces, mode): ((bool, bool, bool), i32),
) -> Result<(), Failure> {
    let l = bra_line(w, r.integer());
    let Some(bounds) = bra_range(r, &l) else {
        return Ok(());
    };
    int(l.nb_pnts()?);
    let range = range_of(spaces, bounds);
    if mode == 0 {
        attempt(|| a.perform_on_surfaces(&w.a.s, &w.b.s, &l, range));
    } else {
        attempt(|| a.perform(&l, range));
    }
    result(a);
    Ok(())
}

/// `bra_approx`: `walk settings xyz u1v1 u2v2 mode n (line range)...`.
pub fn approx_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let w = bra_walk(&mut r)?;
        let mut a = BRepApproxApprox::new();
        settings(&mut r, &mut a);
        let spaces = (r.flag(), r.flag(), r.flag());
        let (mode, n) = (r.integer(), r.integer());
        for _ in 0..n {
            approx_line(&mut r, &w, &mut a, (spaces, mode))?;
        }
        Ok(())
    });
    end();
}

/// The knots and multiplicities `nk (knot mult)...` of a B-spline record.
fn knots(r: &mut Reader) -> (Array1<'static, f64>, Array1<'static, i32>) {
    let nk = r.integer();
    let (mut k, mut m): (Array1<'static, f64>, Array1<'static, i32>) =
        (one_based(nk), one_based(nk));
    for i in 1..=nk {
        k[i] = r.real();
        m[i] = r.integer();
    }
    (k, m)
}

/// `tp_bra_bspline`: `present [degree n (x y z)... nk (knot mult)...]`.
pub fn bra_bspline(r: &mut Reader) -> Result<Handle<GeomBSplineCurve>, Failure> {
    if !r.flag() {
        return Ok(Handle::default());
    }
    let (deg, n) = (r.integer(), r.integer());
    let mut p: Array1<'static, Point3> = one_based(n);
    for i in 1..=n {
        p[i] = r.pnt();
    }
    let (k, m) = knots(r);
    Ok(Handle::new(GeomBSplineCurve::new(&p, &k, &m, deg, false)?))
}

/// `tp_bra_bspline2d`: `present [degree n (u v)... nk (knot mult)...]`.
fn bra_bspline2d(r: &mut Reader) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    if !r.flag() {
        return Ok(Handle::default());
    }
    let (deg, n) = (r.integer(), r.integer());
    let mut p: Array1<'static, Point2> = one_based(n);
    for i in 1..=n {
        p[i] = r.pnt2d();
    }
    let (k, m) = knots(r);
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &p, &k, &m, deg, false,
    )?))
}

/// The line of `tp_bra_line_case`: points or curve poles.
fn read_line(r: &mut Reader) -> Result<Handle<BRepApproxApproxLine>, Failure> {
    if r.integer() != 0 {
        let c = bra_bspline(r)?;
        let c1 = bra_bspline2d(r)?;
        let c2 = bra_bspline2d(r)?;
        return Ok(Handle::new(BRepApproxApproxLine::from_curves(&c, &c1, &c2)));
    }
    let s = Handle::new(IntSurfLineOn2S::new(None));
    for _ in 0..r.integer() {
        let p = r.pnt();
        let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
        let mut q = IntSurfPntOn2S::new();
        q.set_value_all(p, a, b, c, d);
        s.add(&q);
    }
    Ok(Handle::new(BRepApproxApproxLine::from_line(&s, r.flag())))
}

/// `bra_line`: `kind (0 n (x y z u1 v1 u2 v2)... tang | 1 curve3d curve2d curve2d)
/// settings xyz u1v1 u2v2 range`.
pub fn line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let l = read_line(&mut r)?;
        let n = l.nb_pnts()?;
        int(n);
        for i in 0..=n + 1 {
            attempt(|| l.point(i).map(|p| pnt2s(&p)));
        }
        let mut a = BRepApproxApprox::new();
        settings(&mut r, &mut a);
        let spaces = (r.flag(), r.flag(), r.flag());
        let bounds = bra_range(&mut r, &l).unwrap_or((0, 0));
        attempt(|| a.perform(&l, range_of(spaces, bounds)));
        result(&a);
        Ok(())
    });
    end();
}

/// `tp_bra_quadric`.
pub fn bra_quadric(s: &BRepAdaptorSurface) -> Result<IntSurfQuadric, Failure> {
    let mut q = IntSurfQuadric::new();
    match s.get_type()? {
        SurfaceType::Plane => q.set_plane(&s.plane()?),
        SurfaceType::Cylinder => q.set_cylinder(&s.cylinder()?),
        SurfaceType::Sphere => q.set_sphere(&s.sphere()?),
        SurfaceType::Cone => q.set_cone(&s.cone()?),
        _ => {}
    }
    Ok(q)
}
