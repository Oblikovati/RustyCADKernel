//! The adaptor probe operations a2_adaptor, a2_load and a2_offset_load; the
//! counterpart of the *_case functions of tools/oracle/tkg2d_probe_adaptor.cxx.
use super::adaptor::{adaptor, adaptor_summary, built, intervals, make_curve_summary};
use super::bspl::desc;
use super::common::{self, Reader, attempt, ax, ax22, curve, end, flag, int, real, show};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dOffsetCurve};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dOffsetCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg2d::geom2d_eval::RepCurveDesc;

/// Summary, MakeCurve, Trim and ShallowCopy of one adaptor (a2_adaptor_case).
pub fn adaptor_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(a) = built(&mut r) else { return };
    let u = r.reals();
    let (t1, t2) = (r.real(), r.real());
    adaptor_summary(&a, &u);
    make_curve_summary(&*a, &u);
    attempt(|| {
        let t = a.trim(t1, t2, 1e-7)?;
        adaptor_summary(&t, &u);
        make_curve_summary(&*t, &u);
        Ok(())
    });
    attempt(|| {
        adaptor_summary(&a.shallow_copy()?, &u);
        Ok(())
    });
    end();
}

// The members safe on an empty Geom2dAdaptor_Curve (a2_empty_geom).
fn empty_geom(a: &Handle<Geom2dAdaptorCurve>) {
    flag(a.is_initialized());
    flag(a.curve().is_null());
    real(a.first_parameter().unwrap());
    real(a.last_parameter().unwrap());
    int(a.get_type().unwrap() as i32);
    show(a.continuity(), |c| int(c as i32));
    intervals(&**a);
    for r in [1e-3, 4.0] {
        show(a.resolution(r), real);
    }
    show(a.line(), |l| ax(l.position()));
    show(a.circle(), |c| ax22(c.position()));
    show(a.degree(), int);
    show(a.is_rational(), flag);
    show(a.nb_poles(), int);
    show(a.nb_knots(), int);
    flag(a.bezier().unwrap().is_null());
    flag(a.bspline().unwrap().is_null());
    attempt(|| a.trim(0.0, 1.0, 1e-7).map(|_| ()));
    show(a.shallow_copy(), |c| real(c.last_parameter().unwrap()));
    make_curve_summary(&**a, &[]);
}

fn curve_name(a: &Geom2dAdaptorCurve) {
    print!("{} ", a.curve().dynamic_type().unwrap().name());
}

/// Load sequences of Geom2dAdaptor_Curve (a2_load_case).
pub fn load_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c1) = common::built(&mut r) else {
        return;
    };
    let c2 = curve(&mut r).expect("probe curve");
    let u = r.reals();
    let (u1, u2) = (r.real(), r.real());
    let a = Handle::new(Geom2dAdaptorCurve::default());
    let dynamic: Handle<dyn Adaptor2dCurve2d> = a.cast();
    empty_geom(&a);
    let null: Handle<dyn Geom2dCurve> = Handle::default();
    attempt(|| a.load(&null));
    attempt(|| a.load_range(&null, 0.0, 1.0));
    attempt(|| {
        a.load(&c1)?;
        curve_name(&a);
        adaptor_summary(&dynamic, &u);
        Ok(())
    });
    attempt(|| {
        a.load_range(&c1, u1, u2)?;
        adaptor_summary(&dynamic, &u);
        Ok(())
    });
    attempt(|| a.load_range(&c1, u2 + 1.0, u2));
    real(a.first_parameter().unwrap());
    real(a.last_parameter().unwrap());
    attempt(|| {
        a.load_range(&c2, u1, u2)?;
        curve_name(&a);
        adaptor_summary(&dynamic, &u);
        Ok(())
    });
    // `a->Reset()`: `Handle::reset` is `a.reset(new ...)`, so dereference explicitly.
    (*a).reset();
    empty_geom(&a);
    attempt(|| {
        a.load(&c2)?;
        adaptor_summary(&dynamic, &u);
        Ok(())
    });
    end();
}

