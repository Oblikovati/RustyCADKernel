// Rust translation of the private BRepLib_MakeWire::Add(E, IsCheckGeometryProximity)
// of BRepLib_MakeWire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The private `Add(E, IsCheckGeometryProximity)` of `BRepLib_MakeWire`: the
//! connection test, the copy of an edge onto the wire vertices, the orientation
//! decision and the update of the free ends.
// PMN  19/03/1998  For the Problem of performance TopExp::Vertices are not used on wire
// PMN  10/09/1998  In case if the wire is previously closed (or degenerated)
//                  TopExp::Vertices is used to reduce the ambiguity.
use super::BRepLibMakeWire;
use crate::brep_lib::errors::BRepLibWireError;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

/// The connection of a new edge: whether it touches the wire (by a shared vertex or,
/// with the proximity check, by a close one), whether it must be copied onto the wire
/// vertices, and the orientation votes.
#[derive(Default)]
struct Connection {
    connected: bool,
    copyedge: bool,
    forward: bool,
    reverse: bool,
}

/// The children of `shape` (vertices of an edge), with the native iterator defaults.
pub(super) fn children(shape: &Shape) -> Result<Vec<Vertex>, Failure> {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut iterator = ShapeIterator::new(shape, true, true);
    while let Ok(child) = iterator.value() {
        vertices.push(topods::vertex(child)?);
        iterator.next();
    }
    Ok(vertices)
}

impl BRepLibMakeWire {
    /// `Add(E, IsCheckGeometryProximity)`: the first edge creates the wire; a later one
    /// must connect (else `BRepLib_DisconnectedWire` and not done).
    pub(super) fn add_edge_checked(
        &mut self,
        edge: &Edge,
        check_proximity: bool,
    ) -> Result<(), Failure> {
        let init: bool = if self.my_edge.is_null() {
            self.first_edge(edge)?;
            true
        } else {
            let closed: bool = self.my_shape.closed()?; // If it is closed no control
            if !self.connected_edge(edge, check_proximity)? {
                return Ok(());
            }
            closed
        };
        BRepBuilder::default().add(&mut self.base.my_shape, &self.my_edge)?;
        self.my_shape.set_closed(false)?;
        if init {
            (self.vf, self.vl) = topexp::wire_vertices(&Wire::unchecked(self.my_shape.clone()));
        } else {
            self.update_ends()?;
        }
        // Test myShape is closed
        if !self.vf.is_null() && !self.vl.is_null() && self.vf.is_same(&self.vl) {
            self.my_shape.set_closed(true)?;
        }
        self.my_error = BRepLibWireError::WireDone;
        self.set_done();
        Ok(())
    }

    /// The first edge: creates the wire and records the edge vertices.
    fn first_edge(&mut self, edge: &Edge) -> Result<(), Failure> {
        let mut wire = Wire::default();
        BRepBuilder::default().make_wire(&mut wire);
        self.my_shape = wire.into_shape();
        self.my_edge = edge.clone();
        for vertex in children(&self.my_edge)? {
            self.my_vertices.add(vertex.into_shape());
        }
        Ok(())
    }

    /// A later edge: the connection test, then `myEdge` (the FORWARD edge, or its copy
    /// onto close wire vertices) oriented by the votes; false after recording
    /// `BRepLib_DisconnectedWire`.
    fn connected_edge(&mut self, edge: &Edge, check_proximity: bool) -> Result<bool, Failure> {
        let ee: Edge = Edge::unchecked(edge.oriented(Orientation::Forward));
        // test the vertices of the edge
        if self.my_error != BRepLibWireError::NonManifoldWire
            && (self.vf.is_null() || self.vl.is_null())
        {
            self.my_error = BRepLibWireError::NonManifoldWire;
        }
        let connection: Connection = self.connection(&ee, check_proximity)?;
        if !connection.connected {
            self.my_error = BRepLibWireError::DisconnectedWire;
            self.set_not_done();
            return Ok(false);
        }
        if connection.copyedge {
            self.copy_edge(&ee)?;
        } else {
            self.my_edge = ee.clone();
            for vertex in children(&ee)? {
                self.my_vertices.add(vertex.into_shape());
            }
        }
        // If there is an ambiguity (in 3d) preserve the orientation given at input:
        // no vote (closed wire, internal vertex ...) or both votes (closed or
        // degenerated edge).
        let (forward, reverse) = (connection.forward, connection.reverse);
        if (forward == reverse && edge.orientation() == Orientation::Reversed)
            || (reverse && !forward)
        {
            self.my_edge.reverse();
        }
        Ok(true)
    }

    /// The connection test of every vertex of `ee`.
    fn connection(&mut self, ee: &Edge, check_proximity: bool) -> Result<Connection, Failure> {
        let mut connection = Connection::default();
        for ve in children(ee)? {
            if self.my_vertices.contains(&ve) {
                connection.connected = true;
                self.my_vertex = ve.clone();
                self.vote(&ve, &ve, &mut connection);
                continue;
            }
            if !check_proximity {
                continue;
            }
            // search if there is a similar vertex in the edge
            if let Some(vw) = self.close_wire_vertex(&ve)? {
                connection.copyedge = true;
                self.vote(&ve, &vw, &mut connection);
            }
            if connection.copyedge {
                connection.connected = true;
            }
        }
        Ok(connection)
    }

    /// The first wire vertex whose distance to `ve` is below the tolerance of either.
    fn close_wire_vertex(&self, ve: &Vertex) -> Result<Option<Vertex>, Failure> {
        let pe: Point3 = BRepTool::pnt(ve)?;
        for index in 1..=self.my_vertices.extent() {
            let vw: Vertex = topods::vertex(self.my_vertices.find_key(index)?)?;
            let distance: f64 = pe.distance(&BRepTool::pnt(&vw)?);
            if distance < BRepTool::vertex_tolerance(ve)?
                || distance < BRepTool::vertex_tolerance(&vw)?
            {
                return Ok(Some(vw));
            }
        }
        Ok(None)
    }

    /// The orientation vote of the edge vertex `ve` matched to the wire vertex `vw`:
    /// at a free end, reversed when the edge starts there (end VF) or ends there
    /// (end VL); anything else is non-manifold. Skipped once non-manifold.
    fn vote(&mut self, ve: &Vertex, vw: &Vertex, connection: &mut Connection) {
        if self.my_error == BRepLibWireError::NonManifoldWire {
            return;
        }
        if self.vf.is_same(&self.vl) {
            // Orientation indetermined (in 3d) : Preserve the initial
            if !self.vf.is_same(vw) {
                self.my_error = BRepLibWireError::NonManifoldWire;
            }
        } else if self.vf.is_same(vw) {
            let starts: bool = ve.orientation() == Orientation::Forward;
            (connection.reverse, connection.forward) =
                (connection.reverse || starts, connection.forward || !starts);
        } else if self.vl.is_same(vw) {
            let ends: bool = ve.orientation() == Orientation::Reversed;
            (connection.reverse, connection.forward) =
                (connection.reverse || ends, connection.forward || !ends);
        } else {
            self.my_error = BRepLibWireError::NonManifoldWire;
        }
    }
}
