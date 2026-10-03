//! Splits composite-surface wire segments at vertices introduced by cutting lines.
use super::ShapeFixComposeShell;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_fix::ShapeFixWireSegment;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::{ncollection::Sequence, standard::Failure};
use rustycad_tkg3d::top_abs::Orientation;

/// `BreakWires`: partition segments at endpoints of EXTERNAL joint-line edges.
pub(super) fn break_wires(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
) -> Result<(), Failure> {
    let split_vertices = collect_split_vertices(segments)?;
    let closed_mode = composer.closed_mode();
    let mut index = 1;
    while index <= segments.len() {
        let wire = segments.value(index)?.clone();
        index = break_one_wire(segments, index, wire, &split_vertices, closed_mode)?;
    }
    Ok(())
}

/// Collects vertices from external edges using OCCT `IsSame` identity.
fn collect_split_vertices(
    segments: &Sequence<ShapeFixWireSegment>,
) -> Result<Vec<Vertex>, Failure> {
    let mut vertices = Vec::new();
    for segment in segments.iter() {
        let wire_orientation = segment.orientation();
        if !matches!(
            wire_orientation,
            Orientation::External | Orientation::Internal
        ) {
            continue;
        }
        for edge_index in 1..=segment.nb_edges() {
            let edge = segment.edge(edge_index)?;
            if edge_orientation(wire_orientation, &edge) != Orientation::External {
                continue;
            }
            push_unique(&mut vertices, ShapeAnalysisEdge::new().first_vertex(&edge));
            push_unique(&mut vertices, ShapeAnalysisEdge::new().last_vertex(&edge));
        }
    }
    Ok(vertices)
}

/// Selects external-edge direction with the native segment override.
fn edge_orientation(wire: Orientation, edge: &Edge) -> Orientation {
    if wire == Orientation::External {
        Orientation::External
    } else {
        edge.orientation()
    }
}

/// Appends a vertex only when no identical topological vertex was collected.
fn push_unique(vertices: &mut Vec<Vertex>, vertex: Vertex) {
    if !vertices.iter().any(|current| current.is_same(&vertex)) {
        vertices.push(vertex);
    }
}

/// Splits one input segment and returns the next unprocessed sequence index.
fn break_one_wire(
    segments: &mut Sequence<ShapeFixWireSegment>,
    index: usize,
    wire: ShapeFixWireSegment,
    split_vertices: &[Vertex],
    closed_mode: bool,
) -> Result<usize, Failure> {
    if wire.is_vertex() || wire.nb_edges() == 0 {
        return Ok(index + 1);
    }
    let Some(first_split) = first_split_edge(&wire, split_vertices)? else {
        return Ok(index + 1);
    };
    let shift = closed_wire_shift(&wire, first_split, split_vertices, closed_mode)?;
    rebuild_segments(segments, index, wire, split_vertices, shift)
}

/// Returns the first edge whose starting vertex is a recorded split vertex.
fn first_split_edge(
    wire: &ShapeFixWireSegment,
    split_vertices: &[Vertex],
) -> Result<Option<usize>, Failure> {
    for edge_index in 1..=wire.nb_edges() {
        let edge = wire.edge(edge_index)?;
        let vertex = ShapeAnalysisEdge::new().first_vertex(&edge);
        if contains_vertex(split_vertices, &vertex) {
            return Ok(Some(edge_index as usize));
        }
    }
    Ok(None)
}

/// Rotates a closed non-periodic wire when its first vertex is not a split.
fn closed_wire_shift(
    wire: &ShapeFixWireSegment,
    first_split: usize,
    split_vertices: &[Vertex],
    closed_mode: bool,
) -> Result<usize, Failure> {
    if first_split == 1 || closed_mode || !wire.is_closed()? {
        return Ok(0);
    }
    let first = ShapeAnalysisEdge::new().first_vertex(&wire.edge(1)?);
    Ok(if contains_vertex(split_vertices, &first) {
        0
    } else {
        first_split - 1
    })
}

/// Rebuilds ordered segments while copying each edge's four patch limits.
fn rebuild_segments(
    segments: &mut Sequence<ShapeFixWireSegment>,
    index: usize,
    wire: ShapeFixWireSegment,
    split_vertices: &[Vertex],
    shift: usize,
) -> Result<usize, Failure> {
    let mut progress = RebuildProgress::new(wire.orientation());
    let edge_count = wire.nb_edges() as usize;
    for position in 1..=edge_count {
        let edge_index = rotated_index(position, shift, edge_count);
        append_rebuilt_edge(
            segments,
            index,
            &wire,
            edge_index,
            position,
            split_vertices,
            &mut progress,
        )?;
    }
    finish_rebuild(segments, index, progress)
}

/// Mutable accumulators shared by the edge loop and final sequence update.
struct RebuildProgress {
    current: ShapeFixWireSegment,
    orientation: Orientation,
    inserted: usize,
    replacement_needed: bool,
}

