//! Law probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_law.cxx.
use super::common::{Reader, end, flag, int, real, show};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::law::*;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2, Point2};

const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

fn one_based<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut array: Array1<'static, T> =
        Array1::with_bounds(1, values.len() as i32).expect("probe array");
    array.as_mut_slice().copy_from_slice(values);
    array
}

fn reals(r: &mut Reader) -> Array1<'static, f64> {
    one_based(&r.reals())
}

fn ints(r: &mut Reader) -> Array1<'static, i32> {
    let values: Vec<i32> = r.reals().iter().map(|&v| v as i32).collect();
    one_based(&values)
}

/// `degree periodic rational poles [weights] knots mults`: the ga_law_bspline law.
pub fn bspline(r: &mut Reader) -> Result<Handle<LawBSpline>, Failure> {
    let (deg, per, rat) = (r.integer(), r.flag(), r.flag());
    let poles = reals(r);
    let weights = if rat { Some(reals(r)) } else { None };
    let (knots, mults) = (reals(r), ints(r));
    let law = match weights {
        Some(w) => LawBSpline::with_weights(&poles, &w, &knots, &mults, deg, per)?,
        None => LawBSpline::new(&poles, &knots, &mults, deg, per)?,
    };
    Ok(Handle::new(law))
}

/// The ga_law_state printer.
fn state(b: &LawBSpline) {
    int(b.degree());
    flag(b.is_periodic());
    flag(b.is_rational());
    int(b.nb_poles());
    let (mut p, mut w) = (
        one_based(&vec![0.0; b.nb_poles() as usize]),
        one_based(&vec![0.0; b.nb_poles() as usize]),
    );
    b.poles(&mut p).expect("probe poles");
    b.weights(&mut w).expect("probe weights");
    for i in 1..=p.len() as i32 {
        real(p[i]);
        real(w[i]);
    }
    int(b.nb_knots());
    let mut k = one_based(&vec![0.0; b.nb_knots() as usize]);
    let mut m = one_based(&vec![0; b.nb_knots() as usize]);
    b.knots(&mut k).expect("probe knots");
    b.multiplicities(&mut m).expect("probe mults");
    for i in 1..=k.len() as i32 {
        real(k[i]);
        int(m[i]);
    }
    int(b.continuity() as i32);
    int(b.knot_distribution() as i32);
    real(b.first_parameter());
    real(b.last_parameter());
}

fn reals_out(values: &[f64]) {
    values.iter().for_each(|&v| real(v));
}

/// The ga_law_eval printer.
fn eval(b: &LawBSpline, u: f64) {
    show(b.value(u), real);
    show(b.d1(u), |(p, d)| reals_out(&[p, d]));
    show(b.d2(u), |(p, d1, d2)| reals_out(&[p, d1, d2]));
    show(b.d3(u), |(p, d1, d2, d3)| reals_out(&[p, d1, d2, d3]));
    show(b.dn(u, 2), real);
    show(b.dn(u, 5), real);
    for repetition in [false, true] {
        show(b.locate_u(u, 1e-9, repetition), |(i1, i2)| {
            int(i1);
            int(i2);
        });
    }
    show(b.local_value(u, 1, 2), real);
    let n = b.nb_knots();
    show(b.local_d3(u, n - 1, n), |(p, d1, d2, d3)| {
        reals_out(&[p, d1, d2, d3])
    });
    show(b.local_dn(u, n - 1, n, 3), real);
}

/// `law params...`: the state, the queries and the evaluators at each parameter.
pub fn bspline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let b = match bspline(&mut r) {
        Ok(b) => b,
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    state(&b);
    show(b.is_closed(), flag);
    show(b.start_point(), real);
    show(b.end_point(), real);
    for n in -1..=5 {
        show(b.is_cn(n), flag);
    }
    show(b.resolution(1e-3), real);
    real(b.reversed_parameter(0.3));
    real(b.periodic_normalization(7.25));
    for u in r.reals() {
        eval(&b, u);
    }
    end();
}

