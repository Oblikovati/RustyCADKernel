use super::iso_sampling::IsoSampling;
use super::line_fit::LineFitResult;
use super::sampling::{CONTROL_POINTS, CurveSamples};
use super::*;

pub(super) struct ProjectionWalk {
    pub from_cache: bool,
    pub saved: Point2,
    pub reverse: bool,
    pub resolve: [bool; 2],
    point: Point2,
    gap: f64,
    previous_uv: Point2,
    previous_point: Point3,
}
impl ShapeConstructProjectCurveOnSurface {
    pub(super) fn walk_samples(
        &self,
        samples: &mut CurveSamples,
        line: &LineFitResult,
        iso: &IsoSampling,
    ) -> Result<ProjectionWalk, Failure> {
        let mut walk =
            ProjectionWalk::new(self.reverse_samples(samples), self.fields.read().precision);
        let count = samples.points.length();
        for step in 1..=count {
            let index = if walk.reverse { count - step + 1 } else { step };
            if !self.walk_point(samples, index, line, iso, &mut walk)? {
                continue;
            }
            samples.uv[index] = walk.point;
            self.detect_closed_jump(samples, step, index, &mut walk)?;
            walk.previous_point = samples.points[index];
            walk.previous_uv = walk.point;
            if step > 1 {
                walk.extrapolate(samples, index);
            }
        }
        Ok(walk)
    }

    fn walk_point(
        &self,
        samples: &CurveSamples,
        index: i32,
        line: &LineFitResult,
        iso: &IsoSampling,
        walk: &mut ProjectionWalk,
    ) -> Result<bool, Failure> {
        if iso.recognition.recognized && (iso.recognition.same_parameters || !iso.analytic) {
            walk.point = iso.parameterized_point(samples, index);
            return Ok(true);
        }
        let endpoint = sample_endpoint(index, samples.points.length());
        let Some(endpoint) = endpoint else {
            self.walk_next(samples.points[index], walk)?;
            return Ok(true);
        };
        if iso.recognition.recognized || iso.recognition.on_iso[endpoint] {
            walk.point = iso.recognition.endpoints[endpoint];
            return Ok(true);
        }
        if !line.recompute {
            self.reuse_line_endpoint(samples, index, line, walk);
            return Ok(false);
        }
        self.walk_endpoint(samples.points[index], index, walk)?;
        Ok(true)
    }

    fn reuse_line_endpoint(
        &self,
        samples: &CurveSamples,
        index: i32,
        line: &LineFitResult,
        walk: &mut ProjectionWalk,
    ) {
        walk.point = samples.uv[index];
        walk.gap = self.analysis().gap();
        if index == 1 {
            walk.from_cache = line.from_cache;
            walk.saved = walk.point;
        }
    }

    fn walk_next(&self, point: Point3, walk: &mut ProjectionWalk) -> Result<(), Failure> {
        let analysis = self.analysis();
        walk.point = analysis.next_value_of_uv(
            walk.point,
            point,
            self.fields.read().precision,
            precision::confusion() + 1000. * walk.gap,
        )?;
        walk.gap = analysis.gap();
        Ok(())
    }

    fn walk_endpoint(
        &self,
        point: Point3,
        index: i32,
        walk: &mut ProjectionWalk,
    ) -> Result<(), Failure> {
        let analysis = self.analysis();
        let tolerance = self.fields.read().precision;
        let cached = super::point_projection::cached_uv(
            &self.fields.read().cache,
            point,
            tolerance * tolerance,
        );
        if let Some(uv) = cached {
            walk.point = analysis.next_value_of_uv(
                uv,
                point,
                tolerance,
                precision::confusion() + walk.gap,
            )?;
            walk.record_cache(index, uv);
        } else {
            walk.point = analysis.value_of_uv(point, tolerance)?;
        }
        walk.gap = analysis.gap();
        Ok(())
    }

    fn detect_closed_jump(
        &self,
        samples: &CurveSamples,
        step: i32,
        index: i32,
        walk: &mut ProjectionWalk,
    ) -> Result<(), Failure> {
        let count = samples.points.length();
        if count <= CONTROL_POINTS || step <= 2 || step >= count {
            return Ok(());
        }
        let analysis = self.analysis();
        let surface = analysis.surface();
        let tolerance = self.fields.read().precision;
        if walk.previous_point.distance(&samples.points[index]) >= tolerance
            || !surface.is_kind_name("Geom_BSplineSurface")
        {
            return Ok(());
        }
        let (uf, ul, vf, vl) = surface.bounds();
        let jumps = [
            (walk.point.x() - walk.previous_uv.x()).abs() > 0.95 * (ul - uf),
            (walk.point.y() - walk.previous_uv.y()).abs() > 0.95 * (vl - vf),
        ];
        resolve_singular_jumps(&analysis, tolerance, jumps, &mut walk.resolve)
    }

    pub(super) fn reverse_samples(&self, samples: &CurveSamples) -> bool {
        let fields = self.fields.read();
        let Some((point, _)) = fields.cache.first() else {
            return false;
        };
        let last = samples.points[samples.points.length()];
        point.distance(&samples.points[1]) > point.distance(&last)
            && point.distance(&last) < precision::confusion()
    }

    pub(super) fn cache_samples(&self, samples: &CurveSamples, reverse: bool) {
        let count = samples.points.length();
        let mut cache = vec![
            (samples.points[count], samples.uv[count]),
            (samples.points[1], samples.uv[1]),
        ];
        if reverse {
            cache.reverse();
        }
        self.fields.write().cache = cache;
    }
}
impl ProjectionWalk {
    fn new(reverse: bool, gap: f64) -> Self {
        Self {
            from_cache: false,
            saved: Point2::default(),
            reverse,
            resolve: [false; 2],
            point: Point2::default(),
            gap,
            previous_uv: Point2::default(),
            previous_point: Point3::default(),
        }
    }
    fn record_cache(&mut self, index: i32, uv: Point2) {
        if index == 1 {
            self.from_cache = true;
            self.saved = uv;
        }
    }

    fn extrapolate(&mut self, samples: &CurveSamples, index: i32) {
        let previous = samples.uv[if self.reverse { index + 1 } else { index - 1 }];
        self.point = Point2::new(
            2. * self.point.x() - previous.x(),
            2. * self.point.y() - previous.y(),
        );
    }
}

fn sample_endpoint(index: i32, count: i32) -> Option<usize> {
    if index == 1 {
        return Some(0);
    }
    (index == count).then_some(1)
}
fn resolve_singular_jumps(
    analysis: &ShapeAnalysisSurface,
    tolerance: f64,
    jumps: [bool; 2],
    resolve: &mut [bool; 2],
) -> Result<(), Failure> {
    for (coordinate, jump) in jumps.into_iter().enumerate() {
        if !jump {
            continue;
        }
        let closed = if coordinate == 0 {
            analysis.is_u_closed(tolerance)?
        } else {
            analysis.is_v_closed(tolerance)?
        };
        if !closed && analysis.nb_singularities(tolerance)? > 0 {
            resolve[coordinate] = true;
        }
    }
    Ok(())
}
