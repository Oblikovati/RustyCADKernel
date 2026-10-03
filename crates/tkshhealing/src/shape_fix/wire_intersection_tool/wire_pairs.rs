//! Cross-wire intersections from `ShapeFix_IntersectionTool::FixIntersectingWires`.
use super::ShapeFixIntersectionTool;
use super::aggregate::points;
#[path = "wire_pairs/segments.rs"]
mod segments;
use super::boxes::{self, EdgeBox};
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) struct WireRecord {
    pub(super) data: ShapeExtendWireData,
    pub(super) boxes: Vec<EdgeBox>,
}

#[derive(Clone, Copy)]
pub(super) enum PairEffect {
    None,
    RetryFirst(i32),
    RetrySecond(i32),
    Replaced,
}

pub(super) fn fix_intersecting_wires(
    tool: &ShapeFixIntersectionTool,
    face: &mut Face,
) -> Result<bool, Failure> {
    if tool.context().is_null() || face.is_null() {
        return Ok(false);
    }
    let (mut wires, non_wires) = collect_face_children(face)?;
    if wires.len() < 2 {
        return Ok(false);
    }
    let mut max_tolerance = face_vertex_tolerance(face)?;
    let mut modified = false;
    for first in 0..wires.len() - 1 {
        for second in first + 1..wires.len() {
            modified |=
                process_wire_pair(tool, face, &mut wires, first, second, &mut max_tolerance)?;
        }
    }
    if modified {
        replace_face(tool, face, &wires, &non_wires)?;
    }
    Ok(modified)
}

fn collect_face_children(face: &Face) -> Result<(Vec<WireRecord>, Vec<Shape>), Failure> {
    let shape = face.clone().into_shape();
    let mut iterator = ShapeIterator::new(&shape, false, true);
    let mut wires = Vec::new();
    let mut non_wires = Vec::new();
    while iterator.more() {
        let child = iterator.value()?.clone();
        iterator.next();
        if !is_supported_wire(&child)? {
            non_wires.push(child);
            continue;
        }
        wires.push(make_wire_record(topods::wire(&child)?, face)?);
    }
    Ok((wires, non_wires))
}

fn is_supported_wire(shape: &Shape) -> Result<bool, Failure> {
    Ok(shape.shape_type()? == ShapeEnum::Wire
        && matches!(
            shape.orientation(),
            Orientation::Forward | Orientation::Reversed
        ))
}

fn make_wire_record(wire: Wire, face: &Face) -> Result<WireRecord, Failure> {
    let data = ShapeExtendWireData::from_wire(&wire, true, true)?;
    let boxes = boxes::create_edge_boxes(&data, face)?;
    Ok(WireRecord { data, boxes })
}

fn face_vertex_tolerance(face: &Face) -> Result<f64, Failure> {
    let vertices = rustycad_tkbrep::topexp::Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    )?;
    let mut maximum: f64 = 0.0;
    for shape in vertices {
        maximum = maximum.max(BRepTool::vertex_tolerance(&topods::vertex(&shape)?)?);
    }
    Ok(maximum)
}

fn process_wire_pair(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    wires: &mut [WireRecord],
    first_index: usize,
    second_index: usize,
    max_tolerance: &mut f64,
) -> Result<bool, Failure> {
    let (left, right) = wires.split_at_mut(second_index);
    let (first, second) = (&mut left[first_index], &mut right[0]);
    scan_edge_pairs(tool, face, first, second, max_tolerance)
}

fn scan_edge_pairs(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    first: &mut WireRecord,
    second: &mut WireRecord,
    max_tolerance: &mut f64,
) -> Result<bool, Failure> {
    let mut modifications = 0;
    let mut replaced = false;
    let mut first_index = 1;
    while first_index <= first.data.nb_edges() && modifications < 30 {
        let mut second_index = 1;
        while second_index <= second.data.nb_edges() && modifications < 30 {
            let effect = process_edge_pair(
                tool,
                face,
                first,
                second,
                first_index,
                &mut second_index,
                max_tolerance,
            )?;
            match effect {
                PairEffect::None => second_index += 1,
                PairEffect::RetryFirst(count) => {
                    modifications += count;
                    first_index -= 1;
                    break;
                }
                PairEffect::RetrySecond(count) => {
                    modifications += count;
                }
                PairEffect::Replaced => {
                    replaced = true;
                    second_index += 1;
                }
            }
        }
        first_index += 1;
    }
    Ok(modifications > 0 || replaced)
}

