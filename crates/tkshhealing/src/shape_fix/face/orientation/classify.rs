mod periodic;

use self::periodic::{
    PeriodicRanges, classify_vertex_periodically, periodic_copy, toggle_inside_outside,
};
use super::orient::orient_classified_wires;
use super::periodic::shift_wire_pcurves;
use super::{FaceWireSet, ShapeFixFace, ShapeFixFaceWireMap};
use crate::shape_analysis::{ShapeAnalysisSurface, adjust_by_period};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Builder, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

pub(super) struct WireRelation {
    pub(super) containing: State,
    pub(super) infinite: State,
    pub(super) internal: List<Shape>,
}

struct WireClassifier<'a> {
    face: &'a Face,
    wires: &'a [Wire],
    boxes: &'a [BoundingBox2],
    periods: PeriodicRanges,
}

pub(super) fn orient_multiple_wires(
    fixer: &ShapeFixFace,
    face: &Face,
    wires: &FaceWireSet,
    output_map: &mut ShapeFixFaceWireMap,
) -> Result<(Vec<Wire>, Vec<usize>, bool), Failure> {
    let surface = fixer.analysis_surface.read().clone();
    let Some(surface) = surface.get() else {
        return Ok((wires.oriented.clone(), Vec::new(), wires.changed));
    };
    let periods = periodic_ranges(surface)?;
    let boxes = aligned_wire_boxes(face, &wires.oriented, periods)?;
    let relations = classify_all_wires(face, wires, surface, periods, &boxes)?;

    orient_classified_wires(fixer, face, wires, relations, output_map)
}

fn periodic_ranges(surface: &ShapeAnalysisSurface) -> Result<PeriodicRanges, Failure> {
    let bounds = surface.bounds();
    Ok(PeriodicRanges {
        u: surface.is_u_closed(-1.0)?.then_some(bounds.1 - bounds.0),
        v: surface.is_v_closed(-1.0)?.then_some(bounds.3 - bounds.2),
    })
}

fn aligned_wire_boxes(
    face: &Face,
    wires: &[Wire],
    periods: PeriodicRanges,
) -> Result<Vec<BoundingBox2>, Failure> {
    let mut result = Vec::with_capacity(wires.len());
    let mut reference = Point2::default();
    for wire in wires {
        let mut bounds = wire_uv_bounds(face, wire)?;
        align_box(&mut bounds, &mut reference, result.is_empty(), periods)?;
        result.push(bounds);
    }
    Ok(result)
}

fn align_box(
    bounds: &mut BoundingBox2,
    reference: &mut Point2,
    first: bool,
    periods: PeriodicRanges,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let [umin, vmin, umax, vmax] = bounds.get()?;
    let center = Point2::new((umin + umax) * 0.5, (vmin + vmax) * 0.5);
    if first {
        *reference = center;
        return Ok(());
    }
    let (u, v) = center_shifts(center, *reference, periods);
    if u != 0.0 || v != 0.0 {
        bounds.update_bounds([umin + u, vmin + v], [umax + u, vmax + v]);
    }
    Ok(())
}

fn center_shifts(center: Point2, target: Point2, periods: PeriodicRanges) -> (f64, f64) {
    let u = periods
        .u
        .map(|period| adjust_by_period(center.x(), target.x(), period))
        .unwrap_or(0.0);
    let v = periods
        .v
        .map(|period| adjust_by_period(center.y(), target.y(), period))
        .unwrap_or(0.0);
    (u, v)
}

fn wire_uv_bounds(face: &Face, wire: &Wire) -> Result<BoundingBox2, Failure> {
    let mut bounds = BoundingBox2::default();
    for child in ShapeIterator::new(&wire.clone().into_shape(), true, true) {
        let curve = BRepTool::curve_on_face(&topods::edge(&child)?, face)?;
        add_curve_bounds(&curve.pcurve, curve.first, curve.last, &mut bounds)?;
    }
    Ok(bounds)
}

fn add_curve_bounds(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
    bounds: &mut BoundingBox2,
) -> Result<(), Failure> {
    if curve.is_null() {
        return Ok(());
    }
    if use_full_bspline_bounds(curve, first, last) {
        return Add2dCurve::add_curve(curve, rustycad_tkernel::precision::confusion(), bounds);
    }
    Add2dCurve::add_curve_range(
        curve,
        first,
        last,
        rustycad_tkernel::precision::confusion(),
        bounds,
    )
}

