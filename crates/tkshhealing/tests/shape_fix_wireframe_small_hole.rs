//! Regression for ShapeFix_Wireframe::MergeSmallEdges on a hole of four small edges: the
//! native loop keeps joining the edges of a two-edge wire (the previous and the next edge
//! are the same one) until the hole is a single edge.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Wire};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixWireframe};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};

fn rectangle(x0: f64, y0: f64, x1: f64, y1: f64) -> Wire {
    BRepBuilderAPIMakePolygon::from_four_points(
        Point3::new(x0, y0, 0.),
        Point3::new(x1, y0, 0.),
        Point3::new(x1, y1, 0.),
        Point3::new(x0, y1, 0.),
        true,
    )
    .unwrap()
    .wire()
    .unwrap()
}

/// The 4x4 square with the reversed hole of side 1e-4 (`sh_fix_face` planar input 7).
fn face_with_small_hole() -> Face {
    let outer = rectangle(0., 0., 4., 4.);
    let mut hole = rectangle(1., 1., 1.0001, 1.0001);
    hole.reverse();
    let mut face = BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &outer, true)
        .unwrap()
        .face()
        .unwrap();
    BRepBuilder::default().add(&mut face, &hole).unwrap();
    face
}

fn wire_edge_counts(shape: &Shape) -> Vec<usize> {
    Explorer::new(shape, ShapeEnum::Wire, ShapeEnum::Shape)
        .unwrap()
        .map(|wire| ShapeIterator::new(&wire, true, true).count())
        .collect()
}

#[test]
fn small_hole_is_merged_into_one_edge() {
    let face = face_with_small_hole();
    let fixer = ShapeFixWireframe::from_shape(&face);
    FixRoot::set_precision(&fixer, 1e-3);
    FixRoot::set_max_tolerance(&fixer, 0.1);
    fixer.fix_wire_gaps().unwrap();
    assert!(fixer.fix_small_edges().unwrap());
    assert!(fixer.status_small_edges(ShapeExtendStatus::Done1));
    let result = fixer.shape();
    assert!(topods::face(&result).is_ok());
    assert_eq!(wire_edge_counts(&result), vec![4, 1]);
}
