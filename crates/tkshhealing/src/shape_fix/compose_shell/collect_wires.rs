//! Native `CollectWires` segment extraction, greedy closure and short-loop merge.
mod chain;
mod patch;
mod short_segments;

use super::ShapeFixComposeShell;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_extend::ShapeExtendCompositeSurface;
use crate::shape_fix::ShapeFixWireSegment;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Vector2};

const TOLINT: f64 = 1.0e-10;

pub(super) fn collect_wires(
    composer: &ShapeFixComposeShell,
    loops: &mut Sequence<ShapeFixWireSegment>,
    segments: &mut Sequence<ShapeFixWireSegment>,
) -> Result<(), Failure> {
    let (grid, face, location, resolution) = composer.collection_inputs();
    let shortness = classify_segments(segments, loops, &grid, &face, &location, resolution)?;
    chain::collect_chains(composer, segments, loops, &grid, &face, &shortness)?;
    merge_short_segments(segments, loops, &grid, &face, &shortness)
}

fn classify_segments(
    segments: &mut Sequence<ShapeFixWireSegment>,
    loops: &mut Sequence<ShapeFixWireSegment>,
    grid: &Handle<ShapeExtendCompositeSurface>,
    face: &rustycad_tkbrep::topods::Face,
    location: &rustycad_tkmath::toploc::Location,
    resolution: (f64, f64),
) -> Result<Vec<i32>, Failure> {
    let mut shortness = Vec::new();
    let mut index = 1;
    while index <= segments.len() {
        let segment = segments.value(index)?.clone();
        if move_special_segment(segments, loops, index, &segment)? {
            continue;
        }
        let short =
            short_segments::classify_short_segment(&segment, face, grid, location, resolution)?;
        shortness.push(short);
        mark_ignored_short(segments, index, short)?;
        index += 1;
    }
    Ok(shortness)
}

fn move_special_segment(
    segments: &mut Sequence<ShapeFixWireSegment>,
    loops: &mut Sequence<ShapeFixWireSegment>,
    index: usize,
    segment: &ShapeFixWireSegment,
) -> Result<bool, Failure> {
    if segment.is_vertex() || segment.orientation() == Orientation::Internal {
        loops.append(segment.clone());
        segments.remove(index)?;
        return Ok(true);
    }
    Ok(false)
}

fn mark_ignored_short(
    segments: &mut Sequence<ShapeFixWireSegment>,
    index: usize,
    short: i32,
) -> Result<(), Failure> {
    let segment = segments.value(index)?.clone();
    if short <= 0 || !is_ignored_orientation(&segment)? {
        return Ok(());
    }
    segments
        .change_value(index)?
        .set_orientation(Orientation::Internal);
    Ok(())
}

fn is_ignored_orientation(segment: &ShapeFixWireSegment) -> Result<bool, Failure> {
    if segment.orientation() == Orientation::External {
        return Ok(true);
    }
    if segment.nb_edges() != 1 {
        return Ok(false);
    }
    BRepTool::degenerated(&segment.edge(1)?)
}

fn merge_short_segments(
    segments: &Sequence<ShapeFixWireSegment>,
    loops: &mut Sequence<ShapeFixWireSegment>,
    grid: &Handle<ShapeExtendCompositeSurface>,
    face: &rustycad_tkbrep::topods::Face,
    shortness: &[i32],
) -> Result<(), Failure> {
    for index in 1..=segments.length() as usize {
        merge_one_short_segment(segments, loops, grid, face, shortness, index)?;
    }
    Ok(())
}

fn merge_one_short_segment(
    segments: &Sequence<ShapeFixWireSegment>,
    loops: &mut Sequence<ShapeFixWireSegment>,
    grid: &Handle<ShapeExtendCompositeSurface>,
    face: &rustycad_tkbrep::topods::Face,
    shortness: &[i32],
    index: usize,
) -> Result<(), Failure> {
    let segment = segments.value(index)?.clone();
    if shortness.get(index - 1) != Some(&1) || !eligible_short(&segment) {
        return Ok(());
    }
    let candidate = find_short_attachment(&segment, loops, face, grid)?;
    if let Some((wire_index, edge_index)) = candidate {
        append_short_edges(&segment, loops.value(wire_index)?, edge_index)?;
        return Ok(());
    }
    loops.append(ShapeFixWireSegment::from_wire_data(
        segment.wire_data(),
        Orientation::Forward,
    )?);
    Ok(())
}

