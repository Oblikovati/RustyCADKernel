// Rust twin of tools/oracle/tkgeomalgo_probe_gfill_cf.cxx: boundaries, the Coons patch,
// tangent fields, corner states and the constrained filling.
use super::common::{Reader, attempt, captured, end, flag, int, pnt, real, show, xyz};
use super::gfill::read_curve;
use super::gfill_fill::dump_bsurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dLine;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillBoundWithSurf, GeomFillBoundary, GeomFillConstrainedFilling, GeomFillCoonsAlgPatch,
    GeomFillCornerState, GeomFillDegeneratedBound, GeomFillSimpleBound, GeomFillTgtField,
    GeomFillTgtOnCoons,
};
use rustycad_tkmath::gp::{Axis3, Vector3};

fn vec3(v: Vector3) {
    xyz(*v.xyz());
}

fn read_surface(r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind = r.integer();
    let (p, n, x) = (r.pnt(), r.dir(), r.dir());
    let ax = Axis3::new(p, n, x)?;
    Ok(match kind {
        0 => Handle::new(GeomPlane::new(ax)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(ax, r.real())?).cast(),
        _ => Handle::new(GeomSphericalSurface::new(ax, r.real())?).cast(),
    })
}

pub fn read_boundary(r: &mut Reader) -> Result<Handle<dyn GeomFillBoundary>, Failure> {
    let (kind, t3, ta) = (r.integer(), r.real(), r.real());
    if kind == 0 {
        let (f, l) = (r.real(), r.real());
        let c = read_curve(r)?;
        let a = if f < l {
            GeomAdaptorCurve::with_range(&c, f, l)?
        } else {
            GeomAdaptorCurve::with_curve(&c)?
        };
        let a: Handle<dyn Adaptor3dCurve> = Handle::new(a).cast();
        return Ok(Handle::new(GeomFillSimpleBound::new(&a, t3, ta)).cast());
    }
    if kind == 1 {
        let p = r.pnt();
        let (f, l) = (r.real(), r.real());
        return Ok(Handle::new(GeomFillDegeneratedBound::new(p, f, l, t3, ta)).cast());
    }
    let s = read_surface(r)?;
    let (p, d) = (r.pnt2d(), r.dir2d());
    let (f, l) = (r.real(), r.real());
    let line = Handle::new(Geom2dLine::from_point(p, d)).cast();
    let c2: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(&line, f, l)?).cast();
    let a: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(&s)?).cast();
    let cos = Adaptor3dCurveOnSurface::with_curve(&c2, &a)?;
    Ok(Handle::new(GeomFillBoundWithSurf::new(&cos, t3, ta)).cast())
}

fn bound_report(b: &dyn GeomFillBoundary, ts: &[f64]) {
    flag(b.is_degenerated());
    flag(b.has_normals());
    real(b.tol3d());
    real(b.tolang());
    let (mut f, mut l) = (0., 0.);
    show(b.bounds(), |(bf, bl)| {
        (f, l) = (bf, bl);
        real(f);
        real(l);
    });
    show(b.points(), |(p1, p2)| {
        pnt(p1);
        pnt(p2);
    });
    for &t in ts {
        let w = f + t * (l - f);
        show(b.value(w), pnt);
        show(b.d1(w), |(p, v)| {
            pnt(p);
            vec3(v);
        });
        show(b.norm(w), vec3);
        show(b.d1_norm(w), |(n, dn)| {
            vec3(n);
            vec3(dn);
        });
    }
}

pub fn bound_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let b = read_boundary(&mut r).expect("probe boundary");
    let ts = r.reals();
    let (f, l, hf, hl) = (r.real(), r.real(), r.flag(), r.flag());
    let (df, dl, rev) = (r.real(), r.real(), r.flag());
    let (t3, ta) = (r.real(), r.real());
    bound_report(&*b, &ts);
    attempt(|| b.reparametrize(f, l, hf, hl, df, dl, rev));
    bound_report(&*b, &ts);
    b.set_tol3d(t3);
    b.set_tolang(ta);
    real(b.tol3d());
    real(b.tolang());
    end();
}

pub fn corner_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let ops = r.reals();
    let mut c = GeomFillCornerState::new();
    for pair in ops.as_chunks::<2>().0 {
        let (op, v) = (pair[0] as i32, pair[1]);
        match op {
            0 => c.set_gap(v),
            1 => c.set_tgt_ang(v),
            2 => c.set_nor_ang(v),
            3 => c.constraint(),
            _ => c.do_kill(v),
        }
        real(c.gap());
        real(c.tgt_ang());
        real(c.nor_ang());
        flag(c.has_constraint());
        let (kill, s) = c.is_to_kill();
        flag(kill);
        real(s);
    }
    end();
}

