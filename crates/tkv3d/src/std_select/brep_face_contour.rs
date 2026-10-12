//! Native nonplanar fallback follows the first wire and appends analytic circular entities before the final contour.
use super::brep_face_sensitive::FaceSelectionCriteria;
use crate::{select_mgr::SelectMgrEntityOwnerSource, select3d::*};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface},
    brep_tools::WireExplorer,
    topexp::Explorer,
    topods::{self, Face},
};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, catch_native_failure},
};
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dSurface},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::{
    geom_abs::{CurveType, SurfaceType},
    gp::Point3,
};
pub(super) fn brep_face_contour_sensitive(
    face: &Face,
    surface: &BRepAdaptorSurface,
    owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    entities: &mut dyn super::brep_face_destination::BrepFaceEntityDestination,
    criteria: &FaceSelectionCriteria,
) -> Result<bool, Failure> {
    let Some(wire) = first_face_contour_wire(face)? else {
        return Ok(false);
    };
    let mut explorer = WireExplorer::with_wire(&wire)?;
    let context = FaceContourContext {
        owner,
        criteria,
        surface_type: surface.get_type()?,
    };
    collect_face_contour(&mut explorer, &context, entities)
}
fn first_face_contour_wire(face: &Face) -> Result<Option<rustycad_tkbrep::topods::Wire>, Failure> {
    // OCCT 3d097a0 destroys the explorer's topology copies before the first curve callback.
    let wires = Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)?;
    if !wires.more() {
        return Ok(None);
    }
    Ok(Some(topods::wire(wires.current())?))
}
fn collect_face_contour(
    explorer: &mut WireExplorer,
    context: &FaceContourContext<'_>,
    entities: &mut dyn super::brep_face_destination::BrepFaceEntityDestination,
) -> Result<bool, Failure> {
    let mut points = Vec::new();
    while explorer.more() {
        append_face_contour_edge(explorer, context, &mut points, entities)?;
        explorer.next()?;
    }
    append_face_contour_entity(context.owner, &points, context.criteria.interior, entities)?;
    Ok(true)
}

struct FaceContourContext<'a> {
    owner: &'a dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    criteria: &'a FaceSelectionCriteria,
    surface_type: SurfaceType,
}
fn append_face_contour_edge(
    explorer: &WireExplorer,
    context: &FaceContourContext<'_>,
    points: &mut Vec<Point3>,
    entities: &mut dyn super::brep_face_destination::BrepFaceEntityDestination,
) -> Result<(), Failure> {
    let Some(curve) = load_face_contour_curve(explorer)? else {
        return Ok(());
    };
    let range = BRepTool::range(explorer.current())?;
    if (range.0 - range.1).abs() <= precision::confusion() {
        return Ok(());
    }
    append_face_first_point(&curve, range, explorer, points)?;
    append_face_curve_kind(&curve, range, explorer, context, points, entities)
}
fn load_face_contour_curve(explorer: &WireExplorer) -> Result<Option<BRepAdaptorCurve>, Failure> {
    match catch_native_failure(|| BRepAdaptorCurve::with_edge(explorer.current())) {
        Ok(curve) => Ok(Some(curve)),
        Err(failure) if failure.exception_type() == "Standard_NullObject" => Ok(None),
        Err(failure) => Err(failure),
    }
}
fn append_face_first_point(
    curve: &BRepAdaptorCurve,
    range: (f64, f64),
    explorer: &WireExplorer,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    if !points.is_empty() {
        return Ok(());
    }
    let parameter = if explorer.orientation()? == Orientation::Forward {
        range.0
    } else {
        range.1
    };
    points.push(curve.value(parameter)?);
    Ok(())
}
fn append_face_curve_kind(
    curve: &BRepAdaptorCurve,
    range: (f64, f64),
    explorer: &WireExplorer,
    context: &FaceContourContext<'_>,
    points: &mut Vec<Point3>,
    entities: &mut dyn super::brep_face_destination::BrepFaceEntityDestination,
) -> Result<(), Failure> {
    match curve.get_type()? {
        CurveType::Line => append_face_line_endpoint(curve, range, explorer, points),
        CurveType::Circle => append_face_circle(curve, range, explorer, context, points, entities),
        _ => {
            append_face_uniform_points(curve, range, explorer, context.criteria.edge_points, points)
        }
    }
}
fn append_face_line_endpoint(
    curve: &BRepAdaptorCurve,
    range: (f64, f64),
    explorer: &WireExplorer,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    // OCCT 3d097a0 rereads Orientation after first Value; its callback may change underlying edge vertices.
    let parameter = if explorer.orientation()? == Orientation::Forward {
        range.1
    } else {
        range.0
    };
    points.push(curve.value(parameter)?);
    Ok(())
}

