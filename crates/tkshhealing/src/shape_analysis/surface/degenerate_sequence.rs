use super::degenerate::{copy_degenerate_coordinate, degenerate_coordinate};
use super::*;
use rustycad_tkg2d::geom2d::native::std_min;

impl ShapeAnalysisSurface {
    /// Preserves the explicit native count, one-based sequence access and partial gap updates.
    /// `analysis.project_degenerated_sequence(3, &points, &mut uv, 1e-7, true)?;`
    pub fn project_degenerated_sequence(
        &self,
        count: i32,
        points: &[Point3],
        uv: &mut [Point2],
        tolerance: f64,
        direct: bool,
    ) -> Result<bool, Failure> {
        let start = if direct { 1 } else { count };
        let entry = self.sequence_singularity(points, uv, start, tolerance)?;
        let Some(entry) = entry else {
            return Ok(false);
        };
        let boundary = self.sequence_regular_point(count, points, uv, tolerance, direct, entry)?;
        apply_sequence_degeneracy(count, uv, direct, entry.u_iso, boundary)?;
        Ok(true)
    }

    fn sequence_singularity(
        &self,
        points: &[Point3],
        uv: &[Point2],
        index: i32,
        tolerance: f64,
    ) -> Result<Option<SurfaceSingularity>, Failure> {
        let (entry, minimum) =
            self.select_singularity(tolerance, tolerance * tolerance, |entry| {
                let squared = entry.point.square_distance(&sequence_value(points, index)?);
                if squared > tolerance * tolerance {
                    return Ok(std_min(
                        squared,
                        entry
                            .point
                            .square_distance(&self.value_uv(sequence_value(uv, index)?)?),
                    ));
                }
                Ok(squared)
            })?;
        if entry.is_some() {
            self.fields.write().gap = minimum.sqrt();
        }
        Ok(entry)
    }

    fn sequence_regular_point(
        &self,
        count: i32,
        points: &[Point3],
        uv: &[Point2],
        tolerance: f64,
        direct: bool,
        entry: SurfaceSingularity,
    ) -> Result<Option<i32>, Failure> {
        for rank in 1..count {
            let index = if direct { rank + 1 } else { count - rank };
            let parameter = sequence_value(uv, index)?;
            let point = sequence_value(points, index)?;
            if entry.point.square_distance(&point) > tolerance * tolerance
                && entry.point.square_distance(&self.value_uv(parameter)?) > tolerance * tolerance
            {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
}

fn sequence_value<T: Copy>(values: &[T], index: i32) -> Result<T, Failure> {
    let rank = index
        .checked_sub(1)
        .and_then(|rank| usize::try_from(rank).ok());
    rank.and_then(|rank| values.get(rank))
        .copied()
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "ProjectDegenerated: sequence index {index}; expected 1..={}",
                    values.len()
                ),
            )
        })
}

fn apply_sequence_degeneracy(
    count: i32,
    uv: &mut [Point2],
    direct: bool,
    u_iso: bool,
    boundary: Option<i32>,
) -> Result<(), Failure> {
    let Some(boundary) = boundary else {
        return distribute_sequence(count, uv, u_iso);
    };
    let neighbour = sequence_value(uv, boundary)?;
    let range = if direct {
        1..boundary
    } else {
        boundary + 1..count + 1
    };
    for index in range {
        sequence_value(uv, index)?;
        copy_degenerate_coordinate(&mut uv[(index - 1) as usize], neighbour, u_iso);
    }
    Ok(())
}

fn distribute_sequence(count: i32, uv: &mut [Point2], u_iso: bool) -> Result<(), Failure> {
    let first = degenerate_coordinate(sequence_value(uv, 1)?, u_iso);
    let last = degenerate_coordinate(sequence_value(uv, count)?, u_iso);
    for index in 1..=count {
        let value =
            (first * f64::from(count - index) + last * f64::from(index - 1)) / f64::from(count - 1);
        let parameter = &mut uv[(index - 1) as usize];
        if u_iso {
            parameter.set_y(value);
        } else {
            parameter.set_x(value);
        }
    }
    Ok(())
}
