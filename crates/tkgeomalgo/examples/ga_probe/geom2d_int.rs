//! Geom2dInt probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geom2dint.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show};
use super::curve2d::curve2d;
use super::int_curve::{domain, read_conic, read_domain};
use super::int_imp_par_gen::v2;
use super::int_res2d::inter;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_int::{
    Geom2dCurveTool as T, IntConicCurveOfGInter,
    MyImpParToolOfTheIntersectorOfTheIntConicCurveOfGInter,
    TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter, TheIntConicCurveOfGInter,
    TheProjPCurOfGInter,
};
use rustycad_tkgeomalgo::int_curve::IConicTool;
use rustycad_tkmath::math::{
    Function, FunctionSet, FunctionSetWithDerivatives, FunctionWithDerivative, Matrix, Vector,
};

fn adaptor(c: &Handle<dyn Geom2dCurve>) -> Result<Geom2dAdaptorCurve, Failure> {
    Geom2dAdaptorCurve::with_curve(c)
}

/// `curve u0 u1 u eps`: every member of Geom2dInt_Geom2dCurveTool.
pub fn tool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = curve2d(&mut r)?;
        let (u0, u1, u, eps) = (r.real(), r.real(), r.real(), r.real());
        let a = adaptor(&c)?;
        int(T::get_type(&a)? as i32);
        show(T::nb_samples(&a), int);
        show(T::nb_samples_in(&a, u0, u1), int);
        real(T::eps_x(&a));
        show(T::eps_x_of(&a, eps), real);
        real(T::first_parameter(&a)?);
        real(T::last_parameter(&a)?);
        show(T::value(&a, u), pnt2d);
        show(T::d0(&a, u), pnt2d);
        show(T::d1(&a, u), |(p, t)| {
            pnt2d(p);
            v2(t);
        });
        show(T::d2(&a, u), |(p, t, n)| {
            pnt2d(p);
            v2(t);
            v2(n);
        });
        show(T::d3(&a, u), |d| {
            pnt2d(d.0);
            v2(d.1);
            v2(d.2);
            v2(d.3);
        });
        show(T::dn(&a, u, 2), v2);
        attempt(|| intervals(&a));
        show(T::degree(&a), int);
        show(T::line(&a), |l| pnt2d(l.location()));
        show(T::circle(&a), |c| real(c.radius()));
        show(T::ellipse(&a), |e| real(e.minor_radius()));
        show(T::parabola(&a), |p| real(p.focal()));
        show(T::hyperbola(&a), |h| real(h.major_radius()));
        Ok(())
    });
    end();
}

fn intervals(a: &Geom2dAdaptorCurve) -> Result<(), Failure> {
    let nb = T::nb_intervals(a)?;
    int(nb);
    let mut tab: Array1<'static, f64> = Array1::with_bounds(1, nb + 1).unwrap();
    T::intervals(a, &mut tab)?;
    for i in 1..=nb + 1 {
        real(tab[i]);
    }
    let (x, y) = T::get_interval(a, 1, &tab)?;
    real(x);
    real(y);
    Ok(())
}

/// `curve x y low high`: both FindParameter of the projection.
pub fn proj_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = curve2d(&mut r)?;
        let p = r.pnt2d();
        let (low, high) = (r.real(), r.real());
        let a = adaptor(&c)?;
        show(TheProjPCurOfGInter::find_parameter(&a, p, 1e-7), real);
        show(
            TheProjPCurOfGInter::find_parameter_in(&a, p, low, high, 1e-7),
            real,
        );
        Ok(())
    });
    end();
}

