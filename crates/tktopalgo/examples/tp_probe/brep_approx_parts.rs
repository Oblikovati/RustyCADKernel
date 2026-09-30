//! The BRepApprox multi-line, SvSurfaces and IntImp instance probe operations, the Rust
//! twin of tools/oracle/tktopalgo_probe_brepapprox_parts.cxx.
use super::brep_approx::{BraWalk, bra_face, bra_line, bra_quadric, bra_range, bra_walk, partype};
use super::common::{
    Reader, attempt, captured, dir, end, flag, int, pnt, pnt2d, real, show, xy, xyz,
};
use super::int_surf::quadric;
use super::int_walk::pnt2s;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeomalgo::approx_int::{
    ApproxIntMultiLineLayout, ApproxIntOffsets, ApproxIntSvComputed, PatchBounds, SvSurfacesRef,
};
use rustycad_tkgeomalgo::int_imp::IntImpConstIsoparametric;
use rustycad_tkgeombase::app_par_curves::LineTool;
use rustycad_tkmath::gp::{Direction2, Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{FunctionSetRoot, Matrix, Vector};
use rustycad_tktopalgo::brep_approx::{
    BRepApproxApprox, BRepApproxTheImpPrmSvSurfacesOfApprox,
    BRepApproxTheInt2SOfThePrmPrmSvSurfacesOfApprox, BRepApproxTheMultiLineOfApprox,
    BRepApproxThePrmPrmSvSurfacesOfApprox, BRepApproxTheZerImpFuncOfTheImpPrmSvSurfacesOfApprox,
};
use std::cell::RefCell;
use std::rc::Rc;

type MultiLine = BRepApproxTheMultiLineOfApprox;

fn one_based<T: Default + Clone>(n: i32) -> Array1<'static, T> {
    Array1::with_bounds(1, n).expect("probe array")
}

