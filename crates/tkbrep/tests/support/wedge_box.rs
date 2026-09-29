//! The box path of `BRepPrim_GWedge` (TKPrim), replayed with the real BRep types for
//! the `BRepPrimAPI_MakeBox` fake: the same axes, point, line and plane arithmetic, the
//! same edge, wire, face and shell order, the same orientations (every edge holds its
//! end vertex REVERSED before its start vertex FORWARD; every wire holds its edges
//! REVERSED, REVERSED, FORWARD, FORWARD; the Min faces are reversed), the p-curves and
//! the `BRepPrim_Builder` completion steps. The upstream suites address box entities by
//! index, so the graph built from this box must match the native one slot for slot.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{Edge, Face, Shell, Solid, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Direction2, Direction3, Plane3, Point2, Point3, Vector3,
};

// `BRepPrim_Direction`: XMin, XMax, YMin, YMax, ZMin, ZMax.
const X_MIN: usize = 0;
const X_MAX: usize = 1;
const Y_MIN: usize = 2;
const Y_MAX: usize = 3;
const Z_MIN: usize = 4;
const Z_MAX: usize = 5;

// `val` and `tab`: the vertex and edge numbering of the wedge.
const VAL: [usize; 6] = [0, 4, 0, 2, 0, 1];
const TAB: [[i32; 6]; 6] = [
    [-1, -1, 0, 1, 8, 9],
    [-1, -1, 2, 3, 10, 11],
    [0, 2, -1, -1, 4, 5],
    [1, 3, -1, -1, 6, 7],
    [8, 10, 4, 6, -1, -1],
    [9, 11, 5, 7, -1, -1],
];

/// `BRepPrim_GWedge` for a box: `XMin = YMin = ZMin = 0` and the given extents, with
/// the lazily built vertices and edges.
struct Wedge {
    axes: Axis2,
    max: [f64; 3],
    vertices: [Option<Vertex>; 8],
    edges: [Option<Edge>; 12],
}

fn edge_index(d1: usize, d2: usize) -> usize {
    TAB[d1][d2] as usize
}

impl Wedge {
    // `P.Translate(X * VX); P.Translate(Y * VY); P.Translate(Z * VZ)` from the origin.
    fn at(&self, x: f64, y: f64, z: f64) -> Point3 {
        let mut point = self.axes.location();
        point.translate(Vector3::from(self.axes.x_direction()) * x);
        point.translate(Vector3::from(self.axes.y_direction()) * y);
        point.translate(Vector3::from(self.axes.direction()) * z);
        point
    }

    // `Point(d1, d2, d3)` by vertex number.
    fn point(&self, index: usize) -> Point3 {
        let x = if index & 4 != 0 { self.max[0] } else { 0.0 };
        let y = if index & 2 != 0 { self.max[1] } else { 0.0 };
        let z = if index & 1 != 0 { self.max[2] } else { 0.0 };
        self.at(x, y, z)
    }

    // `Vertex(d1, d2, d3)`.
    fn vertex(&mut self, d1: usize, d2: usize, d3: usize) -> Vertex {
        let index = VAL[d1] + VAL[d2] + VAL[d3];
        if self.vertices[index].is_none() {
            let mut vertex = Vertex::default();
            let point = self.point(index);
            BRepBuilder::default()
                .make_vertex_at(&mut vertex, point, precision::confusion())
                .unwrap();
            self.vertices[index] = Some(vertex);
        }
        self.vertices[index].clone().unwrap()
    }

    // `Line(d1, d2)`: Z edges 0-3, X edges 4-7, Y edges 8-11.
    fn line(&self, d1: usize, d2: usize) -> Axis1 {
        let index = edge_index(d1, d2);
        let direction = match index / 4 {
            0 => self.axes.direction(),
            1 => self.axes.x_direction(),
            _ => self.axes.y_direction(),
        };
        let (x, y, z) = match index {
            1 | 6 => (0.0, self.max[1], 0.0),
            2 | 10 => (self.max[0], 0.0, 0.0),
            3 => (self.max[0], self.max[1], 0.0),
            5 | 9 => (0.0, 0.0, self.max[2]),
            7 => (0.0, self.max[1], self.max[2]),
            11 => (self.max[0], 0.0, self.max[2]),
            _ => (0.0, 0.0, 0.0),
        };
        Axis1::new(self.at(x, y, z), direction)
    }

