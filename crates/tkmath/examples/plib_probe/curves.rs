//! CoefficientsPoles and Trimming counterparts, flat, point and grid overloads.
use super::{Fields, array, bounded, fail, filled, points, pole_array};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::plib::{self, PolePoint};

struct Curve {
    dim: i32,
    rational: bool,
    lower: i32,
    u1: f64,
    u2: f64,
    coefficients: Vec<f64>,
    weights: Vec<f64>,
}

pub fn curve_case(v: &[f64], trim: bool) {
    let mut f = Fields::new(v);
    let (dim, rational, n, lower, wlen) = (f.int(), f.int() != 0, f.int(), f.int(), f.int());
    let (u1, u2) = (f.real(), f.real());
    let curve = Curve {
        dim,
        rational,
        lower,
        u1,
        u2,
        coefficients: f.reals(n * dim),
        weights: f.reals(wlen),
    };
    if let Err(failure) = flat(&curve, trim) {
        fail(&failure);
    }
    print!("| ");
    if wlen == n || !rational {
        let result: Result<(), Failure> = match dim {
            3 => point_overload::<Point3>(&curve, trim),
            2 => point_overload::<Point2>(&curve, trim),
            1 => scalar_overload(&curve, trim),
            _ => Ok(()),
        };
        if let Err(failure) = result {
            fail(&failure);
        }
    }
    println!();
}

fn values(lower: i32, source: &[f64]) -> Array1<'static, f64> {
    let mut result: Array1<'static, f64> = bounded(lower, lower + source.len() as i32 - 1);
    result.as_mut_slice().copy_from_slice(source);
    result
}

fn flat(curve: &Curve, trim: bool) -> Result<(), Failure> {
    let mut coefs: Array1<'static, f64> = values(curve.lower, &curve.coefficients);
    let mut wc: Array1<'static, f64> = values(curve.lower, &curve.weights);
    if trim {
        plib::trimming_flat(
            curve.u1,
            curve.u2,
            curve.dim,
            &mut coefs,
            curve.rational.then_some(&mut wc),
        )?;
        print_pair(&coefs, curve.rational.then_some(&wc));
        return Ok(());
    }
    let upper: i32 = curve.lower + curve.coefficients.len() as i32 - 1;
    let mut poles: Array1<'static, f64> = filled(curve.lower, upper, 91.0);
    let mut wp: Array1<'static, f64> = filled(
        curve.lower,
        curve.lower + curve.weights.len() as i32 - 1,
        91.0,
    );
    let weights: Option<&mut Array1<'static, f64>> = curve.rational.then_some(&mut wp);
    plib::coefficients_poles_flat(
        curve.dim,
        &coefs,
        curve.rational.then_some(&wc),
        &mut poles,
        weights,
    )?;
    print_pair(&poles, curve.rational.then_some(&wp));
    Ok(())
}

fn print_pair(first: &Array1<'_, f64>, second: Option<&Array1<'_, f64>>) {
    array(first);
    if let Some(weights) = second {
        array(weights);
    }
}

fn point_overload<P: PolePoint>(curve: &Curve, trim: bool) -> Result<(), Failure> {
    let mut coefs: Array1<'static, P> = pole_array(curve.lower, &curve.coefficients);
    let mut wc: Array1<'static, f64> = weights_or_zero(curve);
    let mut wp: Array1<'static, f64> = filled(coefs.lower(), coefs.upper(), 91.0);
    if trim {
        plib::trimming(
            curve.u1,
            curve.u2,
            &mut coefs,
            curve.rational.then_some(&mut wc),
        )?;
        points(&coefs);
        print_weights(curve.rational, &wc);
        return Ok(());
    }
    let mut poles: Array1<'static, P> = bounded(coefs.lower(), coefs.upper());
    let weights: Option<&mut Array1<'static, f64>> = curve.rational.then_some(&mut wp);
    plib::coefficients_poles(&coefs, curve.rational.then_some(&wc), &mut poles, weights)?;
    points(&poles);
    print_weights(curve.rational, &wp);
    Ok(())
}

fn weights_or_zero(curve: &Curve) -> Array1<'static, f64> {
    let count: usize = curve.coefficients.len() / curve.dim as usize;
    let source: Vec<f64> = if curve.rational {
        curve.weights.clone()
    } else {
        vec![0.0; count]
    };
    values(curve.lower, &source)
}

fn print_weights(rational: bool, weights: &Array1<'_, f64>) {
    if rational {
        array(weights);
    }
}

fn scalar_overload(curve: &Curve, trim: bool) -> Result<(), Failure> {
    let mut a: Array1<'static, f64> = values(curve.lower, &curve.coefficients);
    let mut x: Array1<'static, f64> = weights_or_zero(curve);
    if trim {
        plib::trimming_scalar(curve.u1, curve.u2, &mut a, curve.rational.then_some(&mut x))?;
        array(&a);
        print_weights(curve.rational, &x);
        return Ok(());
    }
    let mut b: Array1<'static, f64> = filled(a.lower(), a.upper(), 91.0);
    let mut y: Array1<'static, f64> = filled(a.lower(), a.upper(), 91.0);
    let weights: Option<&mut Array1<'static, f64>> = curve.rational.then_some(&mut y);
    plib::coefficients_poles_scalar(&a, curve.rational.then_some(&x), &mut b, weights)?;
    array(&b);
    print_weights(curve.rational, &y);
    Ok(())
}

fn grid_out(poles: &Array2<'_, Point3>, weights: &Array2<'_, f64>, rational: bool) {
    for pole in poles.as_slice() {
        super::out(pole.coordinates());
    }
    if rational {
        super::out(weights.as_slice());
    }
}

pub fn grid_case(v: &[f64]) {
    let mut f = Fields::new(v);
    let (rows, cols, rational, lr, lc) = (f.int(), f.int(), f.int() != 0, f.int(), f.int());
    let (u1, u2) = (f.real(), f.real());
    let mut c: Array2<'static, Point3> =
        Array2::with_bounds(lr, lr + rows - 1, lc, lc + cols - 1).unwrap();
    for pole in c.as_mut_slice() {
        *pole = Point3::new(f.real(), f.real(), f.real());
    }
    let mut wc: Array2<'static, f64> =
        Array2::with_bounds(lr, lr + rows - 1, lc, lc + cols - 1).unwrap();
    for weight in wc.as_mut_slice() {
        *weight = f.real();
    }
    let mut p: Array2<'static, Point3> =
        Array2::with_bounds(lr, lr + rows - 1, lc, lc + cols - 1).unwrap();
    let mut wp: Array2<'static, f64> =
        Array2::with_bounds(lr, lr + rows - 1, lc, lc + cols - 1).unwrap();
    wp.init(91.0);
    match plib::coefficients_poles_grid(
        &c,
        rational.then_some(&wc),
        &mut p,
        rational.then_some(&mut wp),
    ) {
        Ok(()) => grid_out(&p, &wp, rational),
        Err(failure) => fail(&failure),
    }
    for trim in [plib::u_trimming, plib::v_trimming] {
        let (mut a, mut aw) = (c.clone(), wc.clone());
        match trim(u1, u2, &mut a, rational.then_some(&mut aw)) {
            Ok(()) => grid_out(&a, &aw, rational),
            Err(failure) => fail(&failure),
        }
    }
    println!();
}
