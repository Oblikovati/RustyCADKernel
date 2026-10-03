//! Native `ShapeFix_Wire::FixSelfIntersection` mode and pass ordering.
use super::ShapeFixWire;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;

impl ShapeFixWire {
    /// Runs enabled per-edge, adjacent-edge, and nonadjacent intersection repair passes.
    /// Example: `let changed = fixer.fix_self_intersection()?`.
    pub fn fix_self_intersection(&self) -> Result<bool, Failure> {
        self.set_intersection_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok(false);
        }
        self.fix_single_edge_intersections()?;
        self.fix_adjacent_intersections()?;
        self.fix_nonadjacent_intersections()?;
        Ok(self.status_self_intersection(ShapeExtendStatus::Done))
    }

    /// Selects native `FixSelfIntersectingEdge` default-call behavior.
    pub fn set_fix_self_intersecting_edge_mode(&self, mode: i32) {
        self.state.write().fix_self_intersecting_edge_mode = mode;
    }

    /// Selects native `FixIntersectingEdges` default-call behavior.
    pub fn set_fix_intersecting_edges_mode(&self, mode: i32) {
        self.state.write().fix_intersecting_edges_mode = mode;
    }

    /// Selects native nonadjacent intersection repair default-call behavior.
    pub fn set_fix_nonadjacent_intersecting_edges_mode(&self, mode: i32) {
        self.state.write().fix_nonadjacent_intersecting_edges_mode = mode;
    }

    /// Reads native per-edge self-intersection dispatch; `fixer.fix_self_intersecting_edge_mode()`.
    pub fn fix_self_intersecting_edge_mode(&self) -> i32 {
        self.state.read().fix_self_intersecting_edge_mode
    }

    /// Reads native adjacent-edge intersection dispatch; `fixer.fix_intersecting_edges_mode()`.
    pub fn fix_intersecting_edges_mode(&self) -> i32 {
        self.state.read().fix_intersecting_edges_mode
    }

    /// Reads native nonadjacent intersection dispatch; `fixer.fix_nonadjacent_intersecting_edges_mode()`.
    pub fn fix_nonadjacent_intersecting_edges_mode(&self) -> i32 {
        self.state.read().fix_nonadjacent_intersecting_edges_mode
    }

    pub(super) fn perform_self_intersection_stage(
        &self,
        reorder_ok: bool,
    ) -> Result<bool, Failure> {
        let saved_mode = self.state.read().fix_intersecting_edges_mode;
        let suppress_default = saved_mode == -1 && !reorder_ok;
        if suppress_default {
            self.set_fix_intersecting_edges_mode(0);
        }
        let changed = self.fix_self_intersection();
        if suppress_default {
            self.set_fix_intersecting_edges_mode(saved_mode);
        }
        let changed = changed?;
        let _ = self.fix_reorder()?;
        Ok(changed)
    }

    fn fix_single_edge_intersections(&self) -> Result<(), Failure> {
        if !is_enabled(self.state.read().fix_self_intersecting_edge_mode, true) {
            return Ok(());
        }
        let mode = self.remove_loop_mode();
        if mode > 1 {
            return Ok(());
        }
        let mut count = self.wire_data()?.nb_edges();
        let mut index = 1;
        while index <= count {
            let _ = self.fix_self_intersecting_edge(index)?;
            self.merge_last_intersection_status();
            if mode == 1 && count > self.wire_data()?.nb_edges() {
                index -= 1;
            }
            count = self.wire_data()?.nb_edges();
            index += 1;
        }
        if mode == 1 {
            let _ = self.fix_closed(self.precision())?;
        }
        Ok(())
    }

    fn fix_adjacent_intersections(&self) -> Result<(), Failure> {
        if !is_enabled(self.state.read().fix_intersecting_edges_mode, true) {
            return Ok(());
        }
        let mut walk = AdjacentIntersectionWalk::new(self);
        while walk.has_edge_pair(self.wire_data()?.nb_edges()) {
            if self.repair_adjacent_pair(&mut walk)? {
                continue;
            }
            walk.advance();
        }
        if !self.context().is_null() {
            self.update_wire()?;
        }
        Ok(())
    }

    fn repair_adjacent_pair(&self, walk: &mut AdjacentIntersectionWalk) -> Result<bool, Failure> {
        let _ = self.fix_intersecting_edges_at(walk.index)?;
        walk.collect_failure_flags(self);
        if !self.last_fix_status(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        walk.collect_done_flags(self);
        let count = self.wire_data()?.nb_edges();
        if count < 3 {
            if self.last_fix_status(ShapeExtendStatus::Done7) {
                let _ = self.fix_intersecting_edges_at(walk.index)?;
            }
            return Ok(false);
        }
        self.remove_crossing_edges(walk, count)
    }

    fn remove_crossing_edges(
        &self,
        walk: &mut AdjacentIntersectionWalk,
        count: i32,
    ) -> Result<bool, Failure> {
        let mut removed = false;
        if self.last_fix_status(ShapeExtendStatus::Done4) {
            self.wire_data()?.remove(walk.index)?;
            removed = true;
        }
        if self.last_fix_status(ShapeExtendStatus::Done3) {
            let previous = if walk.index > 1 {
                walk.index - 1
            } else {
                count + walk.index - 1
            };
            self.wire_data()?.remove(previous)?;
            removed = true;
        }
        if removed {
            self.or_intersection_status(ShapeExtendStatus::Done3);
            walk.reset(self.closed_mode());
            return Ok(true);
        }
        let _ = self.fix_intersecting_edges_at(walk.index)?;
        Ok(false)
    }

    fn fix_nonadjacent_intersections(&self) -> Result<(), Failure> {
        if !is_enabled(
            self.state.read().fix_nonadjacent_intersecting_edges_mode,
            true,
        ) {
            return Ok(());
        }
        let data = self.wire_data()?;
        let face = self.face();
        let context = self.context();
        let tool = super::ShapeFixIntersectionTool::new(&context, self.precision());
        let (mut split, mut cut, mut removed) = (0, 0, 0);
        if tool.fix_self_intersect_wire(&data, &face, &mut split, &mut cut, &mut removed)? {
            self.or_intersection_status(ShapeExtendStatus::Done5);
        }
        self.finish_nonadjacent_intersections(data, split, removed)
    }

    fn finish_nonadjacent_intersections(
        &self,
        data: rustycad_tkernel::standard::Handle<crate::shape_extend::ShapeExtendWireData>,
        split: i32,
        removed: i32,
    ) -> Result<(), Failure> {
        if split == 0 && removed == 0 {
            return Ok(());
        }
        if removed > 0 {
            self.state.write().removed_segment = true;
        }
        self.working_analyzer().load_wire_data(&data);
        if !self.context().is_null() {
            self.update_wire()?;
        }
        self.fix_root_data()
            .with_mut(|root| root.shape = Shape::default());
        Ok(())
    }

    fn merge_last_intersection_status(&self) {
        let mut state = self.state.write();
        state.intersections_status |= state.last_fix_status;
    }

    fn set_intersection_status(&self, status: ShapeExtendStatus) {
        self.state.write().intersections_status = ShapeExtend::encode_status(status);
    }

    fn or_intersection_status(&self, status: ShapeExtendStatus) {
        self.state.write().intersections_status |= ShapeExtend::encode_status(status);
    }
}

struct AdjacentIntersectionWalk {
    index: i32,
    start: i32,
}

impl AdjacentIntersectionWalk {
    fn new(fixer: &ShapeFixWire) -> Self {
        let start = if fixer.closed_mode() { 1 } else { 2 };
        Self {
            index: start,
            start,
        }
    }

    fn has_edge_pair(&self, count: i32) -> bool {
        count > 1 && self.index <= count
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn reset(&mut self, closed: bool) {
        self.start = if closed { 1 } else { 2 };
        self.index = self.start;
    }

    fn collect_failure_flags(&self, fixer: &ShapeFixWire) {
        for status in [ShapeExtendStatus::Fail1, ShapeExtendStatus::Fail2] {
            if fixer.last_fix_status(status) {
                fixer.or_intersection_status(status);
            }
        }
    }

    fn collect_done_flags(&self, fixer: &ShapeFixWire) {
        for status in [
            ShapeExtendStatus::Done1,
            ShapeExtendStatus::Done2,
            ShapeExtendStatus::Done6,
        ] {
            if fixer.last_fix_status(status) {
                fixer.or_intersection_status(status);
            }
        }
    }
}

fn is_enabled(mode: i32, default: bool) -> bool {
    mode < 0 && default || mode > 0
}
