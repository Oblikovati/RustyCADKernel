//! The Extrema surface-surface requests; the counterpart of
//! tools/oracle/tkgeombase_probe_extrema_ss.cxx (same encoding and token order).
use super::common::{Reader, attempt, end, flag, int, real};
use super::extrema_el::ponsurf;
use super::extrema_ps::surface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeombase::extrema::{ExtSS, FuncExtSS, GenExtSS};
use rustycad_tkmath::math::{Matrix, Vector};

fn vector(values: &[f64]) -> Vector<'static> {
    let mut v = Vector::with_bounds(1, values.len() as i32).unwrap();
    for (i, x) in values.iter().enumerate() {
        v[i as i32 + 1] = *x;
    }
    v
}

fn matrix(m: &Matrix<'_>) {
    for i in 1..=4 {
        for j in 1..=4 {
            real(m[(i, j)]);
        }
    }
}

fn fess_step(f: &mut FuncExtSS<'_>, x: &Vector<'_>) {
    let mut m = Matrix::filled(1, 4, 1, 4, 0.0).unwrap();
    attempt(|| {
        let mut v = vector(&[0.0; 4]);
        flag(f.value(x, &mut v)?);
        (1..=4).for_each(|i| real(v[i]));
        Ok(())
    });
    attempt(|| {
        flag(f.derivatives(x, &mut m)?);
        matrix(&m);
        Ok(())
    });
    attempt(|| {
        let mut w = vector(&[0.0; 4]);
        flag(f.values(x, &mut w, &mut m)?);
        (1..=4).for_each(|i| real(w[i]));
        matrix(&m);
        Ok(())
    });
    attempt(|| Ok(int(f.get_state_number()?)));
}

fn fess_body(r: &mut Reader) -> Result<(), Failure> {
    let a = surface(r)?;
    let b = surface(r)?;
    let n = r.integer();
    let mut d = FuncExtSS::new();
    attempt(|| {
        let mut f0 = vector(&[0.0; 4]);
        Ok(flag(d.value(&vector(&[0.5; 4]), &mut f0)?))
    });
    attempt(|| Ok(int(d.get_state_number()?)));
    int(d.nb_ext());
    attempt(|| Ok(real(d.square_distance(1)?)));
    attempt(|| Ok(ponsurf(d.point_on_s1(1)?)));
    attempt(|| Ok(ponsurf(d.point_on_s2(1)?)));
    let mut f = FuncExtSS::with_surfaces(&a, &b);
    int(f.nb_variables());
    int(f.nb_equations());
    attempt(|| Ok(int(f.get_state_number()?)));
    for _ in 0..n {
        let x = vector(&[r.real(), r.real(), r.real(), r.real()]);
        fess_step(&mut f, &x);
    }
    int(f.nb_ext());
    for i in 0..=f.nb_ext() + 1 {
        attempt(|| {
            real(f.square_distance(i)?);
            ponsurf(f.point_on_s1(i)?);
            ponsurf(f.point_on_s2(i)?);
            Ok(())
        });
    }
    f.initialize(&b, &a);
    int(f.nb_ext());
    Ok(())
}

pub fn fess_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| fess_body(&mut r));
    end();
}

fn genss_result(e: &GenExtSS<'_>) {
    flag(e.is_done());
    attempt(|| {
        let n = e.nb_ext()?;
        int(n);
        for i in 1..=n {
            real(e.square_distance(i)?);
            ponsurf(e.point_on_s1(i)?);
            ponsurf(e.point_on_s2(i)?);
        }
        Ok(())
    });
    attempt(|| Ok(real(e.square_distance(0)?)));
}

fn ranges(r: &mut Reader) -> ([f64; 4], [f64; 4]) {
    let p: Vec<f64> = (0..8).map(|_| r.real()).collect();
    ([p[0], p[1], p[2], p[3]], [p[4], p[5], p[6], p[7]])
}

fn genss_body(r: &mut Reader) -> Result<(), Failure> {
    let a = surface(r)?;
    let b = surface(r)?;
    let (nu, nv) = (r.integer(), r.integer());
    let (t1, t2) = (r.real(), r.real());
    let (p1, p2) = ranges(r);
    attempt(|| {
        genss_result(&GenExtSS::with_surfaces(&a, &b, nu, nv, t1, t2)?);
        Ok(())
    });
    attempt(|| {
        genss_result(&GenExtSS::with_ranges(&a, &b, (nu, nv), p1, p2, (t1, t2))?);
        Ok(())
    });
    let mut d = GenExtSS::new();
    flag(d.is_done());
    attempt(|| Ok(int(d.nb_ext()?)));
    attempt(|| {
        d.initialize_range(&b, (nu, nv), (p2[0], p2[1]), (p2[2], p2[3]), t2)?;
        d.perform_range(&a, (p1[0], p1[1]), (p1[2], p1[3]), t1)?;
        genss_result(&d);
        d.perform(&a, t1)?;
        genss_result(&d);
        Ok(())
    });
    Ok(())
}

pub fn genss_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| genss_body(&mut r));
    end();
}

fn ss_result(e: &ExtSS<'_>) {
    flag(e.is_done());
    attempt(|| Ok(flag(e.is_parallel()?)));
    attempt(|| {
        let n = e.nb_ext()?;
        int(n);
        for i in 1..=n {
            real(e.square_distance(i)?);
            attempt(|| {
                let (p, q) = e.points(i)?;
                ponsurf(p);
                ponsurf(q);
                Ok(())
            });
        }
        Ok(())
    });
    attempt(|| Ok(real(e.square_distance(0)?)));
}

fn ss_body(r: &mut Reader) -> Result<(), Failure> {
    let a = surface(r)?;
    let b = surface(r)?;
    let (t1, t2) = (r.real(), r.real());
    let (p1, p2) = ranges(r);
    attempt(|| {
        ss_result(&ExtSS::with_surfaces(&a, &b, t1, t2)?);
        Ok(())
    });
    attempt(|| {
        ss_result(&ExtSS::with_ranges(&a, &b, p1, p2, (t1, t2))?);
        Ok(())
    });
    let mut d = ExtSS::new();
    flag(d.is_done());
    attempt(|| Ok(int(d.nb_ext()?)));
    attempt(|| {
        d.initialize(&b, p2, t2)?;
        d.perform(&a, p1, t1)?;
        ss_result(&d);
        let bounds = [a.first_u_parameter()?, a.last_u_parameter()?, a.first_v_parameter()?, a.last_v_parameter()?];
        d.perform(&a, bounds, t1)?;
        ss_result(&d);
        Ok(())
    });
    Ok(())
}

pub fn ss_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| ss_body(&mut r));
    end();
}
