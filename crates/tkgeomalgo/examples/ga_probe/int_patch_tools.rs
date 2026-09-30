//! IntPatch tool probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intpatch_tools.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, show};
use super::curve2d::curve2d;
use super::ics_geometry::surface;
use super::int_patch::Fixture;
use super::int_surf::quadric;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchArcFunction, IntPatchCSFunction, IntPatchCurvIntSurf, IntPatchHCurve2dTool,
    IntPatchHInterTool, curvature_radius_of_inters_line,
};
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkgeomalgo::int_surf::IntSurfPntOn2S;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Vector2;
use rustycad_tkmath::math::{Matrix, Vector};

/// `ga_ip_arc`.
pub fn arc(r: &mut Reader) -> Result<TheArc, Failure> {
    let c = curve2d(r)?;
    if !r.flag() {
        return Ok(Handle::new(Geom2dAdaptorCurve::with_curve(&c)?).cast());
    }
    let a = r.real();
    Ok(Handle::new(Geom2dAdaptorCurve::with_range(&c, a, r.real())?).cast())
}

fn v2d(v: Vector2) {
    real(v.x());
    real(v.y());
}

type T = IntPatchHInterTool;

fn hinter_arc(fx: &Fixture, a: &TheArc, p: rustycad_tkmath::gp::Point2) {
    show(T::nb_samples_on_arc(a), int);
    show(T::bounds(a), |(f, l)| {
        real(f);
        real(l);
    });
    show(T::project(a, p), |found| {
        flag(found.is_some());
        if let Some((prm, q)) = found {
            real(prm);
            pnt2d(q);
        }
    });
    show(T::tolerance(&fx.vertex(1), a), real);
    show(T::parameter(&fx.vertex(1), a), real);
    flag(T::has_been_seen(a));
    int(T::nb_points(a));
    attempt(|| T::value(a, 1).map(|_| ()));
    flag(T::is_vertex(a, 1));
    attempt(|| T::vertex(a, 1).map(|_| ()));
    int(T::nb_segments(a));
    attempt(|| T::has_first_point(a, 1).map(|_| ()));
    attempt(|| T::has_last_point(a, 1).map(|_| ()));
    flag(T::is_all_solution(a));
}

/// `ga_ip_hinter_case`.
pub fn hinter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let s = surface(&mut r);
        flag(T::singular_on_u_min(&s));
        flag(T::singular_on_u_max(&s));
        flag(T::singular_on_v_min(&s));
        flag(T::singular_on_v_max(&s));
        show(T::nb_samples_u(&s, 0.0, 1.0), int);
        show(T::nb_samples_v(&s, 0.0, 1.0), int);
        attempt(|| {
            let mut tool = T::new();
            let n = tool.nb_sample_points(&s)?;
            int(n);
            for i in 0..=n + 1 {
                let (u, v) = tool.sample_point(&s, i)?;
                real(u);
                real(v);
            }
            Ok(())
        });
        let mut arcs: Vec<TheArc> = Vec::new();
        for _ in 0..r.integer() {
            arcs.push(arc(&mut r)?);
        }
        arcs.push(fx.arc(1));
        let p = r.pnt2d();
        arcs.iter().for_each(|a| hinter_arc(&fx, a, p));
        Ok(())
    });
    end();
}

type C = IntPatchHCurve2dTool;

fn shape(k: i32) -> Shape {
    [Shape::C0, Shape::G1, Shape::C1, Shape::G2][k as usize]
}

fn hc2d_intervals(a: &TheArc, k: i32) -> Result<(), Failure> {
    let n = C::nb_intervals(a, shape(k))?;
    int(n);
    let mut t: Array1<'static, f64> =
        Array1::with_bounds(1, n + 1).map_err(|e| e.into_failure("NCollection_Array1"))?;
    C::intervals(a, &mut t, shape(k))?;
    (1..=n + 1).for_each(|i| real(t[i]));
    Ok(())
}

fn hc2d_derivatives(a: &TheArc, u: f64) {
    show(C::value(a, u), pnt2d);
    show(C::d0(a, u), pnt2d);
    show(C::d1(a, u), |(p, v)| {
        pnt2d(p);
        v2d(v);
    });
    show(C::d2(a, u), |(p, v, w)| {
        pnt2d(p);
        v2d(v);
        v2d(w);
    });
    show(C::d3(a, u), |(p, v, w, x)| {
        pnt2d(p);
        v2d(v);
        v2d(w);
        v2d(x);
    });
    (1..=3).for_each(|n| show(C::dn(a, u, n), v2d));
}

fn hc2d_conics(a: &TheArc) {
    show(C::line(a), |x| pnt2d(x.location()));
    show(C::circle(a), |x| real(x.radius()));
    show(C::ellipse(a), |x| real(x.minor_radius()));
    show(C::hyperbola(a), |x| real(x.major_radius()));
    show(C::parabola(a), |x| real(x.focal()));
    show(C::bezier(a), |x| {
        flag(x.is_null());
        if let Some(b) = x.get() {
            int(b.nb_poles());
        }
    });
    show(C::bspline(a), |x| {
        flag(x.is_null());
        if let Some(b) = x.get() {
            int(b.nb_knots());
        }
    });
}

