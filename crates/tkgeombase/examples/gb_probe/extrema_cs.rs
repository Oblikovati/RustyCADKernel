//! The Extrema curve-surface requests; the counterpart of
//! tools/oracle/tkgeombase_probe_extrema_cs.cxx (same encoding and token order).
#![allow(clippy::unit_arg, clippy::type_complexity)]
use super::common::{Reader, attempt, end, flag, int, real};
use super::extrema_el::{poncurv, ponsurf};
use super::extrema_pc::{Space, Space3};
use super::extrema_ps::surface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::extrema::{
    ExtCS, FuncExtCS, GenExtCS, GlobOptFuncCQuadric, GlobOptFuncCS, GlobOptFuncConicS,
};
use rustycad_tkmath::math::{Matrix, Vector};

fn vector(values: &[f64]) -> Vector<'static> {
    let mut v = Vector::with_bounds(1, values.len() as i32).unwrap();
    for (i, x) in values.iter().enumerate() {
        v[i as i32 + 1] = *x;
    }
    v
}

fn matrix(m: &Matrix<'_>) {
    for i in m.lower_row()..=m.upper_row() {
        for j in m.lower_col()..=m.upper_col() {
            real(m[(i, j)]);
        }
    }
}

fn fecs_step(f: &mut FuncExtCS<'_>, x: &Vector<'_>) {
    let mut m = Matrix::filled(1, 3, 1, 3, 0.0).unwrap();
    attempt(|| {
        let mut v = vector(&[0.0; 3]);
        flag(f.value(x, &mut v)?);
        (1..=3).for_each(|i| real(v[i]));
        Ok(())
    });
    attempt(|| {
        flag(f.derivatives(x, &mut m)?);
        matrix(&m);
        Ok(())
    });
    attempt(|| {
        let mut w = vector(&[0.0; 3]);
        flag(f.values(x, &mut w, &mut m)?);
        (1..=3).for_each(|i| real(w[i]));
        matrix(&m);
        Ok(())
    });
    attempt(|| Ok(int(f.get_state_number()?)));
}

fn fecs_body(r: &mut Reader) -> Result<(), Failure> {
    let c = Space3::adaptor(r)?;
    let s = surface(r)?;
    let n = r.integer();
    let mut d = FuncExtCS::new();
    attempt(|| {
        let mut f0 = vector(&[0.0; 3]);
        Ok(flag(d.value(&vector(&[0.5; 3]), &mut f0)?))
    });
    attempt(|| Ok(int(d.get_state_number()?)));
    int(d.nb_ext());
    attempt(|| Ok(real(d.square_distance(1)?)));
    attempt(|| Ok(poncurv(d.point_on_curve(1)?)));
    attempt(|| Ok(ponsurf(d.point_on_surface(1)?)));
    let mut f = FuncExtCS::with_curve_surface(&*c, &s);
    int(f.nb_variables());
    int(f.nb_equations());
    for _ in 0..n {
        let x = vector(&[r.real(), r.real(), r.real()]);
        fecs_step(&mut f, &x);
    }
    int(f.nb_ext());
    for i in 0..=f.nb_ext() + 1 {
        attempt(|| {
            real(f.square_distance(i)?);
            poncurv(f.point_on_curve(i)?);
            ponsurf(f.point_on_surface(i)?);
            Ok(())
        });
    }
    f.square_distances().clear();
    int(f.nb_ext());
    int(f.points_on_curve().len() as i32);
    int(f.points_on_surf().len() as i32);
    f.initialize(&*c, &s);
    int(f.nb_ext());
    Ok(())
}

pub fn fecs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| fecs_body(&mut r));
    end();
}

fn gofcs_body(r: &mut Reader) -> Result<(), Failure> {
    let c = Space3::adaptor(r)?;
    let s = surface(r)?;
    let n = r.integer();
    let f = GlobOptFuncCS::new(&*c, &s);
    int(f.nb_variables());
    for _ in 0..n {
        let x = vector(&[r.real(), r.real(), r.real()]);
        let mut g = vector(&[0.0; 3]);
        let mut h = Matrix::filled(1, 3, 1, 3, 0.0).unwrap();
        attempt(|| {
            let mut v = 0.0;
            flag(f.value(&x, &mut v)?);
            real(v);
            Ok(())
        });
        attempt(|| {
            flag(f.gradient(&x, &mut g)?);
            (1..=3).for_each(|i| real(g[i]));
            Ok(())
        });
        attempt(|| {
            let mut v = 0.0;
            flag(f.values(&x, &mut v, &mut g)?);
            real(v);
            (1..=3).for_each(|i| real(g[i]));
            Ok(())
        });
        attempt(|| {
            let mut v = 0.0;
            flag(f.values_hessian(&x, &mut v, &mut g, &mut h)?);
            real(v);
            matrix(&h);
            Ok(())
        });
    }
    Ok(())
}

pub fn gofcs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| gofcs_body(&mut r));
    end();
}

fn ranges(r: &mut Reader) -> [f64; 6] {
    [r.real(), r.real(), r.real(), r.real(), r.real(), r.real()]
}

