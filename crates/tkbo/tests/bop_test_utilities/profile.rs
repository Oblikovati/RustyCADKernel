//! BOPTest_Utilities.pxx: the profile commands (the DRAW `profile` emulation) and the wire,
//! polygon and face builders they use.

#![allow(dead_code, non_snake_case)]

use super::*;

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