/// `TpBraSv`: the SvSurfaces of kind 1 to 3 (the ImpPrm one kept apart for
/// FillInitialVectorOfSolution).
enum Sv {
    None,
    PrmPrm(Rc<RefCell<BRepApproxThePrmPrmSvSurfacesOfApprox>>),
    ImpPrm(Rc<RefCell<BRepApproxTheImpPrmSvSurfacesOfApprox>>),
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

/// `tp_bra_sv`.
fn bra_sv(w: &BraWalk, kind: i32) -> Result<Sv, Failure> {
    Ok(match kind {
        1 => Sv::PrmPrm(Rc::new(RefCell::new(
            BRepApproxThePrmPrmSvSurfacesOfApprox::new(&w.a.s, &w.b.s)?,
        ))),
        2 => Sv::ImpPrm(Rc::new(RefCell::new(
            BRepApproxTheImpPrmSvSurfacesOfApprox::quadric_first(&bra_quadric(&w.a.s)?, &w.b.s),
        ))),
        3 => Sv::ImpPrm(Rc::new(RefCell::new(
            BRepApproxTheImpPrmSvSurfacesOfApprox::quadric_second(&w.a.s, &bra_quadric(&w.b.s)?),
        ))),
        _ => Sv::None,
    })
}

/// `tp_bra_ml_value`.
fn ml_value(m: &MultiLine, i: i32) {
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

/// `tp_bra_ml_tangency`.
fn ml_tangency(m: &MultiLine, i: i32) {
    let mut v: Array1<'static, Vector3> = one_based(m.nb_p3d().max(1));
    let mut w: Array1<'static, Vector2> = one_based(m.nb_p2d().max(1));
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

/// `tp_bra_ml_report`.
fn ml_report(m: &MultiLine) {
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

/// `tp_bra_ml_op`.
fn ml_op(r: &mut Reader, m: &MultiLine) {
    let op = r.integer();
    if op == 3 {
        captured(|out| m.dump(out));
        return;
    }
    if op == 4 {
        let (f, l) = (m.first_point(), m.last_point());
        let t = partype(r.integer());
        attempt(|| {
            let mut p = Vector::filled(f, l, 0.0)?;
            BRepApproxApprox::parameters(m, f, l, t, &mut p)?;
            (p.lower()..=p.upper()).for_each(|i| real(p[i]));
            Ok(())
        });
        return;
    }
    let (a, b, c) = (r.integer(), r.integer(), r.integer());
    if op == 1 {
        attempt(|| m.make_ml_between(a, b, c).map(|n| ml_report(&n)));
        return;
    }
    attempt(|| {
        let other = m.make_ml_one_more_point(a, b, c)?;
        flag(other.is_some());
        if let Some(o) = other {
            ml_report(&o);
        }
        Ok(())
    });
}

/// The layout fields `nbp3d nbp2d u1v1 u2v2 xo yo zo u1o v1o u2o v2o p2donfirst`.
fn layout(r: &mut Reader) -> ApproxIntMultiLineLayout {
    let (n3, n2, a1, a2) = (r.integer(), r.integer(), r.flag(), r.flag());
    let o: Vec<f64> = (0..7).map(|_| r.real()).collect();
    ApproxIntMultiLineLayout {
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
        p2d_on_first: r.flag(),
        ind_min: 0,
        ind_max: 0,
    }
}

/// `bra_mline`: `walk svkind line layout range ntan (i)... nops (op args)...`.
pub fn mline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let w = bra_walk(&mut r)?;
        let kind = r.integer();
        let l = bra_line(&w, r.integer());
        let sv = bra_sv(&w, kind)?;
        let mut lay = layout(&mut r);
        let Some((lo, hi)) = bra_range(&mut r, &l) else {
            return Ok(());
        };
        (lay.ind_min, lay.ind_max) = (lo, hi);
        let m = MultiLine::with_sv_surfaces(l, sv.shared(), lay);
        ml_report(&m);
        for _ in 0..r.integer() {
            ml_tangency(&m, r.integer());
        }
        for _ in 0..r.integer() {
            ml_op(&mut r, &m);
        }
        if let Some(s) = sv.shared() {
            flag(s.borrow().get_use_solver());
        }
        Ok(())
    });
    end();
}

/// One SvSurfaces operation of `tp_bra_sv_case` on `u1 v1 u2 v2`.
fn sv_op(w: &BraWalk, sv: &Sv, kind: i32, (op, uv): (i32, [f64; 4])) -> Result<(), Failure> {
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
            fill_initial(w, sv, kind, uv)?;
        }
    }
    Ok(())
}

/// `FillInitialVectorOfSolution` of `tp_bra_sv_op` (op 7).
fn fill_initial(w: &BraWalk, sv: &Sv, kind: i32, uv: [f64; 4]) -> Result<(), Failure> {
    let Sv::ImpPrm(ip) = sv else {
        return Err(Failure::with_message(
            "Standard_NullObject",
            "not an ImpPrm SvSurfaces",
        ));
    };
    let ps = if kind == 2 { &w.b.s } else { &w.a.s };
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

/// `bra_sv`: `walk svkind nops (op args)...`.
pub fn sv_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let w = bra_walk(&mut r)?;
        let kind = r.integer();
        let sv = bra_sv(&w, kind)?;
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
            attempt(|| sv_op(&w, &sv, kind, (op, uv)));
        }
        Ok(())
    });
    end();
}

fn vector(v: &Vector<'_>) {
    (v.lower()..=v.upper()).for_each(|i| real(v[i]));
}

fn matrix(m: &Matrix<'_>) {
    for i in m.lower_row()..=m.upper_row() {
        for j in m.lower_col()..=m.upper_col() {
            real(m[(i, j)]);
        }
    }
}

fn dir2d(d: Direction2) {
    real(d.x());
    real(d.y());
}

type ZerImp = BRepApproxTheZerImpFuncOfTheImpPrmSvSurfacesOfApprox;

