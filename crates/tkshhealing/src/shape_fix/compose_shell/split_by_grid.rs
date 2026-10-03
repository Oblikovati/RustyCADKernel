use super::ShapeFixComposeShell;
use super::grid::patch_index;
use crate::shape_analysis::{ShapeAnalysis, adjust_to_period};
use crate::shape_extend::ShapeExtendCompositeSurface;
use crate::shape_fix::ShapeFixWireSegment;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

const INTERSECTION_TOLERANCE: f64 = 1.0e-10;

pub(super) fn split_by_grid(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
) -> Result<(), Failure> {
    let grid = composer.fields.read().grid.clone();
    let live = grid.get().ok_or_else(null_grid)?;
    let (u_min, u_max, v_min, v_max) = live.bounds();
    constrain_edges(composer, segments, live)?;
    split_u_joints(composer, segments, live, (u_min, u_max))?;
    split_v_joints(composer, segments, live, false, (v_min, v_max))
}

fn constrain_edges(
    composer: &ShapeFixComposeShell,
    segments: &Sequence<ShapeFixWireSegment>,
    grid: &ShapeExtendCompositeSurface,
) -> Result<(), Failure> {
    if composer.closed_mode() {
        constrain_closed_segments(composer, segments, grid)
    } else {
        constrain_face_segments(composer, segments, grid)
    }
}

fn constrain_face_segments(
    composer: &ShapeFixComposeShell,
    segments: &Sequence<ShapeFixWireSegment>,
    grid: &ShapeExtendCompositeSurface,
) -> Result<(), Failure> {
    let face = composer.fields.read().face.clone();
    let (u_min, u_max, v_min, v_max) = rustycad_tkbrep::brep_tools::uv_bounds(&face)?;
    let (u_first, u_last) = face_patch_limits(composer, u_min, u_max, grid, true)?;
    let (v_first, v_last) = face_patch_limits(composer, v_min, v_max, grid, false)?;
    for_each_edge(segments, |segment, edge| {
        set_all_patch_limits(segment, edge, (u_first, u_last), (v_first, v_last))
    })
}

fn face_patch_limits(
    composer: &ShapeFixComposeShell,
    minimum: f64,
    maximum: f64,
    grid: &ShapeExtendCompositeSurface,
    along_u: bool,
) -> Result<(i32, i32), Failure> {
    let (min_index, max_index) = if along_u {
        (
            patch_index(
                minimum + INTERSECTION_TOLERANCE,
                &grid.u_joint_values(),
                axis_closed(composer, true),
            ),
            patch_index(
                maximum - INTERSECTION_TOLERANCE,
                &grid.u_joint_values(),
                axis_closed(composer, true),
            ),
        )
    } else {
        (
            patch_index(
                minimum + INTERSECTION_TOLERANCE,
                &grid.v_joint_values(),
                axis_closed(composer, false),
            ),
            patch_index(
                maximum - INTERSECTION_TOLERANCE,
                &grid.v_joint_values(),
                axis_closed(composer, false),
            ),
        )
    };
    let (min_index, max_index) = (min_index?, max_index?);
    Ok((min_index, max_index + 1))
}

fn constrain_closed_segments(
    composer: &ShapeFixComposeShell,
    segments: &Sequence<ShapeFixWireSegment>,
    grid: &ShapeExtendCompositeSurface,
) -> Result<(), Failure> {
    for index in 1..=segments.length() {
        let segment = segments.value(index as usize)?.clone();
        let (mut u_min, mut u_max, mut v_min, mut v_max) = segment_bounds(composer, &segment)?;
        shift_segment_bounds(
            composer, grid, &mut u_min, &mut u_max, &mut v_min, &mut v_max,
        )?;
        define_segment_limits(
            &segment,
            closed_patch_ranges(composer, grid, (u_min, u_max, v_min, v_max))?,
        )?;
    }
    Ok(())
}

