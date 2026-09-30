//! Behavior tests of BRepBuilderAPI beyond the upstream suites (docs/topalgo-design.md
//! section 10): the virtual MakeShape root, the wrapping makers and their native quirks,
//! the modifications and their histories, Collect and FindPlane. The numeric probe
//! (`bba_*` requests) compares the same members with OCCT bit for bit.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::BRepToolsModification;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Circle3, Direction3, GeneralTransform3, Plane3, Point3, Transform3,
};
use rustycad_tktopalgo::brep_builder_api::{
    self, BRepBuilderAPICollect, BRepBuilderAPICopy, BRepBuilderAPIEdgeError,
    BRepBuilderAPIFaceError, BRepBuilderAPIFindPlane, BRepBuilderAPIGTransform,
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShell, BRepBuilderAPIMakeSolid,
    BRepBuilderAPIMakeWire, BRepBuilderAPIModifyShape, BRepBuilderAPINurbsConvert,
    BRepBuilderAPIShellError, BRepBuilderAPITransform, BRepBuilderAPIWireError,
};

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn z() -> Direction3 {
    Direction3::new(0.0, 0.0, 1.0).unwrap()
}

/// The unit square face on the XOY plane.
fn square() -> Face {
    let plane = Plane3::new(p(0.0, 0.0, 0.0), z()).unwrap();
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, (0.0, 1.0, 0.0, 1.0))
        .unwrap()
        .face()
        .unwrap()
}

fn sub_shapes(shape: &Shape, kind: ShapeEnum) -> IndexedMapOfShape {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut map).unwrap();
    map
}

fn segment(a: Point3, b: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(a, b)
        .unwrap()
        .edge()
        .unwrap()
}

