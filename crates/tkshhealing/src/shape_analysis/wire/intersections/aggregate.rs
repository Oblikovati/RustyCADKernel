use super::*;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;

impl ShapeAnalysisWire {
    /// Run OCCT's eight wire checks in their native order; `analyzer.perform()?`.
    pub fn perform(&self) -> Result<bool, Failure> {
        let mut found = self.check_order(true, true)?;
        found |= self.check_small(0.)?;
        found |= self.check_connected(0.)?;
        found |= self.check_edge_curves()?;
        found |= self.check_degenerated()?;
        found |= self.check_self_intersection()?;
        found |= self.check_lacking()?;
        found |= self.check_closed(0.)?;
        Ok(found)
    }

    /// Check self, adjacent and box-overlapping nonadjacent intersections.
    /// Example: `analyzer.check_self_intersection()?`.
    pub fn check_self_intersection(&self) -> Result<bool, Failure> {
        self.fields.write().status_self_intersection = 0;
        if !self.is_ready() {
            return Ok(false);
        }
        let count = self.wire_data().nb_edges();
        self.check_local_wire_intersections(count)?;
        let boxes = self.intersection_boxes(count)?;
        self.check_nonadjacent_intersections(count, &boxes)?;
        Ok(self.status_self_intersection(ShapeExtendStatus::Done))
    }

    /// Query the aggregate intersection status; `analyzer.status_self_intersection(ShapeExtendStatus::Done3)`.
    pub fn status_self_intersection(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_self_intersection, status)
    }

    fn check_local_wire_intersections(&self, count: i32) -> Result<(), Failure> {
        for index in 1..=count {
            self.check_self_intersecting_edge(index)?;
            self.accumulate_intersection_flags(ShapeExtendStatus::Done1, ShapeExtendStatus::Fail1);
            self.check_adjacent_intersections(index)?;
            self.accumulate_intersection_flags(ShapeExtendStatus::Done2, ShapeExtendStatus::Fail2);
        }
        Ok(())
    }

    fn intersection_boxes(&self, count: i32) -> Result<Vec<BoundingBox2>, Failure> {
        let (surface, location) = BRepTool::surface_with_location(&self.face())?;
        let analysis = ShapeAnalysisEdge::new();
        let wire = self.wire_data();
        let mut boxes = Vec::with_capacity(count as usize);
        for index in 1..=count {
            let mut bounds = BoundingBox2::default();
            let (curve, first, last) =
                analysis.pcurve_on_surface(&wire.edge(index)?, &surface, &location, false)?;
            if !curve.is_null() {
                let adapted = Geom2dAdaptorCurve::with_range(&curve, first, last)?;
                Add2dCurve::add_adaptor(&adapted, precision::confusion(), &mut bounds)?;
            }
            boxes.push(bounds);
        }
        Ok(boxes)
    }

    fn check_nonadjacent_intersections(
        &self,
        count: i32,
        boxes: &[BoundingBox2],
    ) -> Result<(), Failure> {
        let (mut done, mut fail) = (false, false);
        for first in 1..count - 1 {
            let last = if self.check_closed(precision::confusion())? && first == 1 {
                count - 1
            } else {
                count
            };
            for second in first + 2..=last {
                if boxes[(first - 1) as usize].is_out_box(&boxes[(second - 1) as usize]) {
                    continue;
                }
                self.check_intersecting_edges(first, second)?;
                // Native checks FAIL1 here, even though the pair overload reports missing pcurves as FAIL3.
                fail |= self.last_check_status(ShapeExtendStatus::Fail1);
                done |= self.last_check_status(ShapeExtendStatus::Done1);
            }
        }
        if fail {
            self.fields.write().status_self_intersection |=
                ShapeExtend::encode_status(ShapeExtendStatus::Fail3);
        }
        if done {
            self.fields.write().status_self_intersection |=
                ShapeExtend::encode_status(ShapeExtendStatus::Done3);
        }
        Ok(())
    }

    fn accumulate_intersection_flags(&self, done: ShapeExtendStatus, fail: ShapeExtendStatus) {
        if self.last_check_status(ShapeExtendStatus::Done) {
            self.fields.write().status_self_intersection |= ShapeExtend::encode_status(done);
        }
        if self.last_check_status(ShapeExtendStatus::Fail) {
            self.fields.write().status_self_intersection |= ShapeExtend::encode_status(fail);
        }
    }
}
