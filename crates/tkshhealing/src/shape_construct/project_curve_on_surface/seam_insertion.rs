use super::sampling::CurveSamples;
use super::*;
use rustycad_tkmath::elclib;

pub(super) struct SeamInsertion {
    pub coordinate: i32,
    pub period: f64,
    pub tolerance: f64,
    pub adjusting: bool,
}
struct MidpointProjection {
    parameter: f64,
    point: Point3,
    uv: Point2,
}
impl SeamInsertion {
    pub fn apply(
        &mut self,
        projector: &ShapeConstructProjectCurveOnSurface,
        curve: &Handle<dyn GeomCurve>,
        samples: &mut CurveSamples,
        index: &mut i32,
        current: &mut f64,
        previous: f64,
    ) -> Result<(), Failure> {
        let corrected = elclib::in_period(
            *current,
            previous - self.period / 2.,
            previous + self.period / 2.,
        );
        if !self.adjusting {
            self.adjusting = !self.try_insert(
                projector,
                curve,
                samples,
                index,
                (previous, *current),
                corrected,
            )?;
        }
        if self.adjusting {
            *current = corrected;
            samples.uv[*index].set_coord(self.coordinate, *current)?;
        }
        Ok(())
    }

    fn try_insert(
        &self,
        projector: &ShapeConstructProjectCurveOnSurface,
        curve: &Handle<dyn GeomCurve>,
        samples: &mut CurveSamples,
        index: &mut i32,
        ends: (f64, f64),
        corrected: f64,
    ) -> Result<bool, Failure> {
        let (previous, current) = ends;
        let range = (samples.parameters[*index - 1], samples.parameters[*index]);
        let mut middle = MidpointProjection::new(projector, curve, (range.0 + range.1) / 2.)?;
        if self.midpoint_inside(middle.uv, previous, corrected)? {
            return Ok(false);
        }
        if !self.refine(projector, curve, range, &mut middle, (previous, current))? {
            return Ok(false);
        }
        samples.insert(*index, middle.point, middle.parameter, middle.uv);
        *index += 1;
        Ok(true)
    }

    fn midpoint_inside(
        &self,
        middle: Point2,
        previous: f64,
        corrected: f64,
    ) -> Result<bool, Failure> {
        let coordinate = elclib::in_period(
            middle.coord(self.coordinate)?,
            previous - self.period / 2.,
            previous + self.period / 2.,
        );
        let (first, last) = if corrected < previous {
            (corrected, previous)
        } else {
            (previous, corrected)
        };
        Ok(last - first <= self.tolerance || (first <= coordinate && coordinate <= last))
    }

    fn refine(
        &self,
        projector: &ShapeConstructProjectCurveOnSurface,
        curve: &Handle<dyn GeomCurve>,
        mut range: (f64, f64),
        middle: &mut MidpointProjection,
        ends: (f64, f64),
    ) -> Result<bool, Failure> {
        let limit = self.period / 2. - self.tolerance;
        loop {
            let coordinate = middle.uv.coord(self.coordinate)?;
            if !((coordinate - ends.0).abs() >= limit || (ends.1 - coordinate).abs() >= limit) {
                return Ok(true);
            }
            if middle.parameter - range.0 <= precision::p_confusion()
                || range.1 - middle.parameter <= precision::p_confusion()
            {
                return Ok(false);
            }
            if (coordinate - ends.0).abs() >= limit {
                range.1 = (range.0 + range.1) / 2.;
            } else {
                range.0 = (range.0 + range.1) / 2.;
            }
            *middle = MidpointProjection::new(projector, curve, (range.0 + range.1) / 2.)?;
        }
    }
}
impl MidpointProjection {
    fn new(
        projector: &ShapeConstructProjectCurveOnSurface,
        curve: &Handle<dyn GeomCurve>,
        parameter: f64,
    ) -> Result<Self, Failure> {
        let point = curve.value(parameter)?;
        let uv = projector
            .analysis()
            .value_of_uv(point, projector.fields.read().precision)?;
        Ok(Self {
            parameter,
            point,
            uv,
        })
    }
}
impl CurveSamples {
    fn insert(&mut self, index: i32, point: Point3, parameter: f64, uv: Point2) {
        let mut result = Self::with_count(self.points.length() + 1);
        for position in 1..=result.points.length() {
            if position == index {
                result.points[position] = point;
                result.parameters[position] = parameter;
                result.uv[position] = uv;
                continue;
            }
            let source = if position < index {
                position
            } else {
                position - 1
            };
            result.points[position] = self.points[source];
            result.parameters[position] = self.parameters[source];
            result.uv[position] = self.uv[source];
        }
        *self = result;
    }
}