fn patch_fields(k: &Handle<GeomFillCoonsAlgPatch>, b: &[Handle<dyn GeomFillBoundary>]) {
    for (i, bound) in b.iter().enumerate().take(4) {
        let t = GeomFillTgtOnCoons::new(k, i as i32);
        flag(t.is_scalable());
        let (f, l) = bound.bounds().unwrap();
        for s in [0., 0.3, 1.] {
            let w = f + s * (l - f);
            show(t.value(w), vec3);
            show(t.d1(w), vec3);
            show(t.d1_with_value(w), |(a, d)| {
                vec3(a);
                vec3(d);
            });
        }
    }
}

pub fn patch_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let b: Vec<_> = (0..4)
        .map(|_| read_boundary(&mut r).expect("probe boundary"))
        .collect();
    let uv = r.reals();
    let k = match GeomFillCoonsAlgPatch::new([&b[0], &b[1], &b[2], &b[3]]) {
        Ok(k) => Handle::new(k),
        Err(e) => {
            print!("{} ", e.exception_type());
            end();
            return;
        }
    };
    let (u0, u1) = b[0].bounds().unwrap();
    let (v0, v1) = b[1].bounds().unwrap();
    (0..4).for_each(|i| pnt(k.corner(i).unwrap()));
    for pair in uv.as_chunks::<2>().0 {
        let (u, v) = (u0 + pair[0] * (u1 - u0), v0 + pair[1] * (v1 - v0));
        show(k.value(u, v), pnt);
        show(k.d1u(u, v), vec3);
        show(k.d1v(u, v), vec3);
        show(k.duv(u, v), vec3);
        show(k.func(0).and_then(|f| f.value(v)), real);
        show(k.func(1).and_then(|f| f.value(u)), real);
    }
    patch_fields(&k, &b);
    let (f1, f2) = k.func_pair();
    k.set_func(&f2, &f1);
    show(k.value(0.5 * (u0 + u1), 0.5 * (v0 + v1)), pnt);
    end();
}

fn cfill_report(f: &GeomFillConstrainedFilling, b: &[Handle<dyn GeomFillBoundary>]) {
    dump_bsurface(&f.surface());
    for i in 0..4 {
        let s = f.boundary(i).unwrap();
        let found = b
            .iter()
            .position(|x| x.object_address() == s.object_address());
        int(found.map_or(-1, |k| k as i32));
    }
    for w in [0., 0.3, 1.] {
        for ord in 0..2 {
            let mut res = [0.0; 12];
            show(f.eval(w, ord, &mut res), int);
            res.iter().for_each(|&x| real(x));
        }
    }
}

fn init(
    f: &mut GeomFillConstrainedFilling,
    b: &[Handle<dyn GeomFillBoundary>],
    nocheck: bool,
) -> Result<(), Failure> {
    if b.len() == 3 {
        f.init_three([&b[0], &b[1], &b[2]], nocheck)
    } else {
        f.init_four([&b[0], &b[1], &b[2], &b[3]], nocheck)
    }
}

fn cfill_checks(f: &GeomFillConstrainedFilling) {
    for i in 0..4 {
        captured(|_| f.check_coons_alg_patch(i));
        captured(|text| f.check_tgte_field(i, text));
        captured(|text| f.check_approx(i, text));
        captured(|text| f.check_result(i, text));
    }
}

pub fn cfill_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (maxdeg, maxseg, nocheck, checks) = (r.integer(), r.integer(), r.flag(), r.flag());
    let n = r.integer();
    let b: Vec<_> = (0..n)
        .map(|_| read_boundary(&mut r).expect("probe boundary"))
        .collect();
    let doms = r.reals();
    let again = r.flag();
    let mut f = GeomFillConstrainedFilling::new(maxdeg, maxseg);
    flag(f.surface().is_null());
    attempt(|| f.re_build());
    if let Err(e) = init(&mut f, &b, nocheck) {
        print!("{} ", e.exception_type());
        end();
        return;
    }
    cfill_report(&f, &b);
    if checks {
        cfill_checks(&f);
    }
    if !doms.is_empty() {
        for pair in doms.as_chunks::<2>().0 {
            f.set_domain(pair[1], &b[pair[0] as usize].cast()).unwrap();
        }
        attempt(|| f.re_build());
        dump_bsurface(&f.surface());
    }
    if again {
        let n2 = r.integer();
        let b2: Vec<_> = (0..n2)
            .map(|_| read_boundary(&mut r).expect("probe boundary"))
            .collect();
        attempt(|| {
            init(&mut f, &b2, nocheck)?;
            dump_bsurface(&f.surface());
            Ok(())
        });
    }
    end();
}
