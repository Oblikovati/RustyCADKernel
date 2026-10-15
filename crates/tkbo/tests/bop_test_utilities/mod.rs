//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BOPTest_Utilities.pxx (3d097a0):
//! the shape factories and the BRepAlgoAPI / BOPAlgo fixtures of the TKBO GTests.
#![allow(dead_code, non_snake_case)]

use rustycad_tkbo::bop_algo::{BOPAlgoBOP, BOPAlgoPaveFiller, Operation};
use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICommon, BRepAlgoAPICut, BRepAlgoAPIFuse};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::elslib::plane_value;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Circle3, Direction3, Plane3, Point2, Point3, Transform3, Vector2, Vector3,
};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakePrism,
    BRepPrimAPIMakeRevol, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex, BRepBuilderAPIMakeWire,
    BRepBuilderAPINurbsConvert, BRepBuilderAPITransform,
};
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

mod fixtures;
mod profile;
// Each test target includes this module and uses a different subset of it.
#[allow(unused_imports)]
pub use fixtures::*;
#[allow(unused_imports)]
pub use profile::*;

pub fn DefaultTolerance() -> f64 {
    1.0e-6
}

pub fn DefaultFuzzyValue() -> f64 {
    1.0e-8
}

/// Calculate surface area of a shape.
pub fn GetSurfaceArea(shape: &Shape) -> f64 {
    if shape.is_null() {
        return 0.0;
    }
    let mut props = GProps::default();
    BRepGProp::surface_properties(shape, &mut props, false, false).unwrap();
    props.mass()
}

/// Calculate volume of a shape.
pub fn GetVolume(shape: &Shape) -> f64 {
    if shape.is_null() {
        return 0.0;
    }
    let mut props = GProps::default();
    BRepGProp::volume_properties(shape, &mut props, false, false, false).unwrap();
    props.mass()
}

/// Check if shape is effectively empty (very small surface area).
pub fn IsEmpty(shape: &Shape, tolerance: f64) -> bool {
    GetSurfaceArea(shape) <= tolerance
}

pub fn CreateUnitBox() -> Shape {
    CreateBox(Point3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0)
}

