//! Bezier and B-spline edition and queries, and evaluation representations; the
//! counterpart of tools/oracle/tkg3d_probe_bspl.cxx.
use super::common::{
    Reader, attempt, built_curve, curve_summary, dump, end, flag, int, pnt, real, show, trsf, vec,
};
use super::shapes;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomBoundedCurve, GeomCurve, GeomGeometry, GeomOffsetCurve,
};
use rustycad_tkg3d::geom_eval::{
    Domain1d, Map1d, RepCurveDesc, RepCurveDescDerivBounded, RepCurveDescFull, RepCurveDescMapped,
};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

fn print_into<T: Copy + Default>(
    array: &mut Array1<'_, T>,
    fill: impl FnOnce(&mut Array1<'_, T>) -> Result<(), Failure>,
    print: impl Fn(T),
) {
    attempt(|| {
        fill(array)?;
        array.iter().copied().for_each(print);
        Ok(())
    });
}

// The deprecated copying getters, with matching and mismatching destinations.
fn bezier_arrays(b: &GeomBezierCurve) {
    let n = b.nb_poles();
    for lower in [1, 0] {
        let mut p = Array1::<Point3>::with_bounds(lower, lower + n - 1).unwrap();
        print_into(&mut p, |a| b.poles_into(a), pnt);
        let mut w = Array1::<f64>::with_bounds(lower, lower + n - 1).unwrap();
        print_into(&mut w, |a| b.weights_into(a), real);
    }
    let mut p = Array1::<Point3>::with_bounds(1, n + 1).unwrap();
    attempt(|| b.poles_into(&mut p));
    let mut w = Array1::<f64>::with_bounds(1, n + 1).unwrap();
    attempt(|| b.weights_into(&mut w));
}

fn bezier_state(b: &GeomBezierCurve, u: &[f64]) {
    int(b.nb_poles());
    int(b.degree());
    flag(b.is_rational());
    show(b.is_closed(), flag);
    for i in 0..=b.nb_poles() + 1 {
        show(b.pole(i), pnt);
        show(b.weight(i), real);
    }
    b.weights_array(|weights| weights.iter().copied().for_each(real));
    flag(b.weights().is_some());
    b.poles().iter().copied().for_each(pnt);
    b.knots().iter().copied().for_each(real);
    b.multiplicities().unwrap().iter().copied().for_each(int);
    b.knot_sequence().unwrap().iter().copied().for_each(real);
    pnt(b.start_point().unwrap());
    pnt(b.end_point().unwrap());
    real(b.resolution(1e-7).unwrap());
    real(b.resolution(2.5).unwrap());
    bezier_arrays(b);
    curve_summary(b, u);
    dump(b, -1);
}

fn bezier_action(b: &GeomBezierCurve, action: i32, r: &mut Reader) -> Result<(), Failure> {
    let i = r.integer();
    let p = r.pnt();
    let (w, a, z) = (r.real(), r.real(), r.real());
    let t = trsf(r);
    match action {
        1 => b.increase(i),
        2 => b.insert_pole_after_weighted(i, p, w),
        3 => b.insert_pole_before_weighted(i, p, w),
        4 => b.remove_pole(i),
        5 => b.reverse(),
        6 => b.segment(a, z),
        7 => b.set_pole(i, p),
        8 => b.set_pole_weighted(i, p, w),
        9 => b.set_weight(i, w),
        10 => b.transform(t),
        11 => b.insert_pole_after(i, p),
        12 => b.insert_pole_before(i, p),
        _ => Ok(()),
    }
}

pub fn bezier_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built_curve(&mut r) else { return };
    let u = r.reals();
    let b: Handle<GeomBezierCurve> = c.cast();
    let count = r.integer();
    bezier_state(&b, &u);
    for _ in 0..count {
        let action = r.integer();
        attempt(|| bezier_action(&b, action, &mut r));
        bezier_state(&b, &u);
    }
    bezier_state(&GeomBezierCurve::copy_of(&b).unwrap(), &u);
    int(GeomBezierCurve::max_degree());
    end();
}

