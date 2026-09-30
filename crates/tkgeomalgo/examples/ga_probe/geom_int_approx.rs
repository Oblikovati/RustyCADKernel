//! ApproxInt and GeomInt_WLApprox probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geomint_approx.cxx.
use super::common::{Reader, attempt, captured, end, flag, int, pnt, pnt2d, real, xy, xyz};
use super::geom_int::{GiInter, gi_inter};
use super::int_walk::pnt2s;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::approx_int::{
    ApproxIntKnotPointSets, ApproxIntKnotTools, ApproxIntMultiLineLayout, ApproxIntOffsets,
    ApproxIntRange, ApproxIntSvComputed, PatchBounds, SvSurfacesRef,
};
use rustycad_tkgeomalgo::geom_int::{
    GeomIntTheImpPrmSvSurfacesOfWLApprox, GeomIntTheMultiLineOfWLApprox,
    GeomIntThePrmPrmSvSurfacesOfWLApprox, GeomIntWLApprox,
};
use rustycad_tkgeomalgo::int_patch::{IntPatchPointLine, IntPatchWLine};
use rustycad_tkgeomalgo::int_surf::IntSurfQuadric;
use rustycad_tkgeombase::app_par_curves::{LineTool, MultiBSpCurve, MultiCurveEval};
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::Vector;
use std::cell::RefCell;
use std::rc::Rc;

fn one_based<T: Default + Clone>(n: i32) -> Array1<'static, T> {
    Array1::with_bounds(1, n).expect("probe array")
}

/// `ga_gi_mbsp`.
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
        let d = m.dimension(c)?;
        int(d);
        if d == 3 {
            let mut p: Array1<'static, Point3> = one_based(np);
            m.curve_3d(c, &mut p)?;
            p.iter().copied().for_each(pnt);
        } else {
            let mut p: Array1<'static, Point2> = one_based(np);
            m.curve_2d(c, &mut p)?;
            p.iter().copied().for_each(pnt2d);
        }
    }
    Ok(())
}

/// `ga_gi_wline`.
fn gi_wline(g: &GiInter, k: i32) -> Handle<IntPatchWLine> {
    let mut seen = 0;
    for i in 1..=g.inter.nb_lines().unwrap_or(0) {
        let w: Handle<IntPatchWLine> = g.inter.line(i).expect("probe line").cast();
        if !w.is_null() {
            seen += 1;
            if seen == k {
                return w;
            }
        }
    }
    Handle::default()
}

/// `ga_gi_range`: None for a null line.
fn gi_range(r: &mut Reader, w: &Handle<IntPatchWLine>) -> Option<(i32, i32)> {
    let kind = r.integer();
    let (mut a, mut b) = (0, 0);
    if kind == 1 {
        a = r.integer();
        b = r.integer();
    }
    if w.is_null() {
        int(-1);
        return None;
    }
    if kind == 0 {
        a = 1;
        b = w.nb_pnts().expect("probe points");
    }
    Some((a, b))
}

/// `ga_gi_wlapprox_case`.
pub fn wlapprox_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let g = gi_inter(&mut r)?;
        let mut a = GeomIntWLApprox::new();
        if r.flag() {
            let (t3, t2) = (r.real(), r.real());
            let (d0, d1, it, np) = (r.integer(), r.integer(), r.integer(), r.integer());
            let tg = r.flag();
            let par = partype(r.integer());
            a.set_parameters(t3, t2, d0, d1, it, np, tg, par);
        }
        let (x, u1, u2) = (r.flag(), r.flag(), r.flag());
        let (mode, nl) = (r.integer(), r.integer());
        for k in 1..=nl {
            let w = gi_wline(&g, k);
            let Some((lo, hi)) = gi_range(&mut r, &w) else {
                continue;
            };
            int(w.nb_pnts()?);
            let range = ApproxIntRange {
                approx_xyz: x,
                approx_u1v1: u1,
                approx_u2v2: u2,
                indicemin: lo,
                indicemax: hi,
            };
            if mode == 0 {
                attempt(|| a.perform_on_surfaces(&g.s1, &g.s2, &w, range));
            } else {
                attempt(|| a.perform(&w, range));
            }
            flag(a.is_done());
            real(a.tol_reached3d());
            real(a.tol_reached2d());
            int(a.nb_multi_curves());
            attempt(|| mbsp(a.value(1)));
        }
        Ok(())
    });
    end();
}

fn partype(k: i32) -> ParametrizationType {
    match k {
        0 => ParametrizationType::ChordLength,
        1 => ParametrizationType::Centripetal,
        _ => ParametrizationType::IsoParametric,
    }
}

