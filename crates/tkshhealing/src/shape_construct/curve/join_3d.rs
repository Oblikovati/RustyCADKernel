use super::{ShapeConstructCurve, join_math::endpoint_reversals};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve},
    top_abs::Orientation,
};
use rustycad_tkgeombase::geom_convert::CompCurveToBSplineCurve;
use rustycad_tkmath::{convert::ParameterisationType, gp::Point3};

/// Joined 3D curve and the native endpoint-reversal decisions.
pub struct ShapeConstructCurveJoin3d {
    /// Composite output curve.
    pub curve: Handle<GeomBSplineCurve>,
    /// Whether native endpoint matching reverses the first input.
    pub reversed_first: bool,
    /// Whether native endpoint matching reverses the second input.
    pub reversed_second: bool,
}

impl ShapeConstructCurve {
    /// Join two oriented curves as `ShapeConstruct::JoinCurves` does. The input ranges are
    /// updated when an input orientation is reversed; `let joined = tool.join_curves_3d(...) ?;`.
    pub fn join_curves_3d(
        &self,
        first: &Handle<dyn GeomCurve>,
        second: &Handle<dyn GeomCurve>,
        first_orientation: Orientation,
        second_orientation: Orientation,
        first_range: &mut (f64, f64),
        second_range: &mut (f64, f64),
    ) -> Result<Option<ShapeConstructCurveJoin3d>, Failure> {
        let first = oriented_copy(first, first_orientation, first_range)?;
        let second = oriented_copy(second, second_orientation, second_range)?;
        join_oriented_curves(self, &first, &second, first_range, second_range)
    }
}

fn oriented_copy(
    curve: &Handle<dyn GeomCurve>,
    orientation: Orientation,
    range: &mut (f64, f64),
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let copy = curve.copy()?.cast::<dyn GeomCurve>();
    if orientation == Orientation::Reversed {
        reverse_curve_range(curve, &copy, range)?;
    }
    Ok(copy)
}

fn reverse_curve_range(
    curve: &Handle<dyn GeomCurve>,
    copy: &Handle<dyn GeomCurve>,
    range: &mut (f64, f64),
) -> Result<(), Failure> {
    let first = curve.reversed_parameter(range.1)?;
    let last = curve.reversed_parameter(range.0)?;
    *range = (first, last);
    copy.reverse()
}

fn join_oriented_curves(
    tool: &ShapeConstructCurve,
    first: &Handle<dyn GeomCurve>,
    second: &Handle<dyn GeomCurve>,
    first_range: &(f64, f64),
    second_range: &(f64, f64),
) -> Result<Option<ShapeConstructCurveJoin3d>, Failure> {
    let first =
        tool.convert_to_bspline(first, first_range.0, first_range.1, precision::confusion())?;
    let second = tool.convert_to_bspline(
        second,
        second_range.0,
        second_range.1,
        precision::confusion(),
    )?;
    join_bspline_curves(&first, &second)
}

fn join_bspline_curves(
    first: &Handle<GeomBSplineCurve>,
    second: &Handle<GeomBSplineCurve>,
) -> Result<Option<ShapeConstructCurveJoin3d>, Failure> {
    if first.is_null() || second.is_null() {
        return Ok(None);
    }
    let reversals = curve_endpoint_reversals(first, second)?;
    reverse_bspline_if(first, reversals.0)?;
    reverse_bspline_if(second, reversals.1)?;
    join_reversed_curves(first, second, reversals)
}

fn curve_endpoint_reversals(
    first: &GeomBSplineCurve,
    second: &GeomBSplineCurve,
) -> Result<(bool, bool), Failure> {
    let first_start = first.pole(1)?;
    let first_end = first.pole(first.nb_poles())?;
    let second_start = second.pole(1)?;
    let second_end = second.pole(second.nb_poles())?;
    let distances = endpoint_distances(first_start, first_end, second_start, second_end);
    Ok(endpoint_reversals(
        distances.0,
        distances.1,
        distances.2,
        distances.3,
        precision::confusion(),
    ))
}

fn endpoint_distances(
    first_start: Point3,
    first_end: Point3,
    second_start: Point3,
    second_end: Point3,
) -> (f64, f64, f64, f64) {
    (
        first_start.distance(&second_start),
        first_end.distance(&second_start),
        first_start.distance(&second_end),
        second_end.distance(&first_end),
    )
}

fn reverse_bspline_if(curve: &GeomBSplineCurve, reversed: bool) -> Result<(), Failure> {
    if reversed {
        return curve.reverse();
    }
    Ok(())
}

fn join_reversed_curves(
    first: &Handle<GeomBSplineCurve>,
    second: &Handle<GeomBSplineCurve>,
    reversals: (bool, bool),
) -> Result<Option<ShapeConstructCurveJoin3d>, Failure> {
    let midpoint = pole_midpoint(first, second)?;
    let first_last = first.nb_poles();
    first.set_pole(first_last, midpoint)?;
    second.set_pole(1, midpoint)?;
    append_composite_curve(first, second, reversals)
}

fn pole_midpoint(first: &GeomBSplineCurve, second: &GeomBSplineCurve) -> Result<Point3, Failure> {
    let first_point = first.pole(first.nb_poles())?;
    let second_point = second.pole(1)?;
    Ok(Point3::new(
        0.5 * (first_point.x() + second_point.x()),
        0.5 * (first_point.y() + second_point.y()),
        0.5 * (first_point.z() + second_point.z()),
    ))
}

fn append_composite_curve(
    first: &Handle<GeomBSplineCurve>,
    second: &Handle<GeomBSplineCurve>,
    reversals: (bool, bool),
) -> Result<Option<ShapeConstructCurveJoin3d>, Failure> {
    let mut composite = composite_builder(first)?;
    if !composite.add(
        &second.cast::<dyn GeomBoundedCurve>(),
        precision::confusion(),
        true,
        false,
        0,
    )? {
        return Ok(None);
    }
    let curve = composite.bspline_curve();
    Ok(Some(ShapeConstructCurveJoin3d {
        curve,
        reversed_first: reversals.0,
        reversed_second: reversals.1,
    }))
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
    use rustycad_tkg3d::{
        geom::{GeomCurve, GeomLine},
        top_abs::Orientation,
    };
    use rustycad_tkmath::gp::{Axis1, Direction3, Point3};

    #[test]
    fn join_curves_3d_composes_contiguous_lines_and_keeps_ranges() {
        let (joined, first_range, second_range) = adjacent_line_result();
        assert_eq!((first_range, second_range), ((0., 1.), (1., 2.)));
        assert_eq!(
            (joined.reversed_first, joined.reversed_second),
            (false, false)
        );
        assert_eq!(joined.curve.value(0.).unwrap(), Point3::new(0., 0., 0.));
        assert_eq!(joined.curve.value(2.).unwrap(), Point3::new(2., 0., 0.));
    }

    fn adjacent_line_result() -> (super::ShapeConstructCurveJoin3d, (f64, f64), (f64, f64)) {
        let line = line();
        let (mut first_range, mut second_range) = ((0., 1.), (1., 2.));
        let joined = ShapeConstructCurve
            .join_curves_3d(
                &line,
                &line,
                Orientation::Forward,
                Orientation::Forward,
                &mut first_range,
                &mut second_range,
            )
            .unwrap()
            .unwrap();
        (joined, first_range, second_range)
    }

    fn line() -> Handle<dyn GeomCurve> {
        let direction = Direction3::new(1., 0., 0.).unwrap();
        let axis = Axis1::new(Point3::new(0., 0., 0.), direction);
        Handle::new(GeomLine::new(axis)).cast()
    }
}
