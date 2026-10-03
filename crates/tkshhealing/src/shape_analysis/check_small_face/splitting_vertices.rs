use super::spot::ShapeAnalysisCheckSmallFace;
use crate::shape_analysis::curve::ShapeAnalysisCurve;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{Explorer, vertices};
use rustycad_tkbrep::topods::{self, Builder, Compound, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

/// Native map from each splitting vertex to affected edges.
pub type SplittingVertexEdges = DataMap<Shape, List<Shape>, ShapeMapHasher>;
/// Native map from each splitting vertex to edge parameters in matching list order.
pub type SplittingVertexParameters = DataMap<Shape, List<f64>, ShapeMapHasher>;

const PROJECT_TOLERANCE_SCALE: f64 = 10.0;
const END_PARAMETER_TOLERANCE: f64 = 1.0e-6;

impl ShapeAnalysisCheckSmallFace {
    /// Finds vertices projected into nonincident edges and records edge parameters;
    /// `analyzer.check_splitting_vertices(&face, &mut edges, &mut params, &mut vertices)?`.
    pub fn check_splitting_vertices(
        &mut self,
        face: &Face,
        edge_map: &mut SplittingVertexEdges,
        parameter_map: &mut SplittingVertexParameters,
        all_vertices: &mut Compound,
    ) -> Result<i32, Failure> {
        let vertex_shapes = face_vertices(face)?;
        if vertex_shapes.is_empty() {
            return Ok(0);
        }
        let precision = self.split_vertex_precision()?;
        let candidates = split_candidates(&vertex_shapes, precision)?;
        let found = scan_candidates(face, candidates, edge_map, parameter_map, all_vertices)?;
        self.record_split_status(found);
        Ok(found)
    }

    fn split_vertex_precision(&self) -> Result<Option<f64>, Failure> {
        let Some(tolerance) = self.tolerance else {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "ShapeAnalysis_CheckSmallFace::CheckSplittingVertices requires SetTolerance before reading native myPrecision",
            ));
        };
        Ok((tolerance >= 0.0).then_some(tolerance))
    }

    fn record_split_status(&mut self, found: i32) {
        if found != 0 {
            self.split_vertex_status = ShapeExtend::encode_status(ShapeExtendStatus::Done);
        }
    }
}

fn scan_candidates(
    face: &Face,
    candidates: Vec<SplitCandidate>,
    edge_map: &mut SplittingVertexEdges,
    parameter_map: &mut SplittingVertexParameters,
    all_vertices: &mut Compound,
) -> Result<i32, Failure> {
    let mut found = 0;
    for candidate in candidates {
        let splits = find_vertex_splits(face, &candidate)?;
        if splits.is_empty() {
            continue;
        }
        record_vertex_split(candidate, splits, edge_map, parameter_map, all_vertices)?;
        found += 1;
    }
    Ok(found)
}

struct SplitCandidate {
    shape: Shape,
    point: Point3,
    tolerance: f64,
}

struct EdgeSplit {
    edge: Shape,
    parameter: f64,
}

