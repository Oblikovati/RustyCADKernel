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
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    entities: &mut Vec<Handle<dyn Select3DSensitiveEntity>>,
    criteria: &FaceSelectionCriteria,
) -> Result<bool, Failure> {
    let wires = Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)?;
    if !wires.more() {
        return Ok(false);
    }
    let wire = topods::wire(wires.current())?;
    let mut explorer = WireExplorer::with_wire(&wire)?;
    let mut points = Vec::new();
    let context = FaceContourContext {
        owner,
        criteria,
        surface_type: surface.get_type()?,
    };
    while explorer.more() {
        append_face_contour_edge(&explorer, &context, &mut points, entities)?;
        explorer.next()?;
    }
    append_face_contour_entity(owner, &points, criteria.interior, entities)?;
    Ok(true)
}
struct FaceContourContext<'a> {
    owner: &'a Handle<dyn SelectMgrEntityOwnerSource>,
    criteria: &'a FaceSelectionCriteria,
    surface_type: SurfaceType,
}
fn append_face_contour_edge(
    explorer: &WireExplorer,
    context: &FaceContourContext<'_>,
    points: &mut Vec<Point3>,
    entities: &mut Vec<Handle<dyn Select3DSensitiveEntity>>,
) -> Result<(), Failure> {
    let curve = match catch_native_failure(|| BRepAdaptorCurve::with_edge(explorer.current())) {
        Ok(curve) => curve,
        Err(failure) if failure.exception_type() == "Standard_NullObject" => return Ok(()),
        Err(failure) => return Err(failure),
    };
    let range = BRepTool::range(explorer.current())?;
    if (range.0 - range.1).abs() <= precision::confusion() {
        return Ok(());
    }
    let orientation = explorer.orientation()?;
    if points.is_empty() {
        points.push(curve.value(if orientation == Orientation::Forward {
            range.0
        } else {
            range.1
        })?);
    }
    match curve.get_type()? {
        CurveType::Line => points.push(curve.value(if orientation == Orientation::Forward {
            range.1
        } else {
            range.0
        })?),
        CurveType::Circle => {
            append_face_circle(&curve, range, orientation, context, points, entities)?
        }
        _ => append_face_uniform_points(
            &curve,
            range,
            orientation,
            context.criteria.edge_points,
            points,
        )?,
    }
    Ok(())
}
fn append_face_circle(
    curve: &BRepAdaptorCurve,
    range: (f64, f64),
    orientation: Orientation,
    context: &FaceContourContext<'_>,
    points: &mut Vec<Point3>,
    entities: &mut Vec<Handle<dyn Select3DSensitiveEntity>>,
) -> Result<(), Failure> {
    let full = 2. * std::f64::consts::PI - (range.1 - range.0).abs() <= precision::confusion();
    let sampled = matches!(
        context.surface_type,
        SurfaceType::Cylinder
            | SurfaceType::Torus
            | SurfaceType::Cone
            | SurfaceType::BSplineSurface
    );
    if !full || sampled {
        return append_face_uniform_points(
            curve,
            range,
            orientation,
            context.criteria.edge_points,
            points,
        );
    }
    let circle = curve.circle()?;
    if circle.radius() <= precision::confusion() {
        entities.push(
            Handle::new(Select3DSensitivePoint::new(
                context.owner,
                circle.location(),
            ))
            .cast(),
        );
        return Ok(());
    }
    entities.push(
        Handle::new(Select3DSensitiveCircle::new(
            context.owner,
            &circle,
            context.criteria.interior,
        ))
        .cast(),
    );
    Ok(())
}
fn append_face_uniform_points(
    curve: &BRepAdaptorCurve,
    range: (f64, f64),
    orientation: Orientation,
    count: i32,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    // OCCT std::max/min retains its first operand for NaN; f64::max/min would change loop behavior.
    let (first, last) = range;
    let maximum = if first < last { last } else { first };
    let minimum = if last < first { last } else { first };
    let intervals = count
        .checked_sub(1)
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!("edge point count={count}; expected representable count-1"),
            )
        })?
        .max(2);
    let step = (maximum - minimum) / f64::from(intervals);
    match orientation {
        Orientation::Forward => {
            append_face_forward_samples(curve, first + step, last, step, points)
        }
        Orientation::Reversed => {
            append_face_reverse_samples(curve, last - step, first, step, points)
        }
        _ => Ok(()),
    }
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
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    points: &[Point3],
    interior: bool,
    entities: &mut Vec<Handle<dyn Select3DSensitiveEntity>>,
) -> Result<(), Failure> {
    if points.len() == 2 {
        entities.push(Handle::new(Select3DSensitiveCurve::from_points(owner, points)?).cast());
        return Ok(());
    }
    if points.len() > 2 {
        let kind = if interior {
            Select3DTypeOfSensitivity::Interior
        } else {
            Select3DTypeOfSensitivity::Boundary
        };
        entities.push(Handle::new(Select3DSensitiveFace::new(owner, points, kind)?).cast());
    }
    Ok(())
}
