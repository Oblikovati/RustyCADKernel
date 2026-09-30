//! IntWalk_PWalking probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intwalk.cxx (the protected members are public here).
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real, show};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface, GeomPlane,
    GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_imp::IntImpConstIsoparametric as Iso;
use rustycad_tkgeomalgo::int_surf::IntSurfPntOn2S;
use rustycad_tkgeomalgo::int_walk::{IntWalkPWalking, IntWalkStatusDeflection as Status};
use rustycad_tkmath::gp::{Axis3, Point3};

pub const ISOS: [Iso; 4] = [
    Iso::UIsoparametricOnCaro1,
    Iso::VIsoparametricOnCaro1,
    Iso::UIsoparametricOnCaro2,
    Iso::VIsoparametricOnCaro2,
];

fn analytic(r: &mut Reader, kind: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (p, d) = (r.pnt(), r.dir());
    let (r1, r2) = (r.real(), r.real());
    let ax = Axis3::from_normal(p, d)?;
    Ok(match kind {
        0 => Handle::new(GeomPlane::new(ax)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(ax, r1)?).cast(),
        2 => Handle::new(GeomSphericalSurface::new(ax, r1)?).cast(),
        3 => Handle::new(GeomToroidalSurface::new(ax, r1, r2)?).cast(),
        _ => Handle::new(GeomConicalSurface::new(ax, r2, r1)?).cast(),
    })
}

fn knots(n: i32, degree: i32) -> (Array1<'static, f64>, Array1<'static, i32>) {
    let count: i32 = n - degree + 1;
    let mut k: Array1<'static, f64> = Array1::with_bounds(1, count).expect("probe knots");
    let mut m: Array1<'static, i32> = Array1::with_bounds(1, count).expect("probe mults");
    for i in 1..=count {
        k[i] = f64::from(i - 1);
        m[i] = if i == 1 || i == count { degree + 1 } else { 1 };
    }
    (k, m)
}

fn spline(r: &mut Reader, kind: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (mut du, mut dv) = (0, 0);
    if kind == 6 {
        (du, dv) = (r.integer(), r.integer());
    }
    let (nu, nv) = (r.integer(), r.integer());
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(1, nu, 1, nv).expect("probe grid");
    for i in 1..=nu {
        for j in 1..=nv {
            poles[(i, j)] = r.pnt();
        }
    }
    if kind == 5 {
        return Ok(Handle::new(GeomBezierSurface::new(&poles)?).cast());
    }
    let ((uk, um), (vk, vm)) = (knots(nu, du), knots(nv, dv));
    Ok(Handle::new(GeomBSplineSurface::new(
        &poles,
        (&uk, &um),
        (&vk, &vm),
        (du, dv),
        (false, false),
    )?)
    .cast())
}

/// `ga_iw_surface`: the surface of a request on its adaptor bounds.
pub fn surface(r: &mut Reader) -> Result<Adaptor3dSurfaceHandle, Failure> {
    let kind = r.integer();
    let s = if kind <= 4 {
        analytic(r, kind)?
    } else {
        spline(r, kind)?
    };
    let bounds = (r.real(), r.real(), r.real(), r.real());
    Ok(Handle::new(GeomAdaptorSurface::with_bounds(&s, bounds)?).cast())
}

pub fn pnt2s(p: &IntSurfPntOn2S) {
    pnt(p.value());
    let (a, b, c, d) = p.parameters();
    [a, b, c, d].into_iter().for_each(real);
}

fn one_based(values: [f64; 4]) -> Array1<'static, f64> {
    let mut a: Array1<'static, f64> = Array1::with_bounds(1, 4).expect("probe params");
    for (i, v) in values.into_iter().enumerate() {
        a[i as i32 + 1] = v;
    }
    a
}

fn params(r: &mut Reader) -> [f64; 4] {
    [r.real(), r.real(), r.real(), r.real()]
}

fn report(w: &IntWalkPWalking) {
    flag(w.is_done());
    attempt(|| {
        let n = w.nb_points()?;
        int(n);
        for i in 1..=n {
            pnt2s(&w.value(i)?);
        }
        Ok(())
    });
    show(w.tangent_at_first(), flag);
    show(w.tangent_at_last(), flag);
    show(w.is_closed(), flag);
    attempt(|| {
        let (d, idx) = w.tangent_at_line()?;
        dir(d);
        int(idx);
        Ok(())
    });
    for i in -1..=4 {
        show(w.max_step(i), real);
    }
    show(w.value(0), |p| pnt2s(&p));
    show(w.nb_points().and_then(|n| w.value(n + 1)), |p| pnt2s(&p));
    show(w.line(), |l| int(l.nb_points()));
}

/// The four walking tolerances of a request.
fn tolerances(r: &mut Reader) -> [f64; 4] {
    [r.real(), r.real(), r.real(), r.real()]
}

