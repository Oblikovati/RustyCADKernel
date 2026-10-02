// Rust translation foundation for ShapeFix_IntersectionTool::FixSelfIntersectWire.

//! Bounded p-curve intersection primitives used by wire self-intersection repair.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkmath::gp::{Point2, Xy};

/// A strict interior crossing found between two finite line p-curves.
///
/// Use [`line_pcurve_crossing`] to find one from two curves and their parameter ranges.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PcurveCrossing {
    /// Crossing coordinate in the face's UV plane.
    pub point: Point2,
    /// Parameter on the first input p-curve.
    pub first_parameter: f64,
    /// Parameter on the second input p-curve.
    pub second_parameter: f64,
}

/// Intersects two finite, line-backed p-curves and rejects endpoint contacts.
///
/// ```
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// use rustycad_tkshhealing::shape_fix::line_pcurve_crossing;
/// let a: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
///     Point2::new(0.5, -0.5), Direction2::new(-1.0, 1.0).unwrap(),
/// )).cast();
/// let b: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
///     Point2::new(-0.5, -0.5), Direction2::new(1.0, 1.0).unwrap(),
/// )).cast();
/// let hit = line_pcurve_crossing(&a, (0.0, 2.0_f64.sqrt()), &b, (0.0, 2.0_f64.sqrt()), 1.0e-10).unwrap();
/// assert!(hit.point.distance(Point2::new(0.0, 0.0)) < 1.0e-12);
/// ```
pub fn line_pcurve_crossing(
    first: &Handle<dyn Geom2dCurve>,
    first_range: (f64, f64),
    second: &Handle<dyn Geom2dCurve>,
    second_range: (f64, f64),
    tolerance: f64,
) -> Option<PcurveCrossing> {
    let first_line = first.cast::<Geom2dLine>();
    let second_line = second.cast::<Geom2dLine>();
    let (first_line, second_line) = (first_line.get()?, second_line.get()?);
    let first_axis = first_line.position();
    let second_axis = second_line.position();
    let denominator = cross(first_axis.direction().xy(), second_axis.direction().xy());
    if denominator.abs() <= tolerance {
        return None;
    }
    let delta = second_axis.location().xy() - first_axis.location().xy();
    let first_parameter = cross(delta, second_axis.direction().xy()) / denominator;
    let second_parameter = cross(delta, first_axis.direction().xy()) / denominator;
    if !strictly_inside(first_parameter, first_range, tolerance)
        || !strictly_inside(second_parameter, second_range, tolerance)
    {
        return None;
    }
    Some(PcurveCrossing {
        point: first_line.value(first_parameter).ok()?,
        first_parameter,
        second_parameter,
    })
}

fn cross(first: Xy, second: Xy) -> f64 {
    first.x() * second.y() - first.y() * second.x()
}

fn strictly_inside(parameter: f64, range: (f64, f64), tolerance: f64) -> bool {
    let (low, high) = if range.0 <= range.1 {
        range
    } else {
        (range.1, range.0)
    };
    parameter > low + tolerance && parameter < high - tolerance
}

#[cfg(test)]
mod tests {
    use super::line_pcurve_crossing;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
    use rustycad_tkmath::gp::{Direction2, Point2};

    #[test]
    fn diagonal_segments_find_interior_crossing_and_parameters() {
        let descending = line(Point2::new(0.5, -0.5), Direction2::new(-1.0, 1.0).unwrap());
        let ascending = line(Point2::new(-0.5, -0.5), Direction2::new(1.0, 1.0).unwrap());
        let hit = line_pcurve_crossing(
            &descending,
            (0.0, 2.0_f64.sqrt()),
            &ascending,
            (0.0, 2.0_f64.sqrt()),
            1.0e-10,
        )
        .expect("bowtie diagonals cross in the interior");
        assert!(hit.point.distance(Point2::new(0.0, 0.0)) < 1.0e-12);
        assert!((hit.first_parameter - 2.0_f64.sqrt() / 2.0).abs() < 1.0e-12);
        assert!((hit.second_parameter - 2.0_f64.sqrt() / 2.0).abs() < 1.0e-12);
    }

    #[test]
    fn endpoint_contacts_parallel_and_out_of_range_pairs_are_rejected() {
        let horizontal = line(Point2::new(-1.0, 0.0), Direction2::X);
        let vertical = line(Point2::new(0.0, -1.0), Direction2::Y);
        assert!(
            line_pcurve_crossing(&horizontal, (0.0, 1.0), &vertical, (0.0, 1.0), 1.0e-10).is_none()
        );
        assert!(
            line_pcurve_crossing(&horizontal, (0.0, 2.0), &vertical, (0.0, 2.0), 1.0e-10).is_some()
        );
        let parallel = line(Point2::new(-1.0, 1.0), Direction2::X);
        assert!(
            line_pcurve_crossing(&horizontal, (0.0, 2.0), &parallel, (0.0, 2.0), 1.0e-10).is_none()
        );
    }

    fn line(origin: Point2, direction: Direction2) -> Handle<dyn Geom2dCurve> {
        Handle::new(Geom2dLine::from_point(origin, direction)).cast()
    }
}
