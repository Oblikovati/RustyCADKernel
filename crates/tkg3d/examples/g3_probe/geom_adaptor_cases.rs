//! The request cases of tools/oracle/tkg3d_probe_geom_adaptor.cxx (a3_*_case).
use super::bspl::desc;
use super::common::{
    Reader, attempt, ax1, ax3, built_curve, built_surface, dir, end, flag, int, name, real, show,
    trsf,
};
use super::geom_adaptor::{
    curve_adaptor_summary, curve_intervals, curve_kinds, make_curve_summary, make_surface_summary,
    surface_adaptor_summary, surface_kinds,
};
use super::shapes;
use super::swept::surface_desc;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{
    Adaptor3dCurve, Adaptor3dCurveBase, Adaptor3dSurface, Adaptor3dSurfaceBase,
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomCurve,
    GeomOffsetCurve, GeomOffsetSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution,
};
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorCurve, GeomAdaptorSurface, GeomAdaptorSurfaceOfLinearExtrusion,
    GeomAdaptorSurfaceOfRevolution, GeomAdaptorTransformedSurface,
};
use rustycad_tkg3d::geom_eval::{RepCurveDesc, RepSurfaceDesc};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis1, Direction3, Transform3};
use std::f64::consts::PI;

fn report<T>(built: Result<T, Failure>) -> Option<T> {
    match built {
        Ok(value) => Some(value),
        Err(error) => {
            println!("{}", error.exception_type());
            None
        }
    }
}

/// a3_base.
pub fn base_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let u = r.reals();
    let c: Handle<dyn Adaptor3dCurve> = Handle::new(Adaptor3dCurveBase::default()).cast();
    curve_adaptor_summary(&c, &u);
    attempt(|| c.shallow_copy().map(|_| ()));
    attempt(|| c.trim(0., 1., 1e-7).map(|_| ()));
    make_curve_summary(&*c, &u);
    let s: Handle<dyn Adaptor3dSurface> = Handle::new(Adaptor3dSurfaceBase::default()).cast();
    surface_adaptor_summary(&s, &u);
    attempt(|| s.shallow_copy().map(|_| ()));
    attempt(|| s.u_trim(0., 1., 1e-7).map(|_| ()));
    attempt(|| s.v_trim(0., 1., 1e-7).map(|_| ()));
    make_surface_summary(&*s, &u);
    end();
}

// a3_geom_curve: 0 GeomAdaptor_Curve(C) | 1 GeomAdaptor_Curve(C, u1, u2) <curve>.
pub fn geom_curve(r: &mut Reader) -> Result<Handle<GeomAdaptorCurve>, Failure> {
    let kind = r.integer();
    if kind == 0 {
        return Ok(Handle::new(GeomAdaptorCurve::with_curve(&shapes::curve(
            r,
        )?)?));
    }
    let (u1, u2) = (r.real(), r.real());
    Ok(Handle::new(GeomAdaptorCurve::with_range(
        &shapes::curve(r)?,
        u1,
        u2,
    )?))
}

/// a3_curve.
pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(a) = report(geom_curve(&mut r)) else {
        return;
    };
    let u = r.reals();
    let (t1, t2) = (r.real(), r.real());
    let adaptor: Handle<dyn Adaptor3dCurve> = a.clone().cast();
    name(&a.curve());
    curve_adaptor_summary(&adaptor, &u);
    make_curve_summary(&*adaptor, &u);
    attempt(|| {
        let t = a.trim(t1, t2, 1e-7)?;
        curve_adaptor_summary(&t, &u);
        make_curve_summary(&*t, &u);
        Ok(())
    });
    show(a.shallow_copy(), |c| curve_adaptor_summary(&c, &u));
    end();
}

// a3_empty_curve.
fn empty_curve(a: &Handle<GeomAdaptorCurve>) {
    name(&a.curve());
    real(a.first_parameter().unwrap());
    real(a.last_parameter().unwrap());
    int(a.get_type().unwrap() as i32);
    show(a.continuity(), |s| int(s as i32));
    curve_intervals(&**a);
    for r in [1e-3, 4.0] {
        show(a.resolution(r), real);
    }
    curve_kinds(&**a);
    attempt(|| a.trim(0., 1., 1e-7).map(|_| ()));
    show(a.shallow_copy(), |c| real(c.last_parameter().unwrap()));
    make_curve_summary(&**a, &[]);
}