pub fn CreateBox(corner: Point3, x: f64, y: f64, z: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(corner, x, y, z)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

pub fn CreateUnitSphere() -> Shape {
    BRepPrimAPIMakeSphere::new(1.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

pub fn CreateSphere(center: Point3, radius: f64) -> Shape {
    BRepPrimAPIMakeSphere::at_center(center, radius)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

pub fn CreateCylinder(radius: f64, height: f64) -> Shape {
    BRepPrimAPIMakeCylinder::new(radius, height)
        .shape()
        .unwrap()
        .clone()
}

pub fn CreateCone(r1: f64, r2: f64, height: f64) -> Shape {
    BRepPrimAPIMakeCone::new(r1, r2, height)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

/// Convert shape to NURBS; null on failure.
pub fn ConvertToNurbs(shape: &Shape) -> Shape {
    match BRepBuilderAPINurbsConvert::with_shape(shape, false) {
        Ok(mut converter) if converter.is_done() => converter.shape().unwrap().clone(),
        _ => Shape::default(),
    }
}

/// Create rectangular polygon face.
pub fn CreatePolygonFace(p1: Point3, p2: Point3, p3: Point3, p4: Point3) -> Shape {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for point in [p1, p2, p3, p4] {
        polygon.add_point(point).unwrap();
    }
    polygon.close().unwrap();
    if !polygon.is_done() {
        return Shape::default();
    }
    let wire = polygon.wire().unwrap();
    BRepBuilderAPIMakeFace::from_wire(&wire, false)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn transformed(shape: &Shape, trsf: Transform3) -> Shape {
    BRepBuilderAPITransform::with_shape(shape, trsf, false, false)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

/// Rotate shape around axis.
pub fn RotateShape(shape: &Shape, axis: Axis1, angle: f64) -> Shape {
    let mut trsf = Transform3::default();
    trsf.set_rotation(axis, angle).unwrap();
    transformed(shape, trsf)
}

/// Translate shape by vector.
pub fn TranslateShape(shape: &Shape, vector: Vector3) -> Shape {
    let mut trsf = Transform3::default();
    trsf.set_translation(vector);
    transformed(shape, trsf)
}

pub fn CreateVertex(point: Point3) -> Vertex {
    BRepBuilderAPIMakeVertex::new(point)
        .unwrap()
        .vertex()
        .unwrap()
}

pub fn CreateEdge(p1: Point3, p2: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(p1, p2)
        .unwrap()
        .edge()
        .unwrap()
}

/// Create prism by extruding shape along vector; null on failure.
pub fn CreatePrism(profile: &Shape, direction: Vector3) -> Shape {
    match BRepPrimAPIMakePrism::new(profile, direction, false, true) {
        Ok(mut maker) if maker.is_done() => maker.shape().unwrap().clone(),
        _ => Shape::default(),
    }
}

pub fn CreateRectangularPrism(corner: Point3, x: f64, y: f64, z: f64) -> Shape {
    let wire = CreateRectangularProfile(corner, x, y);
    let face = CreateFaceFromWire(&wire);
    CreatePrism(&face, Vector3::new(0.0, 0.0, z))
}

/// Create revolution of a profile around an axis.
pub fn CreateRevolution(profile: &Shape, axis: Axis1, angle: f64) -> Shape {
    let mut maker = BRepPrimAPIMakeRevol::new(profile, axis, angle, false).unwrap();
    assert!(maker.is_done(), "Revolution operation failed");
    maker.shape().unwrap().clone()
}

/// Get a face from a shape by index (for nexplode simulation).
pub fn GetFaceByIndex(shape: &Shape, index: usize) -> Face {
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .nth(index - 1)
        .map_or_else(Face::default, Face::unchecked)
}

fn rotate_about(shape: &Shape, origin: Point3, direction: Direction3, angle_deg: f64) -> Shape {
    RotateShape(shape, Axis1::new(origin, direction), angle_deg * PI / 180.0)
}

pub fn RotateZ(shape: &Shape, angle_deg: f64) -> Shape {
    rotate_about(
        shape,
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
        angle_deg,
    )
}

pub fn RotateY(shape: &Shape, angle_deg: f64) -> Shape {
    rotate_about(
        shape,
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
        angle_deg,
    )
}

pub fn RotateX(shape: &Shape, angle_deg: f64) -> Shape {
    rotate_about(
        shape,
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
        angle_deg,
    )
}

/// Common test rotation: Z(-90deg) then Y(-45deg).
pub fn RotateStandard(shape: &Shape) -> Shape {
    RotateY(&RotateZ(shape, -90.0), -45.0)
}

pub fn Translate(shape: &Shape, dx: f64, dy: f64, dz: f64) -> Shape {
    TranslateShape(shape, Vector3::new(dx, dy, dz))
}

/// Y-axis 90-degree rotation about (0, 0, 1).
pub fn RotateY90(shape: &Shape) -> Shape {
    rotate_about(
        shape,
        Point3::new(0.0, 0.0, 1.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
        90.0,
    )
}

/// Unit sphere and box pair.
pub fn CreateSphereAndBox() -> (Shape, Shape) {
    (CreateUnitSphere(), CreateUnitBox())
}

/// Two identical unit boxes at origin.
pub fn CreateIdenticalBoxes() -> (Shape, Shape) {
    let origin = Point3::new(0.0, 0.0, 0.0);
    (
        CreateBox(origin, 1.0, 1.0, 1.0),
        CreateBox(origin, 1.0, 1.0, 1.0),
    )
}

/// NURBS box and regular box pair with the default corners and sizes.
pub fn CreateNurbsAndRegularBox() -> (Shape, Shape) {
    CreateNurbsAndRegularBoxWith(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        (1.0, 1.0, 1.0),
        (1.0, 0.5, 1.0),
    )
}

/// NURBS box and regular box pair.
pub fn CreateNurbsAndRegularBoxWith(
    nurbs_corner: Point3,
    regular_corner: Point3,
    nurbs: (f64, f64, f64),
    regular: (f64, f64, f64),
) -> (Shape, Shape) {
    let nurbs_box = ConvertToNurbs(&CreateBox(nurbs_corner, nurbs.0, nurbs.1, nurbs.2));
    (
        nurbs_box,
        CreateBox(regular_corner, regular.0, regular.1, regular.2),
    )
}

/// Create a blend (fillet) on specified edge of a shape (like the DRAW `blend` command):
/// the `edge_index`-th edge of the explorer, the DRAW defaults; a null shape when the
/// fillet fails, the shape itself when there is no such edge.
pub fn CreateBlend(shape: &Shape, edge_index: usize, radius: f64) -> Shape {
    use rustycad_tkfillet::brep_fillet_api::BRepFilletAPIMakeFillet;
    use rustycad_tkfillet::chfi3d::FilletShape;
    use rustycad_tkmath::geom_abs::Shape as Continuity;
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut current = 1;
    let mut target: Option<Edge> = None;
    while explorer.more() {
        if current == edge_index {
            target = Some(rustycad_tkbrep::topods::edge(explorer.current()).unwrap());
            break;
        }
        current += 1;
        explorer.next().unwrap();
    }
    let Some(target) = target else {
        return shape.clone();
    };
    let mut maker = BRepFilletAPIMakeFillet::new(shape, FilletShape::Rational).unwrap();
    maker.set_params(1e-2, 1.0e-4, 1.0e-5, 1.0e-4, 1.0e-5, 1.0e-3);
    maker.set_continuity(Continuity::C1, 1.0e-2);
    maker.add_with_radius(radius, &target).unwrap();
    maker.build(&rustycad_tkernel::message::ProgressRange::default()).unwrap();
    if !maker.is_done() {
        return Shape::default();
    }
    maker.shape().unwrap().clone()
}

/// Create a cylinder on a specified plane (like TCL pcylinder command).
pub fn CreateCylinderOnPlane(plane: &Plane3, radius: f64, height: f64) -> Shape {
    let axes = plane.position().ax2().unwrap();
    BRepPrimAPIMakeCylinder::with_axes(axes, radius, height)
        .shape()
        .unwrap()
        .clone()
}
