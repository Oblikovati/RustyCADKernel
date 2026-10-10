//! BRepFill_Section and BRepFill_SectionPlacement probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_brepfill_section.cxx (the records are documented there).
use super::brep_fill_generator::{failed, indexed};
use super::common::{Reader, attempt, end, flag, int, real};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shape_index, tp_shapes};
use rustycad_tkbool::brep_fill::{
    BRepFillEdge3DLaw, BRepFillLocationLaw, BRepFillSection, BRepFillSectionPlacement,
};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillCorrectedFrenet, GeomFillCurveAndTrihedron, GeomFillLocationLaw, GeomFillTrihedronLaw,
};
use rustycad_tkmath::gp::Transform3;

/// The shape of the program at a table index; a null shape for -1.
fn at(t: &TpShapes, index: i32) -> Result<Shape, Failure> {
    if index < 0 {
        return Ok(Shape::default());
    }
    t.shapes
        .get(index as usize)
        .cloned()
        .ok_or_else(|| Failure::out_of_range("bl_probe: shape index"))
}

/// The section of a request.
fn section_of(r: &mut Reader<'_>, t: &TpShapes) -> Result<BRepFillSection, Failure> {
    let profile = at(t, r.integer())?;
    let v: Vertex = topods::vertex(&at(t, r.integer())?)?;
    let (contact, correction) = (r.flag(), r.flag());
    BRepFillSection::with_profile(&profile, &v, contact, correction)
}

/// `bf_section program profile vertex contact correction`.
pub fn section_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let mut section = match section_of(&mut r, &t) {
        Ok(section) => section,
        Err(error) => return failed(error),
    };
    section.set(true);
    flag(section.is_law());
    flag(section.is_punctual());
    flag(section.with_contact());
    flag(section.with_correction());
    int(tp_shape_index(&t, section.original_shape()));
    int(tp_shape_index(&t, section.vertex()));
    attempt(|| shape(section.wire()));
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes(section.wire(), &mut m, true, true);
    for s in &t.shapes {
        attempt(|| indexed(&m, &section.modified_shape(s)?));
    }
    end();
}

/// `bl_trsf`.
fn trsf(t: &Transform3) -> Result<(), Failure> {
    for i in 1..=3 {
        for j in 1..=4 {
            real(t.value(i, j)?);
        }
    }
    Ok(())
}

/// The placement queries of a request.
fn placement(r: &mut Reader<'_>, t: &TpShapes) -> Result<(), Failure> {
    let path = topods::wire(&at(t, r.integer())?)?;
    let section = at(t, r.integer())?;
    let v = at(t, r.integer())?;
    let (contact, correction) = (r.flag(), r.flag());
    let tri: Handle<dyn GeomFillTrihedronLaw> = Handle::new(GeomFillCorrectedFrenet::new()).cast();
    let model: Handle<dyn GeomFillLocationLaw> =
        Handle::new(GeomFillCurveAndTrihedron::new(&tri)).cast();
    let law: Handle<dyn BRepFillLocationLaw> =
        Handle::new(BRepFillEdge3DLaw::new(&path, &model)?).cast();
    let place = BRepFillSectionPlacement::with_vertex(&law, &section, &v, contact, correction)?;
    trsf(place.transformation())?;
    attempt(|| {
        real(place.abscissa_on_path()?);
        Ok(())
    });
    Ok(())
}

/// `bf_section_placement program path section vertex contact correction`.
pub fn section_placement_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    if let Err(error) = placement(&mut r, &t) {
        print!("{} ", error.exception_type());
    }
    end();
}
