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

/// Profile command types (equivalent to TCL profile commands).
#[derive(Clone, Copy, PartialEq)]
pub enum ProfileCmd {
    O,
    P,
    F,
    X,
    Y,
    L,
    XX,
    YY,
    T,
    TT,
    R,
    RR,
    D,
    C,
    W,
    WW,
}

/// Profile command structure.
pub struct ProfileOperation {
    pub cmd: ProfileCmd,
    pub params: Vec<f64>,
}

impl ProfileOperation {
    pub fn new(cmd: ProfileCmd, params: &[f64]) -> Self {
        Self {
            cmd,
            params: params.to_vec(),
        }
    }
}

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

/// Create wire from list of points (closed polygon unless `close` is false).
pub fn CreatePolygonWire(points: &[Point3], close: bool) -> Wire {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for point in points {
        polygon.add_point(*point).unwrap();
    }
    if close {
        polygon.close().unwrap();
    }
    polygon.wire().unwrap()
}

/// The point of the plane at (u, v) (`ElSLib::Value(u, v, gp_Pln)`).
fn on_plane(point: Point2, plane: &Plane3) -> Point3 {
    plane_value(point.x(), point.y(), plane.position())
}

/// The state of a profile under construction.
struct ProfileState {
    wire: BRepBuilderAPIMakeWire,
    current: Point2,
    direction: Vector2,
    first_set: bool,
    first: Point2,
    plane: Plane3,
}

impl ProfileState {
    fn start_if_unset(&mut self) {
        if !self.first_set {
            self.current = Point2::new(0.0, 0.0);
            self.first = self.current;
            self.first_set = true;
        }
    }

    fn line_to(&mut self, target: Point2) {
        let edge = BRepBuilderAPIMakeEdge::from_points(
            on_plane(self.current, &self.plane),
            on_plane(target, &self.plane),
        )
        .unwrap()
        .edge()
        .unwrap();
        self.wire.add_edge(&edge).unwrap();
        self.current = target;
    }

    fn origin(&mut self, params: &[f64]) {
        let origin = Point3::new(params[0], params[1], params[2]);
        let frame = Axis3::new(
            origin,
            self.plane.axis().direction(),
            self.plane.x_axis().direction(),
        )
        .unwrap();
        self.plane = Plane3::from_position(frame);
        if !self.first_set {
            self.current = Point2::new(0.0, 0.0);
            self.first = self.current;
            self.first_set = true;
        }
    }

    fn orient(&mut self, params: &[f64]) {
        let normal = Direction3::new(params[0], params[1], params[2]).unwrap();
        let x_direction = Direction3::new(params[3], params[4], params[5]).unwrap();
        let frame = Axis3::new(self.plane.location(), normal, x_direction).unwrap();
        self.plane = Plane3::from_position(frame);
    }

    fn arc(&mut self, params: &[f64]) {
        let radius = params[0].abs();
        let angle_deg = params[1];
        let angle = angle_deg * PI / 180.0;
        if angle_deg.abs() >= 360.0 {
            let center = if self.first_set {
                self.current
            } else {
                Point2::new(0.0, 0.0)
            };
            let circle = Circle3::new(
                Axis2::new(on_plane(center, &self.plane), self.plane.axis().direction()).unwrap(),
                radius,
            )
            .unwrap();
            let edge = BRepBuilderAPIMakeEdge::from_circ(circle)
                .unwrap()
                .edge()
                .unwrap();
            self.wire.add_edge(&edge).unwrap();
            self.current = Point2::new(center.x() + radius, center.y());
            self.direction = Vector2::new(0.0, 1.0);
            self.first_set = true;
            return;
        }
        if !self.first_set {
            self.current = Point2::new(0.0, 0.0);
            self.direction = Vector2::new(1.0, 0.0);
            self.first = self.current;
            self.first_set = true;
        }
        let normal = Vector2::new(-self.direction.y(), self.direction.x());
        let center = self.current.translated(normal * radius);
        let start = (self.current.y() - center.y()).atan2(self.current.x() - center.x());
        let end = start + angle;
        let end_point = Point2::new(
            center.x() + radius * end.cos(),
            center.y() + radius * end.sin(),
        );
        let circle = Circle3::new(
            Axis2::new(on_plane(center, &self.plane), self.plane.axis().direction()).unwrap(),
            radius,
        )
        .unwrap();
        let edge = BRepBuilderAPIMakeEdge::from_circ_points(
            circle,
            on_plane(self.current, &self.plane),
            on_plane(end_point, &self.plane),
        )
        .unwrap()
        .edge()
        .unwrap();
        self.wire.add_edge(&edge).unwrap();
        self.current = end_point;
        self.direction = Vector2::new((end + PI / 2.0).cos(), (end + PI / 2.0).sin());
    }
}

