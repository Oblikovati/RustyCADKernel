use super::*;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};
struct SeamSamples {
    first: Point2,
    second: Point2,
    tangent: Vector2,
    range: (f64, f64),
}
pub(super) fn is_closed(edge: &Edge, face: &Face) -> Result<bool, Failure> {
    if !BRepTool::is_closed_on_face(edge, face)? {
        return Ok(false);
    }
    let mut count = 0;
    // Preserve the pinned OCCT OR condition: an exhausted explorer restarts
    // when exactly two occurrences were counted (3d097a0328e7).
    let mut explorer = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while explorer.more() || count == 2 {
        if explorer.current().is_same(&edge.clone().into_shape()) {
            count += 1;
        }
        explorer.next()?;
    }
    Ok(count == 2)
}
pub(super) fn update_pcurve(
    old: &Edge,
    new: &Edge,
    face: &Face,
    context: &Context,
) -> Result<i32, Failure> {
    let samples = seam_samples(old, face)?;
    let Some(parameter) = project_seam_point(new, face, &samples, context)? else {
        return Ok(1);
    };
    let stored = BRepTool::curve_on_face(new, face)?;
    let (point, tangent) = stored.pcurve.d1(parameter)?;
    let tolerance = BRepTool::edge_tolerance(new)?;
    let translation = seam_translation(&samples, point, tolerance)?;
    let translated = translated_pcurve(&stored.pcurve, samples.range, translation)?;
    let pair = if tangent.dot(samples.tangent) < 0. {
        (&translated, &stored.pcurve)
    } else {
        (&stored.pcurve, &translated)
    };
    BRepBuilder::default().update_edge_pcurves_on_face(new, pair, face, tolerance)?;
    Ok(0)
}
fn seam_samples(old: &Edge, face: &Face) -> Result<SeamSamples, Failure> {
    let original = BRepTool::curve_on_face(old, face)?;
    let mut reversed = old.clone();
    reversed.set_orientation(Orientation::Reversed);
    let second = BRepTool::curve_on_face(&reversed, face)?;
    let parameter = Tools::intermediate_point(second.first, second.last);
    let (first, tangent) = original.pcurve.d1(parameter)?;
    Ok(SeamSamples {
        first,
        second: second.pcurve.d1(parameter)?.0,
        tangent,
        range: (original.first, original.last),
    })
}
fn project_seam_point(
    edge: &Edge,
    face: &Face,
    samples: &SeamSamples,
    context: &Context,
) -> Result<Option<f64>, Failure> {
    let point = BRepTool::surface(face)?.value(samples.first.x(), samples.first.y())?;
    let projector = context.proj_pc(edge)?;
    let mut projector = projector.write();
    projector.perform(point)?;
    if projector.nb_points() == 0 {
        return Ok(None);
    }
    projector.lower_distance_parameter().map(Some)
}
fn seam_translation(
    samples: &SeamSamples,
    point: Point2,
    tolerance: f64,
) -> Result<Vector2, Failure> {
    let translation = Vector2::new(
        samples.second.x() - samples.first.x(),
        samples.second.y() - samples.first.y(),
    );
    let direction = Direction2::new(translation.x(), translation.y())?;
    let distance = if direction.x().abs() < tolerance {
        point.y() - samples.second.y()
    } else {
        point.x() - samples.second.x()
    };
    Ok(if distance.abs() < tolerance {
        -translation
    } else {
        translation
    })
}
fn translated_pcurve(
    curve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    translation: Vector2,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let copy: Handle<dyn Geom2dCurve> = curve.copy()?.cast();
    let trimmed: Handle<dyn Geom2dCurve> = Handle::new(Geom2dTrimmedCurve::new(
        &copy, range.0, range.1, true, true,
    )?)
    .cast();
    trimmed.translate(translation)?;
    Ok(trimmed)
}