fn process_edge_pair(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    first: &mut WireRecord,
    second: &mut WireRecord,
    first_index: i32,
    second_index: &mut i32,
    max_tolerance: &mut f64,
) -> Result<PairEffect, Failure> {
    let edge1 = first.data.edge(first_index)?;
    let edge2 = second.data.edge(*second_index)?;
    if !edge_pair_may_intersect(&edge1, &edge2, &first.boxes, &second.boxes)? {
        return Ok(PairEffect::None);
    }
    let Some(curves) = points::pair_curves(&edge1, &edge2, face)? else {
        return Ok(PairEffect::None);
    };
    if !curves.intersections.is_done() {
        return Ok(PairEffect::None);
    }
    let segment_count = curves.intersections.nb_segments()?;
    if segment_count == 1 {
        let segment = curves.intersections.segment(1)?;
        if segment.has_first_point() && segment.has_last_point() {
            return segments::repair_cross_wire_segment(
                tool,
                face,
                first,
                second,
                first_index,
                *second_index,
                &edge1,
                &edge2,
                &curves,
                segment,
                max_tolerance,
            );
        }
    }
    let count = curves.intersections.nb_points()?;
    if count == 0 || count >= 3 {
        return Ok(PairEffect::None);
    }
    repair_cross_wire_point(
        tool,
        face,
        first,
        second,
        first_index,
        second_index,
        &edge1,
        &edge2,
        &curves,
        points::selected_point(&curves.intersections, count)?,
        max_tolerance,
    )
}

fn edge_pair_may_intersect(
    first: &rustycad_tkbrep::topods::Edge,
    second: &rustycad_tkbrep::topods::Edge,
    first_boxes: &[EdgeBox],
    second_boxes: &[EdgeBox],
) -> Result<bool, Failure> {
    if first.is_same(second) || BRepTool::degenerated(first)? || BRepTool::degenerated(second)? {
        return Ok(false);
    }
    let (Some(first_box), Some(second_box)) = (
        boxes::find_edge_box(first_boxes, first),
        boxes::find_edge_box(second_boxes, second),
    ) else {
        return Ok(false);
    };
    Ok(!first_box.is_out_box(&second_box))
}

#[allow(clippy::too_many_arguments)]
fn repair_cross_wire_point(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    first: &mut WireRecord,
    second: &mut WireRecord,
    first_index: i32,
    second_index: &mut i32,
    edge1: &rustycad_tkbrep::topods::Edge,
    edge2: &rustycad_tkbrep::topods::Edge,
    curves: &super::aggregate::PairCurves,
    point: rustycad_tkgeomalgo::int_res2d::IntersectionPoint,
    max_tolerance: &mut f64,
) -> Result<PairEffect, Failure> {
    use rustycad_tkgeomalgo::int_res2d::Position;
    let positions = [
        point.transition_of_first().position_on_curve(),
        point.transition_of_second().position_on_curve(),
    ];
    match positions {
        [Position::Middle, Position::Middle] => split_both_edges(
            tool,
            face,
            first,
            second,
            first_index,
            second_index,
            edge1,
            edge2,
            &point,
            curves,
            max_tolerance,
        ),
        [Position::Middle, _] => split_at_other_wire_vertex(
            tool,
            face,
            first,
            edge1,
            edge2,
            first_index,
            point.param_on_first(),
            &curves.pcurves[0],
            max_tolerance,
            true,
        ),
        [_, Position::Middle] => split_at_other_wire_vertex(
            tool,
            face,
            second,
            edge2,
            edge1,
            *second_index,
            point.param_on_second(),
            &curves.pcurves[1],
            max_tolerance,
            false,
        ),
        _ => union_cross_wire_vertices(tool, first, second, edge1, edge2, *second_index),
    }
}

