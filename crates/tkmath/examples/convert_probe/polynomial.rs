//! Composite Bezier and polynomial conversions of tools/oracle/tkmath_convert_probe.cxx.
use super::{arr, arr_i, attempt, grid_pts, grid_reals, igrid, pts, rgrid};
use crate::bspl_probe::{In, oi, report};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::convert::{
    BezierJunctionPoint, CompBezierCurvesToBSplineCurveBase, CompPolynomialToPoles,
    GridPolynomialToPoles, PolynomialGrid, PolynomialSpans,
};
use rustycad_tkmath::gp::{Point2, Point3};

fn sized<P: Default>(lower: i32, count: i32) -> Array1<'static, P> {
    Array1::with_bounds(lower, lower + count - 1).unwrap()
}

fn bezier_out<P: BezierJunctionPoint>(
    conv: &CompBezierCurvesToBSplineCurveBase<P>,
) -> Result<(), Failure> {
    oi(conv.degree());
    oi(conv.nb_poles());
    oi(conv.nb_knots());
    let mut poles: Array1<'static, P> = sized(1, conv.nb_poles());
    let (mut knots, mut mults) = (sized(1, conv.nb_knots()), sized(1, conv.nb_knots()));
    conv.poles(&mut poles)?;
    conv.knots_and_mults(&mut knots, &mut mults)?;
    pts(&poles);
    arr(&knots);
    arr_i(&mults);
    let mut shifted: Array1<'static, P> = sized(0, conv.nb_poles());
    conv.poles(&mut shifted)?;
    pts(&shifted);
    let mut longer: Array1<'static, P> = sized(1, conv.nb_poles() + 1);
    attempt(|| conv.poles(&mut longer));
    Ok(())
}

fn bezier_run<P: BezierJunctionPoint>(f: &mut In<'_>, angular: f64) {
    let mut conv = CompBezierCurvesToBSplineCurveBase::<P>::new(angular);
    conv.perform().unwrap();
    oi(conv.degree());
    oi(conv.nb_poles());
    for _ in 0..f.i() {
        conv.add_curve(&f.points::<P>());
    }
    attempt(|| {
        conv.perform()?;
        bezier_out(&conv)
    });
    attempt(|| {
        conv.perform()?;
        bezier_out(&conv)
    });
}

// bezier: dim angular ncurves (lower count coords...)...
pub fn bezier_case(values: &[f64]) {
    let mut f = In::new(values);
    let (dim, angular) = (f.i(), f.r());
    if dim == 2 {
        bezier_run::<Point2>(&mut f, angular);
    } else {
        bezier_run::<Point3>(&mut f, angular);
    }
    println!();
}

fn comppoly_out(c: &CompPolynomialToPoles) {
    oi(i32::from(c.is_done()));
    oi(c.degree());
    oi(c.nb_poles());
    oi(c.nb_knots());
    report(c.poles(), grid_reals);
    report(c.knots(), arr);
    report(c.multiplicities(), arr_i);
    let (p, k, m) = (c.poles_copy(), c.knots_copy(), c.multiplicities_copy());
    oi(i32::from(p.is_some()));
    p.iter().for_each(|grid| grid_reals(grid));
    oi(i32::from(k.is_some()));
    k.iter().for_each(|array| arr(array));
    oi(i32::from(m.is_some()));
    m.iter().for_each(|array| arr_i(array));
}

// comppoly: mode 0 uniform: ncurves continuity dim maxdeg nullmask counts coeffs intervals true
//           mode 1 continuities: ncurves dim maxdeg continuity counts coeffs intervals true
//           mode 2 single: dim maxdeg degree coeffs intervals true
pub fn comppoly_case(values: &[f64]) {
    let mut f = In::new(values);
    let mode = f.i();
    attempt(|| {
        let conversion = if mode == 0 {
            let (nc, cont, dim, maxdeg, mask) = (f.i(), f.i(), f.i(), f.i(), f.i());
            let (counts, coeffs, intervals, trues) =
                (f.ints(), f.reals(), rgrid(&mut f), f.reals());
            let spans = PolynomialSpans {
                num_coeff_per_curve: &counts,
                coefficients: &coeffs,
                polynomial_intervals: &intervals,
                true_intervals: &trues,
            };
            CompPolynomialToPoles::from_uniform_continuity(
                nc,
                cont,
                dim,
                maxdeg,
                (mask == 0).then_some(spans),
            )?
        } else if mode == 1 {
            let (nc, dim, maxdeg) = (f.i(), f.i(), f.i());
            let (cont, counts, coeffs, intervals, trues) =
                (f.ints(), f.ints(), f.reals(), rgrid(&mut f), f.reals());
            let spans = PolynomialSpans {
                num_coeff_per_curve: &counts,
                coefficients: &coeffs,
                polynomial_intervals: &intervals,
                true_intervals: &trues,
            };
            CompPolynomialToPoles::from_continuities(nc, dim, maxdeg, &cont, spans)?
        } else {
            let (dim, maxdeg, degree) = (f.i(), f.i(), f.i());
            let (coeffs, intervals, trues) = (f.reals(), f.reals(), f.reals());
            CompPolynomialToPoles::from_single_span(
                dim, maxdeg, degree, &coeffs, &intervals, &trues,
            )?
        };
        comppoly_out(&conversion);
        Ok(())
    });
    println!();
}

fn grid_out(c: &GridPolynomialToPoles) {
    oi(i32::from(c.is_done()));
    for count in [
        c.nb_u_poles(),
        c.nb_v_poles(),
        c.u_degree(),
        c.v_degree(),
        c.nb_u_knots(),
        c.nb_v_knots(),
    ] {
        report(count, oi);
    }
    report(c.poles(), grid_pts);
    report(c.u_knots(), arr);
    report(c.v_knots(), arr);
    report(c.u_multiplicities(), arr_i);
    report(c.v_multiplicities(), arr_i);
}

// grid: mode 0 patch: maxu maxv counts coeffs polyu polyv
//       mode 1 grid: nbu nbv ucont vcont maxu maxv countsgrid coeffs polyu polyv trueu truev
pub fn grid_case(values: &[f64]) {
    let mut f = In::new(values);
    let mode = f.i();
    attempt(|| {
        let conversion = if mode == 0 {
            let (mu, mv) = (f.i(), f.i());
            let (counts, coeffs, pu, pv) = (f.ints(), f.reals(), f.reals(), f.reals());
            GridPolynomialToPoles::from_single_patch(mu, mv, &counts, &coeffs, &pu, &pv)?
        } else {
            let (nu, nv, cu, cv, mu, mv) = (f.i(), f.i(), f.i(), f.i(), f.i(), f.i());
            let counts = igrid(&mut f);
            let (coeffs, pu, pv, tu, tv) = (f.reals(), f.reals(), f.reals(), f.reals(), f.reals());
            GridPolynomialToPoles::from_grid(&PolynomialGrid {
                nb_u_surfaces: nu,
                nb_v_surfaces: nv,
                u_continuity: cu,
                v_continuity: cv,
                max_u_degree: mu,
                max_v_degree: mv,
                num_coeff_per_surface: &counts,
                coefficients: &coeffs,
                polynomial_u_intervals: &pu,
                polynomial_v_intervals: &pv,
                true_u_intervals: &tu,
                true_v_intervals: &tv,
            })?
        };
        grid_out(&conversion);
        Ok(())
    });
    println!();
}
