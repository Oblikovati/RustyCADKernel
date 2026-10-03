// Rust translation of ModelingAlgorithms/TKShHealing/ShapeFix/ShapeFix_WireVertex.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeFix_WireVertex`: apply the decisions from `ShapeAnalysis_WireVertex` to a wire.
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWireVertex};
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point3, Xyz};

/// Fixes disconnected wire endpoints using the recorded analyzer decisions.
///
/// ```
/// use rustycad_tkshhealing::shape_fix::ShapeFixWireVertex;
/// let fixer = ShapeFixWireVertex::new();
/// assert!(!fixer.analyzer().is_done());
/// ```
#[derive(Clone, Default)]
pub struct ShapeFixWireVertex {
    analyzer: ShapeAnalysisWireVertex,
}

impl ShapeFixWireVertex {
    /// Construct an uninitialized fixer (`ShapeFix_WireVertex()`);
    /// `ShapeFixWireVertex::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a wire, analyze it with `precision`, and retain its editable edge data;
    /// `fixer.init(&wire, 1e-7)?`.
    pub fn init(&mut self, wire: &Wire, precision: f64) -> Result<(), Failure> {
        self.analyzer.init(wire, precision)?;
        self.analyzer.set_precision(precision);
        self.analyzer.analyze()
    }

    /// Load shared editable wire data, then analyze it; `fixer.init_wire_data(&data, 1e-7)?`.
    pub fn init_wire_data(
        &mut self,
        wire: &Handle<ShapeExtendWireData>,
        precision: f64,
    ) -> Result<(), Failure> {
        self.analyzer.load_wire_data(wire);
        self.analyzer.set_precision(precision);
        self.analyzer.analyze()
    }

    /// Reuse an existing analysis by native handle-sharing semantics;
    /// `fixer.init_from_analyzer(&analysis)`.
    pub fn init_from_analyzer(&mut self, analyzer: &ShapeAnalysisWireVertex) {
        self.analyzer = analyzer.clone();
    }

    /// Borrow the analyzer state; `let analysis = fixer.analyzer()`.
    pub fn analyzer(&self) -> &ShapeAnalysisWireVertex {
        &self.analyzer
    }

    /// Share the mutable edge data; `let data = fixer.wire_data()`.
    pub fn wire_data(&self) -> Handle<ShapeExtendWireData> {
        self.analyzer.wire_data()
    }

    /// Build a wire from the current edge data; `let wire = fixer.wire()?`.
    pub fn wire(&self) -> Result<Wire, Failure> {
        self.analyzer.wire_data().wire()
    }

    /// Join only SameCoords and Close junctions, preserving native parameter quirks;
    /// `let changed = fixer.fix_same()?`.
    pub fn fix_same(&mut self) -> Result<i32, Failure> {
        if !self.analyzer.is_done() {
            return Ok(0);
        }
        let mut fixed = 0;
        for index in 1..=self.analyzer.nb_edges() {
            fixed += self.fix_same_junction(index)?;
        }
        Ok(fixed)
    }

    /// Apply all positive analysis records and reconnect the cyclic edge sequence;
    /// `let changed = fixer.fix()?`.
    pub fn fix(&mut self) -> Result<i32, Failure> {
        if !self.analyzer.is_done() {
            return Ok(0);
        }
        let positive_count =
            count_positive_statuses(&self.analyzer, self.analyzer.nb_edges() as usize)?;
        if positive_count == 0 {
            return Ok(0);
        }
        let snapshot = FixSnapshot::capture(&self.analyzer, positive_count)?;
        self.apply_fix_snapshot(snapshot)
    }

    fn fix_same_junction(&mut self, index: i32) -> Result<i32, Failure> {
        let status = self.analyzer.status(index)?;
        if status != 1 && status != 2 {
            return Ok(0);
        }
        let pair = self.junction_edges(index)?;
        let (previous_vertex, following_vertex) = junction_vertices(&pair);
        if previous_vertex.is_equal(&following_vertex) {
            self.analyzer.set_same_vertex(index)?;
            return Ok(0);
        }
        if status == 2 {
            self.update_close_tolerances(&pair, &previous_vertex)?;
        }
        attach_shared_vertex(pair, previous_vertex)?;
        self.analyzer.set_same_vertex(index)?;
        Ok(1)
    }

