//! BRepOffsetAPI_MakeOffset probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffsetapi_make_offset.cxx (the records are documented
//! there).
use super::common::{Reader, attempt, end, flag, int};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIMakeOffset;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// Prints the exception of a failed step and ends the line.
pub fn failed(error: Failure) {
    print!("{} ", error.exception_type());
    end();
}

/// The shape of the program at a table index.
pub fn at(t: &TpShapes, index: i32) -> Result<Shape, Failure> {
    t.shapes
        .get(index as usize)
        .cloned()
        .ok_or_else(|| Failure::out_of_range("of_probe: shape index"))
}

/// The join of its enumeration value (0 arc, 1 tangent, 2 intersection).
fn join_of(value: i32) -> JoinType {
    match value {
        1 => JoinType::Tangent,
        2 => JoinType::Intersection,
        _ => JoinType::Arc,
    }
}

/// `of_sorted`: the sorted indices and orientations of a list.
fn sorted(map: &IndexedMapOfShape, l: &List<Shape>) {
    let mut ids: Vec<(i32, i32)> = l
        .iter()
        .map(|s| (map.find_index(s) as i32, s.orientation() as i32))
        .collect();
    ids.sort();
    int(ids.len() as i32);
    for (index, orientation) in ids {
        int(index);
        int(orientation);
    }
}

/// `of_make_offset_perform`.
fn perform(maker: &mut BRepOffsetAPIMakeOffset, t: &TpShapes, offset: f64, alt: f64) {
    if let Err(error) = maker.perform(offset, alt) {
        print!("{} ", error.exception_type());
        return;
    }
    flag(maker.is_done());
    if !maker.is_done() {
        return;
    }
    attempt(|| maker.shape().and_then(shape));
    let mut map = IndexedMapOfShape::default();
    let result = maker.shape().cloned().unwrap_or_default();
    topexp::map_shapes(&result, &mut map, true, true);
    for g in &t.shapes {
        attempt(|| maker.generated(g).map(|l| sorted(&map, l)));
    }
}

/// The maker of a mode: Init(face), or Init(join) with AddWire of each wire of the face,
/// or no spine.
fn maker_of(
    face: &Face,
    mode: i32,
    join: JoinType,
    open: bool,
) -> Result<BRepOffsetAPIMakeOffset, Failure> {
    let mut maker = BRepOffsetAPIMakeOffset::new();
    if mode == 0 {
        maker.init_face(face, join, open)?;
        return Ok(maker);
    }
    maker.init(join, open);
    if mode == 1 {
        let mut ex = Explorer::new(face.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)?;
        while ex.more() {
            maker.add_wire(&topods::wire(ex.current())?);
            ex.next()?;
        }
    }
    Ok(maker)
}

/// `oa_make_offset program face mode join open approx offset1 alt1 offset2 alt2`.
pub fn make_offset_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let index = r.integer();
    let (mode, join) = (r.integer(), join_of(r.integer()));
    let (open, approx) = (r.flag(), r.flag());
    let (offset1, alt1, offset2, alt2) = (r.real(), r.real(), r.real(), r.real());
    let face = at(&t, index).and_then(|f| topods::face(&f));
    let mut maker = match face.and_then(|face| maker_of(&face, mode, join, open)) {
        Ok(maker) => maker,
        Err(error) => return failed(error),
    };
    maker.set_approx(approx);
    perform(&mut maker, &t, offset1, alt1);
    perform(&mut maker, &t, offset2, alt2);
    end();
}
