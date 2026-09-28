//! Supplemental BRep construction, sharing, and null-object regressions.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Shape, Vertex},
};
use rustycad_tkmath::{
    gp::{Point3, Transform3},
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::MakeVertex;
#[test]
fn located_copies_share_geometry_without_mutating_original() {
    let builder = MakeVertex::with_precision(Point3::new(1.0, 2.0, 3.0), 0.01);
    assert!(builder.is_done());
    assert_eq!(BRepTool::vertex_tolerance(builder.vertex()).unwrap(), 0.01);
    let moved = builder
        .shape()
        .moved(
            &Location::new(Transform3::translation([5.0, 6.0, 7.0])),
            false,
        )
        .unwrap();
    assert!(builder.shape().is_partner(&moved));
    assert_eq!(
        BRepTool::pnt(&Vertex::unchecked(moved.clone())).unwrap(),
        Point3::new(6.0, 8.0, 10.0)
    );
    assert_eq!(
        BRepTool::pnt(builder.vertex()).unwrap(),
        Point3::new(1.0, 2.0, 3.0)
    );
    assert!(
        !builder
            .shape()
            .is_partner(MakeVertex::new(Point3::default()).shape())
    );
    assert_eq!(
        BRepTool::vertex_tolerance(&negative_tolerance_vertex()).unwrap(),
        1e-7
    );
}
#[test]
fn null_shapes_and_vertices_preserve_native_failures() {
    let null = Shape::default();
    assert!(null.is_null());
    assert!(null.is_partner(&Shape::default()));
    assert!(!null.is_partner(MakeVertex::new(Point3::default()).shape()));
    let vertex = Vertex::default();
    assert_eq!(
        BRepTool::pnt(&vertex).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert!(
        BRepTool::vertex_tolerance(&vertex)
            .unwrap_err()
            .message()
            .starts_with("BRep_Tool:: TopoDS_Vertex hasn't gp_Pnt")
    );
}

/// A vertex made with a negative tolerance: BRep_TVertex keeps `RealEpsilon()` and
/// BRep_Tool floors it at `Precision::Confusion()`.
fn negative_tolerance_vertex() -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::default(), -1.0)
        .unwrap();
    vertex
}