fn segment_bounds(
    composer: &ShapeFixComposeShell,
    segment: &ShapeFixWireSegment,
) -> Result<(f64, f64, f64, f64), Failure> {
    let face = composer.fields.read().face.clone();
    let mut copied = topods::face(&face.into_shape().empty_copied()?)?;
    let wire = segment.wire_data().get().ok_or_else(null_wire)?.wire()?;
    let mut copied_shape = copied.clone().into_shape();
    BRepBuilder::default().add(&mut copied_shape, &wire.into_shape())?;
    copied = topods::face(&copied_shape)?;
    ShapeAnalysis::get_face_uv_bounds(&copied)
}

fn shift_segment_bounds(
    composer: &ShapeFixComposeShell,
    grid: &ShapeExtendCompositeSurface,
    u_min: &mut f64,
    u_max: &mut f64,
    v_min: &mut f64,
    v_max: &mut f64,
) -> Result<(), Failure> {
    let fields = composer.fields.read();
    if fields.u_closed {
        let first = grid.u_joint_value(1)?;
        let second = grid.u_joint_value(2)?;
        let shift = adjust_to_period(*u_max - INTERSECTION_TOLERANCE, first, second);
        *u_min += shift;
        *u_max += shift;
    }
    if fields.v_closed {
        let first = grid.v_joint_value(1)?;
        let second = grid.v_joint_value(2)?;
        let shift = adjust_to_period(*v_max - INTERSECTION_TOLERANCE, first, second);
        *v_min += shift;
        *v_max += shift;
    }
    Ok(())
}

type PatchRanges = ((i32, i32), (i32, i32));

fn closed_patch_ranges(
    composer: &ShapeFixComposeShell,
    grid: &ShapeExtendCompositeSurface,
    bounds: (f64, f64, f64, f64),
) -> Result<PatchRanges, Failure> {
    let (u_min, u_max, v_min, v_max) = bounds;
    Ok((
        closed_axis_range(
            u_min,
            u_max,
            &grid.u_joint_values(),
            axis_closed(composer, true),
        )?,
        closed_axis_range(
            v_min,
            v_max,
            &grid.v_joint_values(),
            axis_closed(composer, false),
        )?,
    ))
}

fn closed_axis_range(
    minimum: f64,
    maximum: f64,
    joints: &crate::shape_extend::JointValues,
    closed: bool,
) -> Result<(i32, i32), Failure> {
    let lower = patch_index(minimum + INTERSECTION_TOLERANCE, joints, closed)?.max(0);
    let upper = patch_index(maximum - INTERSECTION_TOLERANCE, joints, closed)? + 1;
    Ok((lower, upper))
}

fn define_segment_limits(
    segment: &ShapeFixWireSegment,
    ranges: ((i32, i32), (i32, i32)),
) -> Result<(), Failure> {
    let ((u_min, u_max), (v_min, v_max)) = ranges;
    for edge in 1..=segment.nb_edges() {
        segment.define_u_min(edge, u_min)?;
        segment.define_u_max(edge, u_max)?;
        segment.define_v_min(edge, v_min)?;
        segment.define_v_max(edge, v_max)?;
    }
    Ok(())
}

fn set_all_patch_limits(
    segment: &ShapeFixWireSegment,
    edge: i32,
    u: (i32, i32),
    v: (i32, i32),
) -> Result<(), Failure> {
    segment.define_u_min(edge, u.0)?;
    segment.define_u_max(edge, u.1)?;
    segment.define_v_min(edge, v.0)?;
    segment.define_v_max(edge, v.1)
}

fn split_u_joints(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
    grid: &ShapeExtendCompositeSurface,
    face_bounds: (f64, f64),
) -> Result<(), Failure> {
    split_axis_joints(composer, segments, grid, true, face_bounds)
}

fn split_v_joints(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
    grid: &ShapeExtendCompositeSurface,
    along_u: bool,
    face_bounds: (f64, f64),
) -> Result<(), Failure> {
    split_axis_joints(composer, segments, grid, along_u, face_bounds)
}

