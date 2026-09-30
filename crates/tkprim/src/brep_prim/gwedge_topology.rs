// Rust translation of BRepPrim_GWedge.cxx (Face, HasWire, Wire, Edge, Vertex) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction of the faces, wires, edges and vertices of `BRepPrim_GWedge`, each
//! built once and shared by the shapes that contain it.
use super::BRepPrimDirection;
use super::BRepPrimDirection::{XMax, XMin, YMax, YMin, ZMax, ZMin};
use super::gwedge::{BRepPrimGWedge, num_dir1, num_dir2, num_dir3};
use super::gwedge_geometry::{missing_edge, missing_vertex};
use rustycad_tkbrep::topods::{Edge, Face, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

/// The four directions bounding the face of axis group `group` (`i / 2`), as the
/// native `dd1`, `dd2`, `dd3`, `dd4`.
fn face_bounds(group: usize) -> [BRepPrimDirection; 4] {
    match group {
        0 => [ZMin, YMax, ZMax, YMin],
        1 => [XMin, ZMax, XMax, ZMin],
        _ => [YMin, XMax, YMax, XMin],
    }
}

/// The two directions bounding the edge of index `i` (`dd1`, `dd2` by `i / 4`).
fn edge_bounds(i: usize) -> [BRepPrimDirection; 2] {
    match i / 4 {
        0 => [ZMin, ZMax],
        1 => [XMin, XMax],
        _ => [YMin, YMax],
    }
}

impl BRepPrimGWedge {
    /// `Face(d1)`: the face in direction `d1` on its plane, with its wire, reversed on
    /// the Min side, with the p-curves of its edges; built once;
    /// `wedge.face(BRepPrimDirection::ZMax)?`.
    pub fn face(&mut self, d1: BRepPrimDirection) -> Result<Face, Failure> {
        let i = num_dir1(d1);
        if !self.faces_built[i] {
            self.build_face(d1, i)?;
        }
        Ok(self.my_faces[i].clone())
    }

    /// The body of `Face(d1)` for a face not built yet.
    fn build_face(&mut self, d1: BRepPrimDirection, i: usize) -> Result<(), Failure> {
        let plane = self.plane(d1)?;
        let mut face = Face::default();
        self.my_builder.make_face(&mut face, plane)?;
        if self.has_wire(d1)? {
            let wire = self.wire(d1)?;
            self.my_builder.add_face_wire(&mut face, &wire)?;
        }
        if i.is_multiple_of(2) {
            self.my_builder.reverse_face(&mut face);
        }
        let [dd1, dd2, dd3, dd4] = face_bounds(i / 2);
        let (dx, dy) = (plane.x_axis().direction(), plane.y_axis().direction());
        for d2 in [dd4, dd3, dd2, dd1] {
            if !self.has_edge(d1, d2)? {
                continue;
            }
            let line = self.line(d1, d2)?;
            let (u, v) = elslib::plane_parameters(plane.position(), line.location());
            let pcurve = Line2::new(
                Point2::new(u, v),
                Direction2::new(line.direction() * dx, line.direction() * dy)?,
            );
            let edge = &self.my_edges[num_dir2(d1, d2)?];
            self.my_builder.set_pcurve_line(edge, &face, pcurve)?;
        }
        self.my_builder.complete_face(&face)?;
        self.my_faces[i] = face;
        self.faces_built[i] = true;
        Ok(())
    }

    /// `HasWire(d1)`: the face in direction `d1` has a wire when it is not open and one
    /// of its four edges exists; `wedge.has_wire(BRepPrimDirection::YMax)?`.
    pub fn has_wire(&self, d1: BRepPrimDirection) -> Result<bool, Failure> {
        let i = num_dir1(d1);
        if self.my_infinite[i] {
            return Ok(false);
        }
        let [dd1, dd2, dd3, dd4] = face_bounds(i / 2);
        Ok(self.has_edge(d1, dd1)?
            || self.has_edge(d1, dd2)?
            || self.has_edge(d1, dd3)?
            || self.has_edge(d1, dd4)?)
    }

    /// `Wire(d1)`: the wire of the face in direction `d1`, its existing edges in the
    /// order dd4, dd3 (reversed), dd2, dd1; built once; `wedge.wire(BRepPrimDirection::XMin)?`.
    pub fn wire(&mut self, d1: BRepPrimDirection) -> Result<Wire, Failure> {
        let i = num_dir1(d1);
        if !self.wires_built[i] {
            let [dd1, dd2, dd3, dd4] = face_bounds(i / 2);
            let mut wire = Wire::default();
            self.my_builder.make_wire(&mut wire);
            for (d2, direct) in [(dd4, false), (dd3, false), (dd2, true), (dd1, true)] {
                if self.has_edge(d1, d2)? {
                    let edge = self.edge(d1, d2)?;
                    self.my_builder.add_wire_edge(&mut wire, &edge, direct)?;
                }
            }
            self.my_builder.complete_wire(&wire)?;
            self.my_wires[i] = wire;
            self.wires_built[i] = true;
        }
        Ok(self.my_wires[i].clone())
    }

    /// `Edge(d1, d2)`: the edge between the faces `d1` and `d2` on its line, with its
    /// existing vertices; the two edges of a YMax face of null extent are one edge;
    /// built once; Standard_DomainError when the edge does not exist;
    /// `wedge.edge(BRepPrimDirection::XMin, BRepPrimDirection::YMin)?`.
    pub fn edge(&mut self, d1: BRepPrimDirection, d2: BRepPrimDirection) -> Result<Edge, Failure> {
        if !self.has_edge(d1, d2)? {
            return Err(missing_edge("Edge", d1, d2));
        }
        let i = num_dir2(d1, d2)?;
        if !self.edges_built[i] {
            self.build_edge(d1, d2, i)?;
        }
        Ok(self.my_edges[i].clone())
    }

    /// The body of `Edge(d1, d2)` for an edge not built yet.
    fn build_edge(
        &mut self,
        d1: BRepPrimDirection,
        d2: BRepPrimDirection,
        i: usize,
    ) -> Result<(), Failure> {
        let [dd1, dd2] = edge_bounds(i);
        let line = self.line(d1, d2)?;
        let mut edge = Edge::default();
        self.my_builder.make_edge_line(&mut edge, line)?;
        for (d3, direct) in [(dd2, false), (dd1, true)] {
            if self.has_vertex(d1, d2, d3) {
                let vertex = self.vertex(d1, d2, d3)?;
                let parameter = elclib::line_parameter(line.position(), self.point(d1, d2, d3)?);
                self.my_builder
                    .add_edge_vertex(&mut edge, &vertex, parameter, direct)?;
            }
        }
        self.my_edges[i] = edge;
        self.share_flat_edges(i);
        self.my_builder.complete_edge(&self.my_edges[i]);
        self.edges_built[i] = true;
        Ok(())
    }

    /// The edges of a YMax face of null extent: 6 and 7 are one edge when Z2 is flat,
    /// 1 and 3 when X2 is flat.
    fn share_flat_edges(&mut self, i: usize) {
        let pairs: [(bool, usize, usize); 2] = [
            (self.z2max == self.z2min, 6, 7),
            (self.x2max == self.x2min, 1, 3),
        ];
        for (flat, a, b) in pairs {
            if !flat || (i != a && i != b) {
                continue;
            }
            let other = if i == a { b } else { a };
            self.my_edges[other] = self.my_edges[i].clone();
            self.edges_built[other] = true;
        }
    }

    /// `Vertex(d1, d2, d3)`: the vertex of the faces `d1`, `d2`, `d3`; the vertices of a
    /// YMax face of null extent are shared; built once; Standard_DomainError when the
    /// vertex does not exist; `wedge.vertex(XMin, YMin, ZMin)?`.
    pub fn vertex(
        &mut self,
        d1: BRepPrimDirection,
        d2: BRepPrimDirection,
        d3: BRepPrimDirection,
    ) -> Result<Vertex, Failure> {
        if !self.has_vertex(d1, d2, d3) {
            return Err(missing_vertex("Vertex", d1, d2, d3));
        }
        let i = num_dir3(d1, d2, d3)?;
        if !self.vertices_built[i] {
            let point = self.point(d1, d2, d3)?;
            let mut vertex = Vertex::default();
            self.my_builder.make_vertex(&mut vertex, point)?;
            self.my_vertices[i] = vertex;
            self.share_flat_vertices(i);
            self.vertices_built[i] = true;
        }
        Ok(self.my_vertices[i].clone())
    }

    /// The vertices of a YMax face of null extent: when Z2 is flat, 2 and 3 are one
    /// vertex and 6 and 7 another; when X2 is flat, 2 and 6 are one and 3 and 7
    /// another. The native body copies both pairs of a flat side from the built pair.
    fn share_flat_vertices(&mut self, i: usize) {
        if self.z2max == self.z2min {
            if i == 2 || i == 6 {
                self.copy_vertices([(2, 3), (6, 7)]);
            } else if i == 3 || i == 7 {
                self.copy_vertices([(3, 2), (7, 6)]);
            }
        }
        if self.x2max == self.x2min {
            if i == 2 || i == 3 {
                self.copy_vertices([(2, 6), (3, 7)]);
            } else if i == 6 || i == 7 {
                self.copy_vertices([(6, 2), (7, 3)]);
            }
        }
    }

    /// Copies each vertex `from` into `to` and marks `to` built.
    fn copy_vertices(&mut self, pairs: [(usize, usize); 2]) {
        for (from, to) in pairs {
            self.my_vertices[to] = self.my_vertices[from].clone();
            self.vertices_built[to] = true;
        }
    }
}