// The members safe on an Adaptor2d_OffsetCurve without a basis.
fn empty_offset(d: f64, w1: f64, w2: f64) {
    let e = Adaptor2dOffsetCurve::default();
    real(e.offset());
    real(e.first_parameter().unwrap());
    real(e.last_parameter().unwrap());
    flag(e.curve().is_null());
    real(e.resolution(d).unwrap());
    real(e.shallow_copy().unwrap().last_parameter().unwrap());
    show(e.trim(w1, w2, 1e-7), |t| real(t.first_parameter().unwrap()));
    e.load_offset_bounded(d, w1, w2);
    real(e.offset());
    real(e.last_parameter().unwrap());
}

fn offset_bounds(o: &Adaptor2dOffsetCurve) {
    real(o.offset());
    real(o.first_parameter().unwrap());
    real(o.last_parameter().unwrap());
}

/// Load members of Adaptor2d_OffsetCurve (a2_offset_load_case).
pub fn offset_load_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(b1) = built(&mut r) else { return };
    let b2 = adaptor(&mut r).expect("probe adaptor");
    let u = r.reals();
    let (d, w1, w2) = (r.real(), r.real(), r.real());
    empty_offset(d, w1, w2);
    let o = match Adaptor2dOffsetCurve::with_offset(&b1, d) {
        Ok(o) => Handle::new(o),
        Err(error) => {
            println!("{}", error.exception_type());
            return;
        }
    };
    let dynamic: Handle<dyn Adaptor2dCurve2d> = o.cast();
    adaptor_summary(&dynamic, &u);
    o.load_curve(&b2);
    offset_bounds(&o);
    flag(o.curve() == b2);
    adaptor_summary(&dynamic, &u);
    attempt(|| o.load_offset(d));
    offset_bounds(&o);
    adaptor_summary(&dynamic, &u);
    o.load_offset_bounded(-d, w1, w2);
    adaptor_summary(&dynamic, &u);
    end();
}

// Sets, or for a null descriptor clears, the evaluation representation (a2_set_rep).
pub fn set_rep(c: &Handle<dyn Geom2dCurve>, d: &Handle<dyn RepCurveDesc>) -> Result<(), Failure> {
    if let Some(b) = c.cast::<Geom2dBezierCurve>().get() {
        return update_rep(
            d,
            |x| b.set_eval_representation(x),
            || b.clear_eval_representation(),
        );
    }
    if let Some(b) = c.cast::<Geom2dBSplineCurve>().get() {
        return update_rep(
            d,
            |x| b.set_eval_representation(x),
            || b.clear_eval_representation(),
        );
    }
    if let Some(b) = c.cast::<Geom2dOffsetCurve>().get() {
        return update_rep(
            d,
            |x| b.set_eval_representation(x),
            || b.clear_eval_representation(),
        );
    }
    Ok(())
}

fn update_rep(
    d: &Handle<dyn RepCurveDesc>,
    set: impl FnOnce(Handle<dyn RepCurveDesc>) -> Result<(), Failure>,
    clear: impl FnOnce(),
) -> Result<(), Failure> {
    if d.is_null() {
        clear();
        return Ok(());
    }
    set(d.clone())
}

/// An adaptor on a curve with an evaluation representation (a2_evalrep_case).
pub fn evalrep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = common::built(&mut r) else {
        return;
    };
    let rep = curve(&mut r).expect("probe representation curve");
    let d = desc(&mut r, rep);
    let u = r.reals();
    let (u1, u2) = (r.real(), r.real());
    attempt(|| set_rep(&c, &d));
    let a = Handle::new(Geom2dAdaptorCurve::with_range(&c, u1, u2).expect("probe range"));
    let dynamic: Handle<dyn Adaptor2dCurve2d> = a.cast();
    adaptor_summary(&dynamic, &u);
    set_rep(&c, &Handle::default()).unwrap();
    adaptor_summary(&dynamic, &u);
    a.load_range(&c, u1, u2).unwrap();
    adaptor_summary(&dynamic, &u);
    let copy: Handle<dyn Geom2dCurve> = c.copy().unwrap().cast();
    a.load_range(&copy, u1, u2).unwrap();
    adaptor_summary(&dynamic, &u);
    end();
}