/// a3_curve_load.
pub fn curve_load_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c1) = built_curve(&mut r) else {
        return;
    };
    let c2 = shapes::curve(&mut r).unwrap();
    let u = r.reals();
    let (u1, u2) = (r.real(), r.real());
    let a = Handle::new(GeomAdaptorCurve::default());
    let dyn_a: Handle<dyn Adaptor3dCurve> = a.clone().cast();
    empty_curve(&a);
    attempt(|| a.load(&Handle::default()));
    attempt(|| a.load_range(&Handle::default(), 0., 1.));
    attempt(|| {
        a.load(&c1)?;
        name(&a.curve());
        curve_adaptor_summary(&dyn_a, &u);
        Ok(())
    });
    attempt(|| {
        a.load_range(&c1, u1, u2)?;
        curve_adaptor_summary(&dyn_a, &u);
        Ok(())
    });
    attempt(|| a.load_range(&c1, u2 + 1., u2));
    real(a.first_parameter().unwrap());
    real(a.last_parameter().unwrap());
    attempt(|| {
        a.load_range(&c2, u1, u2)?;
        name(&a.curve());
        curve_adaptor_summary(&dyn_a, &u);
        Ok(())
    });
    // `a->Reset()`: `Handle::reset` is `a.reset(new ...)`, so dereference explicitly.
    (*a).reset();
    empty_curve(&a);
    attempt(|| {
        a.load(&c2)?;
        curve_adaptor_summary(&dyn_a, &u);
        Ok(())
    });
    end();
}

// a3_set_curve_rep: set (or clear, for a null descriptor) the representation.
fn set_curve_rep(c: &Handle<dyn GeomCurve>, d: &Handle<dyn RepCurveDesc>) -> Result<(), Failure> {
    macro_rules! apply {
        ($class:ty) => {
            if let Some(curve) = c.cast::<$class>().get() {
                if d.is_null() {
                    curve.clear_eval_representation();
                    return Ok(());
                }
                return curve.set_eval_representation(d.clone());
            }
        };
    }
    apply!(GeomBezierCurve);
    apply!(GeomBSplineCurve);
    apply!(GeomOffsetCurve);
    Ok(())
}

/// a3_curve_evalrep.
pub fn curve_evalrep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built_curve(&mut r) else { return };
    let rep = shapes::curve(&mut r).unwrap();
    let d = desc(&mut r, rep);
    let u = r.reals();
    let (u1, u2) = (r.real(), r.real());
    attempt(|| set_curve_rep(&c, &d));
    let a = Handle::new(GeomAdaptorCurve::with_range(&c, u1, u2).unwrap());
    let dyn_a: Handle<dyn Adaptor3dCurve> = a.clone().cast();
    curve_adaptor_summary(&dyn_a, &u);
    set_curve_rep(&c, &Handle::default()).unwrap();
    curve_adaptor_summary(&dyn_a, &u);
    a.load_range(&c, u1, u2).unwrap();
    curve_adaptor_summary(&dyn_a, &u);
    a.load_range(&c.copy().unwrap().cast(), u1, u2).unwrap();
    curve_adaptor_summary(&dyn_a, &u);
    end();
}

// a3_geom_surface: 0 GeomAdaptor_Surface(S) | 1 (S, u1, u2, v1, v2, tolu, tolv) <surface>.
fn geom_surface(r: &mut Reader) -> Result<Handle<GeomAdaptorSurface>, Failure> {
    let kind = r.integer();
    if kind == 0 {
        return Ok(Handle::new(GeomAdaptorSurface::with_surface(
            &shapes::surface(r)?,
        )?));
    }
    let bounds = (r.real(), r.real(), r.real(), r.real());
    let tolerances = (r.real(), r.real());
    let surface = shapes::surface(r)?;
    Ok(Handle::new(GeomAdaptorSurface::with_range(
        &surface,
        (bounds, tolerances),
    )?))
}

fn print_bounds(bounds: Result<(f64, f64, f64, f64), Failure>) {
    show(bounds, |(u1, u2, v1, v2)| {
        [u1, u2, v1, v2].into_iter().for_each(real)
    });
}