fn eligible_short(segment: &ShapeFixWireSegment) -> bool {
    !segment.is_vertex()
        && segment.orientation() != Orientation::Internal
        && segment.orientation() != Orientation::External
}

fn find_short_attachment(
    segment: &ShapeFixWireSegment,
    loops: &Sequence<ShapeFixWireSegment>,
    face: &rustycad_tkbrep::topods::Face,
    grid: &Handle<ShapeExtendCompositeSurface>,
) -> Result<Option<(usize, i32)>, Failure> {
    let vertex = segment.first_vertex()?;
    let (point, _) = tangent_on_edge(&segment.edge(1)?, face, false, 0.0)?;
    let patch = segment.patch_index(1)?;
    let mut best = ShortAttachment::default();
    for wire_index in 1..=loops.length() as usize {
        scan_attachment_wire(
            loops, wire_index, face, &vertex, point, patch, grid, &mut best,
        )?;
    }
    Ok(best.found().then_some((best.wire_index, best.edge_index)))
}

#[allow(clippy::too_many_arguments)]
fn scan_attachment_wire(
    loops: &Sequence<ShapeFixWireSegment>,
    wire_index: usize,
    face: &rustycad_tkbrep::topods::Face,
    vertex: &rustycad_tkbrep::topods::Vertex,
    point: Point2,
    patch: crate::shape_fix::WirePatchIndex,
    grid: &Handle<ShapeExtendCompositeSurface>,
    best: &mut ShortAttachment,
) -> Result<(), Failure> {
    let wire = loops.value(wire_index)?;
    for edge_index in 1..=wire.nb_edges() {
        let edge = wire.edge(edge_index)?;
        if !vertex.is_same(&ShapeAnalysisEdge::new().first_vertex(&edge)) {
            continue;
        }
        let same_patch = patch::extend_same_patch(
            wire,
            &mut patch.clone(),
            grid.get().ok_or_else(null_grid)?.nb_u_patches()?,
            grid.get().ok_or_else(null_grid)?.nb_v_patches()?,
            false,
        );
        let (candidate_point, _) = tangent_on_edge(&edge, face, false, 0.0)?;
        let distance = point.square_distance(candidate_point);
        best.consider(wire_index, edge_index, same_patch, distance);
    }
    Ok(())
}

fn append_short_edges(
    segment: &ShapeFixWireSegment,
    destination: &ShapeFixWireSegment,
    first_edge: i32,
) -> Result<(), Failure> {
    for index in 1..=segment.nb_edges() {
        destination
            .wire_data()
            .get()
            .ok_or_else(null_wire)?
            .add_edge(&segment.edge(index)?, first_edge + index - 1)?;
    }
    Ok(())
}

#[derive(Default)]
struct ShortAttachment {
    wire_index: usize,
    edge_index: i32,
    distance: f64,
    same_patch: bool,
}

impl ShortAttachment {
    fn found(&self) -> bool {
        self.wire_index != 0
    }

    fn consider(&mut self, wire: usize, edge: i32, same_patch: bool, distance: f64) {
        if better_attachment(self, same_patch, distance) {
            *self = Self {
                wire_index: wire,
                edge_index: edge,
                distance,
                same_patch,
            };
        }
    }
}

fn better_attachment(best: &ShortAttachment, same_patch: bool, distance: f64) -> bool {
    if same_patch && !best.same_patch {
        return true;
    }
    if best.same_patch && !same_patch {
        return false;
    }
    !best.found() || distance < best.distance
}

fn tangent_on_edge(
    edge: &rustycad_tkbrep::topods::Edge,
    face: &rustycad_tkbrep::topods::Face,
    at_end: bool,
    dparam: f64,
) -> Result<(Point2, Vector2), Failure> {
    let mut outputs = crate::shape_analysis::EdgeTangent2d::default();
    ShapeAnalysisEdge::new().get_end_tangent_2d_on_face_into(
        edge,
        face,
        at_end,
        dparam,
        &mut outputs,
    )?;
    Ok((outputs.point, outputs.tangent))
}

pub(super) fn coincident(
    first: Point2,
    second: Point2,
    resolution: (f64, f64),
    tolerance: f64,
) -> bool {
    let u_tolerance = (resolution.0 * tolerance).max(TOLINT);
    let v_tolerance = (resolution.1 * tolerance).max(TOLINT);
    (first.x() - second.x()).abs() <= u_tolerance && (first.y() - second.y()).abs() <= v_tolerance
}

fn null_grid() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::CollectWires: null grid",
    )
}

fn null_wire() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::CollectWires: null wire data",
    )
}