fn gofconics_body(r: &mut Reader) -> Result<(), Failure> {
    let c = Space3::adaptor(r)?;
    let s = surface(r)?;
    let [tf, tl, uf, ul, vf, vl] = ranges(r);
    let n = r.integer();
    let f = GlobOptFuncConicS::with_curve(&*c, &s)?;
    let mut g = GlobOptFuncConicS::with_bounds(&s, (uf, ul), (vf, vl));
    g.load_conic(&*c, tf, tl)?;
    int(f.nb_variables());
    for _ in 0..n {
        let x = vector(&[r.real(), r.real()]);
        for e in [&f, &g] {
            attempt(|| {
                let mut v = 0.0;
                flag(e.value(&x, &mut v)?);
                real(v);
                Ok(())
            });
            attempt(|| Ok(real(e.conic_parameter(&x)?)));
        }
    }
    Ok(())
}

pub fn gofconics_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| gofconics_body(&mut r));
    end();
}

fn gofcquad_body(r: &mut Reader) -> Result<(), Failure> {
    let c = Space3::adaptor(r)?;
    let s = surface(r)?;
    let [tf, tl, uf, ul, vf, vl] = ranges(r);
    let n = r.integer();
    let f = GlobOptFuncCQuadric::with_surface(&*c, &s)?;
    let mut g = GlobOptFuncCQuadric::with_range(&*c, tf, tl);
    g.load_quad(&s, (uf, ul), (vf, vl))?;
    int(f.nb_variables());
    for _ in 0..n {
        let x = vector(&[r.real()]);
        for e in [&f, &g] {
            attempt(|| {
                let mut v = 0.0;
                flag(e.value(&x, &mut v)?);
                real(v);
                Ok(())
            });
            attempt(|| {
                let mut uv = vector(&[0.0, 0.0]);
                e.quadric_parameters(&x, &mut uv)?;
                real(uv[1]);
                real(uv[2]);
                Ok(())
            });
        }
    }
    Ok(())
}

pub fn gofcquad_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| gofcquad_body(&mut r));
    end();
}

fn gencs_result(e: &GenExtCS<'_>) {
    flag(e.is_done());
    attempt(|| {
        let n = e.nb_ext()?;
        int(n);
        for i in 1..=n {
            real(e.square_distance(i)?);
            poncurv(e.point_on_curve(i)?);
            ponsurf(e.point_on_surface(i)?);
        }
        Ok(())
    });
    attempt(|| Ok(real(e.square_distance(0)?)));
}

fn genecs_body(r: &mut Reader) -> Result<(), Failure> {
    let c = Space3::adaptor(r)?;
    let s = surface(r)?;
    let (nt, nu, nv) = (r.integer(), r.integer(), r.integer());
    let (t1, t2) = (r.real(), r.real());
    let [a, b, u1, u2, v1, v2] = ranges(r);
    attempt(|| {
        gencs_result(&GenExtCS::with_curve(&*c, &s, (nt, nu, nv), t1, t2)?);
        Ok(())
    });
    attempt(|| {
        gencs_result(&GenExtCS::with_ranges(
            &*c,
            &s,
            (nt, nu, nv),
            [a, b, u1, u2, v1, v2],
            (t1, t2),
        )?);
        Ok(())
    });
    let mut d = GenExtCS::new();
    flag(d.is_done());
    attempt(|| Ok(int(d.nb_ext()?)));
    attempt(|| {
        d.initialize_range(&s, (nu, nv), (u1, u2), (v1, v2), t2)?;
        d.perform_range(&*c, nt, (a, b), t1)?;
        gencs_result(&d);
        d.perform(&*c, nt, t1)?;
        gencs_result(&d);
        Ok(())
    });
    Ok(())
}

pub fn genecs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| genecs_body(&mut r));
    end();
}

fn cs_result(e: &ExtCS<'_>) {
    flag(e.is_done());
    attempt(|| Ok(flag(e.is_parallel()?)));
    attempt(|| {
        let n = e.nb_ext()?;
        int(n);
        for i in 1..=n {
            real(e.square_distance(i)?);
            attempt(|| {
                let (p, q) = e.points(i)?;
                poncurv(p);
                ponsurf(q);
                Ok(())
            });
        }
        Ok(())
    });
    attempt(|| Ok(real(e.square_distance(0)?)));
}

fn cs_body(r: &mut Reader) -> Result<(), Failure> {
    let c = Space3::adaptor(r)?;
    let s = surface(r)?;
    let (tc, ts) = (r.real(), r.real());
    let [a, b, u1, u2, v1, v2] = ranges(r);
    attempt(|| {
        cs_result(&ExtCS::with_curve(&*c, &s, tc, ts)?);
        Ok(())
    });
    attempt(|| {
        cs_result(&ExtCS::with_ranges(
            &*c,
            &s,
            (a, b),
            (u1, u2),
            (v1, v2),
            (tc, ts),
        )?);
        Ok(())
    });
    let mut d = ExtCS::new();
    flag(d.is_done());
    attempt(|| Ok(int(d.nb_ext()?)));
    attempt(|| {
        d.initialize(&s, tc, ts)?;
        d.perform(&*c, a, b)?;
        cs_result(&d);
        d.perform(&*c, c.first_parameter()?, c.last_parameter()?)?;
        cs_result(&d);
        Ok(())
    });
    Ok(())
}

pub fn cs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| cs_body(&mut r));
    end();
}
