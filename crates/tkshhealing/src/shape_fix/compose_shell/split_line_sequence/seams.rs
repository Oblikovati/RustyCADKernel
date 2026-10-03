use super::super::grid::{define_patch, patch_index};
use super::{GridCut, LineEvent, ShapeFixComposeShell};
use crate::shape_analysis::adjust_to_period;
use crate::shape_build::ShapeBuildVertex;
use crate::shape_extend::ShapeExtendWireData;
use crate::shape_fix::{FixRoot, ShapeFixWireSegment};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::ReShape,
    topods::{self, Edge, Vertex},
};
use rustycad_tkernel::{
    ncollection::Sequence,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn append_seam(
    composer: &ShapeFixComposeShell,
    wires: &mut Sequence<ShapeFixWireSegment>,
    events: &[LineEvent],
    cut: GridCut,
    inner_pair: bool,
) -> Result<(), Failure> {
    let first = context_vertex(composer, &events[0])?;
    let second = context_vertex(composer, &events[1])?;
    let range = (events[0].parameter, events[1].parameter);
    let near = near_seam_vertices(composer, &first, &second)?;
    if range.1 - range.0 < precision::p_confusion() || (inner_pair && near) {
        return merge_seam_vertices(composer, &first, &second);
    }
    let segment = seam_segment(composer, &first, &second, range, cut)?;
    define_patch(&segment, 0, cut.along_u, cut.index, None)?;
    define_other_axis(composer, &segment, range.0, cut.along_u)?;
    wires.append(segment);
    Ok(())
}

fn context_vertex(composer: &ShapeFixComposeShell, event: &LineEvent) -> Result<Vertex, Failure> {
    let context = composer.context();
    let live = context.get().ok_or_else(null_context)?;
    let applied = ReShape::apply(live, &event.vertex, ShapeEnum::Shape)?;
    topods::vertex(&applied)
}

fn near_seam_vertices(
    composer: &ShapeFixComposeShell,
    first: &Vertex,
    second: &Vertex,
) -> Result<bool, Failure> {
    let mut maximum = composer.max_tolerance();
    if maximum <= 2. * precision::confusion() {
        maximum = precision::infinite();
    }
    let first_tolerance = BRepTool::vertex_tolerance(first)?.min(maximum);
    let second_tolerance = BRepTool::vertex_tolerance(second)?.min(maximum);
    let distance = BRepTool::pnt(first)?.square_distance(&BRepTool::pnt(second)?);
    Ok(distance <= first_tolerance * first_tolerance
        || distance <= second_tolerance * second_tolerance)
}

fn merge_seam_vertices(
    composer: &ShapeFixComposeShell,
    first: &Vertex,
    second: &Vertex,
) -> Result<(), Failure> {
    if first.is_same(second) {
        return Ok(());
    }
    let combined = ShapeBuildVertex.combine_vertices(first, second, 1.0001)?;
    let context = composer.context();
    let live = context.get().ok_or_else(null_context)?;
    ReShape::replace(
        live,
        &first.clone().into_shape(),
        &combined.oriented(first.orientation()),
    )?;
    ReShape::replace(
        live,
        &second.clone().into_shape(),
        &combined.oriented(second.orientation()),
    )
}

fn seam_segment(
    composer: &ShapeFixComposeShell,
    first: &Vertex,
    second: &Vertex,
    range: (f64, f64),
    cut: GridCut,
) -> Result<ShapeFixWireSegment, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    builder.add(&mut edge, &first.oriented(Orientation::Forward))?;
    builder.add(&mut edge, &second.oriented(Orientation::Reversed))?;
    let first_curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(cut.line)).cast();
    let second_curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_lin2d(cut.line)).cast();
    let face = composer.fields.read().face.clone();
    builder.update_edge_pcurves_on_face(
        &edge,
        (&first_curve, &second_curve),
        &face,
        precision::confusion(),
    )?;
    builder.range_on_face(&edge, &face, range.0, range.1)?;
    let wire = Handle::new(ShapeExtendWireData::new());
    wire.add_edge(&edge, 0)?;
    ShapeFixWireSegment::from_wire_data(&wire, Orientation::External)
}

fn define_other_axis(
    composer: &ShapeFixComposeShell,
    segment: &ShapeFixWireSegment,
    parameter: f64,
    cut_by_u: bool,
) -> Result<(), Failure> {
    let fields = composer.fields.read();
    let grid = fields.grid.get().ok_or_else(null_grid)?;
    let (joints, closed) = if cut_by_u {
        (grid.v_joint_values(), fields.v_closed)
    } else {
        (grid.u_joint_values(), fields.u_closed)
    };
    let shift = if fields.closed_mode && closed {
        let interval = if cut_by_u {
            (grid.v_joint_value(1)?, grid.v_joint_value(2)?)
        } else {
            (grid.u_joint_value(1)?, grid.u_joint_value(2)?)
        };
        adjust_to_period(parameter - 1e-10, interval.0, interval.1)
    } else {
        0.
    };
    let shifted = parameter + shift;
    let minimum = patch_index(shifted + precision::p_confusion(), &joints, closed)?;
    let maximum = patch_index(shifted - precision::p_confusion(), &joints, closed)? + 1;
    if cut_by_u {
        segment.define_v_min(1, minimum)?;
        segment.define_v_max(1, maximum)
    } else {
        segment.define_u_min(1, minimum)?;
        segment.define_u_max(1, maximum)
    }
}

fn null_context() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::SplitByLine: null context; expected reshape context",
    )
}
fn null_grid() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::SplitByLine: null grid; expected composite surface",
    )
}