fn split_axis_joints(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
    grid: &ShapeExtendCompositeSurface,
    along_u: bool,
    face_bounds: (f64, f64),
) -> Result<(), Failure> {
    let closed = axis_closed(composer, along_u);
    let count = if along_u {
        grid.nb_u_patches()?
    } else {
        grid.nb_v_patches()?
    };
    let start = if closed { 1 } else { 2 };
    for joint in start..=count {
        split_joint(
            composer,
            segments,
            grid,
            along_u,
            joint,
            face_bounds,
            closed,
        )?;
    }
    Ok(())
}

fn split_joint(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
    grid: &ShapeExtendCompositeSurface,
    along_u: bool,
    joint: i32,
    face_bounds: (f64, f64),
    closed: bool,
) -> Result<(), Failure> {
    let value = joint_value(grid, along_u, joint)?;
    if !closed || composer.closed_mode() {
        return split_at(composer, segments, along_u, value, joint);
    }
    split_periodic_joint(composer, segments, grid, along_u, value, face_bounds)
}

fn split_periodic_joint(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
    grid: &ShapeExtendCompositeSurface,
    along_u: bool,
    value: f64,
    face_bounds: (f64, f64),
) -> Result<(), Failure> {
    let (face_first, face_last) = axis_face_bounds(composer, along_u)?;
    let period = face_bounds.1 - face_bounds.0;
    let mut shift = adjust_to_period(value, face_first, face_first + period);
    let mut translated = value + shift;
    while translated <= face_last + INTERSECTION_TOLERANCE {
        let cut = patch_index(
            translated + INTERSECTION_TOLERANCE,
            &axis_joints(grid, along_u),
            axis_closed(composer, along_u),
        )?;
        split_at(composer, segments, along_u, translated, cut)?;
        shift += period;
        translated = value + shift;
    }
    Ok(())
}

fn split_at(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
    along_u: bool,
    position: f64,
    cut_index: i32,
) -> Result<(), Failure> {
    let (origin, direction) = if along_u {
        (Point2::new(position, 0.0), Direction2::new(0.0, 1.0)?)
    } else {
        (Point2::new(0.0, position), Direction2::new(1.0, 0.0)?)
    };
    let line = Line2::new(origin, direction);
    split_segments(composer, segments, line, along_u, cut_index)
}

fn split_segments(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
    line: Line2,
    along_u: bool,
    cut_index: i32,
) -> Result<(), Failure> {
    super::split_line_sequence::split_sequence(composer, segments, line, along_u, cut_index)
}

fn joint_value(
    grid: &ShapeExtendCompositeSurface,
    along_u: bool,
    index: i32,
) -> Result<f64, Failure> {
    if along_u {
        grid.u_joint_value(index)
    } else {
        grid.v_joint_value(index)
    }
}

fn axis_joints(
    grid: &ShapeExtendCompositeSurface,
    along_u: bool,
) -> crate::shape_extend::JointValues {
    if along_u {
        grid.u_joint_values()
    } else {
        grid.v_joint_values()
    }
}

fn axis_closed(composer: &ShapeFixComposeShell, along_u: bool) -> bool {
    let fields = composer.fields.read();
    if along_u {
        fields.u_closed
    } else {
        fields.v_closed
    }
}

fn axis_face_bounds(composer: &ShapeFixComposeShell, along_u: bool) -> Result<(f64, f64), Failure> {
    let face = composer.fields.read().face.clone();
    let bounds = rustycad_tkbrep::brep_tools::uv_bounds(&face)?;
    Ok(if along_u {
        (bounds.0, bounds.1)
    } else {
        (bounds.2, bounds.3)
    })
}

fn for_each_edge(
    segments: &Sequence<ShapeFixWireSegment>,
    mut action: impl FnMut(&ShapeFixWireSegment, i32) -> Result<(), Failure>,
) -> Result<(), Failure> {
    for index in 1..=segments.length() {
        let segment = segments.value(index as usize)?;
        for edge in 1..=segment.nb_edges() {
            action(segment, edge)?;
        }
    }
    Ok(())
}

fn null_grid() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::SplitByGrid: null grid",
    )
}

fn null_wire() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::SplitByGrid: null wire data",
    )
}