/// `ga_ip_hc2d_case`.
pub fn hc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = arc(&mut r)?;
        let (u, u0, u1) = (r.real(), r.real(), r.real());
        real(C::first_parameter(&a)?);
        real(C::last_parameter(&a)?);
        int(C::continuity(&a)? as i32);
        (0..=3).for_each(|k| attempt(|| hc2d_intervals(&a, k)));
        show(C::is_closed(&a), flag);
        show(C::is_periodic(&a), flag);
        show(C::period(&a), real);
        hc2d_derivatives(&a, u);
        show(C::resolution(&a, 1.0e-3), real);
        int(C::get_type(&a)? as i32);
        hc2d_conics(&a);
        show(C::nb_samples(&a, u0, u1), int);
        Ok(())
    });
    end();
}

fn arcf_step(f: &mut IntPatchArcFunction, x: f64) {
    attempt(|| {
        let v = f.value_at(x)?;
        flag(true);
        real(v);
        pnt(f.last_computed_point());
        Ok(())
    });
    attempt(|| {
        let d = f.derivative_at(x)?;
        flag(true);
        real(d);
        pnt(f.last_computed_point());
        Ok(())
    });
    attempt(|| {
        let (v, d) = f.values_at(x, 0.0)?;
        flag(true);
        real(v);
        real(d);
        Ok(())
    });
    int(f.get_state_number());
}

/// `ga_ip_arcf_case`.
pub fn arcf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = surface(&mut r);
        let q = quadric(&mut r);
        let a = arc(&mut r)?;
        let mut f = IntPatchArcFunction::new();
        f.set_quadric(&q);
        f.set_surface(&s);
        f.set_arc(&a);
        int(f.quadric().type_quadric() as i32);
        flag(f.arc() == a);
        flag(f.surface() == s);
        show(f.nb_samples(), int);
        let n = r.integer();
        for _ in 0..n {
            let x = r.real();
            arcf_step(&mut f, x);
        }
        (0..=n + 1).for_each(|i| show(f.valpoint(i), pnt));
        f.set_arc(&a);
        attempt(|| f.valpoint(1).map(|_| ()));
        Ok(())
    });
    end();
}

fn csf_values(f: &mut IntPatchCSFunction, x: &Vector<'_>) -> Result<(), Failure> {
    let mut fx = Vector::with_bounds(1, 3)?;
    let mut d = Matrix::with_bounds(1, 3, 1, 3).expect("probe matrix");
    attempt(|| {
        flag(f.value(x, &mut fx)?);
        (1..=3).for_each(|k| real(fx[k]));
        pnt(f.point());
        real(f.root());
        Ok(())
    });
    let matrix = |d: &Matrix<'_>| (1..=3).for_each(|k| (1..=3).for_each(|j| real(d[(k, j)])));
    attempt(|| {
        flag(f.derivatives(x, &mut d)?);
        matrix(&d);
        Ok(())
    });
    attempt(|| {
        flag(f.values(x, &mut fx, &mut d)?);
        (1..=3).for_each(|k| real(fx[k]));
        matrix(&d);
        Ok(())
    });
    Ok(())
}

fn csf_intersection(f: &IntPatchCSFunction, [u, v, w, tol, margin]: [f64; 5]) {
    attempt(|| {
        let mut e = IntPatchCurvIntSurf::new([u, v, w], f, tol, margin)?;
        flag(e.is_done());
        show(e.is_empty(), flag);
        show(e.point(), pnt);
        show(e.parameter_on_surface(), |(p, q)| {
            real(p);
            real(q);
        });
        show(e.parameter_on_curve(), real);
        real(e.function().root());
        Ok(())
    });
    let e2 = IntPatchCurvIntSurf::from_function(f, tol);
    flag(e2.is_done());
    show(e2.is_empty(), flag);
    show(e2.point(), pnt);
}

/// `ga_ip_csf_case`.
pub fn csf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s1 = surface(&mut r);
        let a = arc(&mut r)?;
        let s2 = surface(&mut r);
        let mut f = IntPatchCSFunction::new(&s1, &a, &s2);
        int(f.nb_variables());
        int(f.nb_equations());
        flag(*f.auxillar_surface() == s1);
        flag(*f.auxillar_curve() == a);
        pnt(f.point());
        real(f.root());
        for _ in 0..r.integer() {
            let mut x = Vector::with_bounds(1, 3)?;
            (x[1], x[2], x[3]) = (r.real(), r.real(), r.real());
            csf_values(&mut f, &x)?;
        }
        let params = [r.real(), r.real(), r.real(), r.real(), r.real()];
        csf_intersection(&f, params);
        Ok(())
    });
    end();
}

/// `ga_ip_curvature_case`.
pub fn curvature_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (s1, s2) = (surface(&mut r), surface(&mut r));
        for _ in 0..r.integer() {
            let x = r.pnt();
            let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
            let mut p = IntSurfPntOn2S::new();
            p.set_value_all(x, a, b, c, d);
            show(curvature_radius_of_inters_line(&s1, &s2, &p), real);
        }
        Ok(())
    });
    end();
}
