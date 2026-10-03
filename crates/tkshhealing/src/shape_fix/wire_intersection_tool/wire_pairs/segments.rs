//! Short cross-wire overlap repair branches from FixIntersectingWires.
use super::{PairEffect, ShapeFixIntersectionTool, WireRecord};
use crate::shape_fix::wire::wire_intersection_tool::aggregate::PairCurves;
use crate::shape_fix::wire::wire_intersection_tool::aggregate::segment::{
    self, SegmentPoints, SelectedEndpoint,
};
use crate::shape_fix::wire::wire_intersection_tool::point_on_edge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::standard::Failure;
#[allow(clippy::too_many_arguments)]
pub(super) fn repair_cross_wire_segment(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    first: &mut WireRecord,
    second: &mut WireRecord,
    first_index: i32,
    second_index: i32,
    edge1: &rustycad_tkbrep::topods::Edge,
    edge2: &rustycad_tkbrep::topods::Edge,
    curves: &PairCurves,
    segment: rustycad_tkgeomalgo::int_res2d::IntersectionSegment,
    max_tolerance: &mut f64,
) -> Result<PairEffect, Failure> {
    let samples = segment::segment_points(segment, edge1, edge2, face, curves)?;
    if segment::separated_overlap(&samples, *max_tolerance) {
        return Ok(PairEffect::None);
    }
    let first_cut = cut_overlap_endpoint(
        tool,
        face,
        edge1,
        curves.ranges[0],
        samples.first_parameters,
        &samples.first,
        *max_tolerance,
    )?;
    let second_cut = cut_overlap_endpoint(
        tool,
        face,
        edge2,
        curves.ranges[1],
        samples.second_parameters,
        &samples.second,
        *max_tolerance,
    )?;
    if first_cut || second_cut {
        return Ok(PairEffect::None);
    }
    split_short_overlap(
        tool,
        face,
        first,
        second,
        first_index,
        second_index,
        edge1,
        edge2,
        curves,
        &samples,
        max_tolerance,
    )
}

fn cut_overlap_endpoint(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    edge: &rustycad_tkbrep::topods::Edge,
    range: (f64, f64),
    parameters: (f64, f64),
    overlap_points: &[rustycad_tkmath::gp::Point3; 2],
    max_tolerance: f64,
) -> Result<bool, Failure> {
    let selected =
        segment::select_existing_endpoint(edge, range, parameters, overlap_points, max_tolerance)?;
    cut_selected_overlap_endpoint(tool, face, edge, range, parameters, selected)
}

fn cut_selected_overlap_endpoint(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    edge: &rustycad_tkbrep::topods::Edge,
    range: (f64, f64),
    parameters: (f64, f64),
    selected: SelectedEndpoint,
) -> Result<bool, Failure> {
    if !selected.found {
        return Ok(false);
    }
    let endpoint = farther_overlap_range(range, parameters);
    let cut = farther_overlap_point(endpoint, parameters);
    let mut cut_line = false;
    if !tool.cut_edge(edge, endpoint, cut, face, &mut cut_line)? {
        return Ok(false);
    }
    let old_tolerance = BRepTool::vertex_tolerance(&selected.vertex)?;
    if selected.tolerance > old_tolerance {
        BRepBuilder::default()
            .update_vertex_tolerance(&selected.vertex, selected.tolerance * 1.00001)?;
    }
    Ok(true)
}

fn farther_overlap_range(range: (f64, f64), overlap: (f64, f64)) -> f64 {
    let first_distance = (range.0 - overlap.0).abs() + (range.0 - overlap.1).abs();
    let last_distance = (range.1 - overlap.0).abs() + (range.1 - overlap.1).abs();
    if first_distance > last_distance {
        range.0
    } else {
        range.1
    }
}

fn farther_overlap_point(endpoint: f64, overlap: (f64, f64)) -> f64 {
    if (endpoint - overlap.0).abs() > (endpoint - overlap.1).abs() {
        overlap.1
    } else {
        overlap.0
    }
}

#[allow(clippy::too_many_arguments)]
fn split_short_overlap(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    first: &mut WireRecord,
    second: &mut WireRecord,
    first_index: i32,
    second_index: i32,
    edge1: &rustycad_tkbrep::topods::Edge,
    edge2: &rustycad_tkbrep::topods::Edge,
    curves: &PairCurves,
    points: &SegmentPoints,
    max_tolerance: &mut f64,
) -> Result<PairEffect, Failure> {
    let first_span = (points.first_parameters.1 - points.first_parameters.0).abs();
    let second_span = (points.second_parameters.1 - points.second_parameters.0).abs();
    let first_range = BRepTool::range_on_face(edge1, face)?;
    let second_range = BRepTool::range_on_face(edge2, face)?;
    let first_is_long = first_span > (first_range.1 - first_range.0).abs() * 0.5;
    let second_is_long = second_span > (second_range.1 - second_range.0).abs() * 0.5;
    if first_is_long && second_is_long {
        return super::long_overlap::replace_full_second_overlap(
            tool,
            face,
            first,
            second,
            first_index,
            second_index,
            edge2,
            points,
            max_tolerance,
        );
    }
    if first_is_long || second_is_long {
        return Ok(PairEffect::None);
    }
    let first_middle = point_on_edge(
        edge1,
        0.5 * (points.first_parameters.0 + points.first_parameters.1),
        &curves.pcurves[0],
        face,
    )?;
    let second_middle = point_on_edge(
        edge2,
        0.5 * (points.second_parameters.0 + points.second_parameters.1),
        &curves.pcurves[1],
        face,
    )?;
    let tolerance = short_overlap_tolerance(points, first_middle, second_middle);
    let mut vertex = topods::Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, first_middle, tolerance)?;
    *max_tolerance = max_tolerance.max(tolerance);
    let second_split = tool.split_edge2(
        &second.data,
        face,
        second_index,
        points.second_parameters,
        &vertex,
        tolerance,
        &mut second.boxes,
    )?;
    let first_split = tool.split_edge2(
        &first.data,
        face,
        first_index,
        points.first_parameters,
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

fn short_overlap_tolerance(
    samples: &SegmentPoints,
    first_middle: rustycad_tkmath::gp::Point3,
    second_middle: rustycad_tkmath::gp::Point3,
) -> f64 {
    let overlap_middle = rustycad_tkmath::gp::Point3::new(
        0.5 * (samples.first[0].x() + samples.first[1].x()),
        0.5 * (samples.first[0].y() + samples.first[1].y()),
        0.5 * (samples.first[0].z() + samples.first[1].z()),
    );
    let first_distance = samples.first[0]
        .distance(&overlap_middle)
        .max(samples.first[1].distance(&first_middle));
    let second_distance = samples.second[0]
        .distance(&overlap_middle)
        .max(samples.second[1].distance(&first_middle));
    first_distance
        .max(second_distance)
        .max(first_middle.distance(&second_middle))
        * 1.00001
}
