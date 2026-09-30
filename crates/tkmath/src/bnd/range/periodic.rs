use super::{Failure, IntersectStatus, List, Range1};
fn real_equal(left: f64, right: f64) -> bool {
    (left - right).abs() < f64::MIN_POSITIVE
}
pub(super) fn classify(range: Range1, value: f64, period: f64) -> IntersectStatus {
    if range.is_void() {
        return IntersectStatus::Out;
    }
    let period = period.abs();
    let first = range.first - value;
    let last = range.last - value;
    if period <= f64::MIN_POSITIVE {
        return nonperiodic(first * last);
    }
    let low = first / period;
    let high = last / period;
    let low_int = low.floor() as i32;
    let high_int = high.floor() as i32;
    if low_int != high_int {
        return if real_equal(high, f64::from(high_int)) {
            IntersectStatus::Boundary
        } else {
            IntersectStatus::In
        };
    }
    if real_equal(low, f64::from(low_int)) {
        IntersectStatus::Boundary
    } else {
        IntersectStatus::Out
    }
}
fn nonperiodic(product: f64) -> IntersectStatus {
    if real_equal(product, 0.0) {
        return IntersectStatus::Boundary;
    }
    if product > 0.0 {
        IntersectStatus::Out
    } else {
        IntersectStatus::In
    }
}
pub(super) fn split(
    range: Range1,
    value: f64,
    list: &mut List<Range1>,
    period: f64,
) -> Result<(), Failure> {
    let period = period.abs();
    if classify(range, value, period) != IntersectStatus::In {
        list.append(range);
        return Ok(());
    }
    if period == 0.0 {
        list.append(Range1::new(range.first, value)?);
        list.append(Range1::new(value, range.last)?);
        return Ok(());
    }
    split_periodic(range, value, list, period)
}
fn split_periodic(
    range: Range1,
    value: f64,
    list: &mut List<Range1>,
    period: f64,
) -> Result<(), Failure> {
    let mut previous = value + period * ((range.first - value) / period).ceil();
    if previous > range.first {
        list.append(Range1::new(range.first, previous)?);
    }
    let mut next = previous + period;
    while next <= range.last {
        list.append(Range1::new(previous, next)?);
        previous = next;
        next += period;
    }
    if previous < range.last {
        list.append(Range1::new(previous, range.last)?);
    }
    Ok(())
}
