//! The `ax_compute` and `ax_bspl` requests.
use super::super::app_par_curves::curve;
use super::super::common::{self as p, Reader, attempt, end};
use super::super::fem_tool::read_vector;
use super::line::AxLine;
use super::{array, bspline, constraint, read_options};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_par_curves::LineTool;
use rustycad_tkgeombase::approx::{BSplComputeLine, ComputeLine, ComputeLineOptions};
use rustycad_tkmath::math::Vector;

/// Also printed for AppDef_Compute (`ad_compute`).
pub fn compute_results(c: &mut ComputeLine) -> Result<(), Failure> {
    p::flag(c.is_all_approximated());
    p::flag(c.is_tolerance_reached());
    p::int(c.parametrization().code());
    let n: i32 = c.nb_multi_curves();
    p::int(n);
    for i in 1..=n {
        curve(c.value(i)?)?;
        let (t3, t2) = c.error(i)?;
        p::real(t3);
        p::real(t2);
        array(c.parameters(i)?);
    }
    attempt(|| bspline(c.spline_value()?));
    attempt(|| curve(c.value(n + 1)?));
    Ok(())
}

/// The ComputeLine of a variant (also AppDef_Compute for `ad_compute`).
pub fn build_compute<L: LineTool + Clone>(
    line: &L,
    variant: i32,
    o: ComputeLineOptions,
    u: &Vector<'static>,
) -> Result<ComputeLine, Failure> {
    Ok(match variant {
        0 => ComputeLine::with_line(line, o)?,
        1 => ComputeLine::with_line_and_parameters(line, u, o)?,
        2 => ComputeLine::new(o),
        _ => ComputeLine::from_parameters(u, o)?,
    })
}

pub fn compute_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = AxLine::read(&mut r);
    let variant = r.integer();
    let o = read_options(&mut r);
    let (fc, lc) = (constraint(r.integer()), constraint(r.integer()));
    let u = read_vector(&mut r);
    let repeat = r.flag();
    attempt(|| {
        let mut c = build_compute(&line, variant, o, &u)?;
        if variant >= 2 {
            c.set_constraints(fc, lc);
            c.perform(&line)?;
        }
        compute_results(&mut c)?;
        if repeat {
            c.set_degrees(o.degree_min, o.degree_max + 1);
            c.set_tolerances(o.tolerance_3d * 10., o.tolerance_2d * 10.);
            c.perform(&line)?;
            compute_results(&mut c)?;
        }
        Ok(())
    });
    end();
}

fn bspl_results(c: &BSplComputeLine) -> Result<(), Failure> {
    p::flag(c.is_all_approximated());
    p::flag(c.is_tolerance_reached());
    let (t3, t2) = c.error();
    p::real(t3);
    p::real(t2);
    bspline(c.value())?;
    array(c.parameters()?);
    Ok(())
}

fn build_bspl(
    line: &AxLine,
    variant: i32,
    o: ComputeLineOptions,
    u: &Vector<'static>,
) -> Result<BSplComputeLine, Failure> {
    Ok(match variant {
        0 => BSplComputeLine::with_line(line, o)?,
        1 => BSplComputeLine::with_line_and_parameters(line, u, o)?,
        3 => BSplComputeLine::from_parameters(u, o)?,
        _ => BSplComputeLine::new(o),
    })
}

/// The setters of the `ax_bspl` request.
struct BsplSetters {
    constraints: [i32; 2],
    continuity: i32,
    periodic: bool,
    knot_kind: i32,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    set_params: bool,
}

fn read_int_array(r: &mut Reader<'_>) -> Array1<'static, i32> {
    let n = r.integer();
    let mut a = Array1::with_bounds(1, n).unwrap();
    (1..=n).for_each(|i| a[i] = r.integer());
    a
}

fn read_real_array(r: &mut Reader<'_>) -> Array1<'static, f64> {
    let n = r.integer();
    let mut a = Array1::with_bounds(1, n).unwrap();
    (1..=n).for_each(|i| a[i] = r.real());
    a
}

fn read_setters(r: &mut Reader<'_>) -> BsplSetters {
    let constraints = [r.integer(), r.integer()];
    let (continuity, periodic, knot_kind) = (r.integer(), r.flag(), r.integer());
    let knots = if knot_kind >= 1 {
        read_real_array(r)
    } else {
        Array1::with_bounds(1, 1).unwrap()
    };
    let mults = if knot_kind == 2 {
        read_int_array(r)
    } else {
        Array1::with_bounds(1, 1).unwrap()
    };
    BsplSetters {
        constraints,
        continuity,
        periodic,
        knot_kind,
        knots,
        mults,
        set_params: false,
    }
}

fn apply_setters(
    c: &mut BSplComputeLine,
    s: &BsplSetters,
    u: &Vector<'static>,
) -> Result<(), Failure> {
    c.set_constraints(constraint(s.constraints[0]), constraint(s.constraints[1]));
    if s.continuity != -2 {
        c.set_continuity(s.continuity);
    }
    c.set_periodic(s.periodic);
    if s.knot_kind == 1 {
        c.set_knots(&s.knots);
    } else if s.knot_kind == 2 {
        c.set_knots_and_multiplicities(&s.knots, &s.mults);
    }
    if s.set_params {
        c.set_parameters(u)?;
    }
    Ok(())
}

pub fn bspl_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let line = AxLine::read(&mut r);
    let variant = r.integer();
    let o = read_options(&mut r);
    let mut setters = read_setters(&mut r);
    let u = read_vector(&mut r);
    setters.set_params = r.flag();
    attempt(|| {
        let mut c = build_bspl(&line, variant, o, &u)?;
        if variant >= 2 {
            apply_setters(&mut c, &setters, &u)?;
            if variant == 4 {
                c.interpol(&line)?;
            } else {
                c.perform(&line)?;
            }
        }
        bspl_results(&c)
    });
    end();
}
