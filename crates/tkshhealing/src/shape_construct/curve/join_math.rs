use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};

/// The B-spline members `SegmentCurve` of ShapeConstruct.cxx uses, for 2D and 3D curves.
pub(super) trait SegmentedSpline {
    fn bounds(&self) -> (f64, f64);
    fn periodic(&self) -> bool;
    fn cut(&self, first: f64, last: f64) -> Result<(), Failure>;
}

impl SegmentedSpline for GeomBSplineCurve {
    fn bounds(&self) -> (f64, f64) {
        (self.first_parameter(), self.last_parameter())
    }
    fn periodic(&self) -> bool {
        self.is_periodic()
    }
    fn cut(&self, first: f64, last: f64) -> Result<(), Failure> {
        self.segment(first, last, precision::p_confusion())
    }
}

impl SegmentedSpline for Geom2dBSplineCurve {
    fn bounds(&self) -> (f64, f64) {
        (self.first_parameter(), self.last_parameter())
    }
    fn periodic(&self) -> bool {
        self.is_periodic()
    }
    fn cut(&self, first: f64, last: f64) -> Result<(), Failure> {
        self.segment(first, last, precision::p_confusion())
    }
}

/// `SegmentCurve(curve, first, last)`: a curve wider than `[first, last]` (by more than
/// PConfusion) is segmented to it, a non-periodic one only within its own range;
/// `segment_curve(&*spline, 0.0, 1.0)?`.
pub(super) fn segment_curve<C: SegmentedSpline + ?Sized>(
    curve: &C,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let (start, end) = curve.bounds();
    let confusion = precision::p_confusion();
    if !(start < first - confusion || end > last + confusion) {
        return Ok(());
    }
    if curve.periodic() {
        return curve.cut(first, last);
    }
    // std::max(FirstParameter, first), std::min(LastParameter, last).
    let low = if start < first { first } else { start };
    let high = if last < end { last } else { end };
    curve.cut(low, high)
}

pub(super) fn endpoint_reversals(
    distance_11: f64,
    distance_21: f64,
    distance_12: f64,
    distance_22: f64,
    confusion: f64,
) -> (bool, bool) {
    // std::min(a, b) is `b < a ? b : a`.
    let minimum_direct = if distance_21 < distance_11 {
        distance_21
    } else {
        distance_11
    };
    let minimum_crossed = if distance_22 < distance_12 {
        distance_22
    } else {
        distance_12
    };
    if (minimum_direct - minimum_crossed).abs() <= confusion || minimum_crossed > minimum_direct {
        return (distance_11 < distance_21, false);
    }
    if minimum_crossed < minimum_direct {
        return (distance_12 < distance_22, true);
    }
    (false, false)
}

#[cfg(test)]
mod tests {
    use super::endpoint_reversals;

    #[test]
    fn endpoint_selection_matches_native_tie_and_crossed_branches() {
        assert_eq!(endpoint_reversals(1., 2., 2., 1., 0.1), (true, false));
        assert_eq!(endpoint_reversals(2., 3., 0.1, 1., 0.1), (true, true));
    }
}
