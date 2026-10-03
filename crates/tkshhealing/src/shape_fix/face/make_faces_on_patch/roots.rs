//! Candidate root-wire sampling and source-ordered endpoint fallback.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

pub(super) fn find_root_wires(
    loops: &Sequence<Shape>,
    support: &Face,
) -> Result<Vec<Shape>, Failure> {
    let mut roots = Vec::new();
    for index in 1..=loops.len() {
        let candidate = loops.value(index)?;
        if !is_manifold_wire(candidate)? {
            continue;
        }
        if wire_is_root(loops, index, support, candidate)? {
            roots.push(candidate.clone());
        }
    }
    Ok(roots)
}

fn wire_is_root(
    loops: &Sequence<Shape>,
    candidate_index: usize,
    support: &Face,
    candidate: &Shape,
) -> Result<bool, Failure> {
    let Some((sample, mut edge_cursor)) = root_sample_and_cursor(candidate, support)? else {
        return Ok(false);
    };
    Ok(
        other_wire_contains_sample(loops, candidate_index, support, sample, &mut edge_cursor)?
            .is_none(),
    )
}

fn other_wire_contains_sample(
    loops: &Sequence<Shape>,
    candidate_index: usize,
    support: &Face,
    sample: Point2,
    edge_cursor: &mut CandidateEdgeCursor,
) -> Result<Option<()>, Failure> {
    for other_index in 1..=loops.len() {
        if candidate_index == other_index {
            continue;
        }
        let other = loops.value(other_index)?;
        if is_manifold_wire(other)? && contains_wire_sample(support, other, sample, edge_cursor)? {
            return Ok(Some(()));
        }
    }
    Ok(None)
}

pub(super) fn is_manifold_wire(shape: &Shape) -> Result<bool, Failure> {
    Ok(shape.shape_type()? == ShapeEnum::Wire
        && matches!(
            shape.orientation(),
            Orientation::Forward | Orientation::Reversed
        ))
}

fn contains_wire_sample(
    support: &Face,
    wire: &Shape,
    point: Point2,
    edge_cursor: &mut CandidateEdgeCursor,
) -> Result<bool, Failure> {
    let Some(candidate_face) = face_with_wire(support, wire)? else {
        return Ok(false);
    };
    let classifier = BRepTopAdaptorFClass2d::new(&candidate_face, precision::p_confusion())?;
    let infinite = classifier.perform_infinite_point()?;
    let mut state = classifier.perform(point, false)?;
    if matches!(state, State::On | State::Unknown) {
        state = classify_edge_endpoints(&classifier, edge_cursor)?;
    }
    Ok(state != infinite)
}

fn face_with_wire(support: &Face, wire: &Shape) -> Result<Option<Face>, Failure> {
    let mut face = support
        .clone()
        .empty_copied()
        .and_then(|shape| topods::face(&shape))?;
    let Some(manifold) = manifold_wire(wire)? else {
        return Ok(None);
    };
    Builder.add(&mut face, &manifold.into_shape())?;
    Ok(Some(face))
}

pub(super) fn manifold_wire(shape: &Shape) -> Result<Option<Wire>, Failure> {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    let mut edge_count = 0;
    for child in ShapeIterator::new(&shape.clone(), true, true) {
        if matches!(
            child.orientation(),
            Orientation::Forward | Orientation::Reversed
        ) {
            Builder.add(&mut wire, &child)?;
            edge_count += 1;
        }
    }
    if edge_count == 0 {
        return Ok(None);
    }
    Ok(Some(wire))
}

fn classify_edge_endpoints(
    classifier: &BRepTopAdaptorFClass2d,
    cursor: &mut CandidateEdgeCursor,
) -> Result<State, Failure> {
    cursor.classify_endpoints(classifier)
}

struct CandidateEdgeCursor {
    edges: Vec<Edge>,
    support: Face,
    index: usize,
    curve: Handle<dyn Geom2dCurve>,
    last: f64,
}

fn root_sample_and_cursor(
    candidate: &Shape,
    support: &Face,
) -> Result<Option<(Point2, CandidateEdgeCursor)>, Failure> {
    let mut cursor = CandidateEdgeCursor::from_wire(candidate, support)?;
    if !cursor.select_initial_curve()? {
        return Ok(None);
    }
    let curve = cursor.curve.get().ok_or_else(null_pcurve)?;
    let first = BRepTool::curve_on_face(&cursor.edges[cursor.index], support)?.first;
    let sample = curve.value(0.5 * (first + cursor.last))?;
    Ok(Some((sample, cursor)))
}

impl CandidateEdgeCursor {
    fn from_wire(wire: &Shape, support: &Face) -> Result<Self, Failure> {
        let mut edges = Vec::new();
        for child in ShapeIterator::new(wire, true, true) {
            if child.shape_type()? == ShapeEnum::Edge {
                edges.push(topods::edge(&child)?);
            }
        }
        Ok(Self {
            edges,
            support: support.clone(),
            index: 0,
            curve: Handle::default(),
            last: 0.0,
        })
    }

    fn select_initial_curve(&mut self) -> Result<bool, Failure> {
        while self.index < self.edges.len()
            && !is_manifold_orientation(self.edges[self.index].orientation())
        {
            self.index += 1;
        }
        if self.index == self.edges.len() {
            return Ok(false);
        }
        let found = BRepTool::curve_on_face(&self.edges[self.index], &self.support)?;
        if found.pcurve.is_null() {
            return Ok(false);
        }
        self.curve = found.pcurve;
        self.last = found.last;
        Ok(true)
    }

    fn classify_endpoints(
        &mut self,
        classifier: &BRepTopAdaptorFClass2d,
    ) -> Result<State, Failure> {
        loop {
            let curve = self.curve.get().ok_or_else(null_pcurve)?;
            let state = classifier.perform(curve.value(self.last)?, false)?;
            if !matches!(state, State::On | State::Unknown) || !self.advance_curve()? {
                return Ok(state);
            }
        }
    }

    fn advance_curve(&mut self) -> Result<bool, Failure> {
        self.index += 1;
        while self.index < self.edges.len() {
            if is_manifold_orientation(self.edges[self.index].orientation()) {
                let found = BRepTool::curve_on_face(&self.edges[self.index], &self.support)?;
                if !found.pcurve.is_null() {
                    self.curve = found.pcurve;
                    self.last = found.last;
                    return Ok(true);
                }
            }
            self.index += 1;
        }
        Ok(false)
    }
}

fn is_manifold_orientation(orientation: Orientation) -> bool {
    matches!(orientation, Orientation::Forward | Orientation::Reversed)
}

fn null_pcurve() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::MakeFacesOnPatch: null pcurve",
    )
}

pub(super) fn recover_lost_roots(
    loops: &mut Sequence<Shape>,
    mut roots: Vec<Shape>,
) -> Result<Vec<Shape>, Failure> {
    if !roots.is_empty() {
        return Ok(roots);
    }
    roots.extend(loops.iter().cloned());
    loops.clear(None);
    Ok(roots)
}
