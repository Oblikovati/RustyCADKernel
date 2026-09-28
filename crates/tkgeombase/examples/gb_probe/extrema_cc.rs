//! The Extrema curve-curve requests; the counterpart of
//! tools/oracle/tkgeombase_probe_extrema_cc.cxx (same encoding and token order).
use super::common::{Reader, attempt, end, flag, int, real};
use super::extrema_pc::{Space, Space2, Space3};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::extrema::{
    CCLocFOfLocECC, CurveTool, ExtCC, ExtCC2d, ExtremaCurveTool, GGenExtCC, GlobOptFuncCCC0,
    GlobOptFuncCCC1, GlobOptFuncCCC2, LocECC, LocateExtCC, POnCurve,
};
use rustycad_tkmath::math::{Matrix, Vector};

type P<S> = <<S as Space>::Tool as ExtremaCurveTool>::Point;

/// The accessors of an Extrema_ExtCC-like result.
trait CcView<S: Space> {
    fn done(&self) -> bool;
    fn parallel(&self) -> Result<bool, Failure>;
    fn count(&self) -> Result<i32, Failure>;
    fn sq(&self, n: i32) -> Result<f64, Failure>;
    fn pair(&self, n: i32) -> Result<(POnCurve<P<S>>, POnCurve<P<S>>), Failure>;
}
macro_rules! cc_view {
    ($t:ty, $s:ty) => {
        impl CcView<$s> for $t {
            fn done(&self) -> bool {
                self.is_done()
            }
            fn parallel(&self) -> Result<bool, Failure> {
                self.is_parallel()
            }
            fn count(&self) -> Result<i32, Failure> {
                self.nb_ext()
            }
            fn sq(&self, n: i32) -> Result<f64, Failure> {
                self.square_distance(n)
            }
            fn pair(&self, n: i32) -> Result<(POnCurve<P<$s>>, POnCurve<P<$s>>), Failure> {
                self.points(n)
            }
        }
    };
}
cc_view!(ExtCC<'_>, Space3);
cc_view!(ExtCC2d<'_>, Space2);
cc_view!(GGenExtCC<'_, CurveTool>, Space3);
cc_view!(
    GGenExtCC<'_, rustycad_tkgeombase::extrema::Curve2dTool>,
    Space2
);

fn cc_result<S: Space>(e: &impl CcView<S>) {
    flag(e.done());
    attempt(|| Ok(flag(e.parallel()?)));
    attempt(|| {
        let n = e.count()?;
        int(n);
        for i in 1..=n {
            real(e.sq(i)?);
            attempt(|| {
                let (a, b) = e.pair(i)?;
                S::pon(a);
                S::pon(b);
                Ok(())
            });
        }
        Ok(())
    });
    attempt(|| Ok(real(e.sq(0)?)));
}

fn trimmed<S: Space>(d: [f64; 4], p: [P<S>; 4]) {
    for i in 0..4 {
        real(d[i]);
        S::pnt(p[i]);
    }
}

fn cc_body(r: &mut Reader) -> Result<(), Failure> {
    let a = Space3::adaptor(r)?;
    let b = Space3::adaptor(r)?;
    let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
    let single = r.flag();
    attempt(|| {
        let e = ExtCC::with_curves(&*a, &*b, 1.0e-10, 1.0e-10)?;
        cc_result::<Space3>(&e);
        if e.is_done() {
            let (d, p) = e.trimmed_square_distances();
            trimmed::<Space3>(d, p);
        }
        Ok(())
    });
    attempt(|| {
        let e = ExtCC::with_ranges(&*a, &*b, (u1, u2), (v1, v2), 1.0e-10, 1.0e-10)?;
        cc_result::<Space3>(&e);
        if e.is_done() {
            let (d, p) = e.trimmed_square_distances();
            trimmed::<Space3>(d, p);
        }
        Ok(())
    });
    let mut e = ExtCC::new(1.0e-9, 1.0e-8);
    e.set_single_solution_flag(single);
    flag(e.get_single_solution_flag());
    attempt(|| {
        e.initialize_ranges(&*a, &*b, (u1, u2), (v1, v2), (1.0e-9, 1.0e-8))?;
        e.perform()?;
        cc_result::<Space3>(&e);
        if e.is_done() {
            let (d, p) = e.trimmed_square_distances();
            trimmed::<Space3>(d, p);
        }
        e.perform()?;
        cc_result::<Space3>(&e);
        Ok(())
    });
    Ok(())
}

pub fn cc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| cc_body(&mut r));
    end();
}

fn cc2d_body(r: &mut Reader) -> Result<(), Failure> {
    let a = Space2::adaptor(r)?;
    let b = Space2::adaptor(r)?;
    let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
    let single = r.flag();
    attempt(|| {
        let e = ExtCC2d::with_curves(&*a, &*b, 1.0e-10, 1.0e-10)?;
        cc_result::<Space2>(&e);
        if e.is_done() {
            let (d, p) = e.trimmed_square_distances();
            trimmed::<Space2>(d, p);
        }
        Ok(())
    });
    attempt(|| {
        let e = ExtCC2d::with_ranges(&*a, &*b, (u1, u2), (v1, v2), 1.0e-10, 1.0e-10)?;
        cc_result::<Space2>(&e);
        if e.is_done() {
            let (d, p) = e.trimmed_square_distances();
            trimmed::<Space2>(d, p);
        }
        Ok(())
    });
    let mut e = ExtCC2d::new();
    e.set_single_solution_flag(single);
    flag(e.get_single_solution_flag());
    attempt(|| {
        e.initialize(&*b, v1, v2, (1.0e-9, 1.0e-8));
        e.perform(&*a, u1, u2)?;
        cc_result::<Space2>(&e);
        if e.is_done() {
            let (d, p) = e.trimmed_square_distances();
            trimmed::<Space2>(d, p);
        }
        e.perform(&*a, u1, u2)?;
        cc_result::<Space2>(&e);
        Ok(())
    });
    Ok(())
}

pub fn cc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| cc2d_body(&mut r));
    end();
}