// The deprecated copying getters, with matching and mismatching destinations.
fn bspline_arrays(b: &GeomBSplineCurve) {
    let (nk, np, nf) = (b.nb_knots(), b.nb_poles(), b.knot_sequence().len() as i32);
    for lower in [1, 2, 0] {
        let mut k = Array1::<f64>::with_bounds(lower, nk).unwrap();
        print_into(&mut k, |a| b.knots_into(a), real);
        let mut f = Array1::<f64>::with_bounds(lower, nf).unwrap();
        print_into(&mut f, |a| b.knot_sequence_into(a), real);
    }
    for extra in [0, 1] {
        let mut m = Array1::<i32>::with_bounds(1, nk + extra).unwrap();
        print_into(&mut m, |a| b.multiplicities_into(a), int);
        let mut p = Array1::<Point3>::with_bounds(1, np + extra).unwrap();
        print_into(&mut p, |a| b.poles_into(a), pnt);
        let mut w = Array1::<f64>::with_bounds(0, np - 1 + extra).unwrap();
        print_into(&mut w, |a| b.weights_into(a), real);
    }
}

fn bspline_header(b: &GeomBSplineCurve) {
    int(b.degree());
    flag(b.is_periodic());
    flag(b.is_rational());
    int(b.knot_distribution() as i32);
    int(b.continuity() as i32);
    int(b.nb_knots());
    int(b.nb_poles());
    show(b.first_u_knot_index(), int);
    show(b.last_u_knot_index(), int);
    for i in 0..=b.nb_knots() + 1 {
        show(b.knot(i), real);
        show(b.multiplicity(i), int);
    }
    for i in 0..=b.nb_poles() + 1 {
        show(b.pole(i), pnt);
        show(b.weight(i), real);
    }
    b.knot_sequence().iter().copied().for_each(real);
    b.weights_array(|weights| weights.iter().copied().for_each(real));
    flag(b.weights().is_some());
    b.knots().iter().copied().for_each(real);
    b.multiplicities().iter().copied().for_each(int);
    b.poles().iter().copied().for_each(pnt);
    show(b.start_point(), pnt);
    show(b.end_point(), pnt);
    show(b.resolution(1e-7), real);
    let (first, last) = (b.first_parameter(), b.last_parameter());
    show(b.is_g1(first, last, 1e-7), flag);
    show(b.is_g1(-1e9, 1e9, 0.5), flag);
}

// IsEqual against itself, an exact copy and a copy with the first pole moved by 1e-9.
fn bspline_equality(b: &GeomBSplineCurve) {
    flag(b.is_equal(b, 1e-7).unwrap());
    attempt(|| {
        let c = GeomBSplineCurve::copy_of(b)?;
        flag(b.is_equal(&c, 0.0)?);
        c.set_pole(1, c.pole(1)?.translated(Vector3::new(1e-9, 0.0, 0.0)))?;
        flag(b.is_equal(&c, 1e-10)?);
        flag(b.is_equal(&c, 1e-8)?);
        Ok(())
    });
}

fn bspline_local_spans(b: &GeomBSplineCurve, v: f64) -> Result<(), Failure> {
    for k1 in b.first_u_knot_index()?..b.last_u_knot_index()? {
        let k2 = k1 + 1;
        show(b.local_value(v, k1, k2), pnt);
        show(b.local_d0(v, k1, k2), pnt);
        show(b.local_d1(v, k1, k2), |(p, d1)| {
            pnt(p);
            vec(d1);
        });
        show(b.local_d2(v, k1, k2), |(p, d1, d2)| {
            pnt(p);
            [d1, d2].into_iter().for_each(vec);
        });
        show(b.local_d3(v, k1, k2), |(p, d1, d2, d3)| {
            pnt(p);
            [d1, d2, d3].into_iter().for_each(vec);
        });
        for n in 1..=4 {
            show(b.local_dn(v, k1, k2, n), vec);
        }
    }
    Ok(())
}

fn bspline_local(b: &GeomBSplineCurve, v: f64) {
    real(b.periodic_normalization(v).unwrap());
    for rep in [false, true] {
        for tolerance in [0.0, 1e-9, 0.25] {
            show(b.locate_u(v, tolerance, rep), |(i1, i2)| {
                int(i1);
                int(i2);
            });
        }
    }
    bspline_local_spans(b, v).unwrap();
    show(b.local_value(v, 1, 1), pnt);
}

