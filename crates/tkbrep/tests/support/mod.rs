//! Named fakes for the TKTopAlgo and TKPrim builders that the upstream TKBRep suites
//! call (BRepBuilderAPI_MakeVertex, BRepBuilderAPI_MakeEdge, BRepPrimAPI_MakeBox).
//!
//! BRep, TKTopAlgo and TKPrim are not ported, so these fakes build the same topology
//! through TopoDS alone: a vertex TShape derived from `TopoDS_TVertex` and an edge
//! TShape derived from `TopoDS_TEdge`, both without geometry. The TopoDS suites only
//! check topology (types, counts, sharing, orientations), which the fakes reproduce;
//! they are replaced by the real builders when those toolkits are ported.
#![allow(dead_code)]
use rustycad_tkbrep::topods::{
    Builder, Edge, Face, Shell, Solid, TEdge, TFace, TShape, TShapeBase, TVertex, Vertex, Wire,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

/// A vertex TShape standing for `BRep_TVertex`: it keeps the point it was made at.
pub struct FakeTVertex {
    parent: TVertex,
    point: Point3,
}
impl_transient!(FakeTVertex, "FakeTVertex", TVertex, parent, interfaces: [dyn TShape]);

impl TShape for FakeTVertex {
    fn tshape_base(&self) -> &TShapeBase {
        self.parent.tshape_base()
    }
    fn empty_copy(&self) -> Handle<dyn TShape> {
        Handle::new(FakeTVertex::at(self.point)).into_cast()
    }
}

impl FakeTVertex {
    fn at(point: Point3) -> Self {
        Self {
            parent: TVertex::new(),
            point,
        }
    }

    /// The point the vertex was made at.
    pub fn point(&self) -> Point3 {
        self.point
    }
}

/// An edge TShape standing for `BRep_TEdge` (a line segment without its curve).
pub struct FakeTEdge {
    parent: TEdge,
}
impl_transient!(FakeTEdge, "FakeTEdge", TEdge, parent, interfaces: [dyn TShape]);

impl TShape for FakeTEdge {
    fn tshape_base(&self) -> &TShapeBase {
        self.parent.tshape_base()
    }
    fn empty_copy(&self) -> Handle<dyn TShape> {
        Handle::new(FakeTEdge {
            parent: TEdge::new(),
        })
        .into_cast()
    }
}

/// `BRepBuilderAPI_MakeVertex(P)`: a new FORWARD vertex at the identity location.
pub fn make_vertex(point: Point3) -> Vertex {
    let mut vertex = Vertex::default();
    Builder.make_shape(
        &mut vertex,
        &Handle::new(FakeTVertex::at(point)).into_cast(),
    );
    vertex
}

/// `BRepBuilderAPI_MakeEdge(P1, P2)`: a new edge holding two new vertices, the first
/// FORWARD and the second REVERSED, as BRepLib_MakeEdge adds them.
pub fn make_edge(first: Point3, last: Point3) -> Edge {
    edge_between(&make_vertex(first), &make_vertex(last))
}

/// An edge between two existing vertices (FORWARD, then REVERSED).
pub fn edge_between(first: &Vertex, last: &Vertex) -> Edge {
    let mut edge = Edge::default();
    Builder.make_shape(
        &mut edge,
        &Handle::new(FakeTEdge {
            parent: TEdge::new(),
        })
        .into_cast(),
    );
    Builder
        .add(&mut edge, &first.oriented(Orientation::Forward))
        .unwrap();
    Builder
        .add(&mut edge, &last.oriented(Orientation::Reversed))
        .unwrap();
    edge
}

/// `BRepPrimAPI_MakeBox(dx, dy, dz).Shape()`: a solid with one shell of six faces,
/// each bounded by one wire of four edges; the twelve edges and the eight vertices
/// are shared as in the native box (each edge by two faces).
pub fn make_box(dx: f64, dy: f64, dz: f64) -> Solid {
    let corners: Vec<Vertex> = (0..8)
        .map(|i| {
            let (x, y, z) = (i & 1, (i >> 1) & 1, (i >> 2) & 1);
            make_vertex(Point3::new(x as f64 * dx, y as f64 * dy, z as f64 * dz))
        })
        .collect();
    // Edges along X (0-3), Y (4-7) and Z (8-11), each from its low to its high corner.
    let pairs = [
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        (0, 2),
        (1, 3),
        (4, 6),
        (5, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    let edges: Vec<Edge> = pairs
        .iter()
        .map(|&(a, b)| edge_between(&corners[a], &corners[b]))
        .collect();
    // Each face lists its four edges around the boundary with their orientation.
    let faces = [
        [(4, true), (6, true), (8, false), (10, true)],
        [(5, true), (7, true), (9, false), (11, true)],
        [(0, true), (2, true), (8, false), (9, true)],
        [(1, true), (3, true), (10, false), (11, true)],
        [(0, true), (1, true), (4, false), (5, true)],
        [(2, true), (3, true), (6, false), (7, true)],
    ];
    let mut shell = Shell::default();
    Builder.make_shell(&mut shell);
    for face_edges in faces {
        let face = face_of(&edges, &face_edges);
        Builder.add(&mut shell, &face).unwrap();
    }
    let mut solid = Solid::default();
    Builder.make_solid(&mut solid);
    Builder.add(&mut solid, &shell).unwrap();
    solid
}

fn face_of(edges: &[Edge], face_edges: &[(usize, bool); 4]) -> Face {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    for &(index, forward) in face_edges {
        let orientation = if forward {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        Builder
            .add(&mut wire, &edges[index].oriented(orientation))
            .unwrap();
    }
    let mut face = Face::default();
    Builder.make_shape(&mut face, &Handle::new(TFace::new()).into_cast());
    Builder.add(&mut face, &wire).unwrap();
    face
}
