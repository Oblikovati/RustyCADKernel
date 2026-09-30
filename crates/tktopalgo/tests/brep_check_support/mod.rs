//! Shapes for the BRepCheck behavior tests, built with BRep_Builder (the port has no
//! BRepBuilderAPI yet): planar faces bounded by vertex polygons (each edge a 3d line
//! with one pcurve per face, isometric to it), cubes and square faces with holes. The
//! Rust side of the corpus builder tools/tktopalgo_corpus_brepcheck_shapes.py.
#![allow(dead_code)]
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Shell, Solid, Vertex, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3, Point2, Point3, Vector3};
use std::collections::BTreeMap;

pub const TOL: f64 = 1e-7;

/// The frame of a planar face: origin, normal, X and Y directions.
#[derive(Clone, Copy)]
pub struct Frame {
    pub origin: Point3,
    pub normal: Vector3,
    pub x: Vector3,
    pub y: Vector3,
}

impl Frame {
    fn uv(&self, p: Point3) -> Point2 {
        let d = Vector3::between(self.origin, p);
        Point2::new(d.dot(self.x), d.dot(self.y))
    }
}

fn unit(v: Vector3) -> Vector3 {
    v.normalized().expect("non-null direction")
}

/// Faces on planes bounded by polygons of shared vertices and edges.
pub struct Planar {
    pub points: Vec<Point3>,
    pub vertices: Vec<Vertex>,
    pub edges: BTreeMap<(usize, usize), Edge>,
    pub faces: Vec<Face>,
    pub wires: Vec<Vec<Wire>>,
    pub surfaces: Vec<Handle<dyn GeomSurface>>,
}

impl Planar {
    pub fn new(points: &[(f64, f64, f64)]) -> Self {
        let points: Vec<Point3> = points
            .iter()
            .map(|&(x, y, z)| Point3::new(x, y, z))
            .collect();
        let vertices = points.iter().map(|&p| vertex(p, TOL)).collect();
        Self {
            points,
            vertices,
            edges: BTreeMap::new(),
            faces: Vec::new(),
            wires: Vec::new(),
            surfaces: Vec::new(),
        }
    }

    fn frame(&self, lp: &[usize], normal: Option<Vector3>) -> Frame {
        let origin = self.points[lp[0]];
        let x = unit(Vector3::between(origin, self.points[lp[1]]));
        let n =
            normal.unwrap_or_else(|| unit(x.crossed(Vector3::between(origin, self.points[lp[2]]))));
        let n = unit(n);
        Frame {
            origin,
            normal: n,
            x,
            y: n.crossed(x),
        }
    }

    /// The edge between points a and b, created once from the lower index.
    pub fn edge(&mut self, a: usize, b: usize) -> (usize, usize) {
        let key = (a.min(b), a.max(b));
        if !self.edges.contains_key(&key) {
            let (start, end) = (self.points[key.0], self.points[key.1]);
            let b_ = BRepBuilder::default();
            let mut e = Edge::default();
            b_.make_edge(&mut e).unwrap();
            let dir = Direction3::from_vector(Vector3::between(start, end)).unwrap();
            let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(start, dir)).cast();
            b_.update_edge_curve(&e, &line, TOL).unwrap();
            b_.add(&mut e, &self.vertices[key.0].oriented(Orientation::Forward))
                .unwrap();
            b_.add(
                &mut e,
                &self.vertices[key.1].oriented(Orientation::Reversed),
            )
            .unwrap();
            self.edges.insert(key, e);
        }
        key
    }

    /// A face bounded by `loops` (the first one outer), with pcurves shifted by
    /// `shift` in v for the edges of `shifted`.
    pub fn face(
        &mut self,
        loops: &[&[usize]],
        normal: Option<Vector3>,
        shifted: &[(usize, usize)],
        shift: f64,
    ) -> Face {
        let frame = self.frame(loops[0], normal);
        let axis = Axis3::new(
            frame.origin,
            Direction3::from_vector(frame.normal).unwrap(),
            Direction3::from_vector(frame.x).unwrap(),
        )
        .unwrap();
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(axis)).cast();
        let b = BRepBuilder::default();
        let mut face = Face::default();
        b.make_face_on_surface(&mut face, &surface, TOL).unwrap();
        let mut wires = Vec::new();
        for lp in loops {
            let mut wire = Wire::default();
            b.make_wire(&mut wire);
            for k in 0..lp.len() {
                let (a, c) = (lp[k], lp[(k + 1) % lp.len()]);
                let key = self.edge(a, c);
                let delta = if shifted.contains(&key) { shift } else { 0.0 };
                self.pcurve(key, &surface, &frame, delta);
                let o = if a < c {
                    Orientation::Forward
                } else {
                    Orientation::Reversed
                };
                b.add(&mut wire, &self.edges[&key].oriented(o)).unwrap();
            }
            b.add(&mut face, &wire).unwrap();
            wires.push(wire);
        }
        self.faces.push(face.clone());
        self.wires.push(wires);
        self.surfaces.push(surface);
        face
    }

    fn pcurve(
        &self,
        key: (usize, usize),
        surface: &Handle<dyn GeomSurface>,
        frame: &Frame,
        shift: f64,
    ) {
        let (a, c) = (frame.uv(self.points[key.0]), frame.uv(self.points[key.1]));
        let start = Point2::new(a.x(), a.y() + shift);
        let dir = Direction2::new(c.x() - a.x(), c.y() - a.y()).unwrap();
        let pc: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(start, dir)).cast();
        BRepBuilder::default()
            .update_edge_pcurve(
                &self.edges[&key],
                &pc,
                surface,
                &Default::default(),
                TOL,
                None,
            )
            .unwrap();
    }

    /// The ranges of every edge on all its representations.
    pub fn finish(&self) {
        for (key, e) in &self.edges {
            let length = self.points[key.0].distance(&self.points[key.1]);
            BRepBuilder::default().range(e, 0.0, length, false).unwrap();
        }
    }

    pub fn shell(&self, faces: &[Face]) -> Shell {
        let b = BRepBuilder::default();
        let mut shell = Shell::default();
        b.make_shell(&mut shell);
        for f in faces {
            b.add(&mut shell, f).unwrap();
        }
        shell
    }
}

