//! The Rust twin of the ComputeApprox, ComputeApproxOnPolarSurface and
//! ProjectedCurve operations of tools/oracle/tkgeombase_probe_projlib_curves.cxx.
use super::super::approx::{adapted_curve, adapted_curve2d, adapted_surface};
use super::super::common::{self as p, Reader, attempt, end};
use super::super::geom_convert::curve2d as print_curve2d;
use super::super::geom2d_convert::{bezier as bezier2d, bspline as bspline2d};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::app_par_curves::Constraint;
use rustycad_tkgeombase::proj_lib::{
    ComputeApprox, ComputeApproxOnPolarSurface, ProjectedCurve, make_pcurve_of_type,
};
use rustycad_tkmath::geom_abs::{CurveType, Shape};

/// `GbPlOptions`: `tol degmin degmax maxseg bndpnt maxdist`.
pub struct Options {
    tol: f64,
    degmin: i32,
    degmax: i32,
    maxseg: i32,
    bnd: Constraint,
    maxdist: f64,
}

fn constraint(code: i32) -> Constraint {
    match code {
        0 => Constraint::NoConstraint,
        1 => Constraint::PassPoint,
        2 => Constraint::TangencyPoint,
        _ => Constraint::CurvaturePoint,
    }
}

fn options(r: &mut Reader<'_>) -> Options {
    Options {
        tol: r.real(),
        degmin: r.integer(),
        degmax: r.integer(),
        maxseg: r.integer(),
        bnd: constraint(r.integer()),
        maxdist: r.real(),
    }
}

type Inputs = (Handle<dyn Adaptor3dCurve>, Handle<dyn Adaptor3dSurface>);

fn inputs(r: &mut Reader<'_>) -> Result<Inputs, Failure> {
    let c = adapted_curve(r)?;
    Ok((c, adapted_surface(r)?))
}

pub fn approx_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let variant = r.integer();
    let o = options(&mut r);
    attempt(|| {
        let (c, s) = inputs(&mut r)?;
        let a = if variant == 1 {
            ComputeApprox::with_curve(&c, &s, o.tol)?
        } else {
            let mut a = ComputeApprox::new();
            a.set_tolerance(o.tol);
            a.set_degree(o.degmin, o.degmax);
            a.set_max_segments(o.maxseg);
            a.set_bnd_pnt(o.bnd);
            a.perform(&c, &s)?;
            a
        };
        bspline2d(&a.bspline());
        bezier2d(&a.bezier());
        p::real(a.tolerance());
        Ok(())
    });
    end();
}

fn polar_results(x: &ComputeApproxOnPolarSurface) {
    p::flag(x.is_done());
    bspline2d(&x.bspline());
    p::show(x.curve2d(), |c| print_curve2d(&c));
    p::real(x.tolerance());
}

fn polar_setup(x: &mut ComputeApproxOnPolarSurface, o: &Options) {
    x.set_tolerance(o.tol);
    x.set_degree(o.degmin, o.degmax);
    x.set_max_segments(o.maxseg);
    x.set_bnd_pnt(o.bnd);
    x.set_max_dist(o.maxdist);
}

