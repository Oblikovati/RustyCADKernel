use super::pcurve_union::{MergedPCurve, OrientedPCurve};
use crate::shape_construct::ShapeConstructProjectCurveOnSurface;
use rustycad_tkbrep::brep::{BRepTool, EdgePCurve};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom2d_convert::curve_to_bspline_curve;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Point2};

pub(super) fn merge_same_support_circles(
    face: &Face,
    merged: &Edge,
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<Option<MergedPCurve>, Failure> {
    let Some(first_circle) = matching_circles(first, second)? else {
        return Ok(None);
    };
    let second_points = pcurve_endpoints(&second.pcurve)?;
    let extension = mapped_range(first_circle, second_points)?;
    let range = extend_range(
        (first.pcurve.first, first.pcurve.last),
        extension,
        first.edge_orientation,
    );
    let result = oriented_circle(first_circle, range, first.edge_orientation)?;
    Ok(Some(reparameterize_circle(face, merged, result)?))
}

fn matching_circles(
    first: &OrientedPCurve,
    second: &OrientedPCurve,
) -> Result<Option<Circle2>, Failure> {
    if !stored(first) || !stored(second) {
        return Ok(None);
    }
    let Some(first_circle) = circle(&first.pcurve)? else {
        return Ok(None);
    };
    let Some(second_circle) = circle(&second.pcurve)? else {
        return Ok(None);
    };
    if same_circle(first_circle, second_circle) {
        return Ok(Some(first_circle));
    }
    Ok(None)
}

fn stored(curve: &OrientedPCurve) -> bool {
    curve.pcurve.is_stored && !curve.pcurve.pcurve.is_null()
}

fn circle(curve: &EdgePCurve) -> Result<Option<Circle2>, Failure> {
    let adaptor = Geom2dAdaptorCurve::with_curve(&curve.pcurve)?;
    if adaptor.get_type()? != rustycad_tkmath::geom_abs::CurveType::Circle {
        return Ok(None);
    }
    Ok(Some(adaptor.circle()?))
}

fn same_circle(first: Circle2, second: Circle2) -> bool {
    first.location().distance(second.location()) <= precision::confusion()
        && (first.radius() - second.radius()).abs() <= precision::confusion()
}

fn pcurve_endpoints(curve: &EdgePCurve) -> Result<[Point2; 2], Failure> {
    Ok([
        curve.pcurve.eval_d0(curve.first)?,
        curve.pcurve.eval_d0(curve.last)?,
    ])
}

fn mapped_range(circle: Circle2, endpoints: [Point2; 2]) -> Result<(f64, f64), Failure> {
    let first = elclib::parameter(&circle, endpoints[0])?;
    let last = elclib::parameter(&circle, endpoints[1])?;
    Ok((first.min(last), first.max(last)))
}

fn extend_range(first: (f64, f64), extension: (f64, f64), orientation: Orientation) -> (f64, f64) {
    if orientation == Orientation::Reversed {
        return (extension.0, first.1);
    }
    (first.0, extension.1)
}

fn oriented_circle(
    circle: Circle2,
    mut range: (f64, f64),
    orientation: Orientation,
) -> Result<MergedPCurve, Failure> {
    let curve = Handle::new(Geom2dCircle::from_circ2d(circle)).cast::<dyn Geom2dCurve>();
    if orientation != Orientation::Reversed {
        return Ok(MergedPCurve { curve, range });
    }
    let geometry = curve.get().ok_or_else(null_curve)?;
    range = (
        geometry.reversed_parameter(range.1)?,
        geometry.reversed_parameter(range.0)?,
    );
    geometry.reverse()?;
    Ok(MergedPCurve { curve, range })
}

fn reparameterize_circle(
    face: &Face,
    merged: &Edge,
    mut result: MergedPCurve,
) -> Result<MergedPCurve, Failure> {
    let (curve, first, last) = BRepTool::curve(merged)?;
    let tolerance = BRepTool::edge_tolerance(merged)?;
    if (last - first - (result.range.1 - result.range.0)).abs() > tolerance {
        if let Some(projected) = project_bad_range(face, &curve, (first, last))? {
            return Ok(projected);
        }
        return reparameterized_spline(result, (first, last));
    }
    let edge_range = (first, last);
    rotate_circle(&mut result.curve, result.range.0 - edge_range.0)?;
    result.range = edge_range;
    Ok(result)
}

fn reparameterized_spline(
    result: MergedPCurve,
    range: (f64, f64),
) -> Result<MergedPCurve, Failure> {
    let trimmed =
        Geom2dTrimmedCurve::new(&result.curve, result.range.0, result.range.1, true, false)?;
    let trimmed = Handle::new(trimmed).cast::<dyn Geom2dCurve>();
    let spline = curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)?;
    let mut knots = spline.knots();
    bsplclib::reparametrize(range.0, range.1, &mut knots)?;
    spline.set_knots(&knots)?;
    Ok(MergedPCurve {
        curve: spline.cast(),
        range,
    })
}

fn project_bad_range(
    face: &Face,
    curve: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    range: (f64, f64),
) -> Result<Option<MergedPCurve>, Failure> {
    let surface = BRepTool::surface(face)?;
    let projector = ShapeConstructProjectCurveOnSurface::new();
    projector.init(&surface, precision::confusion())?;
    let mut projected = Handle::default();
    if !projector.perform(curve, range, &mut projected, (-1.0, -1.0))? {
        return Ok(None);
    }
    Ok(Some(MergedPCurve {
        curve: projected,
        range,
    }))
}

fn rotate_circle(curve: &mut Handle<dyn Geom2dCurve>, offset: f64) -> Result<(), Failure> {
    curve.get().ok_or_else(null_curve)?;
    let circle = Geom2dAdaptorCurve::with_curve(curve)?.circle()?;
    let circle = reparameterized_circle(circle, offset);
    *curve = Handle::new(Geom2dCircle::from_circ2d(circle)).cast();
    Ok(())
}

fn reparameterized_circle(mut circle: Circle2, offset: f64) -> Circle2 {
    let mut position = circle.axis();
    position.rotate(circle.location(), offset);
    circle.set_axis(position);
    circle
}

fn null_curve() -> Failure {
    Failure::with_type("Standard_NullObject", "null circle p-curve", "")
}

#[cfg(test)]
#[path = "pcurve_circle_tests.rs"]
mod tests;
