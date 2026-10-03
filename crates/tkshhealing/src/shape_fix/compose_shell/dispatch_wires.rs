//! Move each collected wire's p-curves onto its selected composite-surface patch.
mod closed_wires;
mod edge_reassignment;
mod middle_point;

use crate::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
use crate::shape_extend::ShapeExtendCompositeSurface;
use crate::shape_fix::{FixRoot, ShapeFixWireSegment};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::toploc::Location;

pub(super) struct PatchLoopGroup {
    pub(super) surface: Handle<dyn GeomSurface>,
    pub(super) loops: Vec<Shape>,
}

pub(super) fn dispatch_wires(
    composer: &super::ShapeFixComposeShell,
    wires: &mut Sequence<ShapeFixWireSegment>,
) -> Result<Vec<PatchLoopGroup>, Failure> {
    let (grid, face, location, _) = composer.collection_inputs();
    let grid = live_grid(&grid)?;
    let context = composer.context();
    closed_wires::prepare_closed_wires(composer, wires, &face)?;
    let middles = middle_point::wire_middle_points(wires, &face)?;
    let middles = adjust_closed_centers(composer, grid, middles);
    let mut copied_edges = Vec::new();
    for index in 1..=wires.length() as usize {
        reassign_segment_edges(
            wires.change_value(index)?,
            face.clone(),
            location.clone(),
            grid,
            &context,
            &mut copied_edges,
        )?;
    }
    group_patch_loops(wires, middles, grid)
}

fn adjust_closed_centers(
    composer: &super::ShapeFixComposeShell,
    grid: &ShapeExtendCompositeSurface,
    mut centers: Vec<Point2>,
) -> Vec<Point2> {
    if !composer.closed_mode() {
        return centers;
    }
    let fields = composer.fields.read();
    let bounds = grid.bounds();
    for center in &mut centers {
        if fields.u_closed {
            let shift = crate::shape_analysis::adjust_to_period(center.x(), bounds.0, bounds.1);
            center.set_x(center.x() + shift);
        }
        if fields.v_closed {
            let shift = crate::shape_analysis::adjust_to_period(center.y(), bounds.2, bounds.3);
            center.set_y(center.y() + shift);
        }
    }
    centers
}

fn live_grid(
    grid: &Handle<ShapeExtendCompositeSurface>,
) -> Result<&ShapeExtendCompositeSurface, Failure> {
    grid.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ShapeFix_ComposeShell::DispatchWires: null grid; expected initialized composite surface",
        )
    })
}

fn reassign_segment_edges(
    segment: &ShapeFixWireSegment,
    source_face: Face,
    location: Location,
    grid: &ShapeExtendCompositeSurface,
    context: &Handle<ShapeBuildReShape>,
    copied_edges: &mut Vec<(Edge, Edge)>,
) -> Result<(), Failure> {
    if segment.is_vertex() {
        return Ok(());
    }
    let center = middle_point::wire_middle_point(segment, &source_face)?;
    let (patch, transform) = patch_face(grid, center, &location)?;
    for edge_index in 1..=segment.nb_edges() {
        let source = segment.edge(edge_index)?;
        let target = copied_or_recorded_edge(&source, &source_face, &patch, context, copied_edges)?;
        edge_reassignment::transform_edge_pcurve(&target, &patch, transform)?;
        segment
            .wire_data()
            .get()
            .ok_or_else(null_wire)?
            .set(&target, edge_index)?;
    }
    Ok(())
}

fn patch_face(
    grid: &ShapeExtendCompositeSurface,
    center: Point2,
    location: &Location,
) -> Result<(Face, (bool, f64, rustycad_tkmath::gp::Transform2)), Failure> {
    let (u, v) = grid.locate_uv_point(center)?;
    let surface = grid.patch(u, v)?;
    let transform = grid.global_to_local_transformation(u, v)?;
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_located_surface(
        &mut face,
        &surface,
        location,
        precision::confusion(),
    )?;
    Ok((face, transform))
}

