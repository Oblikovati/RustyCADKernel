use super::spot::ShapeAnalysisCheckSmallFace;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{Explorer, vertices};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Native map from a first pin edge to its matching second edge.
pub type PinFaceEdgeMap = DataMap<Shape, Shape, ShapeMapHasher>;

impl ShapeAnalysisCheckSmallFace {
    /// Searches face edges for native pin-pair candidates and maps matching edges;
    /// `analyzer.check_pin_face(&face, &mut edge_pairs, -1.0)?`.
    pub fn check_pin_face(
        &mut self,
        face: &Face,
        edge_pairs: &mut PinFaceEdgeMap,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        require_wire(face)?;
        // Native reorders a local ShapeExtend_WireData copy, then never reads that copy.
        scan_face_edges(self, face, edge_pairs, tolerance)
    }
}

fn require_wire(face: &Face) -> Result<(), Failure> {
    let shape = face.clone().into_shape();
    let wires = Explorer::new(&shape, ShapeEnum::Wire, ShapeEnum::Shape)?;
    if wires.more() {
        return Ok(());
    }
    Err(Failure::with_message(
        "Standard_NoSuchObject",
        "ShapeAnalysis_CheckSmallFace::CheckPinFace: face has no wire; expected an outer wire",
    ))
}

fn scan_face_edges(
    analyzer: &mut ShapeAnalysisCheckSmallFace,
    face: &Face,
    edge_pairs: &mut PinFaceEdgeMap,
    tolerance: f64,
) -> Result<bool, Failure> {
    let shape = face.clone().into_shape();
    let mut edges = Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    let mut state = PinFaceState::default();
    while edges.more() {
        let edge = topods::edge(edges.current())?;
        let step = update_pin_face_state(analyzer, &mut state, edge, edge_pairs, tolerance)?;
        if consume_pin_face_step(analyzer, &mut state, step) {
            return Ok(false);
        }
        edges.next()?;
    }
    Ok(state.done)
}

fn consume_pin_face_step(
    analyzer: &mut ShapeAnalysisCheckSmallFace,
    state: &mut PinFaceState,
    step: PinFaceStep,
) -> bool {
    match step {
        PinFaceStep::Abort => true,
        PinFaceStep::Matched => {
            state.done = true;
            analyzer.pin_face_status = Some(ShapeExtend::encode_status(ShapeExtendStatus::Done));
            false
        }
        PinFaceStep::Continue => false,
    }
}

enum PinFaceStep {
    Continue,
    Matched,
    Abort,
}

#[derive(Default)]
struct PinFaceState {
    first: Option<Edge>,
    coefficient: f64,
    done: bool,
}

fn update_pin_face_state(
    analyzer: &ShapeAnalysisCheckSmallFace,
    state: &mut PinFaceState,
    edge: Edge,
    edge_pairs: &mut PinFaceEdgeMap,
    tolerance: f64,
) -> Result<PinFaceStep, Failure> {
    let Some(first) = state.first.as_ref() else {
        return select_initial_edge(state, edge, tolerance);
    };
    let first = first.clone();
    try_pin_pair(analyzer, state, &first, edge, edge_pairs, tolerance)
}

fn select_initial_edge(
    state: &mut PinFaceState,
    edge: Edge,
    tolerance: f64,
) -> Result<PinFaceStep, Failure> {
    let (length, local_tolerance) = endpoint_length(&edge)?;
    if length == 0.0 {
        return Ok(PinFaceStep::Abort);
    }
    let scale = initial_scale(length, local_tolerance, tolerance);
    if scale <= 3.0 {
        return Ok(PinFaceStep::Continue);
    }
    state.first = Some(edge);
    state.coefficient = scale;
    Ok(PinFaceStep::Continue)
}

fn try_pin_pair(
    analyzer: &ShapeAnalysisCheckSmallFace,
    state: &mut PinFaceState,
    first: &Edge,
    second: Edge,
    edge_pairs: &mut PinFaceEdgeMap,
    tolerance: f64,
) -> Result<PinFaceStep, Failure> {
    let second_coefficient = match second_candidate(&second, tolerance)? {
        SecondEdgeResult::Skip => return Ok(PinFaceStep::Continue),
        SecondEdgeResult::ZeroLength => return Ok(PinFaceStep::Abort),
        SecondEdgeResult::Candidate(value) => value,
    };
    if !matches!(
        compare_coefficients(state, &second, second_coefficient),
        PairScale::Compare
    ) {
        return Ok(PinFaceStep::Continue);
    }
    compare_pin_edges(
        analyzer,
        state,
        first,
        second,
        second_coefficient,
        edge_pairs,
        tolerance,
    )
}

