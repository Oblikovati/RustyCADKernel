use super::ShapeConstructCurve;
use super::join_math::{endpoint_reversals, segment_curve};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_api;
use rustycad_tkgeombase::geom_convert::CompCurveToBSplineCurve;
use rustycad_tkmath::{
    convert::ParameterisationType,
    gp::{Direction3, Plane3, Point2, Point3},
};

/// Inputs to native `ShapeConstruct::JoinCurves` for two p-curves.
pub struct ShapeConstructCurveJoin2dRequest {
    /// First p-curve.
    pub first: Handle<dyn Geom2dCurve>,
    /// Second p-curve.
    pub second: Handle<dyn Geom2dCurve>,
    /// First topological orientation.
    pub first_orientation: Orientation,
    /// Second topological orientation.
    pub second_orientation: Orientation,
    /// Mutable first parameter range.
    pub first_range: (f64, f64),
    /// Mutable second parameter range.
    pub second_range: (f64, f64),
    /// Whether invalid-topology crossing protection is enabled.
    pub is_error: bool,
}

/// Joined 2D curve and the native endpoint-reversal decisions (`C2dOut`, `isRev1`, `isRev2`).
pub struct ShapeConstructCurveJoin2d {
    /// Composite output p-curve (GeomAPI::To2d of the 3D join); `None` when the native
    /// call returns false.
    pub curve: Option<Handle<dyn Geom2dCurve>>,
    /// Whether native endpoint matching reverses the first input; also set when the
    /// crossing guard or the composition fails, false when a conversion fails first.
    pub reversed_first: bool,
    /// Whether native endpoint matching reverses the second input (as `reversed_first`).
    pub reversed_second: bool,
}

impl ShapeConstructCurve {
    /// Join two oriented p-curves; `is_error` enables OCCT's invalid-topology crossing guard
    /// and `joined.curve` is `None` where the native call returns false.
    /// `let joined = tool.join_curves_2d(&mut request)?;`.
    pub fn join_curves_2d(
        &self,
        request: &mut ShapeConstructCurveJoin2dRequest,
    ) -> Result<ShapeConstructCurveJoin2d, Failure> {
        let first = oriented_copy(
            &request.first,
            request.first_orientation,
            &mut request.first_range,
        )?;
        let second = oriented_copy(
            &request.second,
            request.second_orientation,
            &mut request.second_range,
        )?;
        join_oriented_curves(
            self,
            &first,
            &second,
            &request.first_range,
            &request.second_range,
            request.is_error,
        )
    }
}

