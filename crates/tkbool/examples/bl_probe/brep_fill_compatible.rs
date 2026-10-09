//! BRepFill_CompatibleWires probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_brepfill_compatible.cxx (the record is documented there).
use super::brep_fill_generator::{failed, list};
use super::common::{Reader, attempt, end, flag, int};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_fill::BRepFillCompatibleWires;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The sections named by the request.
fn sections_of(r: &mut Reader<'_>, t: &TpShapes) -> Result<Vec<Shape>, Failure> {
    let n = r.integer();
    let mut sections = Vec::new();
    for _ in 0..n {
        let index = r.integer() as usize;
        sections.push(
            t.shapes
                .get(index)
                .cloned()
                .ok_or_else(|| Failure::out_of_range("bl_bf_compatible: section index"))?,
        );
    }
    Ok(sections)
}

/// `bf_compatible program n (wire)... percent rotation`.
pub fn compatible_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let sections = match sections_of(&mut r, &t) {
        Ok(sections) => sections,
        Err(error) => return failed(error),
    };
    let mut compatible = BRepFillCompatibleWires::with_sections(&sections);
    compatible.set_percent(r.real());
    let rotation = r.flag();
    if let Err(error) = compatible.perform(rotation) {
        return failed(error);
    }
    int(compatible.get_status() as i32);
    flag(compatible.is_done());
    flag(compatible.is_degenerated_first_section());
    flag(compatible.is_degenerated_last_section());
    int(compatible.shape().len() as i32);
    let mut all = IndexedMapOfShape::default();
    for s in compatible.shape() {
        attempt(|| shape(s));
        if !s.is_null() {
            topexp::map_shapes(s, &mut all, true, true);
        }
    }
    int(compatible.generated().extent() as i32);
    for s in &t.shapes {
        if s.is_null() || s.shape_type().ok() != Some(ShapeEnum::Edge) {
            continue;
        }
        attempt(|| {
            list(
                &all,
                compatible.generated_shapes(&Edge::unchecked(s.clone())),
            )
        });
    }
    end();
}
