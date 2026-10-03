use super::*;

pub(super) struct TailBounds {
    whole: [bool; 2],
    parameters: [f64; 2],
    projections: [CurveProjection; 2],
    points: [Point3; 2],
    distances: [f64; 2],
}

impl TailBounds {
    pub(super) fn new(curves: &[TailCurve; 2], limits: &TailLimits) -> Result<Self, Failure> {
        let mut bounds = Self {
            whole: [true; 2],
            parameters: [0.; 2],
            projections: [CurveProjection::default(); 2],
            points: [Point3::default(); 2],
            distances: [0.; 2],
        };
        for index in 0..2 {
            bounds.find_boundary(curves, limits, index)?;
        }
        Ok(bounds)
    }

    fn find_boundary(
        &mut self,
        curves: &[TailCurve; 2],
        limits: &TailLimits,
        index: usize,
    ) -> Result<(), Failure> {
        let curve = &curves[index];
        self.parameters[index] = curve.range[1 - curve.junction];
        self.points[index] = curve.geometry.value(self.parameters[index])?;
        self.distances[index] =
            curves[1 - index].project(self.points[index], &mut self.projections[index])?;
        if self.distances[index] <= limits.near {
            return Ok(());
        }
        self.whole[index] = false;
        self.bisect_boundary(curves, limits, index)
    }

    fn bisect_boundary(
        &mut self,
        curves: &[TailCurve; 2],
        limits: &TailLimits,
        index: usize,
    ) -> Result<(), Failure> {
        let curve = &curves[index];
        let mut near_parameter = curve.range[curve.junction];
        loop {
            let parameter = (near_parameter + self.parameters[index]) * 0.5;
            self.points[index] = curve.geometry.value(parameter)?;
            let distance =
                curves[1 - index].project(self.points[index], &mut self.projections[index])?;
            if distance <= limits.near {
                near_parameter = parameter;
                continue;
            }
            self.parameters[index] = parameter;
            if distance <= limits.far {
                return Ok(());
            }
        }
    }

    pub(super) fn samples_match(
        &self,
        curves: &[TailCurve; 2],
        limits: &TailLimits,
    ) -> Result<bool, Failure> {
        for index in 0..2 {
            if !self.sample_curve(curves, limits.sample, index)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn sample_curve(
        &self,
        curves: &[TailCurve; 2],
        tolerance: f64,
        index: usize,
    ) -> Result<bool, Failure> {
        let curve = &curves[index];
        let start = curve.range[curve.junction];
        let step = (self.parameters[index] - start) / 23.;
        for rank in 1..23 {
            let parameter = start + f64::from(rank) * step;
            let point = curve.geometry.value(parameter)?;
            let mut projection = CurveProjection {
                parameter,
                point: Point3::default(),
            };
            if curves[1 - index].project(point, &mut projection)? > tolerance {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn both_whole(&self, limits: &TailLimits) -> bool {
        self.whole[0]
            && self.whole[1]
            && self.points[0].square_distance(&self.points[1]) <= limits.far * limits.far
    }

    pub(super) fn cut_parameters(&self) -> [f64; 2] {
        let mut first = 0;
        if self.whole[0] || self.whole[1] {
            first = usize::from(!self.whole[0]);
            if self.distances[1 - first] < self.distances[first] && self.whole[1 - first] {
                first = 1 - first;
            }
        }
        let mut parameters = [0.; 2];
        parameters[first] = self.parameters[first];
        parameters[1 - first] = self.projections[first].parameter;
        parameters
    }
}
