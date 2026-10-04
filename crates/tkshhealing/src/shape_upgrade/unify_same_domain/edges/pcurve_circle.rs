use super::pcurve_union::{MergedPCurve, OrientedPCurve};
use rustycad_tkbrep::brep::{BRepTool, EdgePCurve};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Point2};

pub(super) fn merge_same_support_circles(
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
    Ok(Some(reparameterize_circle(merged, result)?))
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
        return (first.0.min(extension.0), first.1);
    }
    (first.0, first.1.max(extension.1))
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

fn reparameterize_circle(merged: &Edge, mut result: MergedPCurve) -> Result<MergedPCurve, Failure> {
    let edge_range = BRepTool::range(merged)?;
    let tolerance = BRepTool::edge_tolerance(merged)?;
    if range_matches(result.range, edge_range, tolerance) {
        return Ok(result);
    }
    rotate_circle(&mut result.curve, result.range.0 - edge_range.0)?;
    result.range = edge_range;
    Ok(result)
}

fn range_matches(first: (f64, f64), second: (f64, f64), tolerance: f64) -> bool {
    (first.0 - second.0).abs() <= tolerance && (first.1 - second.1).abs() <= tolerance
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