fn polar_variant(
    variant: i32,
    o: &Options,
    r: &mut Reader<'_>,
    (c, s): Inputs,
) -> Result<(), Failure> {
    match variant {
        0 => {
            let mut x = ComputeApproxOnPolarSurface::new();
            polar_setup(&mut x, o);
            x.perform(&c, &s)?;
            polar_results(&x);
        }
        1 => polar_results(&ComputeApproxOnPolarSurface::with_curve(&c, &s, o.tol)?),
        2 => {
            let mut x = ComputeApproxOnPolarSurface::new();
            polar_setup(&mut x, o);
            let init = x.build_initial_curve2d(&c, &s)?;
            p::flag(init.is_null());
            polar_results(&x);
            if !init.is_null() {
                bspline2d(&x.project_using_initial_curve2d(&c, &s, &init)?);
                polar_results(&x);
            }
        }
        3 => {
            let init: Handle<dyn Adaptor2dCurve2d> = adapted_curve2d(r)?;
            polar_results(&ComputeApproxOnPolarSurface::with_initial_curve(
                &init, &c, &s, o.tol,
            )?);
        }
        4 => {
            let init: Handle<dyn Adaptor2dCurve2d> = adapted_curve2d(r)?;
            let bis: Handle<dyn Adaptor2dCurve2d> = adapted_curve2d(r)?;
            polar_results(&ComputeApproxOnPolarSurface::with_seam_curves(
                &init, &bis, &c, &s, o.tol,
            )?);
        }
        _ => {
            let init: Handle<dyn Adaptor2dCurve2d> = adapted_curve2d(r)?;
            let mut x = ComputeApproxOnPolarSurface::new();
            polar_setup(&mut x, o);
            bspline2d(&x.perform_with_initial(&init, &c, &s)?);
            polar_results(&x);
        }
    }
    Ok(())
}

pub fn polar_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let variant = r.integer();
    let o = options(&mut r);
    attempt(|| {
        let ins = inputs(&mut r)?;
        polar_variant(variant, &o, &mut r, ins)
    });
    end();
}

fn projected_typed(x: &ProjectedCurve) {
    p::show(x.line(), p::lin2d);
    p::show(x.circle(), p::circ2d);
    p::show(x.ellipse(), p::elips2d);
    p::show(x.hyperbola(), p::hypr2d);
    p::show(x.parabola(), p::parab2d);
    let kind = x.get_type().expect("probe type");
    let null_spline = (kind == CurveType::BSplineCurve && x.bspline().expect("probe").is_null())
        || (kind == CurveType::BezierCurve && x.bezier().expect("probe").is_null());
    p::flag(null_spline);
    if !null_spline {
        p::show(x.degree(), p::int);
        p::show(x.is_rational(), p::flag);
        p::show(x.nb_poles(), p::int);
        p::show(x.nb_knots(), p::int);
    }
}

/// gb_pl_projected_results.
fn projected_results(x: &ProjectedCurve) -> Result<(), Failure> {
    p::int(x.get_type()? as i32);
    p::real(x.get_tolerance());
    p::flag(x.is_periodic()?);
    p::show(x.first_parameter(), p::real);
    p::show(x.last_parameter(), p::real);
    projected_typed(x);
    bspline2d(&x.bspline()?);
    bezier2d(&x.bezier()?);
    p::show(make_pcurve_of_type(x), |c| print_curve2d(&c));
    p::show(x.continuity(), |s| p::int(s as i32));
    p::show(x.nb_intervals(Shape::C1), p::int);
    p::show(x.is_closed(), p::flag);
    p::show(x.period(), p::real);
    p::show(x.resolution(1.0e-3), p::real);
    attempt(|| x.trim(0., 1., 1.0e-7).map(|_| ()));
    attempt(|| {
        p::pnt2d(x.value(x.first_parameter()?)?);
        Ok(())
    });
    let curve = x.get_curve();
    p::flag(curve.is_null());
    if let Some(c) = curve.get() {
        p::real(c.first_parameter()?);
        p::real(c.last_parameter()?);
    }
    attempt(|| {
        p::int(x.shallow_copy()?.get_type()? as i32);
        Ok(())
    });
    Ok(())
}

pub fn projected_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let variant = r.integer();
    let o = options(&mut r);
    attempt(|| {
        let (c, s) = inputs(&mut r)?;
        let x = match variant {
            0 => {
                let x = ProjectedCurve::new();
                x.load_surface(&s);
                x.load_tolerance(o.tol);
                x.set_degree(o.degmin, o.degmax);
                x.set_max_segments(o.maxseg);
                x.set_bnd_pnt(o.bnd);
                x.set_max_dist(o.maxdist);
                x.perform(&c)?;
                x
            }
            1 => ProjectedCurve::with_curve(&s, &c)?,
            _ => ProjectedCurve::with_tolerance(&s, &c, o.tol)?,
        };
        projected_results(&x)
    });
    end();
}