/// The evaluations of `tp_bra_zerimp_case` after the construction.
fn zerimp_report(z: &mut ZerImp, tol: f64, x: &Vector<'_>) -> Result<(), Failure> {
    z.set_tolerance(tol);
    real(z.tolerance());
    int(z.nb_variables());
    int(z.nb_equations());
    let mut fv = Vector::filled(1, 1, 0.0)?;
    let mut d = Matrix::filled(1, 1, 1, 2, 0.0).expect("probe matrix");
    flag(z.value(x, &mut fv)?);
    real(fv[1]);
    real(z.root());
    pnt(z.point());
    show(z.is_tangent(), flag);
    flag(z.derivatives(x, &mut d)?);
    matrix(&d);
    show(z.is_tangent(), flag);
    let mut x2 = Vector::filled(1, 2, 0.0)?;
    (x2[1], x2[2]) = (x[1] + 0.02, x[2] - 0.01);
    flag(z.values(&x2, &mut fv, &mut d)?);
    real(fv[1]);
    matrix(&d);
    show(z.is_tangent(), flag);
    show(z.direction3d(), |v| xyz(*v.xyz()));
    show(z.direction2d(), dir2d);
    pnt(z.point());
    Ok(())
}

/// `bra_zerimp`: `face restr quadric tol u v`.
pub fn zerimp_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = bra_face(&mut r)?;
        let q = quadric(&mut r);
        let tol = r.real();
        let mut x = Vector::filled(1, 2, 0.0)?;
        (x[1], x[2]) = (r.real(), r.real());
        zerimp_report(&mut ZerImp::with_surfaces(&f.s, &q), tol, &x)
    });
    end();
}

type Int2S = BRepApproxTheInt2SOfThePrmPrmSvSurfacesOfApprox;

const ISOS: [IntImpConstIsoparametric; 4] = [
    IntImpConstIsoparametric::UIsoparametricOnCaro1,
    IntImpConstIsoparametric::VIsoparametricOnCaro1,
    IntImpConstIsoparametric::UIsoparametricOnCaro2,
    IntImpConstIsoparametric::VIsoparametricOnCaro2,
];

/// `tp_bra_int2s_report`.
fn int2s_report(s: &Int2S) {
    flag(s.is_done());
    show(s.is_empty(), flag);
    show(s.point(), pnt2s);
    show(s.is_tangent(), flag);
    show(s.direction(), dir);
    show(s.direction_on_s1(), dir2d);
    show(s.direction_on_s2(), dir2d);
}

/// The function of the Int2S after its Perform (mode 1 of `tp_bra_int2s_case`).
fn zerpar_report(
    s: &mut Int2S,
    iso: IntImpConstIsoparametric,
    param: &Array1<'_, f64>,
) -> Result<(), Failure> {
    let f = s.function();
    let zero3 = || Vector::filled(1, 3, 0.0);
    let (mut uv, mut inf, mut sup, mut tol) = (zero3()?, zero3()?, zero3()?, zero3()?);
    f.compute_parameters(iso, param, &mut uv, [&mut inf, &mut sup, &mut tol]);
    [&uv, &inf, &sup, &tol].into_iter().for_each(vector);
    let mut fv = zero3()?;
    let mut d = Matrix::filled(1, 3, 1, 3, 0.0).expect("probe matrix");
    flag(f.values(&uv, &mut fv, &mut d)?);
    vector(&fv);
    matrix(&d);
    real(f.root());
    pnt(f.point());
    Ok(())
}

/// `bra_int2s`: `face1 restr1 face2 restr2 tol mode iso u1 v1 u2 v2`.
pub fn int2s_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (a, b) = (bra_face(&mut r)?, bra_face(&mut r)?);
        let tol = r.real();
        let (mode, iso) = (r.integer(), ISOS[r.integer() as usize]);
        let mut param: Array1<'static, f64> = one_based(4);
        for i in 1..=4 {
            param[i] = r.real();
        }
        if mode == 0 {
            int2s_report(&Int2S::with_start(&param, &a.s, &b.s, tol)?);
            return Ok(());
        }
        let mut s = Int2S::new(&a.s, &b.s, tol)?;
        let mut rs = FunctionSetRoot::without_tolerance(s.function(), 15)?;
        int(s.perform_iso(&param, &mut rs, iso)? as i32);
        int2s_report(&s);
        zerpar_report(&mut s, iso, &param)
    });
    end();
}
