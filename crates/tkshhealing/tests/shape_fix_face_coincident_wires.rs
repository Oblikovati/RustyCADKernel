//! Native two-coincident-edge wire removal and state preservation.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Edge, Face, ShapeIterator, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::{
    shape_build::ShapeBuildReShape,
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixFace},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
fn source_face() -> Face {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(2., 3., 4.).unwrap();
    topods::face(
        Explorer::new(maker.shape().unwrap(), ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
            .current(),
    )
    .unwrap()
}
fn doubled_wire(edge: &Edge, distinct: bool) -> Wire {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, edge).unwrap();
    let other = if distinct {
        edge.empty_copied().unwrap()
    } else {
        edge.reversed()
    };
    builder.add(&mut wire, &other).unwrap();
    wire
}
fn fixture_face(mode: i32) -> Face {
    let original = source_face();
    let edge = topods::edge(
        Explorer::new(&original, ShapeEnum::Edge, ShapeEnum::Shape)
            .unwrap()
            .current(),
    )
    .unwrap();
    let mut face = topods::face(&original.empty_copied().unwrap()).unwrap();
    let builder = BRepBuilder::default();
    builder.add(&mut face, &doubled_wire(&edge, false)).unwrap();
    if mode != 0 {
        let mut other = doubled_wire(&edge, mode == 2);
        if mode == 3 {
            other.set_orientation(Orientation::Internal);
        }
        builder.add(&mut face, &other).unwrap();
    }
    if mode == 4 {
        face.reverse();
    }
    face
}
fn expected(name: &str) -> Vec<i32> {
    include_str!("../../../tools/oracle/tkshhealing_face_coincident_wires.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|token| token.parse().unwrap())
        .collect()
}
fn run(name: &str, mode: i32, context: bool) {
    let face = fixture_face(mode);
    let tool = ShapeFixFace::new(face.clone().into_shape());
    let reshape = Handle::new(ShapeBuildReShape::new());
    if context {
        tool.set_context(&reshape);
    }
    let changed = tool.fix_wires_two_coinc_edges().unwrap();
    let output = tool.face().unwrap();
    let actual = vec![
        i32::from(changed),
        ShapeIterator::new(&output, false, true).count() as i32,
        output.orientation() as i32,
        i32::from(output.is_same(&face)),
        i32::from(tool.result().is_null()),
        i32::from(tool.status(ShapeExtendStatus::Done)),
        i32::from(reshape.is_recorded(&face)),
    ];
    assert_eq!(actual, expected(name), "{name}");
}
#[test]
fn native_two_edge_removal_policy_and_output_state() {
    for (name, mode, context) in [
        ("single", 0, false),
        ("both", 1, false),
        ("distinct", 2, false),
        ("internal", 3, false),
        ("reversed", 4, false),
        ("context", 2, true),
    ] {
        run(name, mode, context);
    }
}
#[test]
fn single_wire_intersection_delegate_preserves_face() {
    let face = fixture_face(0);
    let tool = ShapeFixFace::new(face.clone().into_shape());
    assert!(!tool.fix_intersecting_wires().unwrap());
    assert!(tool.face().unwrap().is_same(&face));
}
