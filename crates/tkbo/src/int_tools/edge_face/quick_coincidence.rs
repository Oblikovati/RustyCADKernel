use crate::int_tools::{Context, Range};
use rustycad_tkbrep::{
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface},
    topods::Face,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dSurface},
    top_abs::State,
};
use rustycad_tkmath::{
    geom_abs::{CurveType, SurfaceType},
    gp::Point2,
};

pub(super) fn is_coincident(
    curve: &BRepAdaptorCurve,
    surface: &BRepAdaptorSurface,
    context: &Context,
    face: &Face,
    range: Range,
    criteria: f64,
) -> Result<bool, Failure> {
    let segments =
        if curve.get_type()? == CurveType::Line && surface.get_type()? == SurfaceType::Plane {
            2
        } else {
            23
        };
    let (first, step) = sample_parameters(range, segments);
    let (threshold_first, threshold_last) = sample_thresholds(segments);
    let mut accepted = 0;
    let mut classified = false;
    for index in 0..=segments {
        let parameter = first + index as f64 * step;
        let point = curve.value(parameter)?;
        let Some(projected) = project(context, face, point)? else {
            continue;
        };
        if projected.distance > criteria * 100.0 {
            return Ok(false);
        }
        if projected.distance > criteria {
            continue;
        }
        accepted += 1;
        if should_classify(index, segments, threshold_first, threshold_last, classified) {
            if context
                .fclass2d(face)?
                .read()
                .perform(Point2::new(projected.uv.0, projected.uv.1), false)?
                == State::Out
            {
                return Ok(false);
            }
            if index != 0 {
                classified = true;
            }
        }
    }
    Ok(accepted as f64 / (segments + 1) as f64 > 0.5)
}

fn sample_parameters(range: Range, segments: i32) -> (f64, f64) {
    let first = range.first() + 0.01 * (range.last() - range.first());
    let last = range.last() - 0.01 * (range.last() - range.first());
    (first, (last - first) / segments as f64)
}

fn sample_thresholds(segments: i32) -> (i32, i32) {
    (
        ((segments + 1) as f64 * 0.25).round() as i32,
        ((segments + 1) as f64 * 0.75).round() as i32,
    )
}

fn should_classify(index: i32, segments: i32, first: i32, last: i32, classified: bool) -> bool {
    let outside_middle = (index > 0 && index < first) || (last < index && index < segments);
    !outside_middle && (!classified || index == segments)
}

struct ProjectedPoint {
    distance: f64,
    uv: (f64, f64),
}

fn project(
    context: &Context,
    face: &Face,
    point: rustycad_tkmath::gp::Point3,
) -> Result<Option<ProjectedPoint>, Failure> {
    let projector = context.proj_ps(face)?;
    let mut projector = projector.write();
    projector.perform(point)?;
    if !projector.is_done() {
        return Ok(None);
    }
    Ok(Some(ProjectedPoint {
        distance: projector.lower_distance()?,
        uv: projector.lower_distance_parameters()?,
    }))
}