/// a3_surface.
pub fn surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(a) = report(geom_surface(&mut r)) else {
        return;
    };
    let uv = r.reals();
    let (u1, u2, v1, v2, tol) = (r.real(), r.real(), r.real(), r.real(), r.real());
    let adaptor: Handle<dyn Adaptor3dSurface> = a.clone().cast();
    name(&a.surface());
    real(a.tolerance_u());
    real(a.tolerance_v());
    print_bounds(a.bounds());
    surface_adaptor_summary(&adaptor, &uv);
    make_surface_summary(&*adaptor, &uv);
    show(a.u_trim(u1, u2, tol), |t| surface_adaptor_summary(&t, &uv));
    show(a.v_trim(v1, v2, tol), |t| surface_adaptor_summary(&t, &uv));
    show(a.shallow_copy(), |c| surface_adaptor_summary(&c, &uv));
    end();
}

/// a3_surface_load.
pub fn surface_load_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s1) = built_surface(&mut r) else {
        return;
    };
    let s2 = shapes::surface(&mut r).unwrap();
    let uv = r.reals();
    let (u1, u2, v1, v2, tol) = (r.real(), r.real(), r.real(), r.real(), r.real());
    let a = Handle::new(GeomAdaptorSurface::default());
    let dyn_a: Handle<dyn Adaptor3dSurface> = a.clone().cast();
    name(&a.surface());
    real(a.first_u_parameter().unwrap());
    real(a.last_v_parameter().unwrap());
    int(a.get_type().unwrap() as i32);
    surface_kinds(&*dyn_a);
    show(a.u_continuity(), |s| int(s as i32));
    show(a.nb_u_intervals(Shape::C1), int);
    show(a.u_resolution(0.1), real);
    attempt(|| a.load(&Handle::default()));
    attempt(|| a.load_bounds(&Handle::default(), (0., 1., 0., 1.)));
    attempt(|| {
        a.load(&s1)?;
        surface_adaptor_summary(&dyn_a, &uv);
        Ok(())
    });
    attempt(|| {
        a.load_range(&s1, ((u1, u2, v1, v2), (tol, tol)))?;
        surface_adaptor_summary(&dyn_a, &uv);
        Ok(())
    });
    attempt(|| a.load_bounds(&s1, (u2, u1, v1, v2)));
    attempt(|| a.load_bounds(&s1, (u1, u2, v2 + 1., v2)));
    print_bounds(a.bounds());
    real(a.tolerance_u());
    attempt(|| {
        a.load_range(&s2, ((u1, u2, v1, v2), (tol, 0.)))?;
        name(&a.surface());
        surface_adaptor_summary(&dyn_a, &uv);
        Ok(())
    });
    end();
}

// a3_set_surface_rep: set (or clear) the representation of the classes that accept one.
fn set_surface_rep(
    s: &Handle<dyn GeomSurface>,
    d: &Handle<dyn RepSurfaceDesc>,
) -> Result<bool, Failure> {
    macro_rules! apply {
        ($class:ty) => {
            if let Some(surface) = s.cast::<$class>().get() {
                if d.is_null() {
                    surface.clear_eval_representation();
                } else {
                    surface.set_eval_representation(d.clone())?;
                }
                return Ok(true);
            }
        };
    }
    apply!(GeomBezierSurface);
    apply!(GeomBSplineSurface);
    apply!(GeomOffsetSurface);
    apply!(GeomSurfaceOfLinearExtrusion);
    apply!(GeomSurfaceOfRevolution);
    Ok(false)
}

/// a3_surface_evalrep.
pub fn surface_evalrep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = built_surface(&mut r) else {
        return;
    };
    let rep = shapes::surface(&mut r).unwrap();
    let d = surface_desc(&mut r, rep);
    let uv = r.reals();
    let (u1, u2, v1, v2, tol) = (r.real(), r.real(), r.real(), r.real(), r.real());
    show(set_surface_rep(&s, &d), flag);
    let range = ((u1, u2, v1, v2), (tol, tol));
    let a = Handle::new(GeomAdaptorSurface::with_range(&s, range).unwrap());
    let dyn_a: Handle<dyn Adaptor3dSurface> = a.clone().cast();
    surface_adaptor_summary(&dyn_a, &uv);
    set_surface_rep(&s, &Handle::default()).unwrap();
    surface_adaptor_summary(&dyn_a, &uv);
    a.load_range(&s.copy().unwrap().cast(), range).unwrap();
    surface_adaptor_summary(&dyn_a, &uv);
    end();
}

