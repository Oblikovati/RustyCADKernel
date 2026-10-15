//! BRepOffsetAPI_ThruSections probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffsetapi_thru_sections.cxx (the records are
//! documented there).
use super::common::{Reader, attempt, end, flag, int};
use super::make_offset::{at, failed};
use super::make_pipe_shell::list;
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIThruSections;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// The parametrization of its enumeration value.
fn par_type_of(value: i32) -> ParametrizationType {
    match value {
        1 => ParametrizationType::Centripetal,
        2 => ParametrizationType::IsoParametric,
        _ => ParametrizationType::ChordLength,
    }
}

/// The continuity of its enumeration value.
fn continuity_of(value: i32) -> Continuity {
    [
        Continuity::C0,
        Continuity::G1,
        Continuity::C1,
        Continuity::G2,
        Continuity::C2,
        Continuity::C3,
        Continuity::CN,
    ][value.clamp(0, 6) as usize]
}

/// The loft of the request with its settings and sections.
fn loft_of(r: &mut Reader<'_>, t: &TpShapes) -> Result<BRepOffsetAPIThruSections, Failure> {
    let (solid, ruled, pres3d) = (r.flag(), r.flag(), r.real());
    let mut loft = BRepOffsetAPIThruSections::new(solid, ruled, pres3d);
    loft.check_compatibility(r.flag());
    loft.set_smoothing(r.flag());
    loft.set_par_type(par_type_of(r.integer()));
    loft.set_continuity(continuity_of(r.integer()));
    let maxdeg = r.integer();
    if maxdeg >= 0 {
        loft.set_max_degree(maxdeg);
    }
    loft.set_mutable_input(r.flag());
    for _ in 0..r.integer() {
        let kind = r.integer();
        let section = at(t, r.integer())?;
        if kind == 1 {
            loft.add_vertex(&topods::vertex(&section)?)?;
        } else {
            loft.add_wire(&topods::wire(&section)?);
        }
    }
    Ok(loft)
}

/// The records after Build.
fn print_built(loft: &mut BRepOffsetAPIThruSections, t: &TpShapes) {
    let done = loft.is_done();
    flag(done);
    int(loft.get_status() as i32);
    let mut map = IndexedMapOfShape::default();
    if done {
        attempt(|| loft.shape().and_then(shape));
        attempt(|| shape(loft.first_shape().as_ref()));
        attempt(|| shape(loft.last_shape().as_ref()));
        let result = loft.shape().cloned().unwrap_or_default();
        topexp::map_shapes(&result, &mut map, true, true);
    }
    for s in &t.shapes {
        attempt(|| loft.generated(s).map(|l| list(&map, l)));
        attempt(|| {
            let face = loft.generated_face(s);
            int(if face.is_null() {
                -1
            } else {
                map.find_index(&face) as i32
            });
            Ok(())
        });
    }
}

/// `oa_thru_sections program solid ruled pres3d check smoothing partype continuity
/// maxdeg mutable n (kind index)...`.
pub fn thru_sections_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let mut loft = match loft_of(&mut r, &t) {
        Ok(loft) => loft,
        Err(error) => return failed(error),
    };
    if let Err(error) = loft.build(&ProgressRange::default()) {
        return failed(error);
    }
    print_built(&mut loft, &t);
    end();
}