fn face_vertices(face: &Face) -> Result<Vec<Shape>, Failure> {
    let face_shape = face.clone().into_shape();
    let mut vertices = Explorer::new(&face_shape, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    let mut result = Vec::new();
    while vertices.more() {
        result.push(vertices.current().clone());
        vertices.next()?;
    }
    Ok(result)
}

fn split_candidates(
    shapes: &[Shape],
    precision: Option<f64>,
) -> Result<Vec<SplitCandidate>, Failure> {
    shapes
        .iter()
        .map(|shape| split_candidate(shape, precision))
        .collect()
}

fn split_candidate(shape: &Shape, precision: Option<f64>) -> Result<SplitCandidate, Failure> {
    let vertex = topods::vertex(shape)?;
    let tolerance = candidate_tolerance(&vertex, precision)?;
    Ok(SplitCandidate {
        shape: shape.clone(),
        point: BRepTool::pnt(&vertex)?,
        tolerance,
    })
}

fn candidate_tolerance(
    vertex: &rustycad_tkbrep::topods::Vertex,
    precision: Option<f64>,
) -> Result<f64, Failure> {
    if let Some(precision) = precision {
        return Ok(precision);
    }
    BRepTool::vertex_tolerance(vertex)
}

fn find_vertex_splits(face: &Face, candidate: &SplitCandidate) -> Result<Vec<EdgeSplit>, Failure> {
    let face_shape = face.clone().into_shape();
    let mut edges = Explorer::new(&face_shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    let mut result = Vec::new();
    while edges.more() {
        let edge_shape = edges.current().clone();
        if let Some(split) = split_on_edge(&edge_shape, candidate)? {
            result.push(split);
        }
        edges.next()?;
    }
    Ok(result)
}

fn split_on_edge(
    edge_shape: &Shape,
    candidate: &SplitCandidate,
) -> Result<Option<EdgeSplit>, Failure> {
    if is_incident_endpoint(edge_shape, &candidate.shape)? {
        return Ok(None);
    }
    let edge = topods::edge(edge_shape)?;
    let (curve, first, last) = BRepTool::curve(&edge)?;
    if curve.get().is_none() {
        return Ok(None);
    }
    project_candidate(edge_shape, curve, (first, last), candidate)
}

fn is_incident_endpoint(edge: &Shape, vertex: &Shape) -> Result<bool, Failure> {
    let edge = topods::edge(edge)?;
    let (first, last) = vertices(&edge, false);
    Ok(vertex.is_same(&first.into_shape()) || vertex.is_same(&last.into_shape()))
}

fn project_candidate(
    edge: &Shape,
    curve: rustycad_tkernel::standard::Handle<dyn GeomCurve>,
    range: (f64, f64),
    candidate: &SplitCandidate,
) -> Result<Option<EdgeSplit>, Failure> {
    let (_, parameter, found) = project_split_parameter(&curve, candidate, range)?;
    if !found || is_near_curve_end(parameter, range) {
        return Ok(None);
    }
    Ok(Some(EdgeSplit {
        edge: edge.clone(),
        parameter,
    }))
}

fn project_split_parameter(
    curve: &rustycad_tkernel::standard::Handle<dyn GeomCurve>,
    candidate: &SplitCandidate,
    range: (f64, f64),
) -> Result<(f64, f64, bool), Failure> {
    let (distance, _, parameter) = ShapeAnalysisCurve.project(
        curve,
        candidate.point,
        candidate.tolerance * PROJECT_TOLERANCE_SCALE,
        range,
        false,
    )?;
    let found = distance != 0.0
        && distance.partial_cmp(&candidate.tolerance) == Some(std::cmp::Ordering::Less);
    Ok((distance, parameter, found))
}

fn is_near_curve_end(parameter: f64, range: (f64, f64)) -> bool {
    parameter >= range.1
        || parameter <= range.0
        || (parameter - range.0).abs() < END_PARAMETER_TOLERANCE
        || (parameter - range.1).abs() < END_PARAMETER_TOLERANCE
}

fn record_vertex_split(
    candidate: SplitCandidate,
    splits: Vec<EdgeSplit>,
    edge_map: &mut SplittingVertexEdges,
    parameter_map: &mut SplittingVertexParameters,
    all_vertices: &mut Compound,
) -> Result<(), Failure> {
    let (edges, parameters) = split_lists(splits);
    Builder.add(all_vertices, &candidate.shape)?;
    bind_split_data(candidate.shape, edges, parameters, edge_map, parameter_map)?;
    Ok(())
}

fn split_lists(splits: Vec<EdgeSplit>) -> (List<Shape>, List<f64>) {
    let mut edges = List::default();
    let mut parameters = List::default();
    for split in splits {
        edges.append(split.edge);
        parameters.append(split.parameter);
    }
    (edges, parameters)
}

fn bind_split_data(
    vertex: Shape,
    edges: List<Shape>,
    parameters: List<f64>,
    edge_map: &mut SplittingVertexEdges,
    parameter_map: &mut SplittingVertexParameters,
) -> Result<(), Failure> {
    // OCCT ignores Bind's false result when TopExp_Explorer repeats a split vertex.
    let _ = edge_map.bind(vertex.clone(), edges);
    let _ = parameter_map.bind(vertex, parameters);
    Ok(())
}