// a3_empty_extrusion.
fn empty_extrusion(e: &GeomAdaptorSurfaceOfLinearExtrusion) {
    real(e.first_v_parameter().unwrap());
    real(e.last_v_parameter().unwrap());
    int(e.v_continuity().unwrap() as i32);
    int(e.nb_v_intervals(Shape::C2).unwrap());
    flag(e.is_v_closed().unwrap());
    flag(e.is_v_periodic().unwrap());
    show(e.v_period(), real);
    real(e.v_resolution(0.25).unwrap());
    attempt(|| e.cone().map(|_| ()));
    attempt(|| e.sphere().map(|_| ()));
    attempt(|| e.torus().map(|_| ()));
    attempt(|| e.axe_of_revolution().map(|_| ()));
    attempt(|| e.is_u_rational().map(|_| ()));
    attempt(|| e.is_v_rational().map(|_| ()));
    attempt(|| e.bezier().map(|_| ()));
    attempt(|| e.bspline().map(|_| ()));
    dir(e.direction().unwrap());
    name(&e.basis_curve().unwrap());
    show(e.v_degree(), int);
    show(e.nb_v_poles(), int);
    show(e.nb_v_knots(), int);
    attempt(|| e.basis_surface().map(|_| ()));
    show(e.offset_value(), real);
    name(&e.surface());
}

/// a3_extrusion.
pub fn extrusion_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = report(geom_curve(&mut r)) else {
        return;
    };
    let uv = r.reals();
    let d: Direction3 = r.dir();
    let mode = r.integer();
    let (v1, v2, u1, u2) = (r.real(), r.real(), r.real(), r.real());
    let basis: Handle<dyn Adaptor3dCurve> = c.clone().cast();
    let e = Handle::new(match mode {
        0 => GeomAdaptorSurfaceOfLinearExtrusion::with_curve(&basis),
        _ => GeomAdaptorSurfaceOfLinearExtrusion::with_curve_and_direction(&basis, d),
    });
    let dyn_e: Handle<dyn Adaptor3dSurface> = e.clone().cast();
    name(&e.surface());
    surface_adaptor_summary(&dyn_e, if mode == 0 { &[] } else { &uv });
    print_bounds(e.bounds());
    e.load_direction(d);
    surface_adaptor_summary(&dyn_e, &uv);
    make_surface_summary(&*dyn_e, &uv);
    attempt(|| {
        e.load_curve(&c.trim(u1, u2, 1e-7)?);
        surface_adaptor_summary(&dyn_e, &uv);
        Ok(())
    });
    show(e.u_trim(u1, u2, 1e-7), |t| surface_adaptor_summary(&t, &uv));
    show(e.v_trim(v1, v2, 1e-7), |t| surface_adaptor_summary(&t, &uv));
    show(e.shallow_copy(), |t| surface_adaptor_summary(&t, &uv));
    let empty = Handle::new(GeomAdaptorSurfaceOfLinearExtrusion::default());
    empty_extrusion(&empty);
    let copy: Handle<GeomAdaptorSurfaceOfLinearExtrusion> = empty.shallow_copy().unwrap().cast();
    empty_extrusion(&copy);
    empty.load_direction(d);
    empty_extrusion(&empty);
    end();
}

// a3_empty_revolution.
fn empty_revolution(e: &GeomAdaptorSurfaceOfRevolution) {
    ax3(e.axis());
    real(e.first_u_parameter().unwrap());
    real(e.last_u_parameter().unwrap());
    int(e.u_continuity().unwrap() as i32);
    int(e.nb_u_intervals(Shape::C2).unwrap());
    let mut t = Array1::<f64>::with_bounds(3, 4).unwrap();
    e.u_intervals(&mut t, Shape::C1).unwrap();
    t.iter().for_each(|v| real(*v));
    flag(e.is_u_closed().unwrap());
    flag(e.is_u_periodic().unwrap());
    real(e.u_period().unwrap());
    real(e.u_resolution(0.25).unwrap());
    attempt(|| e.nb_v_knots().map(|_| ()));
    attempt(|| e.is_u_rational().map(|_| ()));
    attempt(|| e.is_v_rational().map(|_| ()));
    attempt(|| e.bezier().map(|_| ()));
    attempt(|| e.bspline().map(|_| ()));
    ax1(e.axe_of_revolution().unwrap());
    name(&e.basis_curve().unwrap());
    show(e.u_degree(), int);
    show(e.nb_u_poles(), int);
    show(e.nb_u_knots(), int);
    attempt(|| e.direction().map(|_| ()));
    attempt(|| e.basis_surface().map(|_| ()));
    show(e.offset_value(), real);
    attempt(|| e.u_trim(0.1, 1., 1e-7).map(|_| ()));
    name(&e.surface());
}

