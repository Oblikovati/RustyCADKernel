//! Nonadjacent wire edge intersections; currently exercised for point intersections.
#[path = "aggregate/segment.rs"]
mod segment;
use super::{ShapeFixIntersectionTool, boxes};
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom2d_int::GInter;

const INTERSECTION_TOLERANCE: f64 = 1.0e-10;

#[derive(Clone, Copy)]
pub(super) enum PairAction {
    Next,
    RetryFirst,
    RetrySecond,
    Abort,
}

pub(super) struct RepairCounts<'a> {
    pub(super) split: &'a mut i32,
    pub(super) cut: &'a mut i32,
    pub(super) removed: &'a mut i32,
    pub(super) replaced: &'a mut i32,
}

pub(super) struct PairCurves {
    pub(super) pcurves:
        [rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>; 2],
    pub(super) ranges: [(f64, f64); 2],
    pub(super) intersections: GInter,
}

pub(super) fn fix_self_intersect_wire(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    split_count: &mut i32,
    cut_count: &mut i32,
    removed_count: &mut i32,
) -> Result<bool, Failure> {
    if tool.context().is_null() || face.is_null() {
        return Ok(false);
    }
    *split_count = 0;
    *cut_count = 0;
    let mut max_vertex_tolerance = initial_vertex_tolerance(tool, face)?;
    let mut boxes = boxes::create_edge_boxes(wire, face)?;
    let mut replaced = 0;
    let completed = scan_nonadjacent_pairs(
        tool,
        wire,
        face,
        &mut boxes,
        &mut max_vertex_tolerance,
        &mut RepairCounts {
            split: split_count,
            cut: cut_count,
            removed: removed_count,
            replaced: &mut replaced,
        },
    )?;
    Ok(completed && (*split_count > 0 || *cut_count > 0 || replaced > 0 || *removed_count > 0))
}

fn initial_vertex_tolerance(tool: &ShapeFixIntersectionTool, face: &Face) -> Result<f64, Failure> {
    let applied = tool
        .context()
        .apply(&face.clone().into_shape(), ShapeEnum::Shape)?;
    let vertices = Explorer::new(&applied, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    let mut maximum: f64 = 0.0;
    for shape in vertices {
        let vertex = topods::vertex(&shape)?;
        maximum = maximum.max(BRepTool::vertex_tolerance(&vertex)?);
    }
    Ok(maximum.min(tool.max_tolerance()))
}

fn scan_nonadjacent_pairs(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    boxes: &mut Vec<boxes::EdgeBox>,
    max_vertex_tolerance: &mut f64,
    counts: &mut RepairCounts<'_>,
) -> Result<bool, Failure> {
    let mut first_index = 1;
    while first_index < wire.nb_edges() && *counts.split < 30 {
        let mut second_index = first_index + 2;
        while second_index <= wire.nb_edges() && *counts.split < 30 {
            let action = process_pair(
                tool,
                wire,
                face,
                first_index,
                second_index,
                boxes,
                max_vertex_tolerance,
                counts,
            )?;
            if matches!(action, PairAction::Abort) {
                return Ok(false);
            }
            if matches!(action, PairAction::RetryFirst) {
                first_index -= 1;
                break;
            }
            points::apply_pair_action(action, &mut first_index, &mut second_index);
        }
        first_index += 1;
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn process_pair(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    first_index: i32,
    second_index: i32,
    boxes: &mut Vec<boxes::EdgeBox>,
    max_vertex_tolerance: &mut f64,
    counts: &mut RepairCounts<'_>,
) -> Result<PairAction, Failure> {
    if first_index == 1 && second_index == wire.nb_edges() {
        return Ok(PairAction::Next);
    }
    let (first, second) = (wire.edge(first_index)?, wire.edge(second_index)?);
    if first.is_same(&second) || BRepTool::degenerated(&first)? || BRepTool::degenerated(&second)? {
        return Ok(PairAction::Next);
    }
    let (Some(first_box), Some(second_box)) = (
        boxes::find_edge_box(boxes, &first),
        boxes::find_edge_box(boxes, &second),
    ) else {
        return Ok(PairAction::Next);
    };
    if !first_box.intersects(&second_box) {
        return Ok(PairAction::Next);
    }
    let Some(curves) = points::pair_curves(&first, &second, face)? else {
        return Ok(PairAction::Abort);
    };
    if !curves.intersections.is_done() {
        return Ok(PairAction::Next);
    }
    let point_count = curves.intersections.nb_points()?;
    let segment_count = curves.intersections.nb_segments()?;
    if point_count > 0 && point_count < 3 {
        let candidate = points::selected_point(&curves.intersections, point_count)?;
        return points::repair_point(
            tool,
            wire,
            face,
            first_index,
            second_index,
            &first,
            &second,
            &first_box,
            &curves,
            candidate,
            max_vertex_tolerance,
            counts,
            boxes,
        );
    }
    if segment_count == 1 {
        let segment = curves.intersections.segment(1)?;
        if segment.has_first_point() && segment.has_last_point() {
            return segment::repair(
                tool,
                wire,
                face,
                first_index,
                second_index,
                &first,
                &second,
                &first_box,
                &second_box,
                &curves,
                segment,
                max_vertex_tolerance,
                counts,
                boxes,
            );
        }
    }
    Ok(PairAction::Next)
}

#[path = "aggregate/points.rs"]
mod points;

#[cfg(test)]
#[path = "aggregate/tests.rs"]
mod tests;
