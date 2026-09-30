//! BRepBuilderAPI_FastSewing probe operation, the Rust twin of
//! tools/oracle/tktopalgo_probe_fastsewing.cxx (the request layout is documented there).
use super::brep_lib_common::shape as digest;
use super::brep_lib_package::guarded;
use super::common::{Reader, flag, int, real, text};
use super::face::surface;
use super::shape::tp_shape;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIFastSewing;

/// `tp_fastsew_item`.
fn item(sew: &mut BRepBuilderAPIFastSewing, r: &mut Reader<'_>) -> Result<(), Failure> {
    let added = match r.integer() {
        0 => sew.add_surface(&surface(r)?)?,
        1 => sew.add_shape(&tp_shape(r)?.shape)?,
        2 => sew.add_shape(&Shape::default())?,
        _ => sew.add_surface(&Handle::<dyn GeomSurface>::default())?,
    };
    flag(added);
    Ok(())
}

/// The fast sewing of one request.
fn fastsew_case(r: &mut Reader<'_>) -> Result<(), Failure> {
    let (tol, settol, perform) = (r.real(), r.real(), r.flag());
    let n = r.integer();
    let mut sew = BRepBuilderAPIFastSewing::new(tol);
    if settol >= 0.0 {
        sew.set_tolerance(settol);
    }
    for _ in 0..n {
        item(&mut sew, r)?;
    }
    if perform {
        sew.perform();
    }
    real(sew.get_tolerance());
    int(sew.get_statuses(None)? as i32);
    let mut stream: Vec<u8> = Vec::new();
    int(sew.get_statuses(Some(&mut stream))? as i32);
    text(&stream);
    digest(sew.get_result())
}

/// `bba_fastsew`.
pub fn fastsew_request(values: &[f64]) {
    guarded(values, fastsew_case);
}
