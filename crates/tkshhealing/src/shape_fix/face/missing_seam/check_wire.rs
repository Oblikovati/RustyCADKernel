use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Face, ShapeIterator, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OpenWireDirection {
    pub(super) u: i8,
    pub(super) v: i8,
    pub(super) all_degenerated: bool,
}

pub(super) fn check_wire(
    wire: &Wire,
    face: &Face,
    u_period: f64,
    v_period: f64,
) -> Result<Option<OpenWireDirection>, Failure> {
    let Some((displacement, all_degenerated)) = accumulate_pcurve_displacement(wire, face)? else {
        return Ok(None);
    };
    let direction = OpenWireDirection {
        u: period_open_direction(displacement.x(), u_period),
        v: period_open_direction(displacement.y(), v_period),
        all_degenerated,
    };
    Ok((direction.u != 0 || direction.v != 0).then_some(direction))
}

fn accumulate_pcurve_displacement(
    wire: &Wire,
    face: &Face,
) -> Result<Option<(Vector2, bool)>, Failure> {
    let mut displacement = Vector2::new(0.0, 0.0);
    let mut all_degenerated = true;
    for child in ShapeIterator::new(&wire.clone().into_shape(), true, true) {
        if !accumulate_edge_displacement(&mut displacement, &mut all_degenerated, &child, face)? {
            return Ok(None);
        }
    }
    Ok(Some((displacement, all_degenerated)))
}

fn accumulate_edge_displacement(
    displacement: &mut Vector2,
    all_degenerated: &mut bool,
    shape: &rustycad_tkbrep::topods::Shape,
    face: &Face,
) -> Result<bool, Failure> {
    let edge = topods::edge(shape)?;
    if !BRepTool::degenerated(&edge)? {
        *all_degenerated = false;
    }
    add_pcurve_delta(displacement, &edge, face)
}

fn add_pcurve_delta(
    displacement: &mut Vector2,
    edge: &rustycad_tkbrep::topods::Edge,
    face: &Face,
) -> Result<bool, Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
    let Some(curve) = curve.get() else {
        return Ok(false);
    };
    let first_point = curve.value(first)?;
    let last_point = curve.value(last)?;
    *displacement += point_delta(first_point, last_point);
    Ok(true)
}

fn point_delta(first: Point2, last: Point2) -> Vector2 {
    Vector2::new(last.x() - first.x(), last.y() - first.y())
}

fn period_open_direction(displacement: f64, period: f64) -> i8 {
    let delta = displacement.abs() - period;
    if delta.abs().partial_cmp(&(0.1 * period)) != Some(Ordering::Less) {
        return 0;
    }
    if displacement > 0.0 { 1 } else { -1 }
}

#[cfg(test)]
mod tests {
    use super::{period_open_direction, point_delta};
    use rustycad_tkmath::gp::Point2;

    #[test]
    fn native_open_direction_uses_strict_ten_percent_period_window() {
        assert_eq!(period_open_direction(6.3, 6.0), 1);
        assert_eq!(period_open_direction(-6.3, 6.0), -1);
        assert_eq!(period_open_direction(6.61, 6.0), 0);
        assert_eq!(period_open_direction(0.0, 6.0), 0);
        assert_eq!(period_open_direction(6.0, f64::NAN), 0);
    }

    #[test]
    fn native_pcurve_delta_subtracts_endpoints_componentwise() {
        let delta = point_delta(Point2::new(2.0, -3.0), Point2::new(8.0, 5.0));
        assert_eq!((delta.x(), delta.y()), (6.0, 8.0));
    }
}