fn bspline_state(b: &GeomBSplineCurve, u: &[f64]) {
    bspline_header(b);
    bspline_equality(b);
    bspline_arrays(b);
    for &v in u {
        bspline_local(b, v);
    }
    curve_summary(b, u);
    dump(b, -1);
}

struct Edit {
    i: i32,
    j: i32,
    k: i32,
    p: Point3,
    w: f64,
    a: f64,
    z: f64,
    flag: bool,
    t: Transform3,
}

fn read_edit(r: &mut Reader) -> Edit {
    let (i, j, k) = (r.integer(), r.integer(), r.integer());
    let p = r.pnt();
    let (w, a, z) = (r.real(), r.real(), r.real());
    let flag = r.flag();
    let t = trsf(r);
    Edit {
        i,
        j,
        k,
        p,
        w,
        a,
        z,
        flag,
        t,
    }
}

fn knot_pair(e: &Edit) -> (Array1<'static, f64>, Array1<'static, i32>) {
    let mut ks = Array1::<f64>::with_bounds(1, 2).unwrap();
    ks[1] = e.a;
    ks[2] = e.z;
    let mut ms = Array1::<i32>::with_bounds(1, 2).unwrap();
    ms[1] = e.i;
    ms[2] = e.j;
    (ks, ms)
}

fn bspline_knot_action(b: &GeomBSplineCurve, action: i32, e: &Edit) -> Result<(), Failure> {
    match action {
        1 => b.increase_degree(e.i),
        2 => b.increase_multiplicity(e.i, e.j),
        3 => b.increase_multiplicity_range(e.i, e.j, e.k),
        4 => b.increment_multiplicity(e.i, e.j, e.k),
        5 => b.insert_knot(e.a, e.i, e.w, e.flag),
        6 => {
            let (ks, ms) = knot_pair(e);
            b.insert_knots(&ks, &ms, e.w, e.flag)
        }
        7 => b.remove_knot(e.i, e.j, e.w).map(flag),
        8 => b.set_origin_parameter(e.a, e.w),
        12 => b.segment(e.a, e.z, e.w),
        13 => b.set_knot(e.i, e.a),
        14 => {
            let mut ks = b.knots();
            ks.iter_mut().for_each(|x| *x += e.a);
            b.set_knots(&ks)
        }
        15 => b.set_knot_with_multiplicity(e.i, e.a, e.j),
        _ => bspline_pole_action(b, action, e),
    }
}

fn bspline_pole_action(b: &GeomBSplineCurve, action: i32, e: &Edit) -> Result<(), Failure> {
    match action {
        11 => b.reverse(),
        16 => b.set_periodic(),
        17 => b.set_origin(e.i),
        18 => b.set_not_periodic(),
        19 => b.set_pole(e.i, e.p),
        20 => b.set_pole_weighted(e.i, e.p, e.w),
        21 => b.set_weight(e.i, e.w),
        22 => {
            let (first, last) = b.move_point(e.a, e.p, e.i, e.j)?;
            int(first);
            int(last);
            Ok(())
        }
        23 => {
            let tangent = Vector3::new(e.z, e.w, e.a);
            b.move_point_and_tangent(e.a, e.p, tangent, 1e-9, e.i, e.j)
                .map(int)
        }
        24 => b.transform(e.t),
        _ => Ok(()),
    }
}

// The rational constructor with CheckRational false and true on the curve's own data.
fn bspline_rebuilt(b: &GeomBSplineCurve, u: &[f64]) {
    for check in [false, true] {
        attempt(|| {
            let weights: Array1<'static, f64> = b.weights_array(Clone::clone);
            let c = GeomBSplineCurve::with_weights_checked(
                &b.poles(),
                &weights,
                &b.knots(),
                &b.multiplicities(),
                b.degree(),
                b.is_periodic(),
                check,
            )?;
            flag(c.is_rational());
            curve_summary(&c, u);
            dump(&c, -1);
            Ok(())
        });
    }
}