/// `ga_gi_quadric`.
fn gi_quadric(s: &Adaptor3dSurfaceHandle) -> Result<IntSurfQuadric, Failure> {
    let mut q = IntSurfQuadric::new();
    match s.get_type()? {
        SurfaceType::Plane => q.set_plane(&s.plane()?),
        SurfaceType::Cylinder => q.set_cylinder(&s.cylinder()?),
        SurfaceType::Sphere => q.set_sphere(&s.sphere()?),
        SurfaceType::Cone => q.set_cone(&s.cone()?),
        SurfaceType::Torus => q.set_torus(&s.torus()?),
        _ => {}
    }
    Ok(q)
}

/// `ga_gi_sv`: the SvSurfaces of kind 1 to 3 (with the ImpPrm one kept apart for
/// FillInitialVectorOfSolution).
enum Sv {
    None,
    PrmPrm(Rc<RefCell<GeomIntThePrmPrmSvSurfacesOfWLApprox>>),
    ImpPrm(Rc<RefCell<GeomIntTheImpPrmSvSurfacesOfWLApprox>>),
}

impl Sv {
    fn shared(&self) -> Option<SvSurfacesRef> {
        match self {
            Sv::None => None,
            Sv::PrmPrm(s) => Some(s.clone()),
            Sv::ImpPrm(s) => Some(s.clone()),
        }
    }
}

fn gi_sv(g: &GiInter, kind: i32) -> Result<Sv, Failure> {
    Ok(match kind {
        1 => Sv::PrmPrm(Rc::new(RefCell::new(
            GeomIntThePrmPrmSvSurfacesOfWLApprox::new(&g.s1, &g.s2)?,
        ))),
        2 => Sv::ImpPrm(Rc::new(RefCell::new(
            GeomIntTheImpPrmSvSurfacesOfWLApprox::quadric_first(&gi_quadric(&g.s1)?, &g.s2),
        ))),
        3 => Sv::ImpPrm(Rc::new(RefCell::new(
            GeomIntTheImpPrmSvSurfacesOfWLApprox::quadric_second(&g.s1, &gi_quadric(&g.s2)?),
        ))),
        _ => Sv::None,
    })
}

/// `ga_gi_ml_value`.
fn ml_value(m: &GeomIntTheMultiLineOfWLApprox, i: i32) {
    let (n3, n2) = (m.nb_p3d(), m.nb_p2d());
    let mut p: Array1<'static, Point3> = one_based(n3.max(1));
    let mut q: Array1<'static, Point2> = one_based(n2.max(1));
    attempt(|| {
        if n3 != 0 && n2 != 0 {
            LineTool::value(m, i, &mut p, &mut q)?;
        } else if n2 != 0 {
            m.value_2d(i, &mut q)?;
        } else if n3 != 0 {
            m.value_3d(i, &mut p)?;
        }
        p.iter().copied().for_each(pnt);
        q.iter().copied().for_each(pnt2d);
        Ok(())
    });
}

/// `ga_gi_ml_tangency`.
fn ml_tangency(m: &GeomIntTheMultiLineOfWLApprox, i: i32) {
    let (n3, n2) = (m.nb_p3d().max(1), m.nb_p2d().max(1));
    let mut v: Array1<'static, Vector3> = one_based(n3);
    let mut w: Array1<'static, Vector2> = one_based(n2);
    attempt(|| {
        flag(m.tangency_3d(i, &mut v)?);
        v.iter().for_each(|x| xyz(*x.xyz()));
        Ok(())
    });
    attempt(|| {
        flag(m.tangency_2d(i, &mut w)?);
        w.iter().for_each(|x| xy(x.xy()));
        Ok(())
    });
    attempt(|| {
        flag(m.tangency(i, &mut v, &mut w)?);
        v.iter().for_each(|x| xyz(*x.xyz()));
        w.iter().for_each(|x| xy(x.xy()));
        Ok(())
    });
    attempt(|| LineTool::curvature_3d(m, i, &mut v).map(flag));
}

/// `ga_gi_ml_report`.
fn ml_report(m: &GeomIntTheMultiLineOfWLApprox) {
    let (f, l) = (m.first_point(), m.last_point());
    int(f);
    int(l);
    int(m.nb_p3d());
    int(m.nb_p2d());
    int(m.what_status() as i32);
    for i in f - 1..=l + 1 {
        ml_value(m, i);
    }
}

