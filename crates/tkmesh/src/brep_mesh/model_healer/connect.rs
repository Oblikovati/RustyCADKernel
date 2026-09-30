//! Endpoint associations preserve native alias-sensitive write order.
use super::super::CurvePointReference as PointRef;
use super::*;
use crate::imesh_data::PCurveHandle;
type Ends = [PointRef; 2];
type Closest = (PointRef, PointRef, f64);
fn endpoints(curve: &PCurveHandle) -> Result<Ends, Failure> {
    let result = [
        PointRef::new(curve.downgrade(), 0),
        PointRef::new(curve.downgrade(), curve.parameters_nb() - 1),
    ];
    result[0].get()?;
    result[1].get()?;
    Ok(result)
}
fn other(point: &PointRef, ends: &Ends) -> PointRef {
    ends[usize::from(point == &ends[0])].clone()
}
fn closest(point: &PointRef, ends: &Ends) -> Result<(PointRef, f64), Failure> {
    let first = point.get()?.square_distance(ends[0].get()?);
    let second = point.get()?.square_distance(ends[1].get()?);
    Ok(if first < second {
        (ends[0].clone(), first)
    } else {
        (ends[1].clone(), second)
    })
}
fn closest_pair(first: &Ends, second: &Ends) -> Result<Closest, Failure> {
    let (point1, distance1) = closest(&first[0], second)?;
    let (point2, distance2) = closest(&first[1], second)?;
    Ok(
        if distance1 - distance2 < rustycad_tkmath::gp::resolution() {
            (first[0].clone(), point1, distance1)
        } else {
            (first[1].clone(), point2, distance2)
        },
    )
}
fn adjust(
    major1: PointRef,
    minor1: PointRef,
    mut major2: PointRef,
    mut minor2: PointRef,
    major: &Ends,
    minor: &Ends,
) -> Result<(), Failure> {
    if major2 == major1 {
        major2 = other(&major2, major);
        minor2 = closest(&major2, minor)?.0;
    }
    major1.set(minor1.get()?)?;
    major2.set(minor2.get()?)
}
pub(super) fn connect(
    previous: &PCurveHandle,
    current: &PCurveHandle,
    next: &PCurveHandle,
) -> Result<(), Failure> {
    if previous.is_internal() || current.is_internal() || next.is_internal() {
        return Ok(());
    }
    let prev = endpoints(previous)?;
    if previous == current {
        return prev[0].set(prev[1].get()?);
    }
    let curr = endpoints(current)?;
    let (prev_uv, curr_prev, prev_distance) = closest_pair(&prev, &curr)?;
    if previous == next {
        other(&prev_uv, &prev).set(other(&curr_prev, &curr).get()?)?;
        return prev_uv.set(curr_prev.get()?);
    }
    connect_three(prev, curr, (prev_uv, curr_prev, prev_distance), next)
}
fn connect_three(
    prev: Ends,
    curr: Ends,
    preceding: Closest,
    next: &PCurveHandle,
) -> Result<(), Failure> {
    let (prev_uv, curr_prev, prev_distance) = preceding;
    let following = endpoints(next)?;
    let (next_uv, curr_next, next_distance) = closest_pair(&following, &curr)?;
    if prev_distance - next_distance > rustycad_tkmath::gp::resolution() {
        adjust(curr_next, next_uv, curr_prev, prev_uv, &curr, &prev)
    } else {
        adjust(curr_prev, prev_uv, curr_next, next_uv, &curr, &following)
    }
}

#[cfg(test)]
#[path = "connect_tests.rs"]
mod tests;
