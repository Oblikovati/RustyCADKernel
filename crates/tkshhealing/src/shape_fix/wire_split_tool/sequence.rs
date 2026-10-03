//! Preserve sequence output ordering and endpoint substitutions in SplitEdge.
use super::ShapeFixSplitTool;
use crate::{
    shape_analysis::ShapeAnalysisEdge, shape_build::ShapeBuildReShape,
    shape_extend::ShapeExtendWireData,
};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::{self, ReShape},
    topods::{Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::Point3;
struct EndpointSnapshot {
    vertex: Vertex,
    point: Point3,
    tolerance: f64,
}
impl EndpointSnapshot {
    fn read(vertex: Vertex) -> Result<Self, Failure> {
        Ok(Self {
            point: BRepTool::pnt(&vertex)?,
            tolerance: BRepTool::vertex_tolerance(&vertex)?,
            vertex,
        })
    }
    fn replace(&self, vertex: &Vertex, context: &Handle<ShapeBuildReShape>) -> Result<(), Failure> {
        let tolerance = self.tolerance + self.point.distance(&BRepTool::pnt(vertex)?);
        if BRepTool::vertex_tolerance(vertex)? < tolerance {
            BRepBuilder::default().update_vertex_tolerance(vertex, tolerance)?;
        }
        let replacement = if self.vertex.orientation() == vertex.orientation() {
            vertex.clone().into_shape()
        } else {
            vertex.reversed()
        };
        reshape(context)?.replace(&self.vertex, &replacement)
    }
}
struct SequenceSplit<'a> {
    tool: &'a ShapeFixSplitTool,
    edge: &'a Edge,
    face: &'a Face,
    bounds: (f64, f64),
    parameters: (f64, f64),
    vertices: (&'a Vertex, &'a Vertex),
    tolerances: (f64, f64),
    context: &'a Handle<ShapeBuildReShape>,
    endpoints: (EndpointSnapshot, EndpointSnapshot),
}
impl ShapeFixSplitTool {
    /// Split around a span and record substitutions; `tool.split_edge_sequence(&edge,a,&v1,b,&v2,&face,&mut pieces,&mut index,&context,1e-7,1e-9)?`.
    #[allow(clippy::too_many_arguments)] // OCCT's sequence-producing SplitEdge overload.
    pub fn split_edge_sequence(
        &self,
        edge: &Edge,
        first: f64,
        vertex1: &Vertex,
        last: f64,
        vertex2: &Vertex,
        face: &Face,
        pieces: &mut Sequence<Shape>,
        index: &mut i32,
        context: &Handle<ShapeBuildReShape>,
        tolerance_3d: f64,
        tolerance_2d: f64,
    ) -> Result<bool, Failure> {
        if (last - first).abs() < tolerance_2d {
            return Ok(false);
        }
        *index = 0;
        pieces.clear(None);
        let split = SequenceSplit::new(
            self,
            edge,
            face,
            ((first, last), (vertex1, vertex2)),
            (tolerance_3d, tolerance_2d),
            context,
        )?;
        split.perform(pieces, index)
    }
}
impl<'a> SequenceSplit<'a> {
    fn new(
        tool: &'a ShapeFixSplitTool,
        edge: &'a Edge,
        face: &'a Face,
        span: ((f64, f64), (&'a Vertex, &'a Vertex)),
        tolerances: (f64, f64),
        context: &'a Handle<ShapeBuildReShape>,
    ) -> Result<Self, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let (_, a, b) = analysis.pcurve_on_face(edge, face, true)?;
        let endpoints = (
            EndpointSnapshot::read(analysis.first_vertex(edge))?,
            EndpointSnapshot::read(analysis.last_vertex(edge))?,
        );
        let mut split = Self {
            tool,
            edge,
            face,
            bounds: (a, b),
            parameters: span.0,
            vertices: span.1,
            tolerances,
            context,
            endpoints,
        };
        let forward = (b - a) * (span.0.1 - span.0.0) > 0.;
        if !forward {
            split.parameters = (span.0.1, span.0.0);
            split.vertices = (span.1.1, span.1.0);
        }
        Ok(split)
    }
    fn perform(&self, pieces: &mut Sequence<Shape>, index: &mut i32) -> Result<bool, Failure> {
        let Some(start) = at_endpoint(self.bounds.0 - self.parameters.0, self.tolerances.1) else {
            return Ok(false);
        };
        let Some(end) = at_endpoint(self.bounds.1 - self.parameters.1, self.tolerances.1) else {
            return Ok(false);
        };
        if !self.build(start, end, pieces, index)? {
            return Ok(false);
        }
        record_pieces(self.edge, pieces, self.context)?;
        Ok(true)
    }

    fn split(&self, edge: &Edge, which: bool) -> Result<Option<(Edge, Edge)>, Failure> {
        let (mut first, mut second) = (Edge::default(), Edge::default());
        let (parameter, vertex) = if which {
            (self.parameters.1, self.vertices.1)
        } else {
            (self.parameters.0, self.vertices.0)
        };
        let done = self.tool.split_edge(
            edge,
            parameter,
            vertex,
            self.face,
            &mut first,
            &mut second,
            self.tolerances.0,
            self.tolerances.1,
        )?;
        Ok(done.then_some((first, second)))
    }
    fn build(
        &self,
        start: bool,
        end: bool,
        pieces: &mut Sequence<Shape>,
        index: &mut i32,
    ) -> Result<bool, Failure> {
        match (start, end) {
            (true, true) => {
                self.endpoints.0.replace(self.vertices.0, self.context)?;
                self.endpoints.1.replace(self.vertices.1, self.context)?;
                pieces.append(self.edge.clone().into_shape());
                *index = 1;
            }
            (true, false) | (false, true) => {
                return self.split_once(start, pieces, index);
            }
            (false, false) => {
                return self.split_twice(pieces, index);
            }
        }
        Ok(true)
    }
    fn split_once(
        &self,
        start: bool,
        pieces: &mut Sequence<Shape>,
        index: &mut i32,
    ) -> Result<bool, Failure> {
        let Some((first, second)) = self.split(self.edge, start)? else {
            return Ok(false);
        };
        if start {
            self.endpoints.0.replace(self.vertices.0, self.context)?;
        } else {
            self.endpoints.1.replace(self.vertices.1, self.context)?;
        }
        pieces.append(first.into_shape());
        pieces.append(second.into_shape());
        *index = if start { 1 } else { 2 };
        Ok(true)
    }
    fn split_twice(&self, pieces: &mut Sequence<Shape>, index: &mut i32) -> Result<bool, Failure> {
        let Some((first, second)) = self.split(self.edge, false)? else {
            return Ok(false);
        };
        let Some((middle, last)) = self.split(&second, true)? else {
            return Ok(false);
        };
        for edge in [first, middle, last] {
            pieces.append(edge.into_shape());
        }
        *index = 2;
        Ok(true)
    }
}
fn at_endpoint(delta: f64, tolerance: f64) -> Option<bool> {
    if delta.abs() <= tolerance {
        return Some(true);
    }
    if delta.abs() > tolerance {
        return Some(false);
    }
    None
}
fn reshape(context: &Handle<ShapeBuildReShape>) -> Result<&ShapeBuildReShape, Failure> {
    context.get().ok_or_else(|| {
        Failure::new("ShapeFix_SplitTool::SplitEdge: null context; expected ShapeBuild_ReShape")
    })
}
fn record_pieces(
    edge: &Edge,
    pieces: &Sequence<Shape>,
    context: &Handle<ShapeBuildReShape>,
) -> Result<(), Failure> {
    let wire = ShapeExtendWireData::new();
    for piece in pieces.iter() {
        wire.add_shape(piece, 0)?;
    }
    let replacement = wire.wire()?;
    reshape(context)?.replace(edge, &replacement)?;
    for piece in pieces.iter() {
        brep_tools::update_edge(&rustycad_tkbrep::topods::edge(piece)?);
    }
    Ok(())
}