fn oriented_copy(
    curve: &Handle<dyn Geom2dCurve>,
    orientation: Orientation,
    range: &mut (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let copy = curve.copy()?.cast::<dyn Geom2dCurve>();
    if orientation == Orientation::Reversed {
        reverse_curve_range(curve, &copy, range)?;
    }
    Ok(copy)
}

fn reverse_curve_range(
    curve: &Handle<dyn Geom2dCurve>,
    copy: &Handle<dyn Geom2dCurve>,
    range: &mut (f64, f64),
) -> Result<(), Failure> {
    let first = curve.reversed_parameter(range.1)?;
    let last = curve.reversed_parameter(range.0)?;
    *range = (first, last);
    copy.reverse()
}

fn join_oriented_curves(
    tool: &ShapeConstructCurve,
    first: &Handle<dyn Geom2dCurve>,
    second: &Handle<dyn Geom2dCurve>,
    first_range: &(f64, f64),
    second_range: &(f64, f64),
    is_error: bool,
) -> Result<ShapeConstructCurveJoin2d, Failure> {
    let first =
        tool.convert_to_bspline_2d(first, first_range.0, first_range.1, precision::confusion())?;
    let second = tool.convert_to_bspline_2d(
        second,
        second_range.0,
        second_range.1,
        precision::confusion(),
    )?;
    if first.is_null() || second.is_null() {
        return Ok(not_joined((false, false)));
    }
    segment_curve(&*first, first_range.0, first_range.1)?;
    segment_curve(&*second, second_range.0, second_range.1)?;
    join_bspline_curves(&first, &second, is_error)
}

fn not_joined(reversals: (bool, bool)) -> ShapeConstructCurveJoin2d {
    ShapeConstructCurveJoin2d {
        curve: None,
        reversed_first: reversals.0,
        reversed_second: reversals.1,
    }
}

fn join_bspline_curves(
    first: &Handle<Geom2dBSplineCurve>,
    second: &Handle<Geom2dBSplineCurve>,
    is_error: bool,
) -> Result<ShapeConstructCurveJoin2d, Failure> {
    let reversals = curve_endpoint_reversals(first, second)?;
    reverse_bspline_if(first, reversals.0)?;
    reverse_bspline_if(second, reversals.1)?;
    join_reversed_curves(first, second, reversals, is_error)
}

fn curve_endpoint_reversals(
    first: &Geom2dBSplineCurve,
    second: &Geom2dBSplineCurve,
) -> Result<(bool, bool), Failure> {
    let distances = endpoint_distances(
        first.pole(1)?,
        first.pole(first.nb_poles())?,
        second.pole(1)?,
        second.pole(second.nb_poles())?,
    );
    Ok(endpoint_reversals(
        distances.0,
        distances.1,
        distances.2,
        distances.3,
        precision::confusion(),
    ))
}

fn endpoint_distances(
    first_start: Point2,
    first_end: Point2,
    second_start: Point2,
    second_end: Point2,
) -> (f64, f64, f64, f64) {
    (
        first_start.distance(second_start),
        first_end.distance(second_start),
        first_start.distance(second_end),
        second_end.distance(first_end),
    )
}

fn reverse_bspline_if(curve: &Geom2dBSplineCurve, reversed: bool) -> Result<(), Failure> {
    if reversed {
        return curve.reverse();
    }
    Ok(())
}

fn join_reversed_curves(
    first: &Handle<Geom2dBSplineCurve>,
    second: &Handle<Geom2dBSplineCurve>,
    reversals: (bool, bool),
    is_error: bool,
) -> Result<ShapeConstructCurveJoin2d, Failure> {
    if is_error && crossing_segments(first, second)? {
        return Ok(not_joined(reversals));
    }
    adjust_join_endpoint(first, second)?;
    append_composite_curve(first, second, reversals)
}

fn crossing_segments(
    first: &Geom2dBSplineCurve,
    second: &Geom2dBSplineCurve,
) -> Result<bool, Failure> {
    let (start, end, middle) = first_sample_points(first)?;
    let length = start.distance(end);
    let circle_like = length < start.distance(middle) + precision::p_confusion();
    Ok(start.distance(second.pole(1)?) < length && !circle_like)
}

fn first_sample_points(curve: &Geom2dBSplineCurve) -> Result<(Point2, Point2, Point2), Failure> {
    let first = curve.first_parameter();
    let last = curve.last_parameter();
    Ok((
        curve.value(first)?,
        curve.value(last)?,
        curve.value((first + last) * 0.5)?,
    ))
}

fn adjust_join_endpoint(
    first: &Geom2dBSplineCurve,
    second: &Geom2dBSplineCurve,
) -> Result<(), Failure> {
    let midpoint = pole_midpoint(first, second)?;
    first.set_pole(first.nb_poles(), midpoint)?;
    second.set_pole(1, midpoint)
}

fn pole_midpoint(
    first: &Geom2dBSplineCurve,
    second: &Geom2dBSplineCurve,
) -> Result<Point2, Failure> {
    let first_point = first.pole(first.nb_poles())?;
    let second_point = second.pole(1)?;
    Ok(Point2::new(
        0.5 * (first_point.x() + second_point.x()),
        0.5 * (first_point.y() + second_point.y()),
    ))
}

// abv 01 Sep 99: Geom2dConvert always reparametrises the second curve, so both curves
// go to 3D on the XOY plane (GeomAPI::To3d), are joined there and come back (To2d).
fn append_composite_curve(
    first: &Handle<Geom2dBSplineCurve>,
    second: &Handle<Geom2dBSplineCurve>,
    reversals: (bool, bool),
) -> Result<ShapeConstructCurveJoin2d, Failure> {
    let plane = Plane3::new(Point3::new(0., 0., 0.), Direction3::new(0., 0., 1.)?)?;
    let first_3d = geom_api::to_3d(&first.cast(), &plane)?.cast::<GeomBSplineCurve>();
    let second_3d = geom_api::to_3d(&second.cast(), &plane)?.cast::<GeomBSplineCurve>();
    let mut composite = composite_builder(&first_3d)?;
    let added = composite.add(
        &second_3d.cast::<dyn GeomBoundedCurve>(),
        precision::p_confusion(),
        true,
        false,
        0,
    )?;
    let curve = if added {
        Some(geom_api::to_2d(&composite.bspline_curve().cast(), &plane)?)
    } else {
        None
    };
    Ok(ShapeConstructCurveJoin2d {
        curve,
        reversed_first: reversals.0,
        reversed_second: reversals.1,
    })
}

fn composite_builder(first: &Handle<GeomBSplineCurve>) -> Result<CompCurveToBSplineCurve, Failure> {
    CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn GeomBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )
}

#[cfg(test)]
mod tests {
    use super::ShapeConstructCurve;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
    use rustycad_tkg3d::top_abs::Orientation;
    use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};

    #[test]
    fn join_curves_2d_composes_adjacent_lines_and_reports_reversals() {
        let joined = adjacent_line_result();
        assert_eq!(
            (joined.reversed_first, joined.reversed_second),
            (false, false)
        );
        let curve = joined.curve.unwrap();
        assert_eq!(curve.value(0.).unwrap(), Point2::new(0., 0.));
        assert_eq!(curve.value(2.).unwrap(), Point2::new(2., 0.));
    }

    fn adjacent_line_result() -> super::ShapeConstructCurveJoin2d {
        let line = line();
        let mut request = super::ShapeConstructCurveJoin2dRequest {
            first: line.clone(),
            second: line,
            first_orientation: Orientation::Forward,
            second_orientation: Orientation::Forward,
            first_range: (0., 1.),
            second_range: (1., 2.),
            is_error: true,
        };
        ShapeConstructCurve.join_curves_2d(&mut request).unwrap()
    }

    fn line() -> Handle<dyn Geom2dCurve> {
        let direction = Direction2::new(1., 0.).unwrap();
        let axis = Axis2d::new(Point2::new(0., 0.), direction);
        Handle::new(Geom2dLine::new(axis)).cast()
    }
}
