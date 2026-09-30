use super::sampling::CurveSamples;
use super::seam_insertion::SeamInsertion;
use super::walking::ProjectionWalk;
use super::*;
use crate::shape_analysis::adjust_to_period;

impl ShapeConstructProjectCurveOnSurface {
    pub(super) fn resolve_seams(
        &self,
        curve: &Handle<dyn GeomCurve>,
        samples: &mut CurveSamples,
        walk: &ProjectionWalk,
    ) -> Result<(), Failure> {
        let analysis = self.analysis();
        let surface = analysis.surface();
        let tolerance = self.fields.read().precision;
        let (uf, ul, vf, vl) = surface.bounds();
        if analysis.is_u_closed(tolerance)? || walk.resolve[0] {
            self.resolve_coordinate(curve, samples, walk, 1, (uf, ul), surface.is_u_periodic())?;
        }
        let sphere = surface.is_kind_name("Geom_SphericalSurface");
        if analysis.is_v_closed(tolerance)? || walk.resolve[1] || sphere {
            self.resolve_coordinate(curve, samples, walk, 2, (vf, vl), surface.is_v_periodic())?;
        }
        if analysis.is_v_closed(tolerance)? || sphere {
            snap_v_seam(samples, (vf, vl))?;
        }
        Ok(())
    }

    fn resolve_coordinate(
        &self,
        curve: &Handle<dyn GeomCurve>,
        samples: &mut CurveSamples,
        walk: &ProjectionWalk,
        coordinate: i32,
        range: (f64, f64),
        periodic: bool,
    ) -> Result<(), Failure> {
        let period = range.1 - range.0;
        let first = normalize_first(samples, walk, coordinate, range, periodic)?;
        let seam = SeamInsertion {
            coordinate,
            period,
            tolerance: precision::confusion() * period,
            adjusting: false,
        };
        let (minimum, maximum) = self.walk_seam(curve, samples, seam, first)?;
        if !walk.from_cache {
            center_coordinates(samples, coordinate, range, (minimum + maximum) / 2.)?;
        }
        Ok(())
    }
    fn walk_seam(
        &self,
        curve: &Handle<dyn GeomCurve>,
        samples: &mut CurveSamples,
        mut seam: SeamInsertion,
        first: f64,
    ) -> Result<(f64, f64), Failure> {
        let (mut previous, mut minimum, mut maximum) = (first, first, first);
        let coordinate = seam.coordinate;
        let period = seam.period;
        let mut index = 2;
        while index <= samples.uv.length() {
            let mut current = samples.uv[index].coord(coordinate)?;
            if (current - previous).abs() > period / 2. {
                seam.apply(self, curve, samples, &mut index, &mut current, previous)?;
            }
            previous = current;
            if minimum > current {
                minimum = current;
            } else if maximum < current {
                maximum = current;
            }
            index += 1;
        }
        Ok((minimum, maximum))
    }
}

fn normalize_first(
    samples: &mut CurveSamples,
    walk: &ProjectionWalk,
    coordinate: i32,
    mut range: (f64, f64),
    periodic: bool,
) -> Result<f64, Failure> {
    let period = range.1 - range.0;
    let mut first = samples.uv[1].coord(coordinate)?;
    if !walk.from_cache {
        while first < range.0 {
            first += period;
            samples.uv[1].set_coord(coordinate, first)?;
        }
        while first > range.1 {
            first -= period;
            samples.uv[1].set_coord(coordinate, first)?;
        }
    }
    if periodic && walk.from_cache {
        range = shift_period_window(range, walk.saved.coord(coordinate)?);
        first += adjust_to_period(first, range.0, range.1);
        samples.uv[1].set_coord(coordinate, first)?;
    }
    Ok(first)
}

fn center_coordinates(
    samples: &mut CurveSamples,
    coordinate: i32,
    range: (f64, f64),
    middle: f64,
) -> Result<(), Failure> {
    let period = range.1 - range.0;
    let shift = if middle > range.1 {
        -period
    } else if middle < range.0 {
        period
    } else {
        0.
    };
    if shift == 0. {
        return Ok(());
    }
    for index in 1..=samples.uv.length() {
        let value = samples.uv[index].coord(coordinate)?;
        samples.uv[index].set_coord(coordinate, value + shift)?;
    }
    Ok(())
}

fn snap_v_seam(samples: &mut CurveSamples, range: (f64, f64)) -> Result<(), Failure> {
    for index in 2..=samples.uv.length() {
        let previous = samples.uv[index - 1].y();
        let current = samples.uv[index].y();
        if (current - previous).abs() <= (range.1 - range.0) / 2. {
            continue;
        }
        let distances = [
            (previous - range.0).abs(),
            (previous - range.1).abs(),
            (current - range.0).abs(),
            (current - range.1).abs(),
        ];
        snap_nearest_seam(samples, index, range, nearest_seam(distances))?;
    }
    Ok(())
}

fn shift_period_window(mut range: (f64, f64), saved: f64) -> (f64, f64) {
    let period = range.1 - range.0;
    while range.0 > saved {
        range.0 -= period;
        range.1 -= period;
    }
    while range.1 < saved {
        range.0 += period;
        range.1 += period;
    }
    range
}

fn nearest_seam(distances: [f64; 4]) -> usize {
    let mut selected = 0;
    for candidate in 1..4 {
        if distances[candidate] < distances[selected] {
            selected = candidate;
        }
    }
    selected
}

fn snap_nearest_seam(
    samples: &mut CurveSamples,
    index: i32,
    range: (f64, f64),
    selected: usize,
) -> Result<(), Failure> {
    let point = if selected < 2 { index - 1 } else { index };
    samples.uv[point].set_coord(
        2,
        if selected.is_multiple_of(2) {
            range.0
        } else {
            range.1
        },
    )
}
