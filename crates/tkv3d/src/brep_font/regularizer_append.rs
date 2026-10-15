//! `BRepFont_Regularizer::appendOutline`: copies the outline contours into the vertex,
//! curve, use and loop pools, healing endpoints within the tolerance and dropping
//! point-like segments.
use super::planar_region::{BRepFontCurveId, BRepFontPlanarRegion, BRepFontPlanarRegionLoop};
use super::planar_region::{BRepFontPlanarRegionUse, BRepFontVertexId};
use super::planar_region_curve::BRepFontPlanarRegionCurve;
use super::regularizer::{BRepFontRegularizerStatus as Status, is_valid_real};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Xy;
use rustycad_tkservice::font::{FontGlyphOutline, FontGlyphOutlineContour};
use rustycad_tkservice::font::{
    FontGlyphOutlineSegment as Segment, FontGlyphOutlineSegmentKind as Kind,
};

/// `index`: a size as a compact `u32` index, or InvalidOutline when it does not fit.
pub(super) fn index(value: usize) -> Result<u32, Status> {
    u32::try_from(value).map_err(|_| Status::InvalidOutline)
}

/// The running state of one contour while its segments are appended.
struct ContourAppend {
    first_vertex: u32,
    previous_vertex: u32,
    first_use: u32,
    signed_area: f64,
    bounds: BoundingBox2,
}

/// `appendOutline`: fills the pools of `region`; the error is the failure status.
pub(super) fn append_outline(
    outline: &FontGlyphOutline,
    tolerance: f64,
    region: &mut BRepFontPlanarRegion,
) -> Result<(), Status> {
    let (segments, contours) = (outline.segments(), outline.contours());
    index(segments.len())?;
    index(contours.len())?;
    region.vertices.reserve(segments.len() + contours.len());
    region.curves.reserve(segments.len());
    region.uses.reserve(segments.len());
    region.loops.reserve(contours.len());
    let square_tolerance = tolerance * tolerance;
    let mut expected_first: u64 = 0;
    for contour in contours {
        expected_first = check_contour_range(contour, expected_first, segments.len(), region)?;
        append_contour(outline, contour, square_tolerance, region)?;
    }
    if expected_first != segments.len() as u64 {
        return Err(Status::InvalidOutline);
    }
    Ok(())
}

// Contours must be packed, non-empty and inside the segment array; returns the next start.
fn check_contour_range(
    contour: &FontGlyphOutlineContour,
    expected_first: u64,
    nb_segments: usize,
    region: &BRepFontPlanarRegion,
) -> Result<u64, Status> {
    let first = u64::from(contour.first_segment());
    let last = first + u64::from(contour.nb_segments());
    if first != expected_first || contour.nb_segments() == 0 || last > nb_segments as u64 {
        return Err(Status::InvalidOutline);
    }
    index(region.uses.len())?;
    Ok(last)
}

fn append_contour(
    outline: &FontGlyphOutline,
    contour: &FontGlyphOutlineContour,
    square_tolerance: f64,
    region: &mut BRepFontPlanarRegion,
) -> Result<(), Status> {
    let segments = outline.segments();
    let first_segment = &segments[contour.first_segment() as usize];
    let first_vertex = index(region.vertices.len())?;
    region.vertices.append(first_segment.start());
    let mut state = ContourAppend {
        first_vertex,
        previous_vertex: first_vertex,
        first_use: index(region.uses.len())?,
        signed_area: 0.0,
        bounds: BoundingBox2::default(),
    };
    let nb = contour.nb_segments() as usize;
    for local in 0..nb {
        let segment = &segments[contour.first_segment() as usize + local];
        let is_last_source = local + 1 == nb;
        append_segment(
            outline,
            segment,
            is_last_source,
            square_tolerance,
            region,
            &mut state,
        )?;
    }
    close_contour(outline, square_tolerance, region, state)
}

fn has_valid_points(segment: &Segment) -> bool {
    let valid = |point: Xy| is_valid_real(point.x()) && is_valid_real(point.y());
    valid(segment.start())
        && valid(segment.end())
        && (segment.kind() == Kind::Line || valid(segment.control1()))
        && (segment.kind() != Kind::CubicBezier || valid(segment.control2()))
}

fn append_segment(
    outline: &FontGlyphOutline,
    segment: &Segment,
    is_last_source: bool,
    square_tolerance: f64,
    region: &mut BRepFontPlanarRegion,
    state: &mut ContourAppend,
) -> Result<(), Status> {
    if !has_valid_points(segment) {
        return Err(Status::InvalidOutline);
    }
    let expected_start = region.vertices.as_slice()[state.previous_vertex as usize];
    if (segment.start() - expected_start).square_modulus() > square_tolerance {
        return Err(Status::OpenContour);
    }
    let keeps_end = !is_last_source || outline.is_single_stroke();
    let healed_end = if keeps_end {
        segment.end()
    } else {
        region.vertices.as_slice()[state.first_vertex as usize]
    };
    if !keeps_end && (segment.end() - healed_end).square_modulus() > square_tolerance {
        return Err(Status::OpenContour);
    }
    if is_point_like(segment, expected_start, square_tolerance) {
        return Ok(());
    }
    let mut last_vertex = state.first_vertex;
    if keeps_end {
        last_vertex = index(region.vertices.len())?;
        region.vertices.append(healed_end);
    }
    let curve_index = index(region.curves.len())?;
    let curve =
        BRepFontPlanarRegionCurve::new(&heal_endpoints(segment, expected_start, healed_end));
    region.curves.append(curve);
    region.uses.append(BRepFontPlanarRegionUse::forward(
        BRepFontCurveId::new(curve_index),
        BRepFontVertexId::new(state.previous_vertex),
        BRepFontVertexId::new(last_vertex),
    ));
    accumulate_curve(&curve, state)?;
    state.previous_vertex = last_vertex;
    Ok(())
}