pub fn vertex(p: Point3, tol: f64) -> Vertex {
    let mut v = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut v, p, tol)
        .unwrap();
    v
}

pub const CUBE_POINTS: [(f64, f64, f64); 8] = [
    (0.0, 0.0, 0.0),
    (1.0, 0.0, 0.0),
    (1.0, 1.0, 0.0),
    (0.0, 1.0, 0.0),
    (0.0, 0.0, 1.0),
    (1.0, 0.0, 1.0),
    (1.0, 1.0, 1.0),
    (0.0, 1.0, 1.0),
];
pub const CUBE_FACES: [[usize; 4]; 6] = [
    [0, 3, 2, 1],
    [4, 5, 6, 7],
    [0, 1, 5, 4],
    [1, 2, 6, 5],
    [2, 3, 7, 6],
    [3, 0, 4, 7],
];

/// The faces of the unit cube given by `faces` (indices into CUBE_FACES), finished.
pub fn cube(faces: &[usize]) -> Planar {
    let mut planar = Planar::new(&CUBE_POINTS);
    for &f in faces {
        planar.face(&[&CUBE_FACES[f]], None, &[], 0.0);
    }
    planar.finish();
    planar
}

/// A solid holding `shell`, the shell flagged closed.
pub fn solid_of(shell: &Shell) -> Solid {
    shell.set_closed(true).unwrap();
    let b = BRepBuilder::default();
    let mut solid = Solid::default();
    b.make_solid(&mut solid);
    b.add(&mut solid, shell).unwrap();
    solid
}

/// The square [0, size]^2 on z = 0 with square holes (x0, y0, side) wound clockwise;
/// the pcurves of `shifted` edges moved by `shift`.
pub fn square(
    size: f64,
    holes: &[(f64, f64, f64)],
    shifted: &[(usize, usize)],
    shift: f64,
) -> Planar {
    let mut points = vec![
        (0.0, 0.0, 0.0),
        (size, 0.0, 0.0),
        (size, size, 0.0),
        (0.0, size, 0.0),
    ];
    let mut loops: Vec<Vec<usize>> = vec![vec![0, 1, 2, 3]];
    for &(x0, y0, side) in holes {
        let base = points.len();
        points.extend([
            (x0, y0, 0.0),
            (x0, y0 + side, 0.0),
            (x0 + side, y0 + side, 0.0),
            (x0 + side, y0, 0.0),
        ]);
        loops.push((base..base + 4).collect());
    }
    let mut planar = Planar::new(&points);
    let refs: Vec<&[usize]> = loops.iter().map(Vec::as_slice).collect();
    planar.face(&refs, Some(Vector3::new(0.0, 0.0, 1.0)), shifted, shift);
    planar.finish();
    planar
}

/// A wire of `edges` with their orientations.
pub fn wire_of(edges: &[(&Edge, Orientation)]) -> Wire {
    let b = BRepBuilder::default();
    let mut wire = Wire::default();
    b.make_wire(&mut wire);
    for (e, o) in edges {
        b.add(&mut wire, &e.oriented(*o)).unwrap();
    }
    wire
}

/// `shape` as a plain Shape.
pub fn shape<S: AsRef<Shape>>(s: &S) -> Shape {
    s.as_ref().clone()
}