/// Create profile using typed commands (equivalent to TCL profile).
pub fn CreateProfile(plane: &Plane3, operations: &[ProfileOperation]) -> Shape {
    let mut state = ProfileState {
        wire: BRepBuilderAPIMakeWire::new(),
        current: Point2::new(0.0, 0.0),
        direction: Vector2::new(1.0, 0.0),
        first_set: false,
        first: Point2::new(0.0, 0.0),
        plane: *plane,
    };
    for op in operations {
        match op.cmd {
            ProfileCmd::O if op.params.len() >= 3 => state.origin(&op.params),
            ProfileCmd::P if op.params.len() >= 6 => state.orient(&op.params),
            ProfileCmd::F if op.params.len() >= 2 => {
                state.current = Point2::new(op.params[0], op.params[1]);
                state.first = state.current;
                state.first_set = true;
            }
            ProfileCmd::X if !op.params.is_empty() => {
                state.start_if_unset();
                let target = Point2::new(state.current.x() + op.params[0], state.current.y());
                state.line_to(target);
                state.direction = Vector2::new(if op.params[0] > 0.0 { 1.0 } else { -1.0 }, 0.0);
            }
            ProfileCmd::Y if !op.params.is_empty() => {
                state.start_if_unset();
                let target = Point2::new(state.current.x(), state.current.y() + op.params[0]);
                state.line_to(target);
                state.direction = Vector2::new(0.0, if op.params[0] > 0.0 { 1.0 } else { -1.0 });
            }
            ProfileCmd::C if op.params.len() >= 2 => state.arc(&op.params),
            ProfileCmd::D if op.params.len() >= 2 => {
                state.direction = Vector2::new(op.params[0], op.params[1]);
                state.direction.normalize().unwrap();
            }
            _ => {}
        }
    }
    if state.first_set
        && !state
            .current
            .is_equal(state.first, rustycad_tkernel::precision::confusion())
    {
        let first = state.first;
        state.line_to(first);
    }
    assert!(state.wire.is_done(), "Profile wire creation failed");
    let wire = state.wire.wire().unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&wire, false).unwrap();
    assert!(face.is_done(), "Profile face creation failed");
    face.face().unwrap().into_shape()
}

/// Create profile from operations list on the XY plane.
pub fn CreateProfileFromOperations(operations: &[ProfileOperation]) -> Shape {
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    CreateProfile(&plane, operations)
}

/// Create rectangular profile from corner and dimensions.
pub fn CreateRectangularProfile(corner: Point3, x: f64, y: f64) -> Wire {
    let points = [
        corner,
        Point3::new(corner.x() + x, corner.y(), corner.z()),
        Point3::new(corner.x() + x, corner.y() + y, corner.z()),
        Point3::new(corner.x(), corner.y() + y, corner.z()),
    ];
    CreatePolygonWire(&points, true)
}

/// Create face from wire; null on failure.
pub fn CreateFaceFromWire(wire: &Wire) -> Shape {
    match BRepBuilderAPIMakeFace::from_wire(wire, false) {
        Ok(mut maker) if maker.is_done() => maker.shape().unwrap().clone(),
        _ => Shape::default(),
    }
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

/// Create a cylinder on a specified plane (like TCL pcylinder command).
pub fn CreateCylinderOnPlane(plane: &Plane3, radius: f64, height: f64) -> Shape {
    let axes = plane.position().ax2().unwrap();
    BRepPrimAPIMakeCylinder::with_axes(axes, radius, height)
        .shape()
        .unwrap()
        .clone()
}

fn expect_near(actual: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{what}: {actual} differs from {expected} by more than {tolerance}"
    );
}