fn use_full_bspline_bounds(curve: &Handle<dyn Geom2dCurve>, first: f64, last: f64) -> bool {
    curve.cast::<Geom2dBSplineCurve>().is_some()
        && (first < curve.first_parameter() || last > curve.last_parameter())
}

fn classify_all_wires(
    face: &Face,
    wires: &FaceWireSet,
    surface: &ShapeAnalysisSurface,
    periods: PeriodicRanges,
    boxes: &[BoundingBox2],
) -> Result<Vec<WireRelation>, Failure> {
    let context = WireClassifier {
        face,
        wires: &wires.oriented,
        boxes,
        periods,
    };
    wires
        .oriented
        .iter()
        .enumerate()
        .map(|(index, wire)| classify_one_wire(&context, wire, index, surface, &wires.all))
        .collect()
}

fn classify_one_wire(
    context: &WireClassifier<'_>,
    wire: &Wire,
    index: usize,
    surface: &ShapeAnalysisSurface,
    candidates: &[Shape],
) -> Result<WireRelation, Failure> {
    let classifier = wire_classifier(context.face, wire)?;
    let mut relation = empty_relation(&classifier)?;
    classify_candidates(
        context,
        wire,
        &context.boxes[index],
        surface,
        &classifier,
        &mut relation,
        candidates,
    )?;
    Ok(relation)
}

fn wire_classifier(face: &Face, wire: &Wire) -> Result<BRepTopAdaptorFClass2d, Failure> {
    let dummy = one_wire_face(face, wire)?;
    BRepTopAdaptorFClass2d::new(&dummy, rustycad_tkernel::precision::p_confusion())
}

fn empty_relation(classifier: &BRepTopAdaptorFClass2d) -> Result<WireRelation, Failure> {
    Ok(WireRelation {
        containing: State::Out,
        infinite: classifier.perform_infinite_point()?,
        internal: List::default(),
    })
}

fn classify_candidates(
    context: &WireClassifier<'_>,
    wire: &Wire,
    source_box: &BoundingBox2,
    surface: &ShapeAnalysisSurface,
    classifier: &BRepTopAdaptorFClass2d,
    relation: &mut WireRelation,
    candidates: &[Shape],
) -> Result<(), Failure> {
    for candidate in candidates {
        if skip_candidate(relation, candidate, wire) {
            continue;
        }
        if compare_candidate(
            context, candidate, source_box, surface, classifier, relation,
        )? {
            break;
        }
    }
    Ok(())
}

fn skip_candidate(relation: &WireRelation, candidate: &Shape, wire: &Wire) -> bool {
    relation.containing == State::Unknown || *candidate == wire.clone().into_shape()
}

fn one_wire_face(face: &Face, wire: &Wire) -> Result<Face, Failure> {
    let mut dummy = face.empty_copied()?;
    dummy.set_orientation(Orientation::Forward);
    Builder.add(&mut dummy, &wire.clone().into_shape())?;
    topods::face(&dummy)
}

fn compare_candidate(
    context: &WireClassifier<'_>,
    candidate: &Shape,
    source_box: &BoundingBox2,
    surface: &ShapeAnalysisSurface,
    classifier: &BRepTopAdaptorFClass2d,
    relation: &mut WireRelation,
) -> Result<bool, Failure> {
    match candidate.shape_type()? {
        ShapeEnum::Vertex => {
            classify_vertex(candidate, surface, context.periods, classifier, relation)?;
            Ok(false)
        }
        ShapeEnum::Wire => {
            classify_candidate_wire(context, candidate, source_box, classifier, relation)
        }
        _ => Ok(false),
    }
}

fn classify_vertex(
    candidate: &Shape,
    surface: &ShapeAnalysisSurface,
    periods: PeriodicRanges,
    classifier: &BRepTopAdaptorFClass2d,
    relation: &mut WireRelation,
) -> Result<(), Failure> {
    let vertex = Vertex::unchecked(candidate.clone());
    let point = surface.value_of_uv(
        BRepTool::pnt(&vertex)?,
        rustycad_tkernel::precision::confusion(),
    )?;
    let state = classify_vertex_periodically(point, periods, classifier, relation.infinite)?;
    record_candidate_state(candidate, state, relation);
    Ok(())
}

fn classify_candidate_wire(
    context: &WireClassifier<'_>,
    candidate: &Shape,
    source_box: &BoundingBox2,
    classifier: &BRepTopAdaptorFClass2d,
    relation: &mut WireRelation,
) -> Result<bool, Failure> {
    let Some(other) = candidate_wire_in_bounds(context, candidate, source_box)? else {
        return Ok(false);
    };
    update_candidate_relation(context, candidate, &other, classifier, relation)
}