fn apply_edit(b: &mut Handle<LawBSpline>, op: i32, a: f64, c: f64, d: f64) -> Result<(), Failure> {
    let law: &LawBSpline = b.get().expect("probe law");
    match op {
        0 => law.increase_degree(a as i32),
        1 => law.insert_knot(a, c as i32, d, true),
        2 => law.remove_knot(a as i32, c as i32, d).map(flag),
        3 => law.reverse(),
        4 => law.segment(a, c),
        5 => law.set_periodic(),
        6 => law.set_not_periodic(),
        7 => law.set_origin(a as i32),
        8 => law.set_weight(a as i32, c),
        9 => law.set_knot(a as i32, c),
        10 => law.move_point_and_tangent(a, c, d, 1e-7, 1, 1).map(int),
        11 => law.increase_multiplicity(a as i32, c as i32),
        12 => law.increment_multiplicity(a as i32, c as i32, d as i32),
        13 => law.set_pole_with_weight(a as i32, c, d),
        14 => law.set_knot_with_mult(a as i32, c, d as i32),
        15 => law.increase_multiplicity_range(a as i32, c as i32, d as i32),
        _ => {
            *b = law.copy()?;
            Ok(())
        }
    }
}

/// `law op args... u`: one edit, then the state and the evaluators at u.
pub fn edit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut b = match bspline(&mut r) {
        Ok(b) => b,
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    let (op, a, c, d, u) = (r.integer(), r.real(), r.real(), r.real(), r.real());
    if let Err(error) = apply_edit(&mut b, op, a, c, d) {
        print!("{} ", error.exception_type());
    }
    state(&b);
    eval(&b, u);
    end();
}

/// The ga_law_function printer.
fn function(f: &dyn LawFunction, u: f64) {
    show(f.bounds(), |(a, b)| reals_out(&[a, b]));
    show(f.continuity(), |c| int(c as i32));
    for s in SHAPES {
        let intervals = f.nb_intervals(s).and_then(|n| {
            let mut t = one_based(&vec![0.0; n as usize + 1]);
            f.intervals(&mut t, s)?;
            Ok((n, t))
        });
        show(intervals, |(n, t)| {
            int(n);
            reals_out(t.as_slice());
        });
    }
    show(f.value(u), real);
    show(f.d1(u), |(p, d)| reals_out(&[p, d]));
    show(f.d2(u), |(p, d1, d2)| reals_out(&[p, d1, d2]));
    let trimmed = f.trim(u - 0.25, u + 0.25, 1e-9).and_then(|t| {
        let law = t.get().expect("probe trimmed law");
        let (a, b) = law.bounds()?;
        Ok((a, b, law.value(u)?))
    });
    show(trimmed, |(a, b, v)| reals_out(&[a, b, v]));
}

fn interpol(r: &mut Reader, kind: i32) -> Result<Handle<dyn LawFunction>, Failure> {
    let per = r.flag();
    let v = r.reals();
    let points: Vec<Point2> = v
        .chunks(2)
        .filter(|c| c.len() == 2)
        .map(|c| Point2::new(c[0], c[1]))
        .collect();
    let pts = one_based(&points);
    let law = LawInterpol::new();
    let (x, y) = (r.real(), r.real());
    match kind {
        5 => law.set(&pts, per)?,
        6 => law.set_with_tangents(&pts, x, y, per)?,
        _ => law.set_in_relative(&pts, x, y, per)?,
    }
    Ok(Handle::new(law).cast())
}

fn composite(r: &mut Reader) -> Handle<dyn LawFunction> {
    let (a, b, tol) = (r.real(), r.real(), r.real());
    let law = LawComposite::with_range(a, b, tol);
    if r.flag() {
        law.set_periodic();
    }
    for piece in r.reals().chunks(4).filter(|c| c.len() == 4) {
        let linear = LawLinear::new();
        linear.set(piece[0], piece[1], piece[2], piece[3]);
        law.change_laws(|laws| {
            laws.append(Handle::new(linear).cast());
        });
    }
    Handle::new(law).cast()
}

fn law_function(r: &mut Reader, kind: i32) -> Result<Handle<dyn LawFunction>, Failure> {
    Ok(match kind {
        0 => {
            let l = LawConstant::new();
            let (v, a, b) = (r.real(), r.real(), r.real());
            l.set(v, a, b);
            Handle::new(l).cast()
        }
        1 => {
            let l = LawLinear::new();
            let (a, va, b, vb) = (r.real(), r.real(), r.real(), r.real());
            l.set(a, va, b, vb);
            Handle::new(l).cast()
        }
        2 => {
            let l = LawS::new();
            let (a, va, b, vb) = (r.real(), r.real(), r.real(), r.real());
            l.set(a, va, b, vb)?;
            Handle::new(l).cast()
        }
        3 => {
            let l = LawS::new();
            let (a, va, da) = (r.real(), r.real(), r.real());
            let (b, vb, db) = (r.real(), r.real(), r.real());
            l.set_with_derivatives(a, va, da, b, vb, db)?;
            Handle::new(l).cast()
        }
        4 => {
            let b = bspline(r)?;
            let (a, c) = (r.real(), r.real());
            Handle::new(LawBSpFunc::with_curve(&b, a, c)).cast()
        }
        5..=7 => interpol(r, kind)?,
        _ => composite(r),
    })
}