pub fn bspline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built_curve(&mut r) else { return };
    let u = r.reals();
    let b: Handle<GeomBSplineCurve> = c.cast();
    let count = r.integer();
    bspline_state(&b, &u);
    for _ in 0..count {
        let action = r.integer();
        let edit = read_edit(&mut r);
        attempt(|| bspline_knot_action(&b, action, &edit));
        bspline_state(&b, &u);
    }
    bspline_state(&GeomBSplineCurve::copy_of(&b).unwrap(), &u);
    bspline_rebuilt(&b, &u);
    int(GeomBSplineCurve::max_degree());
    end();
}

pub fn desc(r: &mut Reader, rep: Handle<dyn GeomCurve>) -> Handle<dyn RepCurveDesc> {
    let (kind, order) = (r.integer(), r.integer() as usize);
    let domain = r.flag();
    let (first, last, scale, offset) = (r.real(), r.real(), r.real(), r.real());
    match kind {
        0 => Handle::new(RepCurveDescFull::new(rep)).cast(),
        1 => Handle::new(RepCurveDescDerivBounded::new(rep, order)).cast(),
        _ => {
            let domain = domain.then_some(Domain1d { first, last });
            let map = Map1d { scale, offset };
            Handle::new(RepCurveDescMapped::new(rep, order, domain, map)).cast()
        }
    }
}

/// The Geom classes of this probe that accept an evaluation representation.
trait EvalRepresented: GeomCurve {
    fn set_rep(&self, desc: Handle<dyn RepCurveDesc>) -> Result<(), Failure>;
    fn has_rep(&self) -> bool;
    fn clear_rep(&self);
}
macro_rules! eval_represented {
    ($class:ty) => {
        impl EvalRepresented for $class {
            fn set_rep(&self, desc: Handle<dyn RepCurveDesc>) -> Result<(), Failure> {
                self.set_eval_representation(desc)
            }
            fn has_rep(&self) -> bool {
                self.has_eval_representation()
            }
            fn clear_rep(&self) {
                self.clear_eval_representation()
            }
        }
    };
}
eval_represented!(GeomBezierCurve);
eval_represented!(GeomBSplineCurve);
eval_represented!(GeomOffsetCurve);

fn evalrep_on<C: EvalRepresented + RuntimeTyped>(
    c: &Handle<C>,
    d: &Handle<dyn RepCurveDesc>,
    u: &[f64],
    t: Transform3,
) {
    attempt(|| c.set_rep(d.clone()));
    flag(c.has_rep());
    curve_summary(&**c, u);
    let copy: Handle<C> = c.copy().unwrap().cast();
    flag(copy.has_rep());
    curve_summary(&*copy, u);
    attempt(|| c.set_rep(Handle::new(RepCurveDescFull::new(Handle::default())).cast()));
    let own = Handle::new(RepCurveDescFull::new(c.cast()));
    attempt(|| c.set_rep(own.cast()));
    c.reverse().unwrap();
    flag(c.has_rep());
    curve_summary(&**c, u);
    c.transform(t).unwrap();
    flag(c.has_rep());
    curve_summary(&**c, u);
    c.clear_rep();
    flag(c.has_rep());
}

pub fn evalrep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built_curve(&mut r) else { return };
    let u = r.reals();
    let rep = shapes::curve(&mut r).unwrap();
    let d = desc(&mut r, rep);
    let t = trsf(&mut r);
    if c.cast::<GeomBezierCurve>().is_some() {
        evalrep_on(&c.cast::<GeomBezierCurve>(), &d, &u, t);
    }
    if c.cast::<GeomBSplineCurve>().is_some() {
        evalrep_on(&c.cast::<GeomBSplineCurve>(), &d, &u, t);
    }
    if c.cast::<GeomOffsetCurve>().is_some() {
        evalrep_on(&c.cast::<GeomOffsetCurve>(), &d, &u, t);
    }
    end();
}

pub fn evalrep_edit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built_curve(&mut r) else { return };
    let u = r.reals();
    let rep = shapes::curve(&mut r).unwrap();
    let d = desc(&mut r, rep);
    let b: Handle<GeomBSplineCurve> = c.cast();
    let count = r.integer();
    for _ in 0..count {
        let action = r.integer();
        attempt(|| b.set_eval_representation(d.clone()));
        let edit = read_edit(&mut r);
        attempt(|| bspline_knot_action(&b, action, &edit));
        flag(b.has_eval_representation());
        curve_summary(&*b, &u);
    }
    end();
}
