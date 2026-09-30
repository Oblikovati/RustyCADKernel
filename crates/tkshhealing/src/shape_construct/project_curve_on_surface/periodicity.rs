use super::*;
use crate::shape_analysis::adjust_to_period;

pub(super) fn fix_periodicity(
    points: &mut [Point2; 4],
    coordinate: i32,
    period: f64,
    saved_index: i32,
    saved: f64,
) -> Result<bool, Failure> {
    let (saved, index) = if saved_index < 0 {
        (0.5 * period, 0)
    } else {
        (saved, saved_index as usize)
    };
    let (minimum, maximum) = period_window(saved, period, saved_index >= 0);
    let is_iso =
        maximum - saved < precision::p_confusion() || saved - minimum < precision::p_confusion();
    let iso = is_iso.then_some(saved);
    for point in points.iter_mut() {
        normalize_coordinate(point, coordinate, (minimum, maximum), iso)?;
    }
    let Some(decreasing) = period_jump_direction(points, coordinate)? else {
        return Ok(false);
    };
    repair_period_jumps(points, coordinate, period, index, decreasing)?;
    Ok(true)
}

fn period_window(saved: f64, period: f64, has_saved: bool) -> (f64, f64) {
    let (mut minimum, mut maximum) = (0., period);
    if !has_saved {
        return (minimum, maximum);
    }
    while minimum > saved {
        minimum -= period;
        maximum -= period;
    }
    while maximum < saved {
        minimum += period;
        maximum += period;
    }
    (minimum, maximum)
}

fn normalize_coordinate(
    point: &mut Point2,
    coordinate: i32,
    range: (f64, f64),
    iso: Option<f64>,
) -> Result<(), Failure> {
    let mut parameter = point.coord(coordinate)?;
    parameter += adjust_to_period(parameter, range.0, range.1);
    if let Some(iso) = iso {
        if range.1 - parameter < precision::p_confusion()
            || parameter - range.0 < precision::p_confusion()
        {
            parameter = iso;
        }
    } else {
        if range.1 - parameter < precision::p_confusion() {
            parameter = range.1;
        }
        if parameter - range.0 < precision::p_confusion() {
            parameter = range.0;
        }
    }
    point.set_coord(coordinate, parameter)
}

fn period_jump_direction(points: &[Point2; 4], coordinate: i32) -> Result<Option<bool>, Failure> {
    let mut previous = 0.;
    let mut product = 1.;
    let mut jumped = false;
    for index in 0..3 {
        let difference = points[index + 1].coord(coordinate)? - points[index].coord(coordinate)?;
        if difference < -precision::p_confusion() {
            product *= -1.;
        }
        if difference * previous < -precision::p_confusion() {
            jumped = true;
        }
        previous = difference;
    }
    Ok(jumped.then_some(product > 0.))
}

fn repair_period_jumps(
    points: &mut [Point2; 4],
    coordinate: i32,
    period: f64,
    saved: usize,
    decreasing: bool,
) -> Result<(), Failure> {
    for index in (1..=saved).rev() {
        repair_period_pair(points, (index, index - 1), coordinate, period, decreasing)?;
    }
    for index in saved..3 {
        repair_period_pair(points, (index, index + 1), coordinate, period, !decreasing)?;
    }
    Ok(())
}

fn repair_period_pair(
    points: &mut [Point2; 4],
    indices: (usize, usize),
    coordinate: i32,
    period: f64,
    increase: bool,
) -> Result<(), Failure> {
    let current = points[indices.0].coord(coordinate)?;
    let next = points[indices.1].coord(coordinate)?;
    let repair = if increase {
        current > next
    } else {
        current < next
    };
    if repair {
        points[indices.1].set_coord(coordinate, next + if increase { period } else { -period })?;
    }
    Ok(())
}