fn ecc_body<S: Space>(r: &mut Reader) -> Result<(), Failure>
where
    for<'x> GGenExtCC<'x, S::Tool>: CcView<S>,
{
    let a = S::adaptor(r)?;
    let b = S::adaptor(r)?;
    let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
    let single = r.flag();
    let tol = r.real();
    attempt(|| {
        let mut e = GGenExtCC::<S::Tool>::with_curves(&*a, &*b)?;
        e.set_single_solution_flag(single);
        flag(e.get_single_solution_flag());
        e.set_tolerance(tol);
        e.perform()?;
        cc_result::<S>(&e);
        e.perform()?;
        cc_result::<S>(&e);
        Ok(())
    });
    attempt(|| {
        let mut e = GGenExtCC::<S::Tool>::with_bounds(&*a, &*b, (u1, u2), (v1, v2));
        e.perform()?;
        cc_result::<S>(&e);
        Ok(())
    });
    let mut d = GGenExtCC::<S::Tool>::new();
    attempt(|| Ok(flag(d.is_parallel()?)));
    d.set_params(&*a, &*b, (u1, u2), (v1, v2));
    attempt(|| {
        d.perform()?;
        cc_result::<S>(&d);
        Ok(())
    });
    Ok(())
}

pub fn ecc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| ecc_body::<Space3>(&mut r));
    end();
}
pub fn ecc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| ecc_body::<Space2>(&mut r));
    end();
}

fn locecc_body(r: &mut Reader) -> Result<(), Failure> {
    let a = Space3::adaptor(r)?;
    let b = Space3::adaptor(r)?;
    let n = r.integer();
    for _ in 0..n {
        let (u0, v0, tu, tv) = (r.real(), r.real(), r.real(), r.real());
        attempt(|| {
            let e = LocateExtCC::new(&*a, &*b, u0, v0)?;
            flag(e.is_done());
            attempt(|| Ok(real(e.square_distance()?)));
            attempt(|| {
                let (p, s) = e.point()?;
                Space3::pon(p);
                Space3::pon(s);
                Ok(())
            });
            Ok(())
        });
        attempt(|| {
            let e = LocECC::new::<CurveTool>(&*a, &*b, (u0, v0), (tu, tv))?;
            flag(e.is_done());
            attempt(|| Ok(real(e.square_distance()?)));
            attempt(|| {
                let (p, s) = e.point()?;
                Space3::pon(p);
                Space3::pon(s);
                Ok(())
            });
            Ok(())
        });
    }
    Ok(())
}

