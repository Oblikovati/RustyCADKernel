//! Sampling and primitive emission preserve native point append order and arrow location.
use crate::prs3d::{Prs3dArrow, Prs3dDrawer};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::{
    geom_abs::CurveType,
    gp::{Direction3, Point3},
};
use rustycad_tkservice::graphic3d::*;

pub(super) fn draw_curve(
    curve: &dyn Adaptor3dCurve,
    group: &Handle<dyn Graphic3dGroup>,
    count: i32,
    bounds: [f64; 2],
    points: &mut Sequence<Point3>,
    draw: bool,
) -> Result<(), Failure> {
    let line = curve.get_type()? == CurveType::Line;
    let count = if line { 2 } else { sample_count(curve, count)? };
    let array = primitive_array(line, count, draw)?;
    if line {
        append_line(curve, bounds, points, array.as_deref())?;
    } else {
        append_samples(curve, bounds, count, points, array.as_deref())?;
    }
    if let Some(array) = array {
        group.add_primitive_array(&array, true);
    }
    Ok(())
}
fn primitive_array(
    line: bool,
    count: i32,
    draw: bool,
) -> Result<Option<Handle<Graphic3dArrayOfPrimitives>>, Failure> {
    if !draw {
        return Ok(None);
    }
    let kind = if line {
        Graphic3dTypeOfPrimitiveArray::Segments
    } else {
        Graphic3dTypeOfPrimitiveArray::Polylines
    };
    Ok(Some(Handle::new(Graphic3dArrayOfPrimitives::new(
        kind, count, 0, 0, 0,
    )?)))
}
fn sample_count(curve: &dyn Adaptor3dCurve, count: i32) -> Result<i32, Failure> {
    let intervals = if curve.get_type()? == CurveType::BSplineCurve {
        ((curve.nb_knots()? - 1) / 3).max(1)
    } else {
        1
    };
    Ok((count * intervals).max(2))
}
fn append_line(
    curve: &dyn Adaptor3dCurve,
    bounds: [f64; 2],
    points: &mut Sequence<Point3>,
    array: Option<&Graphic3dArrayOfPrimitives>,
) -> Result<(), Failure> {
    // Native evaluates both endpoints before it modifies the caller's sequence.
    let ends = [curve.value(bounds[0])?, curve.value(bounds[1])?];
    for point in ends {
        append_point(point, points, array)?;
    }
    Ok(())
}
fn append_samples(
    curve: &dyn Adaptor3dCurve,
    bounds: [f64; 2],
    count: i32,
    points: &mut Sequence<Point3>,
    array: Option<&Graphic3dArrayOfPrimitives>,
) -> Result<(), Failure> {
    let step = (bounds[1] - bounds[0]) / f64::from(count - 1);
    for index in 0..count {
        append_point(
            curve.value(bounds[0] + f64::from(index) * step)?,
            points,
            array,
        )?;
    }
    Ok(())
}
fn append_point(
    point: Point3,
    points: &mut Sequence<Point3>,
    array: Option<&Graphic3dArrayOfPrimitives>,
) -> Result<(), Failure> {
    points.append(point);
    if let Some(array) = array {
        array.add_vertex_pnt(&point)?;
    }
    Ok(())
}
pub(super) fn draw_arrow(
    curve: &dyn Adaptor3dCurve,
    group: &Handle<dyn Graphic3dGroup>,
    drawer: &Prs3dDrawer,
) -> Result<(), Failure> {
    if !drawer.line_arrow_draw() {
        return Ok(());
    }
    // OCCT uses the adaptor's last parameter, even for an explicitly trimmed drawing.
    let (tip, tangent) = curve.d1(curve.last_parameter()?)?;
    let aspect = drawer.arrow_aspect();
    Prs3dArrow::draw(
        group,
        &tip,
        &Direction3::from_vector(tangent)?,
        aspect.angle(),
        aspect.length(),
    )
}