impl RebuildProgress {
    /// Starts the segment scan with the source traversal orientation.
    fn new(orientation: Orientation) -> Self {
        Self {
            current: ShapeFixWireSegment::new(),
            orientation,
            inserted: 0,
            replacement_needed: false,
        }
    }
}

/// Applies split-boundary and edge-conversion rules for one source edge.
fn append_rebuilt_edge(
    segments: &mut Sequence<ShapeFixWireSegment>,
    source_index: usize,
    source: &ShapeFixWireSegment,
    edge_index: usize,
    position: usize,
    split_vertices: &[Vertex],
    progress: &mut RebuildProgress,
) -> Result<(), Failure> {
    let mut edge = source.edge(edge_index as i32)?;
    start_segment_at_boundary(
        segments,
        source_index,
        &edge,
        position,
        split_vertices,
        progress,
    )?;
    update_progress_orientation(source.orientation(), &mut edge, progress);
    add_with_source_patch(&mut progress.current, source, edge_index, &edge)
}

/// Flushes an accumulator before a split vertex and resets traversal orientation.
fn start_segment_at_boundary(
    segments: &mut Sequence<ShapeFixWireSegment>,
    source_index: usize,
    edge: &Edge,
    position: usize,
    split_vertices: &[Vertex],
    progress: &mut RebuildProgress,
) -> Result<(), Failure> {
    if position != 1 && !edge_starts_at_split(edge, split_vertices) {
        return Ok(());
    }
    let added = insert_current(
        segments,
        source_index + progress.inserted,
        &mut progress.current,
        progress.orientation,
    )?;
    progress.inserted += added;
    progress.replacement_needed |= added > 0;
    progress.orientation = source_orientation(segments, source_index + progress.inserted)?;
    Ok(())
}

/// Reads the source segment orientation after sequence insertions shift its slot.
fn source_orientation(
    segments: &Sequence<ShapeFixWireSegment>,
    source_index: usize,
) -> Result<Orientation, Failure> {
    Ok(segments.value(source_index)?.orientation())
}

/// Marks an INTERNAL wire conversion and changes its external edge to FORWARD.
fn update_progress_orientation(
    source_orientation: Orientation,
    edge: &mut Edge,
    progress: &mut RebuildProgress,
) {
    let old_orientation = progress.orientation;
    progress.orientation = update_segment_orientation(source_orientation, edge, old_orientation);
    progress.replacement_needed |= progress.orientation != old_orientation;
}

/// Stores the final segment in the source slot after any inserted segments.
fn finish_rebuild(
    segments: &mut Sequence<ShapeFixWireSegment>,
    source_index: usize,
    mut progress: RebuildProgress,
) -> Result<usize, Failure> {
    if progress.replacement_needed {
        progress.current.set_orientation(progress.orientation);
        segments.set_value(source_index + progress.inserted, progress.current)?;
    }
    Ok(source_index + progress.inserted + 1)
}

/// Maps traversal positions through OCCT's one-based cyclic rotation.
fn rotated_index(position: usize, shift: usize, edge_count: usize) -> usize {
    1 + (position - 1 + shift) % edge_count
}

/// Tests whether an edge's forward start is one of the cutting vertices.
fn edge_starts_at_split(edge: &Edge, split_vertices: &[Vertex]) -> bool {
    let start = ShapeAnalysisEdge::new().first_vertex(edge);
    contains_vertex(split_vertices, &start)
}

/// Checks vertex identity in the collected split set.
fn contains_vertex(vertices: &[Vertex], target: &Vertex) -> bool {
    vertices.iter().any(|vertex| vertex.is_same(target))
}

/// Inserts a completed segment before the source and detaches the accumulator.
fn insert_current(
    segments: &mut Sequence<ShapeFixWireSegment>,
    index: usize,
    current: &mut ShapeFixWireSegment,
    orientation: Orientation,
) -> Result<usize, Failure> {
    if current.nb_edges() == 0 {
        return Ok(0);
    }
    current.set_orientation(orientation);
    segments.insert_before(index, current.clone())?;
    current.clear();
    Ok(1)
}

/// Preserves the native INTERNAL parent to EXTERNAL edge transition.
fn update_segment_orientation(
    wire_orientation: Orientation,
    edge: &mut Edge,
    current_orientation: Orientation,
) -> Orientation {
    if wire_orientation == Orientation::Internal && edge.orientation() == Orientation::External {
        edge.set_orientation(Orientation::Forward);
        Orientation::External
    } else {
        current_orientation
    }
}

/// Copies one edge with its original patch interval into the new segment.
fn add_with_source_patch(
    destination: &mut ShapeFixWireSegment,
    source: &ShapeFixWireSegment,
    edge_index: usize,
    edge: &Edge,
) -> Result<(), Failure> {
    destination.add_edge_with_patch(0, edge, source.patch_index(edge_index as i32)?)
}
