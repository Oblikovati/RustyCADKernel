use super::sampling::CurveSamples;
use super::*;

impl ShapeConstructProjectCurveOnSurface {
    pub(super) fn adjust_degenerated_walk(
        &self,
        samples: &mut CurveSamples,
    ) -> Result<(), Failure> {
        if self.fields.read().adjust_over_degen == -1 {
            return Ok(());
        }
        let analysis = self.analysis();
        let tolerance = self.fields.read().precision;
        let Some(coordinate) = degenerated_axis(&analysis, tolerance)? else {
            return Ok(());
        };
        let bounds = analysis.surface().bounds();
        let range = if coordinate == 1 {
            (bounds.0, bounds.1)
        } else {
            (bounds.2, bounds.3)
        };
        self.adjust_degenerated_axis(samples, coordinate, range)
    }
    fn adjust_degenerated_axis(
        &self,
        samples: &mut CurveSamples,
        coordinate: i32,
        range: (f64, f64),
    ) -> Result<(), Failure> {
        let Some((index, previous_bound, current_bound)) =
            find_degenerate_jump(&self.analysis(), samples, coordinate, range)?
        else {
            return Ok(());
        };
        let reference = if self.fields.read().adjust_over_degen != 0 {
            range.0
        } else {
            range.1
        };
        if previous_bound {
            adjust_before(samples, index, coordinate, range, reference)?;
        } else if current_bound {
            adjust_after(samples, index, coordinate, range, reference)?;
        }
        self.add_status(ShapeExtendStatus::Done4);
        Ok(())
    }
}
fn find_degenerate_jump(
    analysis: &ShapeAnalysisSurface,
    samples: &CurveSamples,
    coordinate: i32,
    range: (f64, f64),
) -> Result<Option<(i32, bool, bool)>, Failure> {
    let period = range.1 - range.0;
    let middle = (range.0 + range.1) / 2.;
    let (mut previous, mut previous_bound, mut start) = (0., false, true);
    for index in 1..=samples.uv.length() {
        if analysis.is_degenerated(samples.points[index], precision::confusion())? {
            continue;
        }
        let current = samples.uv[index].coord(coordinate)?;
        let on_bound = ((current - middle).abs() - period / 2.).abs() <= precision::p_confusion();
        if !start && ((current - previous).abs() - period / 2.).abs() <= 0.01 * period {
            return Ok(Some((index, previous_bound, on_bound)));
        }
        start = false;
        previous = current;
        previous_bound = on_bound;
    }
    Ok(None)
}
fn adjust_before(
    samples: &mut CurveSamples,
    index: i32,
    coordinate: i32,
    range: (f64, f64),
    reference: f64,
) -> Result<(), Failure> {
    samples.uv[index - 1].set_coord(coordinate, reference)?;
    for position in (1..=index - 2).rev() {
        wrap_degenerate(
            &mut samples.uv[position],
            coordinate,
            range.1 - range.0,
            reference,
        )?;
    }
    Ok(())
}
fn adjust_after(
    samples: &mut CurveSamples,
    index: i32,
    coordinate: i32,
    range: (f64, f64),
    reference: f64,
) -> Result<(), Failure> {
    samples.uv[index].set_coord(coordinate, reference)?;
    for position in index + 1..=samples.uv.length() {
        wrap_degenerate(
            &mut samples.uv[position],
            coordinate,
            range.1 - range.0,
            reference,
        )?;
    }
    Ok(())
}
fn wrap_degenerate(
    point: &mut Point2,
    coordinate: i32,
    period: f64,
    reference: f64,
) -> Result<(), Failure> {
    let distance = period / 2. + precision::p_confusion();
    let mut current = point.coord(coordinate)?;
    while current < reference - distance {
        current += period;
        point.set_coord(coordinate, current)?;
    }
    while current > reference + distance {
        current -= period;
        point.set_coord(coordinate, current)?;
    }
    Ok(())
}

fn degenerated_axis(
    analysis: &ShapeAnalysisSurface,
    tolerance: f64,
) -> Result<Option<i32>, Failure> {
    let coordinate = if analysis.is_u_closed(tolerance)? {
        1
    } else if analysis.is_v_closed(tolerance)? {
        2
    } else {
        return Ok(None);
    };
    analysis.is_degenerated(Point3::default(), tolerance)?;
    Ok((analysis.nb_singularities(tolerance)? > 0).then_some(coordinate))
}