fn accumulate_curve(
    curve: &BRepFontPlanarRegionCurve,
    state: &mut ContourAppend,
) -> Result<(), Status> {
    let curve_area = curve.signed_area();
    if !is_valid_real(curve_area) || !is_valid_real(state.signed_area + curve_area) {
        return Err(Status::InvalidOutline);
    }
    state.signed_area += curve_area;
    state.bounds.add_box(&curve.bounds());
    Ok(())
}

// Closes a filled contour on its first vertex, then appends the loop.
fn close_contour(
    outline: &FontGlyphOutline,
    square_tolerance: f64,
    region: &mut BRepFontPlanarRegion,
    mut state: ContourAppend,
) -> Result<(), Status> {
    let after_last_use = index(region.uses.len())?;
    let nb_uses = after_last_use - state.first_use;
    if nb_uses == 0 {
        region.vertices.erase_last();
        return Ok(());
    }
    if !outline.is_single_stroke() {
        let last_use = region.uses.as_slice()[after_last_use as usize - 1];
        if last_use.last_vertex().value() != state.first_vertex {
            reattach_last_use(region, after_last_use, square_tolerance, &mut state)?;
        }
    }
    if !outline.is_single_stroke() && state.signed_area.abs() <= square_tolerance {
        return Err(Status::InvalidOutline);
    }
    region.loops.append(BRepFontPlanarRegionLoop::new(
        state.first_use,
        nb_uses,
        state.signed_area,
        state.bounds,
    ));
    Ok(())
}

// The last kept segment ended on a vertex of its own (the closing segment was point-like):
// move its end onto the first vertex, drop that redundant vertex and rebuild the bounds.
fn reattach_last_use(
    region: &mut BRepFontPlanarRegion,
    after_last_use: u32,
    square_tolerance: f64,
    state: &mut ContourAppend,
) -> Result<(), Status> {
    let last_use = region.uses.as_slice()[after_last_use as usize - 1];
    let last_curve = last_use.curve_index().value();
    let redundant_vertex = last_use.last_vertex().value();
    if last_curve as usize >= region.curves.len()
        || u64::from(redundant_vertex) + 1 != region.vertices.len() as u64
    {
        return Err(Status::InvalidOutline);
    }
    let old_curve = region.curves.as_slice()[last_curve as usize];
    let first_point = region.vertices.as_slice()[state.first_vertex as usize];
    if (*old_curve.end() - first_point).square_modulus() > square_tolerance {
        return Err(Status::OpenContour);
    }
    let old_area = old_curve.signed_area();
    let healed = old_curve.with_endpoints(*old_curve.start(), first_point);
    region.curves.as_mut_slice()[last_curve as usize] = healed;
    let healed_area = healed.signed_area();
    if !is_valid_real(old_area)
        || !is_valid_real(healed_area)
        || !is_valid_real(state.signed_area + healed_area - old_area)
    {
        return Err(Status::InvalidOutline);
    }
    state.signed_area += healed_area - old_area;
    region.uses.as_mut_slice()[after_last_use as usize - 1] = BRepFontPlanarRegionUse::new(
        *last_use.curve_index(),
        *last_use.first_vertex(),
        BRepFontVertexId::new(state.first_vertex),
        last_use.first_parameter(),
        last_use.last_parameter(),
        last_use.is_reversed(),
    );
    region.vertices.erase_last();
    recompute_bounds(region, state, after_last_use)
}

fn recompute_bounds(
    region: &BRepFontPlanarRegion,
    state: &mut ContourAppend,
    after_last_use: u32,
) -> Result<(), Status> {
    state.bounds.set_void();
    for use_index in state.first_use..after_last_use {
        let curve_index = region.uses.as_slice()[use_index as usize]
            .curve_index()
            .value();
        let Some(curve) = region.curves.as_slice().get(curve_index as usize) else {
            return Err(Status::InvalidOutline);
        };
        state.bounds.add_box(&curve.bounds());
    }
    Ok(())
}

/// `isPointLike`: every defining point of the segment lies within the tolerance of `point`.
fn is_point_like(segment: &Segment, point: Xy, square_tolerance: f64) -> bool {
    let near = |other: Xy| (other - point).square_modulus() <= square_tolerance;
    if !near(segment.start()) || !near(segment.end()) {
        return false;
    }
    if segment.kind() != Kind::Line && !near(segment.control1()) {
        return false;
    }
    segment.kind() != Kind::CubicBezier || near(segment.control2())
}

/// `healEndpoints`: the same segment kind and controls on new endpoints.
fn heal_endpoints(segment: &Segment, start: Xy, end: Xy) -> Segment {
    match segment.kind() {
        Kind::Line => Segment::line(start, end),
        Kind::QuadraticBezier => Segment::quadratic(start, segment.control1(), end),
        Kind::CubicBezier => Segment::cubic(start, segment.control1(), segment.control2(), end),
    }
}