fn copied_or_recorded_edge(
    source: &Edge,
    source_face: &Face,
    patch_face: &Face,
    context: &Handle<ShapeBuildReShape>,
    copied_edges: &mut Vec<(Edge, Edge)>,
) -> Result<Edge, Failure> {
    let target = match copied_edges.iter().find(|(edge, _)| edge.is_same(source)) {
        Some((_, copied)) => copied.clone(),
        None => record_edge_copy(source, context, copied_edges)?,
    };
    ShapeBuildEdge.reassign_pcurve(&target, source_face, patch_face)?;
    Ok(target)
}

fn record_edge_copy(
    source: &Edge,
    context: &Handle<ShapeBuildReShape>,
    copied_edges: &mut Vec<(Edge, Edge)>,
) -> Result<Edge, Failure> {
    let target = copy_edge_with_native_orientation(source)?;
    context
        .get()
        .ok_or_else(null_wire)?
        .replace(&source.clone().into_shape(), &target.clone().into_shape())?;
    copied_edges.push((source.clone(), target.clone()));
    Ok(target)
}

fn copy_edge_with_native_orientation(source: &Edge) -> Result<Edge, Failure> {
    let (orientation, manifold) = (
        source.orientation(),
        matches!(
            source.orientation(),
            Orientation::Forward | Orientation::Reversed
        ),
    );
    let source = if manifold {
        source.clone()
    } else {
        Edge::unchecked(source.oriented(Orientation::Forward))
    };
    let mut copied = ShapeBuildEdge.copy(&source, false)?;
    if !manifold {
        copied.set_orientation(orientation);
    }
    Ok(copied)
}

fn group_patch_loops(
    wires: &Sequence<ShapeFixWireSegment>,
    middles: Vec<Point2>,
    grid: &ShapeExtendCompositeSurface,
) -> Result<Vec<PatchLoopGroup>, Failure> {
    let mut groups = Vec::new();
    let mut used = vec![false; middles.len()];
    for (index, center) in middles.iter().copied().enumerate() {
        if used[index] {
            continue;
        }
        let (u, v) = grid.locate_uv_point(center)?;
        let surface = grid.patch(u, v)?;
        let mut loops = Vec::new();
        collect_surface_loops(wires, &middles, grid, &surface, &mut used, &mut loops)?;
        groups.push(PatchLoopGroup { surface, loops });
    }
    Ok(groups)
}

fn collect_surface_loops(
    wires: &Sequence<ShapeFixWireSegment>,
    middles: &[Point2],
    grid: &ShapeExtendCompositeSurface,
    surface: &Handle<dyn GeomSurface>,
    used: &mut [bool],
    loops: &mut Vec<Shape>,
) -> Result<(), Failure> {
    for (index, center) in middles.iter().copied().enumerate() {
        if used[index] || !same_patch_surface(grid, center, surface)? {
            continue;
        }
        used[index] = true;
        append_loop_shape(wires.value(index + 1)?, loops)?;
    }
    Ok(())
}

fn same_patch_surface(
    grid: &ShapeExtendCompositeSurface,
    center: Point2,
    expected: &Handle<dyn GeomSurface>,
) -> Result<bool, Failure> {
    let (u, v) = grid.locate_uv_point(center)?;
    Ok(grid.patch(u, v)? == *expected)
}

fn append_loop_shape(segment: &ShapeFixWireSegment, loops: &mut Vec<Shape>) -> Result<(), Failure> {
    if segment.is_vertex() {
        append_internal_vertex(segment, loops);
        return Ok(());
    }
    let data = segment.wire_data().get().ok_or_else(null_wire)?;
    if data.nb_edges() > 0 {
        loops.push(data.wire()?.into_shape());
    }
    Ok(())
}

fn append_internal_vertex(segment: &ShapeFixWireSegment, loops: &mut Vec<Shape>) {
    let vertex = segment.vertex();
    if vertex.orientation() == Orientation::Internal {
        loops.push(vertex.into_shape());
    }
}

fn null_wire() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::DispatchWires: null segment wire; expected wire data",
    )
}
