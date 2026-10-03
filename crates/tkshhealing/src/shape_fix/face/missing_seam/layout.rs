use super::plan::MissingSeamPlan;
use crate::shape_analysis::{ShapeAnalysis, ShapeAnalysisEdge, adjust_by_period};
use crate::shape_extend::ShapeExtendWireData;
use crate::shape_fix::FixRoot;
use crate::shape_fix::ShapeFixWire;
use rustycad_tkbrep::topods::{self, Builder, Face, ShapeIterator, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Vector2;

pub(super) struct OrderedWires {
    pub(super) face: Face,
    pub(super) first: Wire,
    pub(super) second: Wire,
    pub(super) first_bounds: [f64; 4],
    pub(super) second_bounds: [f64; 4],
    pub(super) u_split: f64,
    pub(super) v_split: f64,
}

pub(super) fn order_and_align(
    plan: &mut MissingSeamPlan,
    precision: f64,
) -> Result<Option<OrderedWires>, Failure> {
    let first_bounds = wire_bounds(&plan.face, &plan.first_wire)?;
    let second_bounds = wire_bounds(&plan.face, &plan.second_wire)?;
    reverse_for_surface_orientation(plan, &first_bounds, &second_bounds);
    let (first, second) = reorder_open_wires(plan, precision)?;
    let face = assemble_oriented_face(plan, &first, &second)?;
    plan.first_wire = first.clone();
    plan.second_wire = second.clone();
    plan.face = face.clone();
    let mut ordered = OrderedWires {
        face,
        first,
        second,
        first_bounds,
        second_bounds,
        u_split: plan.surface_bounds.0,
        v_split: plan.surface_bounds.2,
    };
    align_periodic_wires(plan, &mut ordered)?;
    let Some((u_split, v_split)) = select_split_parameter(plan, &ordered)? else {
        return Ok(None);
    };
    ordered.u_split = u_split;
    ordered.v_split = v_split;
    Ok(Some(ordered))
}

fn wire_bounds(face: &Face, wire: &Wire) -> Result<[f64; 4], Failure> {
    let mut temporary = topods::face(&face.clone().into_shape().empty_copied()?)?;
    Builder.add(&mut temporary, &wire.clone().into_shape())?;
    let bounds = ShapeAnalysis::get_face_uv_bounds(&temporary)?;
    Ok([bounds.0, bounds.1, bounds.2, bounds.3])
}

fn reverse_for_surface_orientation(
    plan: &mut MissingSeamPlan,
    first: &[f64; 4],
    second: &[f64; 4],
) {
    if plan.u_closed && plan.v_closed && !is_degenerated_torus(plan) {
        return;
    }
    let coord = if plan.direction.u != 0 { 2 } else { 0 };
    let mode = if plan.direction.u != 0 {
        plan.direction.u
    } else {
        -plan.direction.v
    };
    let delta = midpoint(second, coord) - midpoint(first, coord);
    if delta * (mode as f64) < 0.0 {
        plan.first_wire.reverse();
        plan.second_wire.reverse();
    }
}

fn is_degenerated_torus(plan: &MissingSeamPlan) -> bool {
    plan.surface
        .cast::<rustycad_tkg3d::geom::GeomToroidalSurface>()
        .get()
        .is_some_and(|surface| surface.major_radius() < surface.minor_radius())
}

fn midpoint(bounds: &[f64; 4], index: usize) -> f64 {
    (bounds[index] + bounds[index + 1]) * 0.5
}

fn reorder_open_wires(plan: &MissingSeamPlan, precision: f64) -> Result<(Wire, Wire), Failure> {
    let first = reorder_wire(plan, &plan.first_wire, precision)?;
    let second = reorder_wire(plan, &plan.second_wire, precision)?;
    Ok((first, second))
}

fn reorder_wire(plan: &MissingSeamPlan, wire: &Wire, precision: f64) -> Result<Wire, Failure> {
    let fixer = ShapeFixWire::new();
    fixer.set_face(&plan.face)?;
    fixer.set_precision(precision);
    fixer.load(wire)?;
    fixer.fix_reorder()?;
    fixer.wire()
}

fn assemble_oriented_face(
    plan: &MissingSeamPlan,
    first: &Wire,
    second: &Wire,
) -> Result<Face, Failure> {
    let mut face = empty_face(&plan.face)?;
    face.set_orientation(Orientation::Forward);
    for wire in &plan.wires {
        let prepared = prepared_wire(plan, wire, first, second)?;
        Builder.add(&mut face, &prepared.into_shape())?;
    }
    for shape in &plan.nonmanifold {
        Builder.add(&mut face, shape)?;
    }
    face.set_orientation(plan.face.orientation());
    Ok(face)
}

fn empty_face(face: &Face) -> Result<Face, Failure> {
    topods::face(&face.clone().into_shape().empty_copied()?)
}

fn prepared_wire(
    plan: &MissingSeamPlan,
    wire: &Wire,
    first: &Wire,
    second: &Wire,
) -> Result<Wire, Failure> {
    if wire.is_same(&plan.first_wire) {
        return Ok(first.clone());
    }
    if wire.is_same(&plan.second_wire) {
        return Ok(second.clone());
    }
    reverse_outer_hole_if_needed(&plan.face, wire)
}

fn reverse_outer_hole_if_needed(face: &Face, wire: &Wire) -> Result<Wire, Failure> {
    let mut temporary = empty_face(face)?;
    Builder.add(&mut temporary, &wire.clone().into_shape())?;
    temporary.set_orientation(face.orientation());
    let mut oriented = wire.clone();
    if ShapeAnalysis::is_outer_bound(&temporary)? {
        oriented.reverse();
    }
    Ok(oriented)
}

fn align_periodic_wires(plan: &MissingSeamPlan, ordered: &mut OrderedWires) -> Result<(), Failure> {
    if !plan.u_closed || !plan.v_closed || is_degenerated_torus(plan) {
        return Ok(());
    }
    let (coord, mode, period) = periodic_axis(plan);
    let shift_second = second_wire_shift(ordered, coord, mode, period);
    expand_bounds_for_shift(ordered, coord, shift_second);
    for wire in direct_wires(&ordered.face)? {
        if wire.is_equal(&ordered.first) {
            continue;
        }
        let shift = if wire.is_equal(&ordered.second) {
            shift_second
        } else {
            shift_other_wire(&wire, ordered, coord, period)?
        };
        translate_wire_pcurves(&ordered.face, &wire, coord, shift)?;
    }
    Ok(())
}

fn periodic_axis(plan: &MissingSeamPlan) -> (usize, i8, f64) {
    if plan.direction.u != 0 {
        (1, plan.direction.u, plan.u_period)
    } else {
        (0, -plan.direction.v, plan.v_period)
    }
}

fn second_wire_shift(ordered: &OrderedWires, coord: usize, mode: i8, period: f64) -> f64 {
    let bounds_index = bounds_start(coord);
    let target = midpoint(&ordered.first_bounds, bounds_index)
        + mode as f64 * (period + rustycad_tkernel::precision::p_confusion());
    adjust_by_period(
        midpoint(&ordered.second_bounds, bounds_index),
        target,
        period,
    )
}

fn expand_bounds_for_shift(ordered: &mut OrderedWires, coord: usize, shift: f64) {
    let start = bounds_start(coord);
    ordered.first_bounds[start] =
        ordered.first_bounds[start].min(ordered.second_bounds[start] + shift);
    ordered.first_bounds[start + 1] =
        ordered.first_bounds[start + 1].max(ordered.second_bounds[start + 1] + shift);
}

fn direct_wires(face: &Face) -> Result<Vec<Wire>, Failure> {
    let mut wires = Vec::new();
    for child in ShapeIterator::new(&face.clone().into_shape(), false, true) {
        if child.shape_type()? == rustycad_tkg3d::top_abs::ShapeEnum::Wire {
            wires.push(topods::wire(&child)?);
        }
    }
    Ok(wires)
}

fn shift_other_wire(
    wire: &Wire,
    ordered: &OrderedWires,
    coord: usize,
    period: f64,
) -> Result<f64, Failure> {
    let bounds = wire_bounds(&ordered.face, wire)?;
    Ok(adjust_by_period(
        midpoint(&bounds, bounds_start(coord)),
        midpoint(&ordered.first_bounds, bounds_start(coord)),
        period,
    ))
}

fn translate_wire_pcurves(
    face: &Face,
    wire: &Wire,
    coord: usize,
    shift: f64,
) -> Result<(), Failure> {
    if shift == 0.0 {
        return Ok(());
    }
    let vector = if coord == 1 {
        Vector2::new(shift, 0.0)
    } else {
        Vector2::new(0.0, shift)
    };
    for child in ShapeIterator::new(&wire.clone().into_shape(), true, true) {
        translate_edge_pcurve(face, &topods::edge(&child)?, vector)?;
    }
    Ok(())
}

fn translate_edge_pcurve(
    face: &Face,
    edge: &rustycad_tkbrep::topods::Edge,
    vector: Vector2,
) -> Result<(), Failure> {
    let (curve, _, _) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
    if let Some(curve) = curve.get() {
        curve.translate(vector)?;
    }
    Ok(())
}

fn select_split_parameter(
    plan: &MissingSeamPlan,
    ordered: &OrderedWires,
) -> Result<Option<(f64, f64)>, Failure> {
    let mut u_split = plan.surface_bounds.0;
    let mut v_split = plan.surface_bounds.2;
    let coord = if plan.direction.u != 0 { 1 } else { 0 };
    let period = if coord == 1 {
        plan.u_period
    } else {
        plan.v_period
    };
    if plan.u_closed && plan.v_closed && !is_degenerated_torus(plan) {
        select_midpoint_split(plan, ordered, coord, period, &mut u_split, &mut v_split);
    }
    let Some((found_u, found_v)) =
        find_edge_split_candidates(plan, ordered, &mut u_split, &mut v_split)?
    else {
        return Ok(None);
    };
    let u_bounds = plan.surface_bounds;
    normalize_split(&mut u_split, u_bounds.0, u_bounds.1);
    normalize_split(&mut v_split, u_bounds.2, u_bounds.3);
    if plan.u_closed && found_u == 0 && plan.direction.u != 0 {
        u_split = plan.surface_bounds.0;
    }
    if plan.v_closed && found_v == 0 && plan.direction.v != 0 {
        v_split = plan.surface_bounds.2;
    }
    Ok(Some((u_split, v_split)))
}

fn bounds_start(coord: usize) -> usize {
    if coord == 1 { 0 } else { 2 }
}

fn select_midpoint_split(
    plan: &MissingSeamPlan,
    ordered: &OrderedWires,
    coord: usize,
    period: f64,
    u_split: &mut f64,
    v_split: &mut f64,
) {
    let lower = bounds_start(coord);
    let upper = lower + 1;
    let span = ordered.first_bounds[upper] - ordered.first_bounds[lower];
    if span > period {
        return;
    }
    let other = 0.5 * (ordered.first_bounds[lower] + ordered.first_bounds[upper] - period);
    if plan.direction.u != 0 {
        *v_split = other;
    } else {
        *u_split = other;
    }
}

fn find_edge_split_candidates(
    plan: &MissingSeamPlan,
    ordered: &OrderedWires,
    u_split: &mut f64,
    v_split: &mut f64,
) -> Result<Option<(u8, u8)>, Failure> {
    let first = ShapeExtendWireData::from_wire(&ordered.first, true, true)?;
    let second = ShapeExtendWireData::from_wire(&ordered.second, true, true)?;
    let mut found = (0, 0);
    for index in 1..=first.nb_edges() + second.nb_edges() {
        let (edge, in_first) = selected_edge(&first, &second, index)?;
        let Some(mut point) = edge_end_pcurve(&edge, &ordered.face, false)? else {
            return Ok(None);
        };
        align_candidate_to_period(plan, &mut point);
        let (skip_u, skip_v) = update_best_axis(plan, point, &mut found, u_split, v_split);
        if skip_u && skip_v {
            if in_first {
                continue;
            }
            break;
        }
        if in_first
            && !compare_other_wire_endpoints(
                plan, ordered, point, &second, &mut found, u_split, v_split,
            )?
        {
            return Ok(None);
        }
    }
    Ok(Some(found))
}

fn align_candidate_to_period(plan: &MissingSeamPlan, point: &mut rustycad_tkmath::gp::Point2) {
    if plan.u_closed && plan.direction.u != 0 {
        point.set_x(point.x() + adjust_by_period(point.x(), plan.surface_bounds.0, plan.u_period));
    }
    if plan.v_closed && plan.direction.u == 0 {
        point.set_y(point.y() + adjust_by_period(point.y(), plan.surface_bounds.2, plan.v_period));
    }
}

fn selected_edge(
    first: &ShapeExtendWireData,
    second: &ShapeExtendWireData,
    index: i32,
) -> Result<(rustycad_tkbrep::topods::Edge, bool), Failure> {
    if index <= first.nb_edges() {
        Ok((first.edge(index)?, true))
    } else {
        Ok((second.edge(index - first.nb_edges())?, false))
    }
}

fn edge_end_pcurve(
    edge: &rustycad_tkbrep::topods::Edge,
    face: &Face,
    first: bool,
) -> Result<Option<rustycad_tkmath::gp::Point2>, Failure> {
    let (curve, start, end) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
    let Some(curve) = curve.get() else {
        return Ok(None);
    };
    Ok(Some(curve.value(if first { start } else { end })?))
}

fn update_best_axis(
    plan: &MissingSeamPlan,
    point: rustycad_tkmath::gp::Point2,
    found: &mut (u8, u8),
    u_split: &mut f64,
    v_split: &mut f64,
) -> (bool, bool) {
    let mut skip_u = !plan.u_closed;
    let mut skip_v = !plan.v_closed;
    if plan.u_closed && plan.direction.u != 0 {
        skip_u = point.x().abs() > u_split.abs() && found.0 == 2;
        choose_nearest(point.x(), found, u_split, true);
    }
    if plan.v_closed && plan.direction.u == 0 {
        skip_v = point.y().abs() > v_split.abs() && found.1 == 2;
        choose_nearest(point.y(), found, v_split, false);
    }
    (skip_u, skip_v)
}

fn choose_nearest(value: f64, found: &mut (u8, u8), split: &mut f64, is_u: bool) {
    let rank = if is_u { &mut found.0 } else { &mut found.1 };
    if *rank == 2 && value.abs() > split.abs() {
        return;
    }
    if *rank == 0 || (*rank == 1 && value.abs() < split.abs()) {
        *rank = 1;
        *split = value;
    }
}

fn compare_other_wire_endpoints(
    plan: &MissingSeamPlan,
    ordered: &OrderedWires,
    first_point: rustycad_tkmath::gp::Point2,
    second: &ShapeExtendWireData,
    found: &mut (u8, u8),
    u_split: &mut f64,
    v_split: &mut f64,
) -> Result<bool, Failure> {
    for index in 1..=second.nb_edges() {
        let edge = second.edge(index)?;
        let Some(mut point) = edge_end_pcurve(&edge, &ordered.face, true)? else {
            return Ok(false);
        };
        update_matching_axis(plan, first_point, &mut point, found, u_split, v_split);
    }
    Ok(true)
}

fn update_matching_axis(
    plan: &MissingSeamPlan,
    first: rustycad_tkmath::gp::Point2,
    second: &mut rustycad_tkmath::gp::Point2,
    found: &mut (u8, u8),
    u_split: &mut f64,
    v_split: &mut f64,
) {
    if plan.u_closed && plan.direction.u != 0 {
        second.set_x(second.x() + adjust_by_period(second.x(), first.x(), plan.u_period));
        if (second.x() - first.x()).abs() < rustycad_tkernel::precision::p_confusion() {
            choose_coincident(first.x(), found, u_split, true);
        }
    }
    if plan.v_closed && plan.direction.u == 0 {
        second.set_y(second.y() + adjust_by_period(second.y(), first.y(), plan.v_period));
        if (second.y() - first.y()).abs() < rustycad_tkernel::precision::p_confusion() {
            choose_coincident(first.y(), found, v_split, false);
        }
    }
}

fn choose_coincident(value: f64, found: &mut (u8, u8), split: &mut f64, is_u: bool) {
    let rank = if is_u { &mut found.0 } else { &mut found.1 };
    if *rank != 2 || value.abs() < split.abs() {
        *rank = 2;
        *split = value;
    }
}

fn normalize_split(value: &mut f64, first: f64, last: f64) {
    if *value < first || *value > last {
        *value += crate::shape_analysis::adjust_to_period(*value, first, last);
    }
}
