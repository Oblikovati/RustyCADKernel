//! Vertex identity grouping retains native UV aliases and local destination numbering.
use super::*;
impl TangencyWalk<'_, '_> {
    fn matching_vertex(
        &self,
        index: i32,
        vertex: &TheVertex,
    ) -> Result<Option<ContapThePathPointOfTheSearch>, Failure> {
        if at(self.destination, index)? != 0 {
            return Ok(None);
        }
        let start = self.search.point(index)?;
        if start.is_new() || !self.domain.identical(vertex, &start.vertex()?)? {
            return Ok(None);
        }
        Ok(Some(start.clone()))
    }
    pub(super) fn tangent_point(
        &mut self,
        index: i32,
        start: &ContapThePathPointOfTheSearch,
        mut passing: bool,
    ) -> Result<(), Failure> {
        self.point.set_tangency(true);
        put(self.destination, index, self.appended + 1)?;
        if !start.is_new() {
            let vertex = start.vertex()?;
            for next in index + 1..=self.count {
                if let Some(other) = self.matching_vertex(next, &vertex)? {
                    let orientation = self.domain.arc_orientation(&other.arc())?;
                    passing = passing && unoriented(orientation);
                    let uv = ContapHCurve2dTool::value(&other.arc(), other.parameter())?;
                    self.point.add_uv(uv.x(), uv.y())?;
                    put(self.destination, next, self.appended + 1)?;
                }
            }
        }
        self.append(passing);
        Ok(())
    }
    pub(super) fn vertex_point(
        &mut self,
        index: i32,
        start: &ContapThePathPointOfTheSearch,
        orientation: Orientation,
        boundary: Vector3,
        test: f64,
        group: &mut Crossing,
    ) -> Result<(), Failure> {
        group.reset()?;
        if !unoriented(orientation) {
            group.compare(
                self.domain.vertex_orientation(&start.vertex()?)?,
                orientation,
                boundary,
                test,
            )?;
        }
        put(self.destination, index, self.appended + 1)?;
        self.group_vertex(index, start, group)?;
        self.finish_group(index, group)
    }
    fn finish_group(&mut self, index: i32, group: &mut Crossing) -> Result<(), Failure> {
        let mut keep = group.accept();
        if keep && group.verify {
            keep = self.verify_group(index, group)?;
        }
        if keep {
            self.point
                .set_directions(group.direction, group.direction2d);
            self.append(group.passing);
        } else {
            self.reject_group(index)?;
        }
        Ok(())
    }
    fn group_vertex(
        &mut self,
        index: i32,
        start: &ContapThePathPointOfTheSearch,
        group: &mut Crossing,
    ) -> Result<(), Failure> {
        for next in index + 1..=self.count {
            if let Some(other) = self.matching_vertex(next, &start.vertex()?)? {
                let orientation = self.domain.arc_orientation(&other.arc())?;
                let (uv, tangent) = ContapHCurve2dTool::d1(&other.arc(), other.parameter())?;
                self.point.add_uv(uv.x(), uv.y())?;
                if !unoriented(orientation) {
                    group.passing = false;
                    let boundary = restriction_tangent(tangent, group.du, group.dv);
                    let test = group.direction.dot(group.normal.crossed(boundary));
                    group.compare(
                        self.domain.vertex_orientation(&other.vertex()?)?,
                        orientation,
                        boundary,
                        test,
                    )?;
                }
                put(self.destination, next, self.appended + 1)?;
            }
        }
        Ok(())
    }
    fn verify_group(&self, index: i32, group: &Crossing) -> Result<bool, Failure> {
        for next in index..=self.count {
            if at(self.destination, next)? != self.appended + 1 {
                continue;
            }
            let start = self.search.point(next)?;
            let orientation = self.domain.arc_orientation(&start.arc())?;
            if unoriented(orientation) {
                continue;
            }
            let boundary = self.outgoing_boundary(start, orientation, group)?;
            if group.direction.normalized()?.dot(boundary.normalized()?) >= 1. - ANGULAR_TOLERANCE {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn outgoing_boundary(
        &self,
        start: &ContapThePathPointOfTheSearch,
        orientation: Orientation,
        group: &Crossing,
    ) -> Result<Vector3, Failure> {
        let (_, tangent) = ContapHCurve2dTool::d1(&start.arc(), start.parameter())?;
        let mut boundary = restriction_tangent(tangent, group.du, group.dv);
        let vertex = self.domain.vertex_orientation(&start.vertex()?)?;
        if (orientation == Orientation::Forward && vertex == Orientation::Reversed)
            || (orientation == Orientation::Reversed && vertex == Orientation::Forward)
        {
            boundary.reverse();
        }
        Ok(boundary)
    }
    fn reject_group(&mut self, index: i32) -> Result<(), Failure> {
        for next in index..=self.count {
            if at(self.destination, next)? == self.appended + 1 {
                put(self.destination, next, -(self.appended + 1))?;
            }
        }
        Ok(())
    }
}
