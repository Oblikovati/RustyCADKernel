//! Native early returns deliberately retain prior parameters, length and bounding box.
use super::*;
impl ShrunkRange {
    /// Compute the native working interval, length and splitting eligibility.
    ///
    /// `range.perform()?;`
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.done = false;
        self.splittable = false;
        if self.parameters[1] - self.parameters[0] < precision::p_confusion() {
            return Ok(());
        }
        let points = [
            BRepTool::pnt(&self.vertices[0])?,
            BRepTool::pnt(&self.vertices[1])?,
        ];
        let tolerance = BRepTool::edge_tolerance(&self.edge)?;
        let ends = self.ends(points, tolerance)?;
        let curve = BRepAdaptorCurve::with_edge(&self.edge)?;
        if !self.find_interval(&curve, tolerance, ends)? {
            return Ok(());
        }
        self.finish_interval(&curve, tolerance)
    }
    fn finish_interval(&mut self, curve: &BRepAdaptorCurve, tolerance: f64) -> Result<(), Failure> {
        self.measure_interval(curve, tolerance)?;
        if self.length < precision::confusion() {
            return Ok(());
        }
        self.done = true;
        self.splittable = self.length > 2. * tolerance + 2. * precision::confusion();
        Add3dCurve::add_range(
            curve,
            self.shrunk[0],
            self.shrunk[1],
            tolerance + precision::confusion(),
            &mut self.bounds,
        )
    }
    fn ends(
        &self,
        points: [rustycad_tkmath::gp::Point3; 2],
        edge_tolerance: f64,
    ) -> Result<[RangeEnd; 2], Failure> {
        let mut tolerance = [
            BRepTool::vertex_tolerance(&self.vertices[0])?,
            BRepTool::vertex_tolerance(&self.vertices[1])?,
        ];
        for value in &mut tolerance {
            if *value < edge_tolerance {
                *value = edge_tolerance;
            }
            *value += precision::confusion();
        }
        Ok(std::array::from_fn(|index| RangeEnd {
            parameter: self.parameters[index],
            point: points[index],
            tolerance: tolerance[index],
        }))
    }
    fn find_interval(
        &mut self,
        curve: &BRepAdaptorCurve,
        tolerance: f64,
        ends: [RangeEnd; 2],
    ) -> Result<bool, Failure> {
        let interval = find_valid_range(curve, tolerance, ends[0], ends[1])?;
        if let Some(first) = interval.first {
            self.shrunk[0] = first;
        }
        if let Some(last) = interval.last {
            self.shrunk[1] = last;
        }
        Ok(interval.is_valid
            && (self.shrunk[1] - self.shrunk[0]).partial_cmp(&precision::p_confusion())
                != Some(std::cmp::Ordering::Less))
    }
    fn measure_interval(
        &mut self,
        curve: &BRepAdaptorCurve,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let mut resolution = curve.resolution(tolerance)?;
        let maximum = (self.parameters[1] - self.parameters[0]) / 100.;
        if resolution > maximum {
            resolution = maximum;
        }
        self.length = AbscissaPoint::length_between_with_tolerance(
            curve as &dyn Adaptor3dCurve,
            self.shrunk[0],
            self.shrunk[1],
            resolution,
        )?;
        Ok(())
    }
}