    fn junction_edges(&self, index: i32) -> Result<(Edge, Edge), Failure> {
        let count = self.analyzer.nb_edges();
        let next = if index == count { 1 } else { index + 1 };
        let wire = self.wire_data();
        Ok((wire.edge(index)?, wire.edge(next)?))
    }

    fn update_close_tolerances(
        &self,
        pair: &(Edge, Edge),
        shared_vertex: &Vertex,
    ) -> Result<(), Failure> {
        let analyzer = ShapeAnalysisEdge::new();
        let (_, _, previous_parameter) = analyzer.curve_3d(&pair.0, true)?;
        let (_, following_parameter, _) = analyzer.curve_3d(&pair.1, true)?;
        let builder = BRepBuilder::default();
        let precision = self.analyzer.precision();
        builder.update_vertex_on_edge(shared_vertex, previous_parameter, &pair.0, precision)?;
        builder.update_vertex_on_edge(shared_vertex, following_parameter, &pair.1, precision)
    }

    fn apply_fix_snapshot(&mut self, snapshot: FixSnapshot) -> Result<i32, Failure> {
        detach_old_vertices(&snapshot.edges)?;
        self.attach_fixed_vertices(&snapshot)?;
        let data = self.wire_data();
        for (index, edge) in snapshot.edges.iter().enumerate() {
            data.set(edge, index as i32 + 1)?;
        }
        Ok(snapshot.positive_count)
    }

    fn attach_fixed_vertices(&mut self, snapshot: &FixSnapshot) -> Result<(), Failure> {
        let precision = self.analyzer.precision();
        for index in 0..snapshot.edges.len() {
            self.attach_junction(index, snapshot, precision)?;
        }
        Ok(())
    }

    fn attach_junction(
        &mut self,
        index: usize,
        snapshot: &FixSnapshot,
        precision: f64,
    ) -> Result<(), Failure> {
        let next = (index + 1) % snapshot.edges.len();
        let status = self.analyzer.status(index as i32 + 1)?;
        let vertex = self.prepared_vertex(index, status, snapshot, precision)?;
        reconnect_junction(
            &snapshot.edges[index],
            &snapshot.edges[next],
            vertex,
            status,
        )?;
        self.analyzer.set_same_vertex(index as i32 + 1)
    }

    fn prepared_vertex(
        &self,
        index: usize,
        status: i32,
        snapshot: &FixSnapshot,
        precision: f64,
    ) -> Result<Vertex, Failure> {
        let mut vertex = snapshot.previous_vertices[index].clone();
        let next = (index + 1) % snapshot.edges.len();
        if status > 2 {
            update_vertex_position(
                &vertex,
                self.analyzer.position(index as i32 + 1)?,
                precision,
            )?;
        }
        if status > 0 {
            update_junction_parameters(index, next, &vertex, snapshot, precision)?;
            vertex.set_orientation(Orientation::Forward);
        }
        Ok(vertex)
    }
}

/// Recorded topological and parameter inputs for the two native Fix passes.
struct FixSnapshot {
    edges: Vec<Edge>,
    previous_vertices: Vec<Vertex>,
    previous_parameters: Vec<f64>,
    following_parameters: Vec<f64>,
    positive_count: i32,
}

struct JunctionSnapshot {
    previous_vertices: Vec<Vertex>,
    previous_parameters: Vec<f64>,
    following_parameters: Vec<f64>,
}

impl FixSnapshot {
    fn capture(analyzer: &ShapeAnalysisWireVertex, positive_count: i32) -> Result<Self, Failure> {
        let count = analyzer.nb_edges();
        let data = analyzer.wire_data();
        let edges = (1..=count)
            .map(|index| data.edge(index))
            .collect::<Result<Vec<_>, _>>()?;
        Self::capture_vertices(analyzer, edges, positive_count)
    }

    fn capture_vertices(
        analyzer: &ShapeAnalysisWireVertex,
        edges: Vec<Edge>,
        positive_count: i32,
    ) -> Result<Self, Failure> {
        let junctions = capture_junction_records(analyzer, &edges)?;
        Ok(Self {
            edges,
            previous_vertices: junctions.previous_vertices,
            previous_parameters: junctions.previous_parameters,
            following_parameters: junctions.following_parameters,
            positive_count,
        })
    }
}

