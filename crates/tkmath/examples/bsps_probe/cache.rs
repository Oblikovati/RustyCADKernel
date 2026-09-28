//! Static caches, Bezier coefficients and BSplSLib_Cache.
use super::{Surf, grid1, grid3, od2, og1, og3, ov, pgrid, wgrid};
use crate::bspl_probe::{In, fail, oi, op, report};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplslib::{self, Cache, CacheSource, SurfaceCacheSpan, SurfaceD2};
use rustycad_tkmath::gp::Point3;

// The CacheD2/CoefsD2 layout: P VecU VecV VecUU VecUV VecVV.
fn cache_d2_out(d: &SurfaceD2) {
    op(&d.p);
    [d.vu, d.vv, d.vuu, d.vuv, d.vvv].iter().for_each(ov);
}

// CacheD0/D1/D2 (or CoefsD0/D1/D2 when `span` is None) at (u, v).
fn cache_evals(
    u: f64,
    v: f64,
    span: Option<&SurfaceCacheSpan>,
    cp: &Array2<'_, Point3>,
    cw: Option<&Array2<'_, f64>>,
) {
    let d0 = match span {
        Some(span) => bsplslib::cache_d0(u, v, span, cp, cw),
        None => bsplslib::coefs_d0(u, v, cp, cw),
    };
    report(d0, |p| op(&p));
    let d1 = match span {
        Some(span) => bsplslib::cache_d1(u, v, span, cp, cw),
        None => bsplslib::coefs_d1(u, v, cp, cw),
    };
    report(d1, |(p, vu, vv)| {
        op(&p);
        ov(&vu);
        ov(&vv);
    });
    let d2 = match span {
        Some(span) => bsplslib::cache_d2(u, v, span, cp, cw),
        None => bsplslib::coefs_d2(u, v, cp, cw),
    };
    report(d2, |d| cache_d2_out(&d));
}

// Cache class: IsCacheValid, BuildCache when invalid, D0..D2, then D0Local..D2Local.
fn cache_class(s: &Surf, cache: &mut Cache, [u, v, lu, lv]: [f64; 4]) {
    let valid: bool = cache.is_cache_valid(u, v);
    oi(i32::from(valid));
    if let Err(failure) = class_evals(s, cache, valid, [u, v, lu, lv]) {
        fail(&failure);
    }
}

fn class_evals(
    s: &Surf,
    cache: &mut Cache,
    valid: bool,
    [u, v, lu, lv]: [f64; 4],
) -> Result<(), Failure> {
    if !valid {
        cache.build_cache(u, v, &s.uk, &s.vk, &s.p, s.w())?;
    }
    op(&cache.d0(u, v)?);
    let (p, vu, vv) = cache.d1(u, v)?;
    op(&p);
    ov(&vu);
    ov(&vv);
    od2(&cache.d2(u, v)?);
    op(&cache.d0_local(lu, lv)?);
    let (p, vu, vv) = cache.d1_local(lu, lv)?;
    op(&p);
    ov(&vu);
    ov(&vv);
    od2(&cache.d2_local(lu, lv)?);
    Ok(())
}

// cache: surface(flat knots) u v udomain vdomain uindex vindex ucache vcache ulength vlength
//        count (u v lu lv)...
pub fn cache_case(v: &[f64]) {
    let mut f = In::new(v);
    let s = Surf::read(&mut f);
    let (u, vv, ud, vd, ui, vi) = (f.r(), f.r(), f.r(), f.r(), f.i(), f.i());
    let span = SurfaceCacheSpan {
        u_degree: s.ud,
        v_degree: s.vd,
        u_cache_parameter: f.r(),
        v_cache_parameter: f.r(),
        u_span_length: f.r(),
        v_span_length: f.r(),
    };
    let (mx, mn, dim) = (
        s.ud.max(s.vd) + 1,
        s.ud.min(s.vd) + 1,
        if s.hw { 4 } else { 3 },
    );
    let source = CacheSource {
        u,
        v: vv,
        u_span_domain: ud,
        v_span_domain: vd,
        u_periodic: s.up,
        v_periodic: s.vp,
        u_degree: s.ud,
        v_degree: s.vd,
        u_index: ui,
        v_index: vi,
        u_flat_knots: &s.uk,
        v_flat_knots: &s.vk,
    };
    let (mut cp, mut cw) = (pgrid(mx, mn), wgrid(mx, mn));
    let cw_out = if s.hw { Some(&mut cw) } else { None };
    report(
        bsplslib::build_cache(&source, &s.p, s.w(), &mut cp, cw_out),
        |()| {
            og3(&cp);
            if s.hw {
                og1(&cw);
            }
        },
    );
    let mut ca = wgrid(mx, dim * mn);
    report(
        bsplslib::build_cache_matrix(&source, &s.p, s.w(), &mut ca),
        |()| og1(&ca),
    );
    let n: i32 = f.i();
    let points: Vec<[f64; 4]> = (0..n).map(|_| [f.r(), f.r(), f.r(), f.r()]).collect();
    let cwr = if s.hw { Some(&cw) } else { None };
    for [pu, pv, _, _] in &points {
        cache_evals(*pu, *pv, Some(&span), &cp, cwr);
    }
    match Cache::new(s.ud, s.up, &s.uk, s.vd, s.vp, &s.vk, s.w()) {
        Ok(mut cache) => points
            .iter()
            .for_each(|point| cache_class(&s, &mut cache, *point)),
        Err(failure) => fail(&failure),
    }
    println!();
}

// bezier: poles hasweights weights count (u v)...
pub fn bezier_case(v: &[f64]) {
    let mut f = In::new(v);
    let p = grid3(&mut f);
    let hw: bool = f.b();
    let w = grid1(&mut f);
    // The coefficients take the shape of the poles (see the PolesCoefficients quirk).
    let (rows, cols) = (p.nb_rows() as i32, p.nb_columns() as i32);
    let (mut cp, mut cw) = (pgrid(rows, cols), wgrid(rows, cols));
    report(
        bsplslib::poles_coefficients(&p, None, &mut cp, None),
        |()| og3(&cp),
    );
    let weights = if hw { Some(&w) } else { None };
    let cw_out = if hw { Some(&mut cw) } else { None };
    report(
        bsplslib::poles_coefficients(&p, weights, &mut cp, cw_out),
        |()| {
            og3(&cp);
            if hw {
                og1(&cw);
            }
        },
    );
    let cwr = if hw { Some(&cw) } else { None };
    for _ in 0..f.i() {
        let (u, v) = (f.r(), f.r());
        cache_evals(u, v, None, &cp, cwr);
    }
    println!();
}
