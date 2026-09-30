//! GeomAdaptor_TransformedCurve, Adaptor3d_HVertex and Adaptor3d_TopolTool; the
//! counterpart of tools/oracle/tkg3d_probe_topology.cxx.
use super::adaptor3d_inputs::{adaptor2d, inputs, pnt2, surface};
use super::common::{Reader, attempt, end, flag, int, name, pnt, real, show, trsf};
use super::geom_adaptor::{curve_adaptor_summary, make_curve_summary};
use super::geom_adaptor_cases::geom_curve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{
    Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dHVertex, Adaptor3dHVertexBase,
    Adaptor3dSurface, Adaptor3dTopolTool,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorTransformedCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Cone3, Point2, Transform3};

fn report<T>(built: Result<T, Failure>) -> Option<T> {
    match built {
        Ok(value) => Some(value),
        Err(error) => {
            println!("{}", error.exception_type());
            None
        }
    }
}

// a3_transformed_curve_state.
fn transformed_state(t: &Handle<GeomAdaptorTransformedCurve>, u: &[f64]) {
    flag(t.is_3d_curve());
    flag(t.is_curve_on_surface());
    int(t.trsf().form() as i32);
    name(&t.geom_curve());
    real(t.curve().first_parameter().unwrap());
    real(t.curve().last_parameter().unwrap());
    flag(t.change_curve() == t.curve());
    curve_adaptor_summary(&t.clone().cast(), u);
}

fn trimmed_and_copied(t: &Handle<GeomAdaptorTransformedCurve>, u: &[f64], (t1, t2): (f64, f64)) {
    show(t.trim(t1, t2, 1e-7), |x| curve_adaptor_summary(&x, u));
    show(t.shallow_copy(), |x| curve_adaptor_summary(&x, u));
}

// The curve-on-surface part of a3_transformed_curve_case.
fn curve_on_surface_part(
    t: &Handle<GeomAdaptorTransformedCurve>,
    u: &[f64],
    range: (f64, f64),
    pair: (Handle<dyn Adaptor2dCurve2d>, Handle<dyn Adaptor3dSurface>),
) {
    let built = Adaptor3dCurveOnSurface::with_curve(&pair.0, &pair.1);
    let o = match built {
        Ok(o) => Handle::new(o),
        Err(error) => {
            print!("{} ", error.exception_type());
            return;
        }
    };
    t.load_curve_on_surface(&o);
    flag(t.curve_on_surface().unwrap() == o);
    transformed_state(t, u);
    trimmed_and_copied(t, u, range);
    t.load_curve_on_surface(&Handle::default());
    flag(t.is_3d_curve());
}

/// a3_transformed_curve.
pub fn transformed_curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = report(geom_curve(&mut r)) else {
        return;
    };
    let u = r.reals();
    let m: Transform3 = trsf(&mut r);
    let range = (r.real(), r.real());
    let cos = r.flag();
    let pair = if cos {
        let Some(pair) = inputs(&mut r) else { return };
        Some(pair)
    } else {
        None
    };
    let (first, last) = (c.first_parameter().unwrap(), c.last_parameter().unwrap());
    let t =
        Handle::new(GeomAdaptorTransformedCurve::with_range(&c.curve(), first, last, m).unwrap());
    transformed_state(&t, &u);
    make_curve_summary(&*t, &u);
    trimmed_and_copied(&t, &u, range);
    if let Some(pair) = pair {
        curve_on_surface_part(&t, &u, range, pair);
    }
    t.set_trsf(Transform3::default());
    transformed_state(&t, &u);
    reloads(&t, &c.curve(), &u, range);
    empty_transformed();
    end();
}

fn reloads(
    t: &Handle<GeomAdaptorTransformedCurve>,
    curve: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    u: &[f64],
    (t1, t2): (f64, f64),
) {
    attempt(|| {
        t.load(curve)?;
        transformed_state(t, u);
        Ok(())
    });
    attempt(|| {
        t.load_range(curve, t1, t2)?;
        transformed_state(t, u);
        Ok(())
    });
    attempt(|| {
        t.change_curve().load(curve)?;
        transformed_state(t, u);
        Ok(())
    });
    attempt(|| t.load(&Handle::default()));
}

fn empty_transformed() {
    let e = GeomAdaptorTransformedCurve::default();
    flag(e.is_3d_curve());
    int(e.trsf().form() as i32);
    name(&e.geom_curve());
    show(e.first_parameter(), real);
    attempt(|| e.offset_curve().map(|_| ()));
    attempt(|| e.trim(0., 1., 1e-7).map(|_| ()));
}

// a3_hvertex_state.
fn hvertex_state(
    v: &Handle<dyn Adaptor3dHVertex>,
    c: &Handle<dyn Adaptor2dCurve2d>,
    o: &Handle<dyn Adaptor3dHVertex>,
) {
    let p: Point2 = v.value();
    real(p.x());
    real(p.y());
    int(v.orientation() as i32);
    real(v.resolution(c).expect("the base resolution cannot fail"));
    show(v.parameter(c), real);
    flag(v.is_same(o).unwrap());
    flag(v.is_same(v).unwrap());
}

fn orientation(value: i32) -> Orientation {
    Orientation::ALL[value as usize]
}