#[allow(clippy::too_many_arguments)] // Mirrors native FindVertAndSplitEdge input state.
fn split_at_other_wire_vertex(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    wire: &mut WireRecord,
    edge: &rustycad_tkbrep::topods::Edge,
    other: &rustycad_tkbrep::topods::Edge,
    index: i32,
    parameter: f64,
    pcurve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    max_tolerance: &mut f64,
    retry_first: bool,
) -> Result<PairEffect, Failure> {
    let mut mutable_index = index;
    let split = tool.find_vertex_and_split_edge(
        parameter,
        edge,
        other,
        pcurve,
        max_tolerance,
        &mut mutable_index,
        &wire.data,
        face,
        &mut wire.boxes,
        true,
    )?;
    Ok(if split && retry_first {
        PairEffect::RetryFirst(1)
    } else if split {
        PairEffect::RetrySecond(1)
    } else {
        PairEffect::None
    })
}

#[allow(clippy::too_many_arguments)] // Mirrors native SplitEdge1 calls for each wire.
fn split_both_edges(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    first: &mut WireRecord,
    second: &mut WireRecord,
    first_index: i32,
    second_index: &mut i32,
    edge1: &rustycad_tkbrep::topods::Edge,
    edge2: &rustycad_tkbrep::topods::Edge,
    point: &rustycad_tkgeomalgo::int_res2d::IntersectionPoint,
    curves: &super::aggregate::PairCurves,
    max_tolerance: &mut f64,
) -> Result<PairEffect, Failure> {
    let first_point =
        super::point_on_edge(edge1, point.param_on_first(), &curves.pcurves[0], face)?;
    let second_point =
        super::point_on_edge(edge2, point.param_on_second(), &curves.pcurves[1], face)?;
    let midpoint = rustycad_tkmath::gp::Point3::new(
        (first_point.x() + second_point.x()) * 0.5,
        (first_point.y() + second_point.y()) * 0.5,
        (first_point.z() + second_point.z()) * 0.5,
    );
    let tolerance = (first_point.distance(&second_point) * 0.5 * 1.00001)
        .max(rustycad_tkernel::precision::confusion());
    let mut vertex = topods::Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, midpoint, tolerance)?;
    *max_tolerance = max_tolerance.max(tolerance);
    let second_split = tool.split_edge1(
        &second.data,
        face,
        *second_index,
        point.param_on_second(),
        &vertex,
        tolerance,
        &mut second.boxes,
    )?;
    let first_split = tool.split_edge1(
        &first.data,
        face,
        first_index,
        point.param_on_first(),
        &vertex,
        tolerance,
        &mut first.boxes,
    )?;
    if first_split {
        return Ok(PairEffect::RetryFirst(i32::from(second_split) + 1));
    }
    Ok(if second_split {
        PairEffect::RetrySecond(1)
    } else {
        PairEffect::None
    })
}

fn union_cross_wire_vertices(
    tool: &ShapeFixIntersectionTool,
    first: &mut WireRecord,
    second: &mut WireRecord,
    edge1: &rustycad_tkbrep::topods::Edge,
    edge2: &rustycad_tkbrep::topods::Edge,
    second_index: i32,
) -> Result<PairEffect, Failure> {
    let Some(second_box) = boxes::find_edge_box(&second.boxes, edge2) else {
        return Ok(PairEffect::None);
    };
    let replaced = tool.union_vertices(
        &second.data,
        edge1,
        &mut edge2.clone(),
        second_index,
        &mut second.boxes,
        &second_box,
    )?;
    let _ = first;
    Ok(if replaced {
        PairEffect::Replaced
    } else {
        PairEffect::None
    })
}

fn replace_face(
    tool: &ShapeFixIntersectionTool,
    face: &mut Face,
    wires: &[WireRecord],
    non_wires: &[Shape],
) -> Result<(), Failure> {
    let original_orientation = face.orientation();
    let mut replacement = face.clone().into_shape().empty_copied()?;
    replacement.set_orientation(Orientation::Forward);
    for wire in wires {
        BRepBuilder::default().add(&mut replacement, &wire.data.wire()?.into_shape())?;
    }
    for child in non_wires {
        BRepBuilder::default().add(&mut replacement, child)?;
    }
    replacement.set_orientation(original_orientation);
    tool.context()
        .replace(&face.clone().into_shape(), &replacement)?;
    *face = topods::face(&replacement)?;
    Ok(())
}

#[cfg(test)]
#[path = "wire_pairs_tests.rs"]
mod tests;