/// Validate result properties against expected values (`ValidateResult` of both fixtures).
pub fn ValidateResult(result: &Shape, surface_area: f64, volume: f64, empty: bool) {
    let tolerance = DefaultTolerance();
    if empty {
        assert!(IsEmpty(result, tolerance), "Result should be empty");
        return;
    }
    assert!(!result.is_null(), "Result shape should not be null");
    if surface_area >= 0.0 {
        expect_near(
            GetSurfaceArea(result),
            surface_area,
            5000.0,
            "Surface area mismatch",
        );
    }
    if volume >= 0.0 {
        expect_near(GetVolume(result), volume, tolerance, "Volume mismatch");
    }
}

/// `BRepAlgoAPI_TestBase::PerformCut`.
pub fn PerformCut(object: &Shape, tool: &Shape) -> Shape {
    let cutter = BRepAlgoAPICut::from_shapes(object, tool).unwrap();
    assert!(cutter.is_done(), "BRepAlgoAPI_Cut operation failed");
    cutter.shape().clone()
}

/// `BRepAlgoAPI_TestBase::PerformFuse`.
pub fn PerformFuse(first: &Shape, second: &Shape) -> Shape {
    let fuser = BRepAlgoAPIFuse::from_shapes(first, second).unwrap();
    assert!(fuser.is_done(), "BRepAlgoAPI_Fuse operation failed");
    fuser.shape().clone()
}

/// `BRepAlgoAPI_TestBase::PerformCommon`.
pub fn PerformCommon(first: &Shape, second: &Shape) -> Shape {
    let common = BRepAlgoAPICommon::from_shapes(first, second).unwrap();
    assert!(common.is_done(), "BRepAlgoAPI_Common operation failed");
    common.shape().clone()
}

/// `BOPAlgo_TestBase::PerformDirectBOP`.
pub fn PerformDirectBOP(first: &Shape, second: &Shape, operation: Operation) -> Shape {
    let mut bop = BOPAlgoBOP::new();
    bop.add_argument(first);
    bop.add_tool(second);
    bop.set_operation(operation);
    bop.options_mut().set_fuzzy_value(DefaultFuzzyValue());
    bop.options_mut().set_run_parallel(false);
    bop.set_non_destructive(false);
    bop.perform();
    assert!(!bop.options().has_errors(), "Direct BOP operation failed");
    bop.shape().clone()
}

/// `BOPAlgo_TestBase::PreparePaveFiller`.
pub fn PreparePaveFiller(first: &Shape, second: &Shape) -> BOPAlgoPaveFiller {
    let arguments: List<Shape> = [first.clone(), second.clone()].into_iter().collect();
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(&arguments);
    filler.options_mut().set_fuzzy_value(DefaultFuzzyValue());
    filler.options_mut().set_run_parallel(false);
    filler.set_non_destructive(false);
    filler.perform();
    assert!(
        !filler.options().has_errors(),
        "PaveFiller preparation failed"
    );
    filler
}

/// `BOPAlgo_TestBase::PerformBOPWithPaveFiller`.
pub fn PerformBOPWithPaveFiller(filler: &mut BOPAlgoPaveFiller, operation: Operation) -> Shape {
    let arguments: Vec<Shape> = filler.arguments().iter().cloned().collect();
    assert_eq!(arguments.len(), 2, "Wrong number of arguments");
    let mut bop = BOPAlgoBOP::new();
    bop.add_argument(&arguments[0]);
    bop.add_tool(&arguments[1]);
    bop.set_operation(operation);
    bop.options_mut().set_run_parallel(false);
    bop.perform_with_filler(filler);
    assert!(
        !bop.options().has_errors(),
        "BOP operation with PaveFiller failed"
    );
    bop.shape().clone()
}

/// `BOPAlgo_TestBase::PerformTwoStepBOP`.
pub fn PerformTwoStepBOP(first: &Shape, second: &Shape, operation: Operation) -> Shape {
    let mut filler = PreparePaveFiller(first, second);
    PerformBOPWithPaveFiller(&mut filler, operation)
}