/// a3_hvertex.
pub fn hvertex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = pnt2(&mut r);
    let ori = r.integer();
    let res = r.real();
    let q = pnt2(&mut r);
    let Some(c) = report(adaptor2d(&mut r)) else {
        return;
    };
    let v = Handle::new(Adaptor3dHVertexBase::new(p, orientation(ori), res)).cast();
    let o = Handle::new(Adaptor3dHVertexBase::new(q, Orientation::Reversed, 0.)).cast();
    hvertex_state(&v, &c, &o);
    hvertex_state(&o, &c, &v);
    hvertex_state(&Handle::new(Adaptor3dHVertexBase::default()).cast(), &c, &v);
    end();
}

// a3_topol_restrictions.
fn restrictions(t: &Adaptor3dTopolTool) {
    t.init();
    while t.more() {
        let c: Handle<dyn Adaptor2dCurve2d> = t.value().unwrap();
        name(&c);
        real(c.first_parameter().unwrap());
        real(c.last_parameter().unwrap());
        let l = c.line().unwrap();
        [
            l.location().x(),
            l.location().y(),
            l.direction().x(),
            l.direction().y(),
        ]
        .into_iter()
        .for_each(real);
        int(t.curve_orientation(&c) as i32);
        show(t.curve_tol_3d(&c), real);
        t.initialize_curve(&c).unwrap();
        vertices(t, &c);
        attempt(|| t.vertex().map(|_| ()));
        t.next();
    }
    attempt(|| t.value().map(|_| ()));
}

fn vertices(t: &Adaptor3dTopolTool, c: &Handle<dyn Adaptor2dCurve2d>) {
    let mut first: Handle<dyn Adaptor3dHVertex> = Handle::default();
    t.init_vertex_iterator();
    while t.more_vertex() {
        let v = t.vertex().unwrap();
        if first.is_null() {
            first = v.clone();
        }
        hvertex_state(&v, c, &first);
        int(t.vertex_orientation(&v).unwrap() as i32);
        flag(t.identical(&v, &first).unwrap());
        show(t.vertex_tol_3d(&v), real);
        show(t.pnt(&v), pnt);
        t.next_vertex();
    }
}

// a3_topol_samples.
fn samples(t: &Adaptor3dTopolTool) -> Result<(), Failure> {
    let (nu, nv) = (t.nb_samples_u()?, t.nb_samples_v()?);
    int(nu);
    int(nv);
    int(t.nb_samples()?);
    for i in [1, 2, nu + 1, nu.wrapping_mul(nv)] {
        show(t.sample_point(i), |(p, q)| {
            real(p.x());
            real(p.y());
            pnt(q);
        });
    }
    Ok(())
}

// a3_topol_parameters.
fn parameters(t: &Adaptor3dTopolTool) -> Result<(), Failure> {
    let (a, b) = (t.u_parameters()?, t.v_parameters()?);
    for array in [a, b] {
        int(array.lower());
        int(array.upper());
        array.iter().for_each(|x| real(*x));
    }
    Ok(())
}

/// a3_topol.
pub fn topol_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = report(surface(&mut r)) else {
        return;
    };
    let pts = r.reals();
    let (tol, defl) = (r.real(), r.real());
    let (numin, nvmin) = (r.integer(), r.integer());
    let Some(t) = report(Adaptor3dTopolTool::with_surface(&s)) else {
        return;
    };
    restrictions(&t);
    for pair in pts.as_chunks::<2>().0 {
        let p = Point2::new(pair[0], pair[1]);
        int(t.classify(p, tol, true) as i32);
        flag(t.is_the_point_on(p, tol, true));
    }
    flag(t.has_3d());
    flag(t.domain_is_infinite());
    flag(t.edge().is_none());
    attempt(|| t.initialize());
    show(t.is_uniform_sampling(), flag);
    attempt(|| samples(&t));
    attempt(|| {
        t.compute_sample_points()?;
        samples(&t)
    });
    attempt(|| {
        t.sample_pnts(defl, numin, nvmin)?;
        samples(&t)?;
        parameters(&t)
    });
    spline_and_cone(&t, &s, (defl, numin, nvmin));
    empty_tool(&s, tol);
    end();
}

fn spline_and_cone(
    t: &Adaptor3dTopolTool,
    s: &Handle<dyn Adaptor3dSurface>,
    (defl, numin, nvmin): (f64, i32, i32),
) {
    let kind = s.get_type().unwrap();
    if kind == SurfaceType::BSplineSurface {
        attempt(|| {
            t.bspl_sample_pnts(defl, nvmin, numin)?;
            samples(t)?;
            parameters(t)
        });
    }
    if kind == SurfaceType::Cone {
        let (u, v) = Adaptor3dTopolTool::get_cone_apex_param(&s.cone().unwrap());
        real(u);
        real(v);
    }
}

fn empty_tool(s: &Handle<dyn Adaptor3dSurface>, tol: f64) {
    let e = Adaptor3dTopolTool::default();
    e.init();
    flag(e.more());
    e.init_vertex_iterator();
    flag(e.more_vertex());
    attempt(|| e.value().map(|_| ()));
    attempt(|| e.vertex().map(|_| ()));
    int(e.classify(Point2::new(0., 0.), tol, true) as i32);
    flag(e.is_the_point_on(Point2::new(0., 0.), tol, true));
    flag(e.domain_is_infinite());
    e.initialize_surface(s).unwrap();
    restrictions(&e);
}

/// a3_cone_apex.
pub fn cone_apex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let frame = r.ax3();
    let (radius, angle) = (r.real(), r.real());
    show(Cone3::new(frame, angle, radius), |c| {
        let (u, v) = Adaptor3dTopolTool::get_cone_apex_param(&c);
        real(u);
        real(v);
    });
    end();
}
