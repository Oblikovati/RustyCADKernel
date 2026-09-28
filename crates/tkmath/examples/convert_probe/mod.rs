//! Convert counterparts of tools/oracle/tkmath_convert_probe.cxx.
//!
//! Arrays arrive as "lower count values..."; grids as "lowerRow rows lowerCol cols
//! values..."; arrays print as "lower length values...", grids as "rows cols values...".
use crate::bspl_probe::{In, fail, o, oa, oi, oia, op, opa, report};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::convert::{
    ConicToBSplineCurve, ParameterisationType, build_polynomial_cos_and_sin,
};
use rustycad_tkmath::gp::{
    Axis3, Axis22d, Circle2, Direction2, Direction3, Ellipse2, Hyperbola2, Parabola2, Point2,
    Point3, Xy, Xyz,
};
use rustycad_tkmath::plib::PolePoint;

mod polynomial;
mod surfaces;
pub use polynomial::{bezier_case, comppoly_case, grid_case};
pub use surfaces::surface_case;

/// Prints the failure type of a fallible block, as the native `CV_TRY`.
pub fn attempt(block: impl FnOnce() -> Result<(), Failure>) {
    if let Err(error) = block() {
        fail(&error);
    }
}
pub fn arr(array: &Array1<'_, f64>) {
    oi(array.lower());
    oi(array.len() as i32);
    oa(array);
}
pub fn arr_i(array: &Array1<'_, i32>) {
    oi(array.lower());
    oi(array.len() as i32);
    oia(array);
}
pub fn pts<P: PolePoint>(array: &Array1<'_, P>) {
    oi(array.lower());
    oi(array.len() as i32);
    opa(array);
}
pub fn grid_pts(grid: &Array2<'_, Point3>) {
    oi(grid.nb_rows() as i32);
    oi(grid.nb_columns() as i32);
    grid.as_slice().iter().for_each(op);
}
pub fn grid_reals(grid: &Array2<'_, f64>) {
    oi(grid.nb_rows() as i32);
    oi(grid.nb_columns() as i32);
    grid.as_slice().iter().for_each(|value| o(*value));
}
pub fn rgrid(f: &mut In<'_>) -> Array2<'static, f64> {
    let (rl, nr, cl, nc) = (f.i(), f.i(), f.i(), f.i());
    let mut grid: Array2<'static, f64> =
        Array2::with_bounds(rl, rl + nr - 1, cl, cl + nc - 1).unwrap();
    for value in grid.as_mut_slice() {
        *value = f.r();
    }
    grid
}
pub fn igrid(f: &mut In<'_>) -> Array2<'static, i32> {
    let (rl, nr, cl, nc) = (f.i(), f.i(), f.i(), f.i());
    let mut grid: Array2<'static, i32> =
        Array2::with_bounds(rl, rl + nr - 1, cl, cl + nc - 1).unwrap();
    for value in grid.as_mut_slice() {
        *value = f.i();
    }
    grid
}
pub fn frame3(v: &[f64; 10]) -> Result<Axis3, Failure> {
    let normal = Direction3::from_xyz(Xyz::new(v[3], v[4], v[5]))?;
    let x = Direction3::from_xyz(Xyz::new(v[6], v[7], v[8]))?;
    let mut frame = Axis3::new(Point3::new(v[0], v[1], v[2]), normal, x)?;
    if v[9] != 0.0 {
        frame.y_reverse();
    }
    Ok(frame)
}
fn frame2(v: &[f64; 5]) -> Result<Axis22d, Failure> {
    let x = Direction2::from_xy(Xy::new(v[2], v[3]))?;
    Axis22d::new(Point2::new(v[0], v[1]), x, v[4] != 0.0)
}
pub fn parameterisation(code: i32) -> ParameterisationType {
    use ParameterisationType::*;
    [
        TgtThetaOver2,
        TgtThetaOver2_1,
        TgtThetaOver2_2,
        TgtThetaOver2_3,
        TgtThetaOver2_4,
        QuasiAngular,
        RationalC1,
        Polynomial,
    ][code as usize]
}

fn conic_out(c: &ConicToBSplineCurve) {
    oi(c.degree());
    oi(i32::from(c.is_periodic()));
    oi(c.nb_poles());
    oi(c.nb_knots());
    pts(c.poles());
    arr(c.weights());
    arr(c.knots());
    arr_i(c.multiplicities());
    for k in [0, 1, c.nb_poles(), c.nb_poles() + 1] {
        report(c.pole(k), |p| op(&p));
        report(c.weight(k), o);
    }
    for k in [0, 1, c.nb_knots(), c.nb_knots() + 1] {
        report(c.knot(k), o);
        report(c.multiplicity(k), oi);
    }
}

// conic: kind(0 circle,1 ellipse,2 hyperbola,3 parabola) arc frame2 a b u1 u2 param
pub fn conic_case(values: &[f64]) {
    let mut f = In::new(values);
    let (kind, arc) = (f.i(), f.b());
    let frame: [f64; 5] = std::array::from_fn(|_| f.r());
    let (a, b, u1, u2) = (f.r(), f.r(), f.r(), f.r());
    let p = parameterisation(f.i());
    attempt(|| {
        let ax = frame2(&frame)?;
        let conversion = match kind {
            0 if arc => ConicToBSplineCurve::from_circle_arc(&Circle2::new(ax, a)?, u1, u2, p)?,
            0 => ConicToBSplineCurve::from_circle(&Circle2::new(ax, a)?, p)?,
            1 if arc => {
                ConicToBSplineCurve::from_ellipse_arc(&Ellipse2::new(ax, a, b)?, u1, u2, p)?
            }
            1 => ConicToBSplineCurve::from_ellipse(&Ellipse2::new(ax, a, b)?, p)?,
            2 => ConicToBSplineCurve::from_hyperbola(&Hyperbola2::new(ax, a, b)?, u1, u2)?,
            _ => ConicToBSplineCurve::from_parabola(&Parabola2::new(ax, a)?, u1, u2)?,
        };
        conic_out(&conversion);
        Ok(())
    });
    println!();
}

// cossin: periodic param u1 u2 (the public legacy BuildCosAndSin overloads)
pub fn cossin_case(values: &[f64]) {
    let mut f = In::new(values);
    let periodic = f.b();
    let p = parameterisation(f.i());
    let (u1, u2) = (f.r(), f.r());
    attempt(|| {
        let cs = if periodic {
            ConicToBSplineCurve::build_cos_and_sin(p)?
        } else {
            ConicToBSplineCurve::build_cos_and_sin_trimmed(p, u1, u2)?
        };
        arr(&cs.cos_numerator);
        arr(&cs.sin_numerator);
        arr(&cs.denominator);
        oi(cs.degree);
        arr(&cs.knots);
        arr_i(&cs.mults);
        Ok(())
    });
    println!();
}

// polycs: u1 u2 (BuildPolynomialCosAndSin on 8 poles)
pub fn polycs_case(values: &[f64]) {
    let mut f = In::new(values);
    let (u1, u2) = (f.r(), f.r());
    let filled = || {
        let mut array: Array1<'static, f64> = Array1::with_bounds(1, 8).unwrap();
        array.init(91.0);
        array
    };
    let (mut cs, mut sn, mut den) = (filled(), filled(), filled());
    attempt(|| {
        build_polynomial_cos_and_sin(u1, u2, 8, &mut cs, &mut sn, &mut den)?;
        arr(&cs);
        arr(&sn);
        arr(&den);
        Ok(())
    });
    println!();
}