#[test]
fn a_maker_that_is_not_done_raises_not_done_from_shape_and_check() {
    let mut maker = BRepBuilderAPIMakeEdge::new();
    assert!(!maker.is_done());
    assert_eq!(
        maker.error(),
        BRepBuilderAPIEdgeError::PointProjectionFailed
    );
    assert_eq!(
        maker.check().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        maker.shape().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        maker.edge().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    let same = BRepBuilderAPIMakeEdge::from_points(p(1.0, 0.0, 0.0), p(1.0, 0.0, 0.0)).unwrap();
    assert_eq!(
        same.error(),
        BRepBuilderAPIEdgeError::LineThroughIdenticPoints
    );
}

#[test]
fn a_failing_init_keeps_the_edge_of_an_earlier_one() {
    let circle = Circle3::new(Axis2::new(p(0.0, 0.0, 0.0), z()).unwrap(), 1.0).unwrap();
    let mut maker = BRepBuilderAPIMakeEdge::from_circ(circle).unwrap();
    let first = maker.shape().unwrap().clone();
    let curve = BRepTool::curve(&maker.edge().unwrap()).unwrap().0;
    maker
        .init_curve_points(&curve, p(5.0, 5.0, 5.0), p(1.0, 0.0, 0.0))
        .unwrap();
    assert!(maker.is_done());
    assert_eq!(
        maker.error(),
        BRepBuilderAPIEdgeError::PointProjectionFailed
    );
    assert!(maker.shape().unwrap().is_same(&first));
    assert!(maker.generated(&first).unwrap().is_empty());
    assert!(maker.modified(&first).unwrap().is_empty());
    assert!(!maker.is_deleted(&first).unwrap());
}

#[test]
fn the_empty_solid_maker_is_done_with_a_null_shape() {
    let mut maker = BRepBuilderAPIMakeSolid::new().unwrap();
    assert!(maker.is_done());
    assert_eq!(
        maker.check().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert!(maker.shape().unwrap().is_null());
    assert!(!maker.solid().unwrap().is_null());
}

#[test]
fn closing_a_polygon_that_is_not_done_raises_not_done() {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    polygon.add_point(p(0.0, 0.0, 0.0)).unwrap();
    assert!(!polygon.is_done() && !polygon.added());
    assert_eq!(
        polygon.close().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    polygon.add_point(p(1.0, 0.0, 0.0)).unwrap();
    polygon.add_point(p(1.0, 1.0, 0.0)).unwrap();
    polygon.close().unwrap();
    assert_eq!(polygon.wire().unwrap().nb_children(), 3);
    let wire = polygon.wire().unwrap().into_shape();
    assert!(polygon.shape().unwrap().is_same(&wire));
}

#[test]
fn the_wire_errors_follow_the_brep_lib_maker() {
    let mut wire = BRepBuilderAPIMakeWire::new();
    assert_eq!(wire.error(), BRepBuilderAPIWireError::EmptyWire);
    wire.add_edge(&segment(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0)))
        .unwrap();
    assert_eq!(wire.error(), BRepBuilderAPIWireError::WireDone);
    wire.add_edge(&segment(p(5.0, 5.0, 0.0), p(6.0, 5.0, 0.0)))
        .unwrap();
    assert_eq!(wire.error(), BRepBuilderAPIWireError::DisconnectedWire);
    // The BRepLib maker is not done any more, but the command flag stays set: Check
    // passes and Shape() returns the wire of the first Add (natively too).
    assert!(!wire.is_done());
    assert!(wire.check().is_ok());
    assert_eq!(wire.shape().unwrap().nb_children(), 1);
}

#[test]
fn the_face_and_shell_makers_start_without_a_shape() {
    assert_eq!(
        BRepBuilderAPIMakeFace::new().error(),
        BRepBuilderAPIFaceError::NoFace
    );
    assert_eq!(
        BRepBuilderAPIMakeShell::new().error(),
        BRepBuilderAPIShellError::EmptyShell
    );
    let face = square();
    let mut rebuilt = BRepBuilderAPIMakeFace::from_face(&face).unwrap();
    // Init(F) copies the face but does not mark the maker done, natively too.
    assert!(!rebuilt.is_done());
    assert_eq!(rebuilt.error(), BRepBuilderAPIFaceError::FaceDone);
    assert_eq!(
        rebuilt.shape().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}

#[test]
fn the_package_defaults_are_the_brep_lib_globals() {
    assert_eq!(brep_builder_api::precision(), 1e-7);
    let plane = brep_builder_api::plane();
    assert_eq!(
        plane.object_address(),
        rustycad_tktopalgo::brep_lib::plane().object_address()
    );
}

#[test]
fn a_rigid_transformation_moves_the_shape_and_keeps_its_geometry() {
    let face = square();
    let trsf = Transform3::translation([0.0, 0.0, 2.0]);
    let mut moved = BRepBuilderAPITransform::with_shape(&face, trsf, false, false).unwrap();
    let result = moved.shape().unwrap().clone();
    assert!(result.is_partner(&face));
    let edge = sub_shapes(&face, ShapeEnum::Edge)
        .find_key(1)
        .unwrap()
        .clone();
    let image = moved.modified_shape(&edge).unwrap();
    assert!(image.is_partner(&edge) && !image.is_same(&edge));
    assert_eq!(moved.modified(&edge).unwrap().extent(), 1);
    let point = BRepTool::pnt(
        &topods::vertex(sub_shapes(&result, ShapeEnum::Vertex).find_key(1).unwrap()).unwrap(),
    );
    assert_eq!(point.unwrap().z(), 2.0);
}

#[test]
fn a_copying_or_scaling_transformation_rebuilds_the_geometry() {
    let face = square();
    let mut scale = Transform3::default();
    scale.set_scale(p(0.0, 0.0, 0.0), 2.0).unwrap();
    let mut scaled = BRepBuilderAPITransform::with_shape(&face, scale, false, false).unwrap();
    let result = scaled.shape().unwrap().clone();
    assert!(!result.is_partner(&face));
    let foreign = segment(p(0.0, 0.0, 5.0), p(1.0, 0.0, 5.0));
    let error = scaled.modified_shape(&foreign).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
    let mut mirror = Transform3::default();
    mirror.set_mirror_axis(Axis1::new(p(0.0, 0.0, 0.0), z()));
    let mut copied = BRepBuilderAPITransform::with_shape(&face, mirror, true, false).unwrap();
    assert!(!copied.shape().unwrap().is_partner(&face));
}

#[test]
fn copy_shares_the_geometry_only_without_the_geometry_copy() {
    let face = square();
    let surface_of = |shape: &Shape| {
        BRepTool::surface_with_location(&topods::face(shape).unwrap())
            .unwrap()
            .0
            .object_address()
    };
    let mut shallow = BRepBuilderAPICopy::with_shape(&face, false, false).unwrap();
    let shallow_face = shallow.shape().unwrap().clone();
    assert!(!shallow_face.is_same(&face));
    assert_eq!(surface_of(&shallow_face), surface_of(&face));
    let mut deep = BRepBuilderAPICopy::with_shape(&face, true, false).unwrap();
    let deep_face = deep.shape().unwrap().clone();
    assert_ne!(surface_of(&deep_face), surface_of(&face));
    // Perform marks the command not done to force a new copy, but the modifier keeps the
    // images of the shapes it already recorded, so the same copy comes back (natively too).
    deep.perform(&face, true, false).unwrap();
    assert!(deep.shape().unwrap().is_same(&deep_face));
}

#[test]
fn nurbs_convert_gives_b_spline_geometry() {
    let face = square();
    let mut convert = BRepBuilderAPINurbsConvert::with_shape(&face, false).unwrap();
    let result = topods::face(convert.shape().unwrap()).unwrap();
    assert!(
        BRepTool::surface(&result)
            .unwrap()
            .is_instance_name("Geom_BSplineSurface")
    );
    let edge = sub_shapes(&face, ShapeEnum::Edge)
        .find_key(1)
        .unwrap()
        .clone();
    let image = convert.modified_shape(&edge).unwrap();
    assert!(sub_shapes(&result.into_shape(), ShapeEnum::Edge).contains(&image));
}

#[test]
fn a_general_transformation_stretches_the_converted_shape() {
    let edge = segment(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0));
    let mut stretch = GeneralTransform3::default();
    stretch.set_affinity_plane(
        Axis2::new(p(0.0, 0.0, 0.0), Direction3::new(1.0, 0.0, 0.0).unwrap()).unwrap(),
        3.0,
    );
    let mut deformed = BRepBuilderAPIGTransform::with_shape(&edge, stretch, false).unwrap();
    let result = deformed.shape().unwrap().clone();
    let vertices = sub_shapes(&result, ShapeEnum::Vertex);
    let far = BRepTool::pnt(&topods::vertex(vertices.find_key(2).unwrap()).unwrap()).unwrap();
    assert!((far.x() - 3.0).abs() <= 1e-12);
    assert!(!deformed.modified_shape(&edge).unwrap().is_null());
}

#[test]
fn modify_shape_needs_a_shape_and_a_modification() {
    let face = square();
    let mut modify = BRepBuilderAPIModifyShape::new();
    let error = modify.do_modif_shape(&face).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    let copy: Handle<dyn BRepToolsModification> = Handle::new(
        rustycad_tkbrep::brep_tools::CopyModification::new(true, false),
    )
    .cast();
    modify.do_modif_modification(&copy).unwrap();
    assert!(modify.is_done());
    assert!(
        modify
            .modified_shape(&face)
            .unwrap()
            .is_same(modify.shape().unwrap())
    );
}

#[test]
fn find_plane_keeps_only_planes_every_sampled_point_lies_in() {
    let planar = BRepBuilderAPIMakePolygon::from_three_points(
        p(0.0, 0.0, 1.0),
        p(1.0, 0.0, 1.0),
        p(0.0, 1.0, 1.0),
        true,
    )
    .unwrap()
    .wire()
    .unwrap();
    let finder = BRepBuilderAPIFindPlane::from_shape(&planar, -1.0).unwrap();
    assert!(finder.found());
    let plane = finder.plane().get().unwrap().pln();
    assert!(plane.square_distance(p(5.0, 5.0, 1.0)) <= 1e-20);
    let skew = BRepBuilderAPIMakePolygon::from_four_points(
        p(0.0, 0.0, 0.0),
        p(1.0, 0.0, 0.0),
        p(1.0, 1.0, 0.0),
        p(0.0, 1.0, 0.5),
        true,
    )
    .unwrap()
    .wire()
    .unwrap();
    assert!(
        !BRepBuilderAPIFindPlane::from_shape(&skew, -1.0)
            .unwrap()
            .found()
    );
    assert!(!BRepBuilderAPIFindPlane::new().found());
}

#[test]
fn collect_traces_the_images_through_a_chain() {
    let face = square();
    let trsf = Transform3::translation([1.0, 0.0, 0.0]);
    let mut moved = BRepBuilderAPITransform::with_shape(&face, trsf, true, false).unwrap();
    let mut collect = BRepBuilderAPICollect::new();
    collect.add(&face, &mut moved).unwrap();
    let step1 = moved.shape().unwrap().clone();
    let mut copy = BRepBuilderAPICopy::with_shape(&step1, true, false).unwrap();
    collect.add(&step1, &mut copy).unwrap();
    let last = copy.shape().unwrap().clone();
    let images = collect.modification().find(&face).unwrap();
    assert_eq!(images.extent(), 1);
    assert!(images.first().unwrap().is_same(&last));
    collect.add_generated(&last, &step1).unwrap();
    assert_eq!(collect.generated().find(&face).unwrap().extent(), 1);
    collect.filter(&last).unwrap();
    assert!(collect.generated().find(&face).unwrap().is_empty());
}
