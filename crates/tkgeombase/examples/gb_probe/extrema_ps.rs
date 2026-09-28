//! The Extrema point-surface requests; the counterpart of
//! tools/oracle/tkgeombase_probe_extrema_ps.cxx (same encoding and token order).
#![allow(clippy::unit_arg, clippy::type_complexity)]
use super::bndlib;
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::extrema_el::ponsurf;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::extrema::{
    ExtAlgo, ExtFlag, ExtPExtS, ExtPRevS, ExtPS, FuncPSDist, FuncPSNorm, GenExtPS, GenLocateExtPS,
    POnSurf, Sampling,
};
use rustycad_tkmath::gp::Vector3;
use rustycad_tkmath::math::{Matrix, Vector};

/// gb_ex_surface: a surface spec, a ranged flag and the adaptor bounds.
pub fn surface(r: &mut Reader) -> Result<GeomAdaptorSurface, Failure> {
    let s = bndlib::surface(r)?;
    if r.flag() {
        let bounds = (r.real(), r.real(), r.real(), r.real());
        return GeomAdaptorSurface::with_bounds(&s, bounds);
    }
    GeomAdaptorSurface::with_surface(&s)
}

/// The accessors of a result with NbExt, SquareDistance(N) and Point(N).
pub(super) trait PsView {
    fn done(&self) -> bool;
    fn count(&self) -> Result<i32, Failure>;
    fn sq(&self, n: i32) -> Result<f64, Failure>;
    fn pon(&self, n: i32) -> Result<POnSurf, Failure>;
}

macro_rules! ps_view {
    ($t:ty) => {
        impl PsView for $t {
            fn done(&self) -> bool {
                self.is_done()
            }
            fn count(&self) -> Result<i32, Failure> {
                self.nb_ext()
            }
            fn sq(&self, n: i32) -> Result<f64, Failure> {
                self.square_distance(n)
            }
            fn pon(&self, n: i32) -> Result<POnSurf, Failure> {
                self.point(n)
            }
        }
    };
}
ps_view!(GenExtPS<'_>);
ps_view!(ExtPS<'_>);
ps_view!(ExtPExtS);
ps_view!(ExtPRevS);

pub(super) fn ps_result(e: &impl PsView) {
    flag(e.done());
    attempt(|| {
        let n = e.count()?;
        int(n);
        for i in 1..=n {
            real(e.sq(i)?);
            ponsurf(e.pon(i)?);
        }
        Ok(())
    });
    attempt(|| Ok(real(e.sq(0)?)));
}

fn vec2(a: f64, b: f64) -> Vector<'static> {
    let mut v = Vector::with_bounds(1, 2).unwrap();
    v[1] = a;
    v[2] = b;
    v
}

fn matrix(m: &Matrix<'_>) {
    for i in 1..=2 {
        for j in 1..=2 {
            real(m[(i, j)]);
        }
    }
}

fn fpsn_step(f: &mut FuncPSNorm<'_>, uv: &Vector<'_>) {
    let mut m = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    attempt(|| {
        let mut v = vec2(0.0, 0.0);
        flag(f.value(uv, &mut v)?);
        real(v[1]);
        real(v[2]);
        Ok(())
    });
    attempt(|| {
        flag(f.derivatives(uv, &mut m)?);
        matrix(&m);
        Ok(())
    });
    attempt(|| {
        let mut w = vec2(0.0, 0.0);
        flag(f.values(uv, &mut w, &mut m)?);
        real(w[1]);
        real(w[2]);
        matrix(&m);
        Ok(())
    });
    attempt(|| Ok(int(f.get_state_number()?)));
}

fn fpsn_body(r: &mut Reader) -> Result<(), Failure> {
    let s = surface(r)?;
    let p = r.pnt();
    let n = r.integer();
    let mut d = FuncPSNorm::new();
    attempt(|| {
        let mut f0 = vec2(0.0, 0.0);
        Ok(flag(d.value(&vec2(0.5, 0.5), &mut f0)?))
    });
    attempt(|| Ok(int(d.get_state_number()?)));
    int(d.nb_ext());
    attempt(|| Ok(real(d.square_distance(1)?)));
    attempt(|| Ok(ponsurf(d.point(1)?)));
    let mut f = FuncPSNorm::with_point_surface(p, &s);
    int(f.nb_variables());
    int(f.nb_equations());
    for _ in 0..n {
        let uv = vec2(r.real(), r.real());
        fpsn_step(&mut f, &uv);
    }
    int(f.nb_ext());
    for i in 0..=f.nb_ext() + 1 {
        attempt(|| {
            real(f.square_distance(i)?);
            ponsurf(f.point(i)?);
            Ok(())
        });
    }
    f.set_point(p.translated(Vector3::new(1.0, 0.0, 0.0)));
    int(f.nb_ext());
    attempt(|| {
        let mut v = vec2(0.0, 0.0);
        flag(f.value(&vec2(0.25, 0.25), &mut v)?);
        int(f.get_state_number()?);
        int(f.nb_ext());
        Ok(())
    });
    f.initialize(&s);
    int(f.nb_ext());
    Ok(())
}

pub fn fpsn_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| fpsn_body(&mut r));
    end();
}

fn fpsd_body(r: &mut Reader) -> Result<(), Failure> {
    let s = surface(r)?;
    let p = r.pnt();
    let n = r.integer();
    let f = FuncPSDist::new(&s, p);
    int(f.nb_variables());
    for _ in 0..n {
        let x = vec2(r.real(), r.real());
        attempt(|| {
            let mut v = 0.0;
            flag(f.value(&x, &mut v)?);
            real(v);
            Ok(())
        });
        let mut g = vec2(0.0, 0.0);
        attempt(|| {
            flag(f.gradient(&x, &mut g)?);
            real(g[1]);
            real(g[2]);
            Ok(())
        });
        attempt(|| {
            let mut v = 0.0;
            let mut g = vec2(0.0, 0.0);
            flag(f.values(&x, &mut v, &mut g)?);
            real(v);
            real(g[1]);
            real(g[2]);
            Ok(())
        });
    }
    Ok(())
}

