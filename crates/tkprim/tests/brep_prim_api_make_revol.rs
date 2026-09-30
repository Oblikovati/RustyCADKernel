//! 1:1 translation of src/ModelingAlgorithms/TKPrim/GTests/BRepPrimAPI_MakeRevol_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeRevol, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex,
    BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use std::f64::consts::PI;

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

/// `BRepCheck_Analyzer(aShape).IsValid()`.
fn is_valid(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

fn check_toroidal_revolution(the_x_direction: Direction3, the_radius: f64) {
    let an_circle_axis = Axis2::with_x_direction(
        Point3::new(0.0, 100.00001, 0.0),
        dir(1.0, 0.0, 0.0),
        the_x_direction,
    )
    .unwrap();
    let a_circle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(an_circle_axis, the_radius).unwrap()).cast();
    let a_make_edge = BRepBuilderAPIMakeEdge::from_curve(&a_circle).unwrap();
    assert!(a_make_edge.is_done());

    let a_revolution_axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let edge: Shape = a_make_edge.edge().unwrap().into();
    let mut a_make_revol =
        BRepPrimAPIMakeRevol::new(&edge, a_revolution_axis, PI / 2.0, false).unwrap();
    assert!(a_make_revol.is_done());

    let a_result = a_make_revol.shape().unwrap().clone();
    assert!(!a_result.is_null());
    assert!(is_valid(&a_result));

    let an_explorer = Explorer::new(&a_result, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    assert!(an_explorer.more());
    let a_face = topods::face(an_explorer.current()).unwrap();
    let a_surface = BRepTool::surface(&a_face).unwrap();
    assert!(!a_surface.is_null());
    assert!(a_surface.is_kind_name("Geom_ToroidalSurface"));
    assert!(!a_surface.is_kind_name("Geom_SurfaceOfRevolution"));
}

// Migrated from tests/bugs/modalg_5/bug23954.  Revolving circles in the four
// orientations used by DRAW must preserve their toroidal surface type.
#[test]
fn BRepPrimAPI_MakeRevolTest__ModalgBug_23954_CircleRevolutionsKeepToroidalSurface() {
    check_toroidal_revolution(dir(0.0, 1.0, 0.0), 100.0);
    check_toroidal_revolution(dir(0.0, 0.0, 1.0), 100.0);
    check_toroidal_revolution(dir(0.0, -1.0, 0.0), 100.0);
    check_toroidal_revolution(dir(0.0, 0.0, 1.0), 10.0);
}

fn vertex_at(x: f64, y: f64, z: f64) -> Vertex {
    BRepBuilderAPIMakeVertex::new(Point3::new(x, y, z))
        .unwrap()
        .vertex()
        .unwrap()
}

// Migrated from tests/bugs/modalg_5/bug24328.  Revolving the two-edge wire
// must produce a shape accepted by the Boolean-operation validity checker.
#[test]
fn BRepPrimAPI_MakeRevolTest__ModalgBug_24328_RevolvedWireIsValid() {
    let a_vertex1 = vertex_at(23.5, 11.123, 0.0);
    let a_vertex2 = vertex_at(22.0, 11.1409, 0.0);
    let a_vertex3 = vertex_at(20.5, 11.1807, 0.0);
    let an_edge1: Edge = BRepBuilderAPIMakeEdge::from_vertices(&a_vertex1, &a_vertex2)
        .unwrap()
        .edge()
        .unwrap();
    let an_edge2: Edge = BRepBuilderAPIMakeEdge::from_vertices(&a_vertex2, &a_vertex3)
        .unwrap()
        .edge()
        .unwrap();
    let a_wire: Wire = BRepBuilderAPIMakeWire::from_two_edges(&an_edge1, &an_edge2)
        .unwrap()
        .wire()
        .unwrap();

    let a_wire: Shape = a_wire.into();
    let axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let mut a_make_revol = BRepPrimAPIMakeRevol::new(&a_wire, axis, 2.0 * PI, false).unwrap();
    assert!(a_make_revol.is_done());
    let a_result = a_make_revol.shape().unwrap().clone();
    assert!(!a_result.is_null());
    assert!(is_valid(&a_result));
}

// Migrated from tests/bugs/modalg_5/bug25578.  Revolving the partial-sphere
// face around both axes must complete without a singularity crash.
#[test]
fn BRepPrimAPI_MakeRevolTest__ModalgBug_25578_PartialSphereRevolutions() {
    let mut a_make_sphere =
        BRepPrimAPIMakeSphere::with_latitudes_angle(5.0, 0.0, PI / 2.0, PI / 2.0).unwrap();
    let a_sphere = a_make_sphere.shape().unwrap().clone();
    assert!(a_make_sphere.is_done());

    let a_face_explorer = Explorer::new(&a_sphere, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    assert!(a_face_explorer.more());
    let a_face: Shape = topods::face(a_face_explorer.current()).unwrap().into();

    let an_axis_z = Axis1::new(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let mut a_revol_z = BRepPrimAPIMakeRevol::new(&a_face, an_axis_z, 2.0 * PI, false).unwrap();
    assert!(a_revol_z.is_done());
    assert!(!a_revol_z.shape().unwrap().is_null());

    let an_axis_x = Axis1::new(Point3::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let mut a_revol_x = BRepPrimAPIMakeRevol::new(&a_face, an_axis_x, PI / 2.0, false).unwrap();
    assert!(a_revol_x.is_done());
    assert!(!a_revol_x.shape().unwrap().is_null());
}
