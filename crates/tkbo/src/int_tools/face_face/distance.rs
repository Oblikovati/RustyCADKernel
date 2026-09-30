// FindMaxDistance and MaxDistance from IntTools_FaceFace.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Native eleven-interval golden-section search using the shared face projector.
use super::Context;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
const GOLDEN: f64 = 0.61803398874989484820458683436564;
pub(super) fn face_distance(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
    face: &Face,
    context: &Context,
) -> Result<f64, Failure> {
    let step = (last - first) / 11.;
    let epsilon = 1e-4 * step;
    let mut maximum = 0.;
    let cache = context.proj_ps(face)?;
    let mut projector = cache.write();
    let mut end = first;
    loop {
        let start = end;
        end += step;
        if end > last {
            break;
        }
        let distance = interval_distance(curve, start, end, &mut projector, epsilon)?;
        if distance > maximum {
            maximum = distance;
        }
    }
    Ok(maximum)
}
struct GoldenInterval {
    lower: f64,
    upper: f64,
    points: [f64; 2],
    distances: [f64; 2],
}
impl GoldenInterval {
    fn new(
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        projector: &mut ProjectPointOnSurf,
    ) -> Result<Self, Failure> {
        let points = [
            last - GOLDEN * (last - first),
            first + GOLDEN * (last - first),
        ];
        let distances = [
            point_distance(curve, points[0], projector)?,
            point_distance(curve, points[1], projector)?,
        ];
        Ok(Self {
            lower: first,
            upper: last,
            points,
            distances,
        })
    }
    fn advance(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        projector: &mut ProjectPointOnSurf,
    ) -> Result<(), Failure> {
        if self.distances[0] > self.distances[1] {
            self.upper = self.points[1];
            self.points[1] = self.points[0];
            self.distances[1] = self.distances[0];
            self.points[0] = self.upper - GOLDEN * (self.upper - self.lower);
            self.distances[0] = point_distance(curve, self.points[0], projector)?;
        } else {
            self.lower = self.points[0];
            self.points[0] = self.points[1];
            self.distances[0] = self.distances[1];
            self.points[1] = self.lower + GOLDEN * (self.upper - self.lower);
            self.distances[1] = point_distance(curve, self.points[1], projector)?;
        }
        Ok(())
    }
}
fn interval_distance(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
    projector: &mut ProjectPointOnSurf,
    epsilon: f64,
) -> Result<f64, Failure> {
    let mut interval = GoldenInterval::new(curve, first, last, projector)?;
    while (interval.points[0] - interval.points[1]).abs() > epsilon {
        interval.advance(curve, projector)?;
    }
    let parameter = 0.5 * (interval.lower + interval.upper);
    let mut maximum = point_distance(curve, parameter, projector)?;
    for distance in interval.distances {
        if distance > maximum {
            maximum = distance;
        }
    }
    Ok(maximum)
}
fn point_distance(
    curve: &Handle<dyn GeomCurve>,
    parameter: f64,
    projector: &mut ProjectPointOnSurf,
) -> Result<f64, Failure> {
    projector.perform(curve.eval_d0(parameter)?)?;
    if projector.nb_points()? > 0 {
        projector.lower_distance()
    } else {
        Ok(0.)
    }
}