/// The multi-line operations of `ga_gi_mline_case`.
fn mline_ops(r: &mut Reader, m: &GeomIntTheMultiLineOfWLApprox) {
    for _ in 0..r.integer() {
        let op = r.integer();
        if op == 3 {
            captured(|out| m.dump(out));
            continue;
        }
        let (a, b, c) = (r.integer(), r.integer(), r.integer());
        if op == 1 {
            attempt(|| m.make_ml_between(a, b, c).map(|n| ml_report(&n)));
        } else {
            attempt(|| {
                let other = m.make_ml_one_more_point(a, b, c)?;
                flag(other.is_some());
                if let Some(o) = other {
                    ml_report(&o);
                }
                Ok(())
            });
        }
    }
}

/// `ga_gi_mline_case`.
pub fn mline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let g = gi_inter(&mut r)?;
        let kind = r.integer();
        let w = gi_wline(&g, r.integer());
        let sv = gi_sv(&g, kind)?;
        let (n3, n2, a1, a2) = (r.integer(), r.integer(), r.flag(), r.flag());
        let o: Vec<f64> = (0..7).map(|_| r.real()).collect();
        let first = r.flag();
        let Some((lo, hi)) = gi_range(&mut r, &w) else {
            return Ok(());
        };
        let layout = ApproxIntMultiLineLayout {
            nb_p3d: n3,
            nb_p2d: n2,
            approx_u1v1: a1,
            approx_u2v2: a2,
            offsets: ApproxIntOffsets {
                xo: o[0],
                yo: o[1],
                zo: o[2],
                u1o: o[3],
                v1o: o[4],
                u2o: o[5],
                v2o: o[6],
            },
            p2d_on_first: first,
            ind_min: lo,
            ind_max: hi,
        };
        let m = GeomIntTheMultiLineOfWLApprox::with_sv_surfaces(w, sv.shared(), layout);
        ml_report(&m);
        for _ in 0..r.integer() {
            ml_tangency(&m, r.integer());
        }
        mline_ops(&mut r, &m);
        if let Some(s) = sv.shared() {
            flag(s.borrow().get_use_solver());
        }
        Ok(())
    });
    end();
}

/// One SvSurfaces operation of `ga_gi_sv_case` on `u1 v1 u2 v2`.
fn sv_op(g: &GiInter, sv: &Sv, kind: i32, (op, uv): (i32, [f64; 4])) -> Result<(), Failure> {
    let s = sv.shared().expect("probe SvSurfaces");
    let mut s = s.borrow_mut();
    match op {
        0 => {
            let mut t = uv;
            let mut out = ApproxIntSvComputed::default();
            flag(s.compute(&mut t, &mut out)?);
            t.into_iter().for_each(real);
            pnt(out.point);
            xyz(*out.tangent.xyz());
            xy(out.tangent_uv1.xy());
            xy(out.tangent_uv2.xy());
        }
        1 => pnt(s.pnt(uv)?),
        2 => {
            let p = s.seek_point(uv)?;
            flag(p.is_some());
            if let Some(p) = p {
                pnt2s(&p);
            }
        }
        3 => {
            let (f, t) = s.tangency(uv)?;
            flag(f);
            xyz(*t.xyz());
        }
        4 | 5 => {
            let (f, t) = if op == 4 {
                s.tangency_on_surf1(uv)?
            } else {
                s.tangency_on_surf2(uv)?
            };
            flag(f);
            xy(t.xy());
        }
        _ => {
            drop(s);
            fill_initial(g, sv, kind, uv)?;
        }
    }
    Ok(())
}

/// `FillInitialVectorOfSolution` of `ga_gi_sv_case` (op 7).
fn fill_initial(g: &GiInter, sv: &Sv, kind: i32, uv: [f64; 4]) -> Result<(), Failure> {
    let Sv::ImpPrm(ip) = sv else {
        return Err(Failure::with_message(
            "Standard_NullObject",
            "not an ImpPrm SvSurfaces",
        ));
    };
    let ps: &Adaptor3dSurfaceHandle = if kind == 2 { &g.s2 } else { &g.s1 };
    let bounds = PatchBounds {
        binfu: ps.first_u_parameter()?,
        bsupu: ps.last_u_parameter()?,
        binfv: ps.first_v_parameter()?,
        bsupv: ps.last_v_parameter()?,
    };
    let mut x = Vector::filled(1, 2, 0.0)?;
    let res = ip
        .borrow()
        .fill_initial_vector_of_solution(uv, bounds, &mut x)?;
    flag(res.is_some());
    if let Some((tu, tv)) = res {
        real(x[1]);
        real(x[2]);
        real(tu);
        real(tv);
    }
    Ok(())
}