/// `s1 s2 toltang eps defl incr mode start(4) [box(8)] post minpts`.
pub fn walk_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (s1, s2) = (surface(&mut r)?, surface(&mut r)?);
        let [tt, eps, defl, incr] = tolerances(&mut r);
        let mode = r.integer();
        let start = params(&mut r);
        let mut bx = [0.0; 8];
        if mode == 1 {
            bx = bx.map(|_| r.real());
        }
        let (post, minpts) = (r.integer(), r.integer());
        let mut w = if mode == 2 {
            IntWalkPWalking::with_start(&s1, &s2, [tt, eps, defl, incr], start)?
        } else {
            let mut w = IntWalkPWalking::new(&s1, &s2, tt, eps, defl, incr)?;
            if mode == 1 {
                w.perform_bounded(
                    &one_based(start),
                    [bx[0], bx[1], bx[2], bx[3]],
                    [bx[4], bx[5], bx[6], bx[7]],
                )?;
            } else {
                w.perform(&one_based(start))?;
            }
            w
        };
        if w.is_done() && post & 1 != 0 {
            show(w.put_to_boundary(&s1, &s2), flag);
        }
        if w.is_done() && post & 2 != 0 {
            show(w.seek_additional_points(&s1, &s2, minpts), flag);
        }
        report(&w);
        Ok(())
    });
    end();
}

/// `s1 s2 toltang eps defl incr start(4)`: PerformFirstPoint.
pub fn first_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (s1, s2) = (surface(&mut r)?, surface(&mut r)?);
        let [tt, eps, defl, incr] = tolerances(&mut r);
        let start = params(&mut r);
        let mut w = IntWalkPWalking::new(&s1, &s2, tt, eps, defl, incr)?;
        let first = w.perform_first_point(&one_based(start))?;
        flag(first.is_some());
        pnt2s(&first.unwrap_or_default());
        for i in 0..=3 {
            real(w.max_step(i)?);
        }
        Ok(())
    });
    end();
}

fn edit_steps(w: &mut IntWalkPWalking, start: [f64; 4], ci: Iso) {
    show(w.test_deflection(ci, Status::OK), |s| int(s as i32));
    show(w.test_deflection(ci, Status::PasTropGrand), |s| {
        int(s as i32)
    });
    for deja in [false, true] {
        let mut p = one_based(start);
        let mut c = ci;
        attempt(|| {
            flag(w.test_arret(deja, &mut p, &mut c)?);
            (1..=4).for_each(|i| real(p[i]));
            int(c as i32);
            Ok(())
        });
    }
    for arrive in [false, true] {
        let (mut dr, mut a, mut c) = (false, arrive, ci);
        attempt(|| {
            w.repartir_ou_diviser(&mut dr, &mut c, &mut a)?;
            flag(dr);
            flag(a);
            int(c as i32);
            Ok(())
        });
    }
}

/// `s1 s2 toltang eps defl incr start(4) iso remove`: the public stepping members.
pub fn edit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (s1, s2) = (surface(&mut r)?, surface(&mut r)?);
        let [tt, eps, defl, incr] = tolerances(&mut r);
        let start = params(&mut r);
        let (iso, remove) = (r.integer(), r.integer());
        let mut w = IntWalkPWalking::new(&s1, &s2, tt, eps, defl, incr)?;
        w.perform(&one_based(start))?;
        flag(w.is_done());
        edit_steps(&mut w, start, ISOS[iso as usize]);
        attempt(|| w.remove_a_point(remove));
        attempt(|| {
            if w.is_done() && w.nb_points()? > 0 {
                let first = w.value(1)?;
                w.add_a_point(&first)?;
            }
            Ok(())
        });
        report(&w);
        Ok(())
    });
    end();
}

fn min_op(
    op: i32,
    r: &mut Reader,
    w: &mut IntWalkPWalking,
    surfaces: (&Adaptor3dSurfaceHandle, &Adaptor3dSurfaceHandle),
    mut p: [f64; 4],
) -> Result<(), Failure> {
    let (s1, s2) = surfaces;
    match op {
        0 => {
            let has = r.integer();
            let st = [r.real(), r.real(), r.real(), r.real()];
            flag(w.distance_minimize_by_gradient(s1, s2, &mut p, (has != 0).then_some(st))?);
            p.into_iter().for_each(real);
        }
        1 => {
            let has = r.integer();
            let st = [r.real(), r.real()];
            let (mut u, mut v) = (p[0], p[1]);
            let p0 = s2.value(p[2], p[3])?;
            flag(w.distance_minimize_by_extrema(
                s1,
                p0,
                (&mut u, &mut v),
                (has != 0).then_some(st),
            )?);
            real(u);
            real(v);
        }
        2 => {
            let tol = r.real();
            flag(w.handle_single_singular_point(s1, s2, tol, &mut p)?);
            p.into_iter().for_each(real);
        }
        3 => {
            let d = [r.real(), r.real(), r.real(), r.real()];
            w.compute_pas_init(d)?;
            for i in 0..=3 {
                real(w.max_step(i)?);
            }
        }
        4 => {
            let uv = [r.real(), r.real(), r.real(), r.real()];
            let first = r.integer();
            w.perform(&one_based(p))?;
            show(w.seek_point_on_boundary(s1, s2, uv, first != 0), flag);
            report(w);
        }
        _ => {
            let (iso, fl) = (r.integer(), r.integer());
            w.perform(&one_based(p))?;
            show(
                w.extend_line_in_common_zone(ISOS[iso as usize], fl != 0),
                flag,
            );
            report(w);
        }
    }
    Ok(())
}

/// `op s1 s2 toltang eps defl incr pnt(4) ...`: the protected searches.
pub fn min_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let op = r.integer();
    attempt(|| {
        let (s1, s2) = (surface(&mut r)?, surface(&mut r)?);
        let [tt, eps, defl, incr] = tolerances(&mut r);
        let p = params(&mut r);
        let mut w = IntWalkPWalking::new(&s1, &s2, tt, eps, defl, incr)?;
        min_op(op, &mut r, &mut w, (&s1, &s2), p)
    });
    end();
}