/// `kind args... u`: the ga_law_function_case laws.
pub fn function_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let law = law_function(&mut r, kind);
    if let Err(error) = &law {
        print!("{} ", error.exception_type());
    }
    let u = r.real();
    if let Ok(law) = law {
        function(law.get().expect("probe law"), u);
    }
    end();
}

fn linear(r: &mut Reader) -> Handle<LawLinear> {
    let l = LawLinear::new();
    let (a, va, b, vb) = (r.real(), r.real(), r.real(), r.real());
    l.set(a, va, b, vb);
    Handle::new(l)
}

fn mix(r: &mut Reader, kind: i32) -> Result<(), Failure> {
    match kind {
        0 => {
            let f = mix_bnd_law(&linear(r))?;
            let law = f.get().expect("probe law");
            state(law.curve().get().expect("probe curve"));
            real(law.value(r.real())?);
        }
        1 | 2 => {
            let (deg, k, m) = (r.integer(), reals(r), ints(r));
            let p = if kind == 1 {
                mix_bnd(deg, &k, &m, &linear(r))?
            } else {
                let (nr, idx) = (r.flag(), r.integer());
                mix_tgt(deg, &k, &m, nr, idx)?
            };
            let poles = p.get().expect("probe poles").array1().clone();
            int(poles.len() as i32);
            reals_out(poles.as_slice());
        }
        3 | 4 => {
            let (a, b, hf, hl, va, vb) =
                (r.real(), r.real(), r.flag(), r.flag(), r.real(), r.real());
            let s = if kind == 3 {
                scale(a, b, hf, hl, va, vb)?
            } else {
                scale_cub(a, b, hf, hl, va, vb)?
            };
            state(s.get().expect("probe law"));
        }
        5 => reparametrize_case(r)?,
        _ => splitting_case(r)?,
    }
    Ok(())
}

fn reparametrize_case(r: &mut Reader) -> Result<(), Failure> {
    let (rad, f, l, hdf, hdl) = (r.real(), r.real(), r.real(), r.flag(), r.flag());
    let (df, dl, rev, nb) = (r.real(), r.real(), r.flag(), r.integer());
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(Axis2::default(), rad)?).cast();
    let adaptor = GeomAdaptorCurve::with_range(&circle, 0.5, 4.0)?;
    let s = reparametrize(&adaptor, f, l, hdf, hdl, df, dl, rev, nb)?;
    state(s.get().expect("probe law"));
    Ok(())
}

fn splitting_case(r: &mut Reader) -> Result<(), Failure> {
    let b = bspline(r)?;
    let ks = LawBSplineKnotSplitting::new(&b, r.integer())?;
    int(ks.nb_splits());
    let mut sv = one_based(&vec![0; ks.nb_splits() as usize]);
    ks.splitting(&mut sv)?;
    sv.as_slice().iter().for_each(|&v| int(v));
    show(ks.split_value(ks.nb_splits() + 1), int);
    Ok(())
}

/// `kind args...`: the ga_law_mix_case class functions.
pub fn mix_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    if let Err(error) = mix(&mut r, kind) {
        print!("{} ", error.exception_type());
    }
    end();
}

/// `periodic points [params] tangent-mode ...`: Law_Interpolate, then its law or status.
pub fn interpolate_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let per = r.flag();
    let pts = Handle::new(HArray1::from_owned(reals(&mut r)));
    let with_params = r.flag();
    let par = Handle::new(HArray1::from_owned(reals(&mut r)));
    let (mode, t0, t1) = (r.integer(), r.real(), r.real());
    let (tg, fl) = (reals(&mut r), r.reals());
    let run = || -> Result<(), Failure> {
        let mut inter = if with_params {
            LawInterpolate::with_parameters(&pts, &par, per, 1e-9)?
        } else {
            LawInterpolate::new(&pts, per, 1e-9)?
        };
        if mode == 1 {
            inter.load_ends(t0, t1)?;
        }
        if mode == 2 {
            let flags: Vec<bool> = fl.iter().map(|&v| v != 0.0).collect();
            inter.load(&tg, &Handle::new(HArray1::from_owned(one_based(&flags))))?;
        }
        inter.perform()?;
        flag(inter.is_done());
        if inter.is_done() {
            state(inter.curve()?.get().expect("probe law"));
        }
        Ok(())
    };
    if let Err(error) = run() {
        print!("{} ", error.exception_type());
    }
    end();
}