pub fn fpsd_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| fpsd_body(&mut r));
    end();
}

fn glps_body(r: &mut Reader) -> Result<(), Failure> {
    let s = surface(r)?;
    let p = r.pnt();
    let (tu, tv) = (r.real(), r.real());
    let n = r.integer();
    let conf = rustycad_tkernel::precision::p_confusion();
    let mut e = GenLocateExtPS::new(&s, tu, tv);
    let mut d = GenLocateExtPS::new(&s, conf, conf);
    flag(e.is_done());
    attempt(|| Ok(real(e.square_distance()?)));
    attempt(|| Ok(ponsurf(e.point()?)));
    for _ in 0..n {
        let (u0, v0) = (r.real(), r.real());
        for crit in [false, true] {
            attempt(|| {
                e.perform(p, u0, v0, crit)?;
                flag(e.is_done());
                attempt(|| {
                    real(e.square_distance()?);
                    ponsurf(e.point()?);
                    Ok(())
                });
                Ok(())
            });
        }
        attempt(|| {
            d.perform(p, u0, v0, false)?;
            flag(d.is_done());
            attempt(|| Ok(real(d.square_distance()?)));
            Ok(())
        });
        attempt(|| Ok(flag(GenLocateExtPS::is_min_dist(p, &s, u0, v0)?)));
    }
    Ok(())
}

pub fn glps_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| glps_body(&mut r));
    end();
}

fn ext_flag(value: i32) -> ExtFlag {
    match value {
        0 => ExtFlag::Min,
        1 => ExtFlag::Max,
        _ => ExtFlag::MinMax,
    }
}

fn ext_algo(value: i32) -> ExtAlgo {
    if value == 0 {
        ExtAlgo::Grad
    } else {
        ExtAlgo::Tree
    }
}

fn other_algo(algo: ExtAlgo) -> ExtAlgo {
    if algo == ExtAlgo::Grad {
        ExtAlgo::Tree
    } else {
        ExtAlgo::Grad
    }
}

fn genps_body(r: &mut Reader) -> Result<(), Failure> {
    let s = surface(r)?;
    let p = r.pnt();
    let (nu, nv) = (r.integer(), r.integer());
    let (tu, tv) = (r.real(), r.real());
    let (fl, al) = (ext_flag(r.integer()), ext_algo(r.integer()));
    let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
    let p2 = r.pnt();
    let sampling = Sampling {
        nb_u: nu,
        nb_v: nv,
        tol_u: tu,
        tol_v: tv,
    };
    attempt(|| {
        let mut e = GenExtPS::with_point(p, &s, sampling, fl, al)?;
        ps_result(&e);
        attempt(|| {
            e.perform(p2)?;
            ps_result(&e);
            Ok(())
        });
        e.set_flag(ExtFlag::MinMax);
        attempt(|| {
            e.perform(p)?;
            ps_result(&e);
            Ok(())
        });
        Ok(())
    });
    attempt(|| {
        let mut e = GenExtPS::with_range(p, &s, sampling, (u1, u2), (v1, v2), (fl, al))?;
        ps_result(&e);
        e.set_algo(other_algo(al));
        attempt(|| {
            e.perform(p2)?;
            ps_result(&e);
            Ok(())
        });
        e.set_algo(al);
        attempt(|| {
            e.perform(p)?;
            ps_result(&e);
            Ok(())
        });
        Ok(())
    });
    let mut d = GenExtPS::new();
    flag(d.is_done());
    attempt(|| Ok(int(d.nb_ext()?)));
    attempt(|| {
        d.initialize(
            &s,
            Sampling {
                nb_u: 1,
                ..sampling
            },
        )
    });
    Ok(())
}

pub fn genps_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| genps_body(&mut r));
    end();
}

fn ps_body(r: &mut Reader) -> Result<(), Failure> {
    let s = surface(r)?;
    let p = r.pnt();
    let (tu, tv) = (r.real(), r.real());
    let (fl, al) = (ext_flag(r.integer()), ext_algo(r.integer()));
    let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
    let p2 = r.pnt();
    attempt(|| {
        let mut e = ExtPS::with_surface(p, &s, tu, tv, fl, al)?;
        ps_result(&e);
        attempt(|| {
            e.perform(p2)?;
            ps_result(&e);
            Ok(())
        });
        Ok(())
    });
    attempt(|| {
        let mut e = ExtPS::with_range(p, &s, (u1, u2), (v1, v2), (tu, tv), (fl, al))?;
        ps_result(&e);
        e.set_flag(ExtFlag::MinMax);
        e.set_algo(other_algo(al));
        attempt(|| {
            e.perform(p2)?;
            ps_result(&e);
            Ok(())
        });
        Ok(())
    });
    let mut d = ExtPS::new();
    flag(d.is_done());
    attempt(|| Ok(int(d.nb_ext()?)));
    attempt(|| {
        d.initialize(&s, (u1, u2), (v1, v2), (tu, tv))?;
        d.perform(p)?;
        ps_result(&d);
        let (t, q) = d.trimmed_square_distances();
        for i in 0..4 {
            real(t[i]);
            pnt(q[i]);
        }
        Ok(())
    });
    Ok(())
}

pub fn ps_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| ps_body(&mut r));
    end();
}
