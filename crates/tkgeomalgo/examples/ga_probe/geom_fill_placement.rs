//! GeomFill_SectionPlacement probe operation, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geomfill_placement.cxx.
use super::common::{Reader, attempt, end, flag, real, show};
use super::geom_fill_laws::gfl_curve;
use super::geom_fill_trihedron::{gft_curve, gft_law};
use super::ics_geometry::geom_curve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomGeometry};
use rustycad_tkgeomalgo::geom_fill::*;
use rustycad_tkmath::gp::Transform3;

fn state(p: &GeomFillSectionPlacement) {
    flag(p.is_done());
    [
        p.parameter_on_path(),
        p.parameter_on_section(),
        p.distance(),
        p.angle(),
    ]
    .into_iter()
    .for_each(real);
}

fn trsf(t: Transform3) -> Result<(), Failure> {
    for i in 1..=3 {
        for j in 1..=4 {
            real(t.value(i, j)?);
        }
    }
    Ok(())
}

/// `ga_gfp_outputs`.
fn outputs(p: &GeomFillSectionPlacement, curve: bool) {
    state(p);
    for k in 0..4 {
        attempt(|| trsf(p.transformation(k & 1 != 0, k & 2 != 0)?));
    }
    if !curve {
        return;
    }
    for k in 0..2 {
        show(p.section(k != 0), gfl_curve);
        show(p.modified_section(k != 0), gfl_curve);
    }
}

/// `law path point|curve tol param`.
pub fn placement_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let law: Handle<dyn GeomFillLocationLaw> = match gft_law(&mut r) {
        Ok(t) => Handle::new(GeomFillCurveAndTrihedron::new(&t)).cast(),
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    let l = law.get().expect("probe law");
    let c = gft_curve(&mut r);
    show(l.set_curve(&c), flag);
    let point = r.flag();
    let s: Handle<dyn GeomGeometry> = if point {
        Handle::new(GeomCartesianPoint::from_pnt(r.pnt())).cast()
    } else {
        geom_curve(&mut r).cast()
    };
    let (tol, param) = (r.real(), r.real());
    let mut p = match GeomFillSectionPlacement::new(&law, &s) {
        Ok(p) => p,
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    state(&p);
    attempt(|| p.perform(tol));
    outputs(&p, !point);
    attempt(|| p.perform_at(param, tol));
    outputs(&p, !point);
    p.set_location(&law);
    attempt(|| p.perform_on_path(&c, tol / 10.0));
    outputs(&p, !point);
    end();
}