fn append_face_circle(
    curve: &BRepAdaptorCurve,
    range: (f64, f64),
    explorer: &WireExplorer,
    context: &FaceContourContext<'_>,
    points: &mut Vec<Point3>,
    entities: &mut dyn super::brep_face_destination::BrepFaceEntityDestination,
) -> Result<(), Failure> {
    let full = 2. * std::f64::consts::PI - (range.1 - range.0).abs() <= precision::confusion();
    let sampled = sampled_face_circle_surface(context.surface_type);
    if !full || sampled {
        return append_sampled_face_circle(curve, range, explorer, context, points);
    }
    entities.push(face_full_circle_entity(curve, context)?);
    Ok(())
}
fn append_sampled_face_circle(
    curve: &BRepAdaptorCurve,
    range: (f64, f64),
    explorer: &WireExplorer,
    context: &FaceContourContext<'_>,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    append_face_uniform_points(curve, range, explorer, context.criteria.edge_points, points)
}
fn sampled_face_circle_surface(kind: SurfaceType) -> bool {
    matches!(
        kind,
        SurfaceType::Cylinder
            | SurfaceType::Torus
            | SurfaceType::Cone
            | SurfaceType::BSplineSurface
    )
}
fn face_full_circle_entity(
    curve: &BRepAdaptorCurve,
    context: &FaceContourContext<'_>,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let circle = curve.circle()?;
    if circle.radius() <= precision::confusion() {
        let point = Select3DSensitivePoint::new(&context.owner.observe(), circle.location());
        return Ok(Handle::new(point).cast());
    }
    let sensitive =
        Select3DSensitiveCircle::new(&context.owner.observe(), &circle, context.criteria.interior);
    Ok(Handle::new(sensitive).cast())
}

fn append_face_uniform_points(
    curve: &BRepAdaptorCurve,
    range: (f64, f64),
    explorer: &WireExplorer,
    count: i32,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    let (first, last) = range;
    let step = face_uniform_sample_step(range, count)?;
    match face_sampling_orientation(explorer)? {
        Orientation::Forward => {
            append_face_forward_samples(curve, first + step, last, step, points)
        }
        Orientation::Reversed => {
            append_face_reverse_samples(curve, last - step, first, step, points)
        }
        _ => Ok(()),
    }
}
fn face_uniform_sample_step(range: (f64, f64), count: i32) -> Result<f64, Failure> {
    // OCCT std::max/min retains its first operand for NaN; f64::max/min would change loop behavior.
    let (first, last) = range;
    let maximum = if first < last { last } else { first };
    let minimum = if last < first { last } else { first };
    let intervals = face_sample_intervals(count)?;
    Ok((maximum - minimum) / f64::from(intervals))
}
fn face_sample_intervals(count: i32) -> Result<i32, Failure> {
    let intervals = count.checked_sub(1).ok_or_else(|| {
        Failure::with_message(
            "Standard_OutOfRange",
            &format!("edge point count={count}; expected representable count-1"),
        )
    })?;
    Ok(intervals.max(2))
}
fn append_face_forward_samples(
    curve: &BRepAdaptorCurve,
    mut parameter: f64,
    last: f64,
    step: f64,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    while parameter <= last {
        points.push(curve.value(parameter)?);
        parameter += step;
    }
    Ok(())
}
fn append_face_reverse_samples(
    curve: &BRepAdaptorCurve,
    mut parameter: f64,
    first: f64,
    step: f64,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    while parameter >= first {
        points.push(curve.value(parameter)?);
        parameter -= step;
    }
    Ok(())
}
fn append_face_contour_entity(
    owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    points: &[Point3],
    interior: bool,
    entities: &mut dyn super::brep_face_destination::BrepFaceEntityDestination,
) -> Result<(), Failure> {
    if points.len() == 2 {
        let curve = Select3DSensitiveCurve::from_points(&owner.observe(), points)?;
        entities.push(Handle::new(curve).cast());
        return Ok(());
    }
    if points.len() > 2 {
        entities.push(contour_sensitive_face(owner, points, interior)?);
    }
    Ok(())
}
fn contour_sensitive_face(
    owner: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    points: &[Point3],
    interior: bool,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let kind = if interior {
        Select3DTypeOfSensitivity::Interior
    } else {
        Select3DTypeOfSensitivity::Boundary
    };
    let face = Select3DSensitiveFace::new(&owner.observe(), points, kind)?;
    Ok(Handle::new(face).cast())
}

fn face_sampling_orientation(explorer: &WireExplorer) -> Result<Orientation, Failure> {
    if explorer.orientation()? == Orientation::Forward {
        return Ok(Orientation::Forward);
    }
    if explorer.orientation()? == Orientation::Reversed {
        return Ok(Orientation::Reversed);
    }
    Ok(Orientation::Internal)
}
