//! The Hermit probe; the counterpart of tools/oracle/tkgeombase_probe_hermit.cxx.
use super::common::{self as p, Reader, attempt, end};
use super::lprop::build;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkgeombase::hermit;

fn curve(c: &Handle<Geom2dBSplineCurve>) -> Result<(), Failure> {
    p::int(c.degree());
    p::int(c.nb_poles());
    for i in 1..=c.nb_poles() {
        p::pnt2d(c.pole(i)?);
    }
    p::int(c.nb_knots());
    for i in 1..=c.nb_knots() {
        p::real(c.knot(i)?);
        p::int(c.multiplicity(i)?);
    }
    Ok(())
}

pub fn hermit_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, tp, tk, kmin, kmax) = (r.integer(), r.real(), r.real(), r.real(), r.real());
    attempt(|| {
        if mode == 1 {
            let c: Handle<Geom2dBSplineCurve> = build::curve2d(&mut r)?.cast();
            return curve(&hermit::solution_2d(&c, tp, tk)?);
        }
        let c: Handle<GeomBSplineCurve> = build::curve3d(&mut r)?.cast();
        if mode == 0 {
            return curve(&hermit::solution_3d(&c, tp, tk)?);
        }
        let (mut a, mut b) = (kmin, kmax);
        hermit::solutionbis(&c, &mut a, &mut b, tp, tk)?;
        p::real(a);
        p::real(b);
        Ok(())
    });
    end();
}