    // `Edge(d1, d2)`: the line edge with its end vertex (REVERSED) then its start vertex
    // (FORWARD), each placed by `UpdateVertex` at its line parameter.
    fn edge(&mut self, d1: usize, d2: usize) -> Edge {
        let index = edge_index(d1, d2);
        if let Some(edge) = &self.edges[index] {
            return edge.clone();
        }
        let (dd1, dd2) = match index / 4 {
            0 => (Z_MIN, Z_MAX),
            1 => (X_MIN, X_MAX),
            _ => (Y_MIN, Y_MAX),
        };
        let axis = self.line(d1, d2);
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).into_cast();
        let mut edge = Edge::default();
        let builder = BRepBuilder::default();
        builder
            .make_edge_with_curve(&mut edge, &curve, precision::confusion())
            .unwrap();
        for (d3, direct) in [(dd2, false), (dd1, true)] {
            let vertex = self.vertex(d1, d2, d3);
            let parameter = elclib::line_parameter(axis, BRepTool::pnt(&vertex).unwrap());
            add_edge_vertex(&mut edge, &vertex, parameter, direct);
        }
        self.edges[index] = Some(edge.clone());
        edge
    }

    // The four sides of a face in wire order `dd4, dd3, dd2, dd1`.
    fn sides(d1: usize) -> [usize; 4] {
        match d1 / 2 {
            0 => [Y_MIN, Z_MAX, Y_MAX, Z_MIN],
            1 => [Z_MIN, X_MAX, Z_MAX, X_MIN],
            _ => [X_MIN, Y_MAX, X_MAX, Y_MIN],
        }
    }

    // `Wire(d1)`: the first two edges reversed, the last two forward; then closed.
    fn wire(&mut self, d1: usize) -> Wire {
        let builder = BRepBuilder::default();
        let mut wire = Wire::default();
        builder.make_wire(&mut wire);
        for (slot, d2) in Self::sides(d1).into_iter().enumerate() {
            let orientation = if slot < 2 {
                Orientation::Reversed
            } else {
                Orientation::Forward
            };
            builder
                .add(&mut wire, &self.edge(d1, d2).oriented(orientation))
                .unwrap();
        }
        let closed = BRepTool::is_closed(&wire).unwrap();
        wire.set_closed(closed).unwrap();
        wire
    }

    // `Plane(d1)`: `gp_Pln(P, D)` on the side's corner and axis.
    fn plane(&self, d1: usize) -> Plane3 {
        let direction = match d1 / 2 {
            0 => self.axes.x_direction(),
            1 => self.axes.y_direction(),
            _ => self.axes.direction(),
        };
        let (x, y, z) = match d1 {
            X_MAX => (self.max[0], 0.0, 0.0),
            Y_MAX => (0.0, self.max[1], 0.0),
            Z_MAX => (0.0, 0.0, self.max[2]),
            _ => (0.0, 0.0, 0.0),
        };
        Plane3::new(self.at(x, y, z), direction).unwrap()
    }

    // `Face(d1)`: the planar face with its wire, reversed for a Min side, then one
    // p-curve per edge (in the order dd4, dd3, dd2, dd1) and `CompleteFace`.
    fn face(&mut self, d1: usize) -> Face {
        let plane = self.plane(d1);
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).into_cast();
        let builder = BRepBuilder::default();
        let mut face = Face::default();
        builder
            .make_face_on_surface(&mut face, &surface, precision::confusion())
            .unwrap();
        let wire = self.wire(d1);
        builder.add(&mut face, &wire).unwrap();
        if d1.is_multiple_of(2) {
            face.reverse();
        }
        for d2 in Self::sides(d1) {
            let pcurve = pcurve_of(&plane, self.line(d1, d2));
            let edge = self.edge(d1, d2);
            builder
                .update_edge_pcurve_on_face(&edge, &pcurve, &face, precision::confusion())
                .unwrap();
        }
        brep_tools::update_face(&face).unwrap();
        face
    }
}

// `BRepPrim_Builder::AddEdgeVertex(E, V, P, direct)`.
fn add_edge_vertex(edge: &mut Edge, vertex: &Vertex, parameter: f64, direct: bool) {
    let orientation = if direct {
        Orientation::Forward
    } else {
        Orientation::Reversed
    };
    let mut oriented = vertex.clone();
    oriented.set_orientation(orientation);
    let builder = BRepBuilder::default();
    builder.add(edge, &oriented).unwrap();
    builder
        .update_vertex_on_edge(&oriented, parameter, edge, precision::confusion())
        .unwrap();
}

// The p-curve of a line on a plane: `ElSLib::Parameters` of its origin and the
// components of its direction along the plane's X and Y axes.
fn pcurve_of(plane: &Plane3, line: Axis1) -> Handle<dyn Geom2dCurve> {
    let (u, v) = elslib::plane_parameters(plane.position(), line.location());
    let du = line.direction().dot(plane.x_axis().direction());
    let dv = line.direction().dot(plane.y_axis().direction());
    let axis = Axis2d::new(Point2::new(u, v), Direction2::new(du, dv).unwrap());
    Handle::new(Geom2dLine::new(axis)).into_cast()
}

/// `pmin(P, dx, dy, dz)`: the low corner of a box given by a corner and signed extents.
fn low_corner(origin: Point3, dx: f64, dy: f64, dz: f64) -> Point3 {
    let mut point = origin;
    if dx < 0.0 {
        point.set_x(point.x() + dx);
    }
    if dy < 0.0 {
        point.set_y(point.y() + dy);
    }
    if dz < 0.0 {
        point.set_z(point.z() + dz);
    }
    point
}

/// `BRepPrimAPI_MakeBox(P, dx, dy, dz).Solid()`: the wedge shell (faces XMin, XMax,
/// YMin, YMax, ZMin, ZMax) closed and completed, in a new solid.
pub fn make_wedge_box(origin: Point3, dx: f64, dy: f64, dz: f64) -> Solid {
    let axes =
        Axis2::with_x_direction(low_corner(origin, dx, dy, dz), Direction3::Z, Direction3::X)
            .unwrap();
    let mut wedge = Wedge {
        axes,
        max: [dx.abs(), dy.abs(), dz.abs()],
        vertices: Default::default(),
        edges: Default::default(),
    };
    let builder = BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    shell.set_closed(true).unwrap();
    for d1 in [X_MIN, X_MAX, Y_MIN, Y_MAX, Z_MIN, Z_MAX] {
        let face = wedge.face(d1);
        builder.add(&mut shell, &face).unwrap();
    }
    // `Shell()` and `CompleteShell` both store the computed closure.
    let closed = BRepTool::is_closed(&shell).unwrap();
    shell.set_closed(closed).unwrap();
    brep_tools::update_shell(&shell).unwrap();
    let mut solid = Solid::default();
    builder.make_solid(&mut solid);
    builder.add(&mut solid, &shell).unwrap();
    solid
}
