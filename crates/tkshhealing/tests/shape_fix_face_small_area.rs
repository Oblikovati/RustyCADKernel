//! Native face, context, root, and result effects of removing small contours.
#[path = "support/surfaces.rs"]
mod surfaces;
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topods::{Builder, Face, ShapeIterator, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::{
    shape_build::ShapeBuildReShape,
    shape_fix::{FixRoot, ShapeFixFace},
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};

fn square(side: f64, offset: f64) -> Wire {
    let points = [
        Point3::new(offset, 0., 0.),
        Point3::new(offset + side, 0., 0.),
        Point3::new(offset + side, side, 0.),
        Point3::new(offset, side, 0.),
    ];
    let mut builder = BRepBuilderAPIMakeWire::new();
    for index in 0..4 {
        builder
            .add_edge(
                &BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % 4])
                    .unwrap()
                    .edge()
                    .unwrap(),
            )
            .unwrap();
    }
    builder.wire().unwrap()
}

fn fixture_face(mode: usize) -> Face {
    let mut face = surfaces::planar_face();
    if mode != 1 && mode != 2 {
        Builder.add(&mut face, &square(2., 0.)).unwrap();
    }
    if mode != 0 {
        Builder.add(&mut face, &square(1e-5, 0.5)).unwrap();
    }
    if mode == 4 {
        face.set_orientation(Orientation::Reversed);
    }
    face
}

fn fixture_row(mode: usize) -> Vec<i32> {
    let face = fixture_face(mode);
    let fixer = ShapeFixFace::new(face.clone().into_shape());
    fixer.set_precision(1e-3);
    if mode != 5 {
        fixer.set_context(&Handle::new(ShapeBuildReShape::new()));
    }
    if mode == 6 {
        let mut replacement = surfaces::planar_face();
        Builder.add(&mut replacement, &square(1e-5, 0.5)).unwrap();
        ReShape::replace(
            fixer.context().get().unwrap(),
            &face.clone().into_shape(),
            &replacement.into_shape(),
        )
        .unwrap();
    }
    let before = fixer.result();
    let changed = fixer.fix_small_area_wire(mode != 2).unwrap();
    let working = fixer.face().unwrap();
    let count = ShapeIterator::new(&working.clone().into_shape(), true, true)
        .filter(|child| child.shape_type().unwrap() == ShapeEnum::Wire)
        .count();
    let root = fixer.fix_root_data().with(|fields| fields.shape.clone());
    let removed = fixer.context().get().is_some_and(|context| {
        ReShape::apply(context, &face.clone().into_shape(), ShapeEnum::Shape)
            .unwrap()
            .is_null()
    });
    vec![
        i32::from(changed),
        count as i32,
        i32::from(working.is_same(&face)),
        working.orientation() as i32,
        i32::from(root.is_same(&face.into_shape())),
        i32::from(fixer.result().is_same(&before)),
        i32::from(removed),
    ]
}

#[test]
fn small_area_wire_removal_matches_native_state_and_false_return_for_whole_face() {
    for (mode, row) in include_str!("../../../tools/oracle/tkshhealing_face_small_area.expected")
        .lines()
        .enumerate()
    {
        let expected: Vec<i32> = row
            .split_whitespace()
            .skip(1)
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(fixture_row(mode), expected, "native case {row}");
    }
}
