//! Supplemental BRep construction, sharing, and null-object regressions.
use rustycad_tkbrep::{
    brep::{Builder, Tool},
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
    assert_eq!(Tool::tolerance(builder.vertex()).unwrap(), 0.01);
    let moved = builder
        .shape()
        .moved(
            &Location::new(Transform3::translation([5.0, 6.0, 7.0])),
            false,
        )
        .unwrap();
    assert!(builder.shape().is_partner(&moved));
    assert_eq!(
        Tool::point(&Vertex::unchecked(moved.clone())).unwrap(),
        Point3::new(6.0, 8.0, 10.0)
    );
    assert_eq!(
        Tool::point(builder.vertex()).unwrap(),
        Point3::new(1.0, 2.0, 3.0)
    );
    assert!(
        !builder
            .shape()
            .is_partner(MakeVertex::new(Point3::default()).shape())
    );
    assert_eq!(
        Tool::tolerance(&Builder::make_vertex(Point3::default(), -1.0)).unwrap(),
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
        Tool::point(&vertex).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(
        Tool::tolerance(&vertex).unwrap_err().message(),
        "BRep_Tool:: TopoDS_Vertex hasn't gp_Pnt"
    );
}