/// `ga_gi_sv_case`.
pub fn sv_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let g = gi_inter(&mut r)?;
        let kind = r.integer();
        let sv = gi_sv(&g, kind)?;
        let shared = sv.shared().expect("probe SvSurfaces");
        flag(shared.borrow().get_use_solver());
        for _ in 0..r.integer() {
            let op = r.integer();
            if op == 6 {
                let f = r.flag();
                shared.borrow_mut().set_use_solver(f);
                flag(shared.borrow().get_use_solver());
                continue;
            }
            let uv = [r.real(), r.real(), r.real(), r.real()];
            attempt(|| sv_op(&g, &sv, kind, (op, uv)));
        }
        Ok(())
    });
    end();
}

/// `ga_gi_pars`.
fn gi_pars(r: &mut Reader, n: i32) -> Vector<'static> {
    let lower = r.integer();
    let mut p = Vector::filled(lower, lower + n - 1, 0.0).expect("probe pars");
    for i in lower..lower + n {
        p[i] = r.real();
    }
    p
}

fn bounded<T: Default + Clone>(lower: i32, n: i32) -> Array1<'static, T> {
    Array1::with_bounds(lower, lower + n - 1).expect("probe array")
}

/// BuildKnots of `ga_gi_knots_case` (mode 1).
fn build_knots(r: &mut Reader) -> Result<(), Failure> {
    let (n, lower) = (r.integer(), r.integer());
    let (mut x, mut a, mut b) = (
        bounded::<Point3>(lower, n),
        bounded::<Point2>(lower, n),
        bounded::<Point2>(lower, n),
    );
    for i in lower..lower + n {
        x[i] = r.pnt();
        a[i] = r.pnt2d();
        b[i] = r.pnt2d();
    }
    let mut p = Vector::filled(lower, lower + n - 1, 0.0)?;
    for i in lower..lower + n {
        p[i] = r.real();
    }
    let (f1, f2, f3) = (r.flag(), r.flag(), r.flag());
    let mn = r.integer();
    let mut k: Vec<i32> = vec![-7];
    let sets = ApproxIntKnotPointSets {
        xyz: &x,
        u1v1: &a,
        u2v2: &b,
        approx_xyz: f1,
        approx_u1v1: f2,
        approx_u2v2: f3,
    };
    ApproxIntKnotTools::build_knots(sets, &p, mn, &mut k)?;
    int(k.len() as i32);
    k.into_iter().for_each(int);
    Ok(())
}

/// `ga_gi_knots_case`.
pub fn knots_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    attempt(|| {
        if mode != 0 {
            return build_knots(&mut r);
        }
        let (dim, n) = (r.integer(), r.integer());
        let c: Vec<f64> = (0..n * dim).map(|_| r.real()).collect();
        let p = gi_pars(&mut r, n);
        let mut cv: Array1<'static, f64> =
            Array1::with_bounds(p.lower(), p.upper()).expect("probe curv");
        let mx = ApproxIntKnotTools::build_curvature(&c, dim, &p, &mut cv)?;
        cv.iter().copied().for_each(real);
        real(mx);
        Ok(())
    });
    end();
}

/// `ga_gi_partype_case`.
pub fn partype_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let g = gi_inter(&mut r)?;
        let (x, u1, u2) = (r.flag(), r.flag(), r.flag());
        for k in 1..=r.integer() {
            let w = gi_wline(&g, k);
            let Some((lo, hi)) = gi_range(&mut r, &w) else {
                continue;
            };
            attempt(|| {
                ApproxIntKnotTools::define_par_type(&w, lo, hi, (x, u1, u2)).map(|t| int(t as i32))
            });
            let layout = ApproxIntMultiLineLayout {
                nb_p3d: i32::from(x),
                nb_p2d: i32::from(u1) + i32::from(u2),
                approx_u1v1: u1,
                approx_u2v2: u2,
                p2d_on_first: u1,
                ind_min: lo,
                ind_max: hi,
                ..Default::default()
            };
            let m = GeomIntTheMultiLineOfWLApprox::with_line(w.clone(), layout);
            for t in 0..3 {
                attempt(|| {
                    let mut p = Vector::filled(lo, hi, 0.0)?;
                    GeomIntWLApprox::parameters(&m, lo, hi, partype(t), &mut p)?;
                    (p.lower()..=p.upper()).for_each(|i| real(p[i]));
                    Ok(())
                });
            }
        }
        Ok(())
    });
    end();
}