/// `conic domain curve domain tolconf tol reversed which`.
pub fn conic_curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (k, sa) = (read_conic(&mut r), read_domain(&mut r));
        let c = curve2d(&mut r)?;
        let sb = read_domain(&mut r);
        let (tc, t, rev, which) = (r.real(), r.real(), r.flag(), r.integer());
        let ac = adaptor(&c)?;
        let da = domain(&sa, |u| k.value(u))?;
        let db = domain(&sb, |u| ac.value(u))?;
        if which == 0 {
            let mut i = IntConicCurveOfGInter::new();
            i.set_reversed_parameters(rev);
            match k.kind {
                0 => i.perform_line(k.lin(), &da, &ac, &db, tc, t)?,
                1 => i.perform_circle(k.circ()?, &da, &ac, &db, tc, t)?,
                2 => i.perform_ellipse(k.elips()?, &da, &ac, &db, tc, t)?,
                3 => i.perform_parabola(k.parab()?, &da, &ac, &db, tc, t)?,
                _ => i.perform_hyperbola(k.hypr()?, &da, &ac, &db, tc, t)?,
            }
            return inter(&i);
        }
        let mut i = TheIntConicCurveOfGInter::new();
        i.set_reversed_parameters(rev);
        match k.kind {
            0 => i.perform_line(k.lin(), &da, &ac, &db, tc, t)?,
            1 => i.perform_circle(k.circ()?, &da, &ac, &db, tc, t)?,
            2 => i.perform_ellipse(k.elips()?, &da, &ac, &db, tc, t)?,
            3 => i.perform_parabola(k.parab()?, &da, &ac, &db, tc, t)?,
            _ => i.perform_hyperbola(k.hypr()?, &da, &ac, &db, tc, t)?,
        }
        inter(&i)
    });
    end();
}

fn matrix_out(d: &Matrix<'_>) {
    for i in 1..=2 {
        for j in 1..=2 {
            real(d[(i, j)]);
        }
    }
}

/// `curve1 curve2 x1 x2 conic u`: the distance function and the conic function.
pub fn functions_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (c1, c2) = (curve2d(&mut r)?, curve2d(&mut r)?);
        let (x1, x2) = (r.real(), r.real());
        let k = read_conic(&mut r);
        let u = r.real();
        let (a1, a2) = (adaptor(&c1)?, adaptor(&c2)?);
        let mut f = TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter::new(&a1, &a2);
        let mut x: Vector<'static> = Vector::with_bounds(1, 2)?;
        (x[1], x[2]) = (x1, x2);
        let mut v: Vector<'static> = Vector::filled(1, 2, -3.0)?;
        let mut d: Matrix<'static> = Matrix::filled(1, 2, 1, 2, -4.0).unwrap();
        int(f.nb_variables());
        int(f.nb_equations());
        attempt(|| {
            kept(FunctionSet::value(&mut f, &x, &mut v), f.take_failure()).map(|ok| {
                flag(ok);
                real(v[1]);
                real(v[2]);
            })
        });
        attempt(|| {
            kept(f.derivatives(&x, &mut d), f.take_failure()).map(|ok| {
                flag(ok);
                matrix_out(&d);
            })
        });
        attempt(|| {
            kept(f.values(&x, &mut v, &mut d), f.take_failure()).map(|ok| {
                flag(ok);
                real(v[1]);
                real(v[2]);
                matrix_out(&d);
            })
        });
        conic_function(&super::int_curve::itool_of(&k)?, &a1, u);
        Ok(())
    });
    end();
}

// The result of a math callback, or the curve exception it kept.
fn kept(ok: bool, failure: Option<Failure>) -> Result<bool, Failure> {
    match failure {
        Some(failure) => Err(failure),
        None => Ok(ok),
    }
}

fn conic_function(tool: &IConicTool, a1: &dyn Adaptor2dCurve2d, u: f64) {
    let mut g = MyImpParToolOfTheIntersectorOfTheIntConicCurveOfGInter::new(tool, a1);
    let (mut fv, mut fd) = (-1.0, -2.0);
    attempt(|| {
        kept(Function::value(&mut g, u, &mut fv), g.take_failure()).map(|ok| {
            flag(ok);
            real(fv);
        })
    });
    attempt(|| {
        kept(g.derivative(u, &mut fd), g.take_failure()).map(|ok| {
            flag(ok);
            real(fd);
        })
    });
    (fv, fd) = (-1.0, -2.0);
    attempt(|| {
        kept(g.values(u, &mut fv, &mut fd), g.take_failure()).map(|ok| {
            flag(ok);
            real(fv);
            real(fd);
        })
    });
}
