use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Face, Wire},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, geom_adaptor::GeomAdaptorSurface};

type StoredPCurve = (
    rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    (f64, f64),
);

pub(super) fn split_wire(face: &Face, wire: &Wire) -> Result<Vec<Wire>, Failure> {
    let edges = collect_edges(wire)?;
    let mut state = SplitState::new(face, edges)?;
    state.build_components()
}

struct SplitState<'a> {
    face: &'a Face,
    edges: Vec<rustycad_tkbrep::topods::Edge>,
    used: Vec<bool>,
    analysis: ShapeAnalysisEdge,
}

impl<'a> SplitState<'a> {
    fn new(face: &'a Face, edges: Vec<rustycad_tkbrep::topods::Edge>) -> Result<Self, Failure> {
        let used = vec![false; edges.len()];
        Ok(Self {
            face,
            edges,
            used,
            analysis: ShapeAnalysisEdge::new(),
        })
    }

    fn build_components(&mut self) -> Result<Vec<Wire>, Failure> {
        let mut result = Vec::new();
        for index in 0..self.edges.len() {
            if self.used[index] {
                continue;
            }
            if let Some(wire) = self.collect_component(index)? {
                result.push(wire);
            }
        }
        Ok(result)
    }

    fn collect_component(&mut self, first: usize) -> Result<Option<Wire>, Failure> {
        self.used[first] = true;
        let first_edge = self.edges[first].clone();
        let start = self.analysis.first_vertex(&first_edge);
        let mut last = self.analysis.last_vertex(&first_edge);
        let mut component = make_wire(&first_edge)?;
        for _ in 1..self.edges.len() {
            let Some(next) = self.next_edge(&last) else {
                return Ok(Some(component));
            };
            let edge = self.edges[next].clone();
            BRepBuilder::default().add(&mut component, &edge.clone().into_shape())?;
            self.used[next] = true;
            last = self.analysis.last_vertex(&edge);
            if !last.is_same(&start) {
                continue;
            }
            if self.pcurves_close(&first_edge, &edge, &start, &last)? {
                return Ok(Some(component));
            }
            if self.pcurves_missing(&first_edge, &edge)? {
                return Ok(None);
            }
        }
        Ok(Some(component))
    }

    fn next_edge(&self, last: &rustycad_tkbrep::topods::Vertex) -> Option<usize> {
        self.edges.iter().enumerate().find_map(|(index, edge)| {
            (!self.used[index] && self.analysis.first_vertex(edge).is_same(last)).then_some(index)
        })
    }

    fn pcurves_close(
        &self,
        first: &rustycad_tkbrep::topods::Edge,
        last: &rustycad_tkbrep::topods::Edge,
        start: &rustycad_tkbrep::topods::Vertex,
        end: &rustycad_tkbrep::topods::Vertex,
    ) -> Result<bool, Failure> {
        let (first_curve, first_range) = pcurve(first, self.face)?;
        let (last_curve, last_range) = pcurve(last, self.face)?;
        let (Some(first_curve), Some(last_curve)) = (first_curve.get(), last_curve.get()) else {
            return Ok(false);
        };
        let first_parameter = oriented_start_parameter(first, first_range);
        let last_parameter = oriented_end_parameter(last, last_range);
        let first_point = first_curve.value(first_parameter)?;
        let last_point = last_curve.value(last_parameter)?;
        let tolerance = BRepTool::vertex_tolerance(start)?.max(BRepTool::vertex_tolerance(end)?);
        let resolution = surface_resolution(self.face, tolerance)?;
        Ok(first_point.square_distance(last_point) < resolution)
    }

    fn pcurves_missing(
        &self,
        first: &rustycad_tkbrep::topods::Edge,
        last: &rustycad_tkbrep::topods::Edge,
    ) -> Result<bool, Failure> {
        let (a, _) = pcurve(first, self.face)?;
        let (b, _) = pcurve(last, self.face)?;
        Ok(a.is_null() || b.is_null())
    }
}

fn collect_edges(wire: &Wire) -> Result<Vec<rustycad_tkbrep::topods::Edge>, Failure> {
    let data = ShapeExtendWireData::from_wire(wire, true, true)?;
    (1..=data.nb_edges())
        .map(|index| data.edge(index))
        .collect()
}

fn make_wire(edge: &rustycad_tkbrep::topods::Edge) -> Result<Wire, Failure> {
    let mut wire = Wire::default();
    BRepBuilder::default().make_wire(&mut wire);
    BRepBuilder::default().add(&mut wire, &edge.clone().into_shape())?;
    Ok(wire)
}

fn pcurve(edge: &rustycad_tkbrep::topods::Edge, face: &Face) -> Result<StoredPCurve, Failure> {
    let curve = BRepTool::curve_on_face(edge, face)?;
    if !curve.is_stored {
        return Ok((Default::default(), (curve.first, curve.last)));
    }
    Ok((curve.pcurve, (curve.first, curve.last)))
}

fn oriented_start_parameter(edge: &rustycad_tkbrep::topods::Edge, range: (f64, f64)) -> f64 {
    if edge.orientation() == rustycad_tkg3d::top_abs::Orientation::Reversed {
        range.1
    } else {
        range.0
    }
}

fn oriented_end_parameter(edge: &rustycad_tkbrep::topods::Edge, range: (f64, f64)) -> f64 {
    if edge.orientation() == rustycad_tkg3d::top_abs::Orientation::Reversed {
        range.0
    } else {
        range.1
    }
}

fn surface_resolution(face: &Face, tolerance: f64) -> Result<f64, Failure> {
    let (surface, _) = BRepTool::surface_with_location(face)?;
    let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    Ok(2.
        * adaptor
            .u_resolution(tolerance)?
            .max(adaptor.v_resolution(tolerance)?))
}