pub fn locecc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| locecc_body(&mut r));
    end();
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

fn fcc_step(f: &mut CCLocFOfLocECC<'_>, uv: &Vector<'_>) {
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
    int(f.get_state_number());
}

fn fcc_body(r: &mut Reader) -> Result<(), Failure> {
    let a = Space3::adaptor(r)?;
    let b = Space3::adaptor(r)?;
    let tol = r.real();
    let n = r.integer();
    let mut f = CCLocFOfLocECC::with_curves(&*a, &*b, tol)?;
    int(f.nb_variables());
    int(f.nb_equations());
    real(f.tolerance());
    attempt(|| Ok(real(f.search_of_tolerance(&*a)?)));
    attempt(|| Ok(real(f.search_of_tolerance(&*b)?)));
    let same = |c: Option<&dyn rustycad_tkg3d::adaptor3d::Adaptor3dCurve>,
                t: &dyn rustycad_tkg3d::adaptor3d::Adaptor3dCurve| {
        c.is_some_and(|c| std::ptr::addr_eq(c, t))
    };
    flag(same(f.curve_ptr(1)?, &*a));
    flag(same(f.curve_ptr(2)?, &*b));
    attempt(|| f.curve_ptr(3).map(|_| ()));
    for _ in 0..n {
        let uv = vec2(r.real(), r.real());
        fcc_step(&mut f, &uv);
    }
    int(f.nb_ext());
    for i in 0..=f.nb_ext() + 1 {
        attempt(|| {
            real(f.square_distance(i)?);
            let (p, s) = f.points(i)?;
            Space3::pon(p);
            Space3::pon(s);
            Ok(())
        });
    }
    f.set_tolerance(2.0 * tol);
    real(f.tolerance());
    attempt(|| f.set_curve(3, &*a));
    f.set_curve(1, &*b)?;
    flag(same(f.curve_ptr(1)?, &*b));
    f.set_curve(2, &*b)?;
    attempt(|| {
        let mut v = vec2(0.0, 0.0);
        flag(f.value(&vec2(0.25, 0.5), &mut v)?);
        real(v[1]);
        real(v[2]);
        Ok(())
    });
    let g = CCLocFOfLocECC::new(1.0e-10);
    real(g.tolerance());
    attempt(|| Ok(real(g.search_of_tolerance(&*a)?)));
    Ok(())
}

pub fn fcc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| fcc_body(&mut r));
    end();
}

fn gofcc_body<S: Space>(r: &mut Reader) -> Result<(), Failure> {
    let a = S::adaptor(r)?;
    let b = S::adaptor(r)?;
    let n = r.integer();
    let f0 = GlobOptFuncCCC0::<S::Tool>::new(&*a, &*b);
    let f1 = GlobOptFuncCCC1::<S::Tool>::new(&*a, &*b);
    let f2 = GlobOptFuncCCC2::<S::Tool>::new(&*a, &*b);
    int(f0.nb_variables());
    int(f1.nb_variables());
    int(f2.nb_variables());
    for _ in 0..n {
        let x = vec2(r.real(), r.real());
        let mut g = vec2(0.0, 0.0);
        let mut h = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
        let mut v = 0.0;
        attempt(|| {
            flag(f0.value(&x, &mut v)?);
            real(v);
            Ok(())
        });
        attempt(|| {
            v = 0.0;
            flag(f1.value(&x, &mut v)?);
            real(v);
            Ok(())
        });
        attempt(|| {
            flag(f1.gradient(&x, &mut g)?);
            real(g[1]);
            real(g[2]);
            Ok(())
        });
        attempt(|| {
            v = 0.0;
            flag(f1.values(&x, &mut v, &mut g)?);
            real(v);
            Ok(())
        });
        attempt(|| {
            v = 0.0;
            flag(f2.values(&x, &mut v, &mut g)?);
            real(v);
            real(g[1]);
            real(g[2]);
            Ok(())
        });
        attempt(|| {
            v = 0.0;
            flag(f2.values_hessian(&x, &mut v, &mut g, &mut h)?);
            real(v);
            matrix(&h);
            Ok(())
        });
    }
    Ok(())
}

pub fn gofcc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| gofcc_body::<Space3>(&mut r));
    end();
}
pub fn gofcc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| gofcc_body::<Space2>(&mut r));
    end();
}
