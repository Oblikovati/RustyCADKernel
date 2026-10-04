use super::*;
use crate::bop_tools::algo_tools3d;
use rustycad_tkg3d::top_abs::Orientation;
pub(super) fn face_direction(
    edge: &Edge,
    face: &Face,
    frame: &mut FaceFrame,
    context: &Context,
) -> Result<(bool, Direction3, Direction3), Failure> {
    let mut normal = Direction3::default();
    algo_tools3d::get_normal_to_face_on_edge_at(
        edge,
        face,
        frame.parameter,
        &mut normal,
        Some(context),
    )?;
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    let mut binormal = normal.crossed(frame.tangent)?;
    let mut point = Point3::default();
    let found = !frame.small
        && find_point(
            face,
            frame,
            context,
            &mut binormal,
            &mut point,
            BRepTool::edge_tolerance(edge)?,
        )?;
    if found {
        return Ok((true, normal, binormal));
    }
    let found = algo_tools3d::approx_normal_with_context_step(
        edge,
        face,
        frame.parameter,
        frame.step,
        &mut point,
        &mut normal,
        context,
    )?;
    frame.plane.perform(point)?;
    if !frame.plane.is_done() {
        return Ok((false, normal, binormal));
    }
    point = frame.plane.nearest_point()?;
    binormal = Direction3::from_vector(Vector3::between(frame.point, point))?;
    Ok((found, normal, binormal))
}
fn find_point(
    face: &Face,
    frame: &mut FaceFrame,
    context: &Context,
    direction: &mut Direction3,
    output: &mut Point3,
    tolerance: f64,
) -> Result<bool, Failure> {
    let magnitude = frame.point.xyz().modulus();
    let threshold = if magnitude > 1000. {
        5e-16 * magnitude
    } else {
        precision::angular()
    };
    let cached = context.proj_ps(face)?;
    let mut projection = cached.write();
    let Some(start) = project_to_plane(&mut projection, &mut frame.plane, frame.point)? else {
        return Ok(false);
    };
    let shifted = start.translated(Vector3::from(direction.xyz()) * (2. * tolerance));
    let Some(start) = project_to_plane(&mut projection, &mut frame.plane, shifted)? else {
        return Ok(false);
    };
    for iteration in 0..15 {
        let point = start.translated(Vector3::from(direction.xyz()) * frame.step);
        projection.perform(point)?;
        if !projection.is_done() {
            return Ok(false);
        }
        *output = projection.nearest_point()?;
        let distance = projection.lower_distance()?;
        frame.plane.perform(*output)?;
        *output = frame.plane.nearest_point()?;
        let vector = Vector3::between(start, *output);
        if vector.square_magnitude() < precision::square_confusion() {
            return Ok(false);
        }
        *direction = Direction3::from_vector(vector)?;
        if distance.partial_cmp(&threshold) != Some(std::cmp::Ordering::Greater) || iteration == 14
        {
            return Ok(distance < threshold);
        }
    }
    Ok(false)
}
fn project_to_plane(
    surface: &mut ProjectPointOnSurf,
    plane: &mut ProjectPointOnSurf,
    point: Point3,
) -> Result<Option<Point3>, Failure> {
    surface.perform(point)?;
    if !surface.is_done() {
        return Ok(None);
    }
    plane.perform(surface.nearest_point()?)?;
    Ok(Some(plane.nearest_point()?))
}