fn update_candidate_relation(
    context: &WireClassifier<'_>,
    candidate: &Shape,
    other: &Wire,
    classifier: &BRepTopAdaptorFClass2d,
    relation: &mut WireRelation,
) -> Result<bool, Failure> {
    let (state, inconsistent) = classify_other_wire(context, other, classifier, relation.infinite)?;
    record_candidate_state(candidate, state, relation);
    if inconsistent && state != relation.infinite {
        relation.containing = State::Unknown;
    }
    Ok(inconsistent)
}

fn candidate_wire_in_bounds(
    context: &WireClassifier<'_>,
    candidate: &Shape,
    source_box: &BoundingBox2,
) -> Result<Option<Wire>, Failure> {
    if candidate_bounds(context, candidate)?.is_out_box(source_box) {
        return Ok(None);
    }
    Ok(Some(topods::wire(candidate)?))
}

fn candidate_bounds(
    context: &WireClassifier<'_>,
    candidate: &Shape,
) -> Result<BoundingBox2, Failure> {
    if let Some(index) = context
        .wires
        .iter()
        .position(|wire| wire.clone().into_shape() == *candidate)
    {
        return Ok(context.boxes[index].clone());
    }
    wire_uv_bounds(context.face, &topods::wire(candidate)?)
}

fn classify_other_wire(
    context: &WireClassifier<'_>,
    wire: &Wire,
    classifier: &BRepTopAdaptorFClass2d,
    infinite: State,
) -> Result<(State, bool), Failure> {
    let mut scan = CandidateWireScan::new(infinite);
    for child in ShapeIterator::new(&wire.clone().into_shape(), true, true) {
        if scan.classify_edge(context, wire, &child, classifier)? {
            return Ok(scan.result(true));
        }
    }
    Ok(scan.result(false))
}

struct CandidateWireScan {
    infinite: State,
    found: State,
    check_shift: bool,
}

impl CandidateWireScan {
    fn new(infinite: State) -> Self {
        Self {
            infinite,
            found: State::Unknown,
            check_shift: true,
        }
    }

    fn result(&self, inconsistent: bool) -> (State, bool) {
        (self.found, inconsistent)
    }

    fn classify_edge(
        &mut self,
        context: &WireClassifier<'_>,
        wire: &Wire,
        child: &Shape,
        classifier: &BRepTopAdaptorFClass2d,
    ) -> Result<bool, Failure> {
        let Some(point) = edge_midpoint(context.face, child)? else {
            return Ok(false);
        };
        let state = classifier.perform(point, false)?;
        if !merge_candidate_state(state, &mut self.found)? {
            return Ok(true);
        }
        self.try_shift(context, wire, point, classifier)
    }

    fn try_shift(
        &mut self,
        context: &WireClassifier<'_>,
        wire: &Wire,
        point: Point2,
        classifier: &BRepTopAdaptorFClass2d,
    ) -> Result<bool, Failure> {
        if self.found != self.infinite || !self.check_shift {
            return Ok(false);
        }
        self.check_shift = false;
        let Some((_, shifted)) = periodic_copy(point, context.periods, classifier, self.infinite)?
        else {
            return Ok(false);
        };
        self.found = toggle_inside_outside(self.found);
        shift_wire_pcurves(context.face, wire, Vector2::between(point, shifted))?;
        Ok(false)
    }
}

fn edge_midpoint(face: &Face, child: &Shape) -> Result<Option<Point2>, Failure> {
    let pcurve = BRepTool::curve_on_face(&topods::edge(child)?, face)?;
    if pcurve.pcurve.is_null() {
        return Ok(None);
    }
    Ok(Some(
        pcurve.pcurve.value((pcurve.first + pcurve.last) * 0.5)?,
    ))
}

fn merge_candidate_state(state: State, found: &mut State) -> Result<bool, Failure> {
    if state != State::In && state != State::Out {
        return Ok(true);
    }
    if *found == State::Unknown {
        *found = state;
        return Ok(true);
    }
    Ok(*found == state)
}

fn record_candidate_state(candidate: &Shape, state: State, relation: &mut WireRelation) {
    if state == relation.infinite {
        relation.containing = State::In;
    } else {
        relation.internal.append(candidate.clone());
    }
}
