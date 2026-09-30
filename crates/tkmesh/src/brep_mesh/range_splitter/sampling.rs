use super::*;
pub(super) fn update_range(first: f64, last: f64, periodic: bool, range: &mut (f64, f64)) {
    if !(range.0 < first || range.1 > last) {
        return;
    }
    if periodic {
        if range.1 - range.0 > last - first {
            range.1 = range.0 + (last - first);
        }
        return;
    }
    // OCCT #23675: wholly disjoint domains are intentionally retained.
    if range.0 < last && range.1 > first {
        if first > range.0 {
            range.0 = first;
        }
        if last < range.1 {
            range.1 = last;
        }
    }
}
pub(super) fn sample_length(
    surface: &BRepAdaptorSurface,
    along: (f64, f64),
    across: (f64, f64),
    along_u: bool,
) -> Result<f64, Failure> {
    let step = 0.05 * (along.1 - along.0);
    let mut previous = sample_triplet(surface, along.0, across, along_u)?;
    let mut parameter = along.0 + step;
    let mut length = 0.0;
    for _ in 1..=20 {
        let current = sample_triplet(surface, parameter, across, along_u)?;
        length += previous[0].distance(&current[0])
            + previous[1].distance(&current[1])
            + previous[2].distance(&current[2]);
        previous = current;
        parameter += step;
    }
    Ok(length / 3.0)
}
fn sample_triplet(
    surface: &BRepAdaptorSurface,
    parameter: f64,
    across: (f64, f64),
    along_u: bool,
) -> Result<[Point3; 3], Failure> {
    let coordinates = [across.0, 0.5 * (across.1 + across.0), across.1];
    let mut points = [Point3::default(); 3];
    for (point, coordinate) in points.iter_mut().zip(coordinates) {
        *point = if along_u {
            surface.value(parameter, coordinate)?
        } else {
            surface.value(coordinate, parameter)?
        };
    }
    Ok(points)
}