fn count_positive_statuses(
    analyzer: &ShapeAnalysisWireVertex,
    count: usize,
) -> Result<i32, Failure> {
    (1..=count).try_fold(0, |sum, index| {
        Ok(sum + i32::from(analyzer.status(index as i32)? > 0))
    })
}

fn capture_junction_records(
    analyzer: &ShapeAnalysisWireVertex,
    edges: &[Edge],
) -> Result<JunctionSnapshot, Failure> {
    let count = edges.len();
    let edge_analysis = ShapeAnalysisEdge::new();
    let mut previous_vertices = Vec::with_capacity(count);
    let mut previous_parameters = Vec::with_capacity(count);
    let mut following_parameters = vec![0.0; count];
    for index in 0..count {
        let status = analyzer.status(index as i32 + 1)?;
        let next = (index + 1) % count;
        previous_vertices.push(edge_analysis.last_vertex(&edges[index]));
        let parameters = junction_parameters(analyzer, &edge_analysis, edges, index, next, status)?;
        previous_parameters.push(parameters.0);
        following_parameters[next] = parameters.1;
    }
    Ok(JunctionSnapshot {
        previous_vertices,
        previous_parameters,
        following_parameters,
    })
}

fn junction_parameters(
    analyzer: &ShapeAnalysisWireVertex,
    edge_analysis: &ShapeAnalysisEdge,
    edges: &[Edge],
    index: usize,
    next: usize,
    status: i32,
) -> Result<(f64, f64), Failure> {
    let mut previous_parameter = analyzer.u_previous(index as i32 + 1)?;
    let mut following_parameter = analyzer.u_following(next as i32 + 1)?;
    if status < 4 {
        previous_parameter = edge_analysis.curve_3d(&edges[index], true)?.2;
    }
    if status < 3 || status == 4 {
        following_parameter = edge_analysis.curve_3d(&edges[next], true)?.1;
    }
    Ok((previous_parameter, following_parameter))
}

fn junction_vertices(pair: &(Edge, Edge)) -> (Vertex, Vertex) {
    let analyzer = ShapeAnalysisEdge::new();
    (
        analyzer.last_vertex(&pair.0),
        analyzer.first_vertex(&pair.1),
    )
}

fn attach_shared_vertex(pair: (Edge, Edge), mut vertex: Vertex) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    vertex.set_orientation(pair.1.orientation());
    builder.add(&mut pair.1.clone(), &vertex)?;
    vertex.set_orientation(pair.0.orientation());
    vertex.reverse();
    builder.add(&mut pair.0.clone(), &vertex)
}

fn detach_old_vertices(edges: &[Edge]) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    for edge in edges {
        let mut forward = edge.clone();
        forward.set_orientation(Orientation::Forward);
        let (first, last) = topexp::vertices(&forward, false);
        forward.set_free(true)?;
        builder.remove(&mut forward, &first)?;
        builder.remove(&mut forward, &last)?;
    }
    Ok(())
}

fn update_vertex_position(vertex: &Vertex, position: Xyz, precision: f64) -> Result<(), Failure> {
    BRepBuilder::default().update_vertex_point(vertex, Point3::from(position), precision)
}

fn update_junction_parameters(
    index: usize,
    next: usize,
    vertex: &Vertex,
    snapshot: &FixSnapshot,
    precision: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    builder.update_vertex_on_edge(
        vertex,
        snapshot.previous_parameters[index],
        &snapshot.edges[index],
        precision,
    )?;
    builder.update_vertex_on_edge(
        vertex,
        snapshot.following_parameters[next],
        &snapshot.edges[next],
        precision,
    )?;
    Ok(())
}

fn reconnect_junction(
    previous: &Edge,
    following: &Edge,
    mut vertex: Vertex,
    status: i32,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut next = following.clone();
    next.set_free(true)?;
    if status > 0 {
        vertex.set_orientation(Orientation::Forward);
    }
    builder.add(&mut next, &vertex)?;
    let mut previous = previous.clone();
    previous.set_free(true)?;
    vertex.set_orientation(Orientation::Reversed);
    builder.add(&mut previous, &vertex)
}
