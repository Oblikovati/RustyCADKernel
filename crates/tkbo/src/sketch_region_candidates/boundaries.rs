// SPDX-License-Identifier: GPL-2.0-only
//! Captured boundary occurrences retain source parents and exact stored polynomial coefficients.
use super::{generation::CandidateParents, outcome::*};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_tools::WireExplorer,
    topods::{Edge, Face, Shape, ShapeIterator, face, wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
pub(super) fn capture_candidate(
    shape: &Shape,
    parents: &CandidateParents,
    occurrence: &mut u64,
) -> Result<RegionCandidate, CandidateCause> {
    let face = face(shape).map_err(CandidateCause::Mapped)?;
    let wires = candidate_wires(&face, parents, occurrence)?;
    let relations = super::ancestry::candidate_relations(shape, parents)?;
    Ok(RegionCandidate {
        face,
        wires,
        relations,
    })
}
fn candidate_wires(
    face: &Face,
    parents: &CandidateParents,
    occurrence: &mut u64,
) -> Result<Vec<CandidateWire>, CandidateCause> {
    let mut wires = Vec::new();
    for child in fallible_children(face)? {
        if child.shape_type().map_err(CandidateCause::Mapped)? != ShapeEnum::Wire {
            return Err(CandidateCause::Incomplete(
                CandidateGap::UnclassifiedOccurrence { member: child },
            ));
        }
        wires.push(capture_wire(&child, face, parents, occurrence)?);
    }
    Ok(wires)
}
pub(super) fn capture_wire(
    shape: &Shape,
    face: &Face,
    parents: &CandidateParents,
    next: &mut u64,
) -> Result<CandidateWire, CandidateCause> {
    let wire = wire(shape).map_err(CandidateCause::Mapped)?;
    let mut capture = WireCapture::new(&wire)?;
    let mut explorer =
        WireExplorer::with_wire_on_face(&wire, face).map_err(CandidateCause::Mapped)?;
    capture_walk(&mut explorer, face, parents, next, &mut capture)?;
    capture.finish()
}
struct WireCapture {
    segments: Vec<CandidateSegment>,
    omitted: Vec<Shape>,
    unexpected: Vec<Shape>,
}
impl WireCapture {
    fn new(wire: &rustycad_tkbrep::topods::Wire) -> Result<Self, CandidateCause> {
        Ok(Self {
            segments: Vec::new(),
            omitted: fallible_children(wire)?,
            unexpected: Vec::new(),
        })
    }
    fn finish(self) -> Result<CandidateWire, CandidateCause> {
        if !self.omitted.is_empty() || !self.unexpected.is_empty() {
            return Err(CandidateCause::Incomplete(CandidateGap::IncompleteWire {
                omitted: self.omitted,
                unexpected: self.unexpected,
            }));
        }
        Ok(CandidateWire {
            segments: self.segments,
        })
    }
}
fn capture_walk(
    explorer: &mut WireExplorer,
    face: &Face,
    parents: &CandidateParents,
    next: &mut u64,
    capture: &mut WireCapture,
) -> Result<(), CandidateCause> {
    while explorer.more() {
        capture_visited(explorer.current(), face, parents, next, capture)?;
        explorer.next().map_err(CandidateCause::Mapped)?;
    }
    Ok(())
}
fn capture_segment(
    edge: &Edge,
    face: &Face,
    parents: &CandidateParents,
    id: u64,
) -> Result<CandidateSegment, CandidateCause> {
    let ancestry = segment_parents(edge, parents, id)?;
    let representation = BRepTool::curve_on_face(edge, face).map_err(CandidateCause::Mapped)?;
    let bounds = [representation.first, representation.last];
    let (controls, weights, interval) = capture_coefficients(&representation.pcurve, bounds, id)?;
    Ok(owned_segment(
        edge,
        id,
        ancestry,
        bounds,
        (controls, weights, interval),
    ))
}
fn owned_segment(
    edge: &Edge,
    id: u64,
    parents: Vec<crate::sketch_region::OriginalMemberIdentity>,
    bounds: [f64; 2],
    coefficients: CapturedCoefficients,
) -> CandidateSegment {
    CandidateSegment {
        occurrence: id,
        edge: edge.clone().into_shape(),
        parents,
        controls: coefficients.0,
        weights: coefficients.1,
        interval: coefficients.2,
        original_interval: bounds,
        reversed: edge.orientation() == Orientation::Reversed,
    }
}

fn segment_parents(
    edge: &Edge,
    parents: &CandidateParents,
    id: u64,
) -> Result<Vec<crate::sketch_region::OriginalMemberIdentity>, CandidateCause> {
    let mut original = Vec::new();
    append_segment_parents(edge, parents, &mut original);
    if original.is_empty() {
        return Err(CandidateCause::Incomplete(CandidateGap::MissingParents {
            occurrence: id,
        }));
    }
    Ok(original)
}
type CapturedCoefficients = (Vec<[f64; 2]>, Option<Vec<f64>>, [f64; 2]);
fn capture_coefficients(
    curve: &Handle<dyn Geom2dCurve>,
    bounds: [f64; 2],
    id: u64,
) -> Result<CapturedCoefficients, CandidateCause> {
    if let Some(bezier) = curve.cast::<Geom2dBezierCurve>().get() {
        return bezier_coefficients(bezier, bounds);
    }
    if let Some(line) = curve.cast::<Geom2dLine>().get() {
        let points = [
            line.value(bounds[0]).map_err(CandidateCause::Mapped)?,
            line.value(bounds[1]).map_err(CandidateCause::Mapped)?,
        ];
        return Ok((points.map(|p| [p.x(), p.y()]).to_vec(), None, [0., 1.]));
    }
    Err(CandidateCause::Incomplete(CandidateGap::UnsupportedCurve {
        occurrence: id,
        dynamic_type: format!("{:?}", curve.get().map(|c| c.dynamic_type().name())),
    }))
}
fn bezier_coefficients(
    curve: &Geom2dBezierCurve,
    bounds: [f64; 2],
) -> Result<CapturedCoefficients, CandidateCause> {
    let mut controls = Vec::new();
    let mut weights = Vec::new();
    for index in 1..=curve.nb_poles() {
        let point = curve.pole(index).map_err(CandidateCause::Mapped)?;
        controls.push([point.x(), point.y()]);
        weights.push(curve.weight(index).map_err(CandidateCause::Mapped)?);
    }
    Ok((controls, curve.is_rational().then_some(weights), bounds))
}

fn record_visited(edge: &Edge, omitted: &mut Vec<Shape>, unexpected: &mut Vec<Shape>) {
    if let Some(index) = omitted.iter().position(|child| child.is_equal(edge)) {
        omitted.remove(index);
        return;
    }
    unexpected.push(edge.clone().into_shape());
}

fn capture_visited(
    edge: &Edge,
    face: &Face,
    parents: &CandidateParents,
    next: &mut u64,
    capture: &mut WireCapture,
) -> Result<(), CandidateCause> {
    record_visited(edge, &mut capture.omitted, &mut capture.unexpected);
    capture
        .segments
        .push(capture_segment(edge, face, parents, *next)?);
    *next += 1;
    Ok(())
}

fn fallible_children(wire: &Shape) -> Result<Vec<Shape>, CandidateCause> {
    let mut children = ShapeIterator::new(wire, true, true);
    let mut occurrences = Vec::new();
    while children.more() {
        occurrences.push(children.value().map_err(CandidateCause::Mapped)?.clone());
        children.next();
    }
    Ok(occurrences)
}

fn append_segment_parents(
    edge: &Edge,
    parents: &CandidateParents,
    original: &mut Vec<crate::sketch_region::OriginalMemberIdentity>,
) {
    for relation in parents
        .iter()
        .filter(|relation| relation.output.is_same(edge))
    {
        for key in relation.modified.iter().chain(&relation.generated) {
            if !original.contains(key) {
                original.push(key.clone());
            }
        }
    }
}