fn compare_pin_edges(
    analyzer: &ShapeAnalysisCheckSmallFace,
    state: &mut PinFaceState,
    first: &Edge,
    second: Edge,
    second_coefficient: f64,
    edge_pairs: &mut PinFaceEdgeMap,
    tolerance: f64,
) -> Result<PinFaceStep, Failure> {
    let matched = check_and_bind_pair(
        analyzer,
        first,
        &second,
        state.coefficient,
        second_coefficient,
        edge_pairs,
        tolerance,
    )?;
    advance_first_edge(state, second, second_coefficient);
    Ok(if matched {
        PinFaceStep::Matched
    } else {
        PinFaceStep::Continue
    })
}

enum PairScale {
    Skip,
    Replace,
    Compare,
}

fn compare_coefficients(state: &mut PinFaceState, edge: &Edge, coefficient: f64) -> PairScale {
    if state.coefficient > coefficient * 10.0 {
        return PairScale::Skip;
    }
    if coefficient > state.coefficient * 10.0 {
        state.first = Some(edge.clone());
        state.coefficient = coefficient;
        return PairScale::Replace;
    }
    PairScale::Compare
}

fn check_and_bind_pair(
    analyzer: &ShapeAnalysisCheckSmallFace,
    first: &Edge,
    second: &Edge,
    first_coefficient: f64,
    second_coefficient: f64,
    edge_pairs: &mut PinFaceEdgeMap,
    tolerance: f64,
) -> Result<bool, Failure> {
    let matched = analyzer.check_pin_edges(
        first,
        second,
        first_coefficient,
        second_coefficient,
        tolerance,
    )?;
    if matched {
        bind_pin_pair(edge_pairs, first, second)?;
    }
    Ok(matched)
}

fn advance_first_edge(state: &mut PinFaceState, edge: Edge, coefficient: f64) {
    state.first = Some(edge);
    state.coefficient = coefficient;
}

enum SecondEdgeResult {
    Skip,
    ZeroLength,
    Candidate(f64),
}

fn second_candidate(edge: &Edge, tolerance: f64) -> Result<SecondEdgeResult, Failure> {
    let (length, local_tolerance) = endpoint_length(edge)?;
    let active_tolerance = second_edge_tolerance(local_tolerance, tolerance);
    if length > active_tolerance {
        return Ok(SecondEdgeResult::Skip);
    }
    if length == 0.0 {
        return Ok(SecondEdgeResult::ZeroLength);
    }
    Ok(second_edge_scale(length, active_tolerance))
}

fn second_edge_tolerance(local: f64, fixed: f64) -> f64 {
    if fixed == -1.0 { local } else { fixed }
}

fn second_edge_scale(length: f64, tolerance: f64) -> SecondEdgeResult {
    let coefficient = length / tolerance;
    if coefficient > 3.0 {
        return SecondEdgeResult::Candidate(coefficient);
    }
    SecondEdgeResult::Skip
}

fn endpoint_length(edge: &Edge) -> Result<(f64, f64), Failure> {
    let (first, last) = vertices(edge, false);
    let first_point = BRepTool::pnt(&first)?;
    let last_point = BRepTool::pnt(&last)?;
    let first_tolerance = BRepTool::vertex_tolerance(&first)?;
    let last_tolerance = BRepTool::vertex_tolerance(&last)?;
    let tolerance = if first_tolerance < last_tolerance {
        last_tolerance
    } else {
        first_tolerance
    };
    Ok((first_point.distance(&last_point), tolerance))
}

fn initial_scale(length: f64, local_tolerance: f64, tolerance: f64) -> f64 {
    let active_tolerance = if tolerance > 0.0 {
        tolerance
    } else {
        local_tolerance
    };
    if length / active_tolerance >= 1.0 {
        length / active_tolerance
    } else {
        0.0
    }
}

fn bind_pin_pair(map: &mut PinFaceEdgeMap, first: &Edge, second: &Edge) -> Result<(), Failure> {
    let first_shape = first.clone().into_shape();
    if map.is_bound(&first_shape) {
        return Err(Failure::with_message(
            "Standard_DomainError",
            &format!(
                "ShapeAnalysis_CheckSmallFace::CheckPinFace: duplicate first edge {first_shape:?}; expected a new key"
            ),
        ));
    }
    map.bind(first_shape, second.clone().into_shape());
    Ok(())
}
