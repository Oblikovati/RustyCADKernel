//! Compare both native outer-bound construction modes on every oriented box face.
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{self, Face, ShapeIterator, Wire},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::{shape_analysis::ShapeAnalysisWire, shape_extend::ShapeExtendStatus};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_outer_bound.expected");

#[test]
fn box_outer_bound_modes_and_reversed_wires_match_native() {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(100., 200., 300.).unwrap();
    let shape = maker.shape().unwrap().clone();
    let mut faces = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut index = 0;
    let mut rows = Vec::new();
    while faces.more() {
        let face = topods::face(faces.current()).unwrap();
        append_face_rows(&face, &mut index, &mut rows);
        faces.next().unwrap();
    }
    assert_eq!(index, 6);
    assert_eq!(rows.join("\n") + "\n", EXPECTED);
}

fn append_face_rows(face: &Face, index: &mut i32, rows: &mut Vec<String>) {
    for child in ShapeIterator::new(&face.clone().into_shape(), true, true) {
        if child.shape_type().unwrap() != ShapeEnum::Wire {
            continue;
        }
        *index += 1;
        let wire = topods::wire(&child).unwrap();
        for reversed in [false, true] {
            append_wire_rows(face, &wire, *index, reversed, rows);
        }
    }
}

fn append_wire_rows(
    face: &Face,
    original: &Wire,
    index: i32,
    reversed: bool,
    rows: &mut Vec<String>,
) {
    let mut wire = original.clone();
    if reversed {
        wire.reverse();
    }
    let analyzer = ShapeAnalysisWire::from_wire(&wire, face, 1e-7).unwrap();
    for api in [false, true] {
        let found = analyzer.check_outer_bound(api).unwrap();
        let done = analyzer.last_check_status(ShapeExtendStatus::Done1);
        rows.push(format!(
            "{index} {} {} {} {}",
            u8::from(reversed),
            u8::from(api),
            u8::from(found),
            u8::from(done)
        ));
    }
}

#[test]
fn unready_outer_bound_check_preserves_ok_status() {
    let analyzer = ShapeAnalysisWire::new();
    for api in [false, true] {
        assert!(!analyzer.check_outer_bound(api).unwrap());
        assert!(analyzer.last_check_status(ShapeExtendStatus::Ok));
    }
}
