//! The FilletSurf builder (`fl_fillet_surf`); the twin of the FilletSurf probe of
//! tools/oracle/tkfillet_probe_api.cxx.
use super::api::api_solid;
use super::common::{Reader, attempt, end, int, real};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::fillet_surf::{FilletSurfBuilder, StatusDone};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Null curves or surfaces of the builder.
fn null() -> Failure {
    Failure::with_message("Standard_NullObject", "fl_fillet_surf")
}

/// The sections of the simulation of the `i`-th surface: their number and the range of
/// the first one.
fn sections(b: &FilletSurfBuilder, i: i32) -> Result<(), Failure> {
    int(b.nb_section(i)?);
    let c = b.section(i, 1)?;
    let c = c.get().ok_or_else(null)?;
    real(c.first_parameter());
    real(c.last_parameter());
    Ok(())
}

/// The `i`-th surface: its tolerance, the ranges of its curves and its bounds.
fn surface(b: &FilletSurfBuilder, i: i32) -> Result<(), Failure> {
    real(b.tol_app3d(i)?);
    let c = b.curve_on_face1(i)?;
    let c = c.get().ok_or_else(null)?;
    real(c.first_parameter());
    real(c.last_parameter());
    real(
        b.pcurve_on_face2(i)?
            .get()
            .ok_or_else(null)?
            .first_parameter(),
    );
    real(
        b.pcurve2_on_fillet(i)?
            .get()
            .ok_or_else(null)?
            .last_parameter(),
    );
    let (u1, u2, v1, v2) = b.surface_fillet(i)?.get().ok_or_else(null)?.bounds();
    for v in [u1, u2, v1, v2] {
        real(v);
    }
    Ok(())
}

/// The results of a computed or simulated builder.
fn results(b: &FilletSurfBuilder, simul: bool) -> Result<(), Failure> {
    int(b.is_done() as i32);
    int(b.status_error() as i32);
    if b.is_done() == StatusDone::IsNotOk {
        return Ok(());
    }
    let n = b.nb_surface()?;
    int(n);
    for i in 1..=n {
        if simul {
            sections(b, i)?
        } else {
            surface(b, i)?
        }
    }
    real(b.first_parameter()?);
    real(b.last_parameter()?);
    int(b.start_section_status()? as i32);
    int(b.end_section_status()? as i32);
    Ok(())
}

/// `fl_fillet_surf solid e1 e2 e3 r simulate`: see tools/oracle/tkfillet_probe_api.cxx.
pub fn fillet_surf_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let kind = r.integer();
    let ie = [r.integer(), r.integer(), r.integer()];
    let (rad, simul) = (r.real(), r.flag());
    attempt(|| {
        let s = api_solid(kind)?;
        let mut em = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&s, ShapeEnum::Edge, &mut em)?;
        let mut e: Vec<Shape> = Vec::new();
        for k in ie {
            if k > 0 && k as usize <= em.extent() {
                e.push(em.find_key(k as usize)?.clone());
            }
        }
        let mut b = FilletSurfBuilder::new(&s, &e, rad, 1.0e-2, 1.0e-4, 1.0e-5)?;
        if simul {
            b.simulate()?;
        } else {
            b.perform()?;
        }
        results(&b, simul)
    });
    end();
}