/// a3_revolution.
pub fn revolution_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = report(geom_curve(&mut r)) else {
        return;
    };
    let uv = r.reals();
    let axis: Axis1 = r.ax1();
    let mode = r.integer();
    let (v1, v2, u1, u2) = (r.real(), r.real(), r.real(), r.real());
    let basis: Handle<dyn Adaptor3dCurve> = c.clone().cast();
    let built = match mode {
        0 => Ok(GeomAdaptorSurfaceOfRevolution::with_curve(&basis)),
        _ => GeomAdaptorSurfaceOfRevolution::with_curve_and_axis(&basis, axis),
    };
    let Some(e) = report(built) else { return };
    let e = Handle::new(e);
    let dyn_e: Handle<dyn Adaptor3dSurface> = e.clone().cast();
    ax3(e.axis());
    if mode != 0 {
        surface_adaptor_summary(&dyn_e, &uv);
    }
    attempt(|| e.load_axis(axis));
    ax3(e.axis());
    surface_adaptor_summary(&dyn_e, &uv);
    make_surface_summary(&*dyn_e, &uv);
    attempt(|| {
        e.load_curve(&c.trim(v1, v2, 1e-7)?)?;
        ax3(e.axis());
        surface_adaptor_summary(&dyn_e, &uv);
        Ok(())
    });
    show(e.u_trim(u1, u2, 1e-7), |t| surface_adaptor_summary(&t, &uv));
    show(e.u_trim(0., 2. * PI, 1e-7), |t| {
        surface_adaptor_summary(&t, &uv)
    });
    show(e.v_trim(v1, v2, 1e-7), |t| surface_adaptor_summary(&t, &uv));
    show(e.shallow_copy(), |t| surface_adaptor_summary(&t, &uv));
    let empty = Handle::new(GeomAdaptorSurfaceOfRevolution::default());
    empty_revolution(&empty);
    let copy: Handle<GeomAdaptorSurfaceOfRevolution> = empty.shallow_copy().unwrap().cast();
    empty_revolution(&copy);
    end();
}

// a3_transformed_accessors.
fn transformed_accessors(t: &GeomAdaptorTransformedSurface) {
    flag(t.has_trsf());
    name(&t.geom_surface_original());
    real(t.tolerance_u());
    real(t.tolerance_v());
    attempt(|| {
        let transformed = t.geom_surface_transformed()?;
        name(&transformed);
        flag(transformed == t.geom_surface_transformed()?);
        flag(transformed == t.geom_surface_original());
        Ok(())
    });
    attempt(|| {
        let s = t.adaptor_surface_transformed()?;
        flag(s == t.adaptor_surface_original());
        real(s.first_u_parameter()?);
        int(s.get_type()? as i32);
        Ok(())
    });
}

fn transformed_state(t: &Handle<GeomAdaptorTransformedSurface>, uv: &[f64]) {
    transformed_accessors(t);
    surface_adaptor_summary(&t.clone().cast(), uv);
}

/// a3_transformed.
pub fn transformed_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(a) = report(geom_surface(&mut r)) else {
        return;
    };
    let uv = r.reals();
    let (m, m2): (Transform3, Transform3) = (trsf(&mut r), trsf(&mut r));
    let tol = r.real();
    let (b0, b1, b2, b3) = a.bounds().unwrap();
    let range = ((b0, b1, b2, b3), (tol, tol));
    let built = GeomAdaptorTransformedSurface::with_range(&a.surface(), range, m);
    let Some(t) = report(built) else { return };
    let t = Handle::new(t);
    transformed_state(&t, &uv);
    make_surface_summary(&*t, &uv);
    show(t.u_trim(b0, (b0 + b1) / 2., tol), |c| {
        surface_adaptor_summary(&c, &uv)
    });
    show(t.v_trim(b2, (b2 + b3) / 2., tol), |c| {
        surface_adaptor_summary(&c, &uv)
    });
    show(t.shallow_copy(), |c| surface_adaptor_summary(&c, &uv));
    t.set_trsf(m2);
    transformed_state(&t, &uv);
    t.set_trsf(Transform3::default());
    transformed_state(&t, &uv);
    attempt(|| {
        t.load(&a.surface(), m)?;
        transformed_state(&t, &uv);
        Ok(())
    });
    let empty = GeomAdaptorTransformedSurface::default();
    transformed_accessors(&empty);
    empty.set_trsf(m);
    transformed_accessors(&empty);
    end();
}
