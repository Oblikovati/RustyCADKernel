use super::ShapeFixWire;
use crate::shape_analysis::ShapeAnalysisWireOrder;
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use crate::shape_fix::FixRoot;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};

struct AggregateWireState {
    fixed: bool,
    reorder_ok: bool,
}

impl ShapeFixWire {
    /// Runs aggregate wire repairs in native order; `fixer.perform()?`.
    pub fn perform(&self) -> Result<bool, Failure> {
        self.perform_with_progress(&ProgressRange::default())
    }

    /// Runs aggregate repairs, returning false when the supplied progress is interrupted.
    /// Example: `fixer.perform_with_progress(&range)?`.
    pub fn perform_with_progress(&self, progress: &ProgressRange<'_>) -> Result<bool, Failure> {
        self.clear_statuses();
        if !self.is_loaded() {
            return Ok(false);
        }
        self.share_context_with_edge_tool();
        self.perform_free_wire_stages(progress)
    }

    fn share_context_with_edge_tool(&self) {
        let context = self.context();
        if !context.is_null() {
            self.edge_fixer.set_context(&context);
        }
    }

    fn perform_free_wire_stages(&self, progress: &ProgressRange<'_>) -> Result<bool, Failure> {
        let mut state = AggregateWireState::check_reorder(self)?;
        if progress.user_break() {
            return Ok(false);
        }
        state.fix_small(self)?;
        if progress.user_break() {
            return Ok(false);
        }
        state.fix_connected(self)?;
        if progress.user_break() {
            return Ok(false);
        }
        state.fix_edge_curves(self)?;
        if progress.user_break() {
            return Ok(false);
        }
        self.finish_free_wire_stages(progress, &mut state)
    }

    fn finish_free_wire_stages(
        &self,
        progress: &ProgressRange<'_>,
        state: &mut AggregateWireState,
    ) -> Result<bool, Failure> {
        state.fix_degenerated(self)?;
        if progress.user_break() {
            return Ok(false);
        }
        self.run_notched_edges(state)?;
        if progress.user_break() {
            return Ok(false);
        }
        if self.state.read().fix_tail_mode != 0 {
            self.run_tails(state)?;
        }
        if progress.user_break() {
            return Ok(false);
        }
        self.fix_self_intersections(state)?;
        if progress.user_break() {
            return Ok(false);
        }
        state.fix_lacking(self)?;
        if progress.user_break() {
            return Ok(false);
        }
        self.finish_vertex_tolerances(progress, state)
    }

    fn run_notched_edges(&self, state: &mut AggregateWireState) -> Result<(), Failure> {
        let should_fix = self.state.read().fix_tail_mode <= 0
            && needs_fix(self.state.read().fix_notched_edges_mode, state.reorder_ok);
        if should_fix {
            state.fixed |= self.fix_notched_edges()?;
            if state.fixed {
                let _ = self.fix_shifted()?;
            }
        }
        Ok(())
    }

    fn run_tails(&self, state: &mut AggregateWireState) -> Result<(), Failure> {
        if self.fix_tails()? {
            state.fixed = true;
            let _ = self.fix_shifted()?;
        }
        Ok(())
    }

    fn fix_self_intersections(&self, state: &mut AggregateWireState) -> Result<(), Failure> {
        if needs_fix(
            self.state.read().fix_self_intersection_mode,
            self.closed_mode(),
        ) {
            state.fix_self_intersection(self)?;
            let _ = self.fix_reorder()?;
        }
        Ok(())
    }

    fn finish_vertex_tolerances(
        &self,
        progress: &ProgressRange<'_>,
        state: &mut AggregateWireState,
    ) -> Result<bool, Failure> {
        state.fixed |= self.fix_free_wire_vertex_tolerances()?;
        if progress.user_break() {
            return Ok(false);
        }
        if !self.context().is_null() {
            self.update_wire()?;
        }
        Ok(state.fixed)
    }

    fn fix_free_wire_vertex_tolerances(&self) -> Result<bool, Failure> {
        let wire_data: Handle<ShapeExtendWireData> = self.wire_data()?;
        if wire_data.is_null() {
            return Err(null_wire_data());
        }
        let face = self.face();
        (1..=wire_data.nb_edges()).try_fold(false, |changed, index| {
            Ok(changed
                | self
                    .edge_fixer
                    .fix_vertex_tolerance_on_face(&wire_data.edge(index)?, &face)?)
        })
    }
}

impl AggregateWireState {
    fn check_reorder(fixer: &ShapeFixWire) -> Result<Self, Failure> {
        let mut order = ShapeAnalysisWireOrder::new();
        fixer
            .working_analyzer()
            .check_order_with(&mut order, fixer.closed_mode(), true, false)?;
        let mut state = Self {
            fixed: false,
            reorder_ok: order.status() == 0,
        };
        if needs_fix(fixer.state.read().fix_reorder_mode, !state.reorder_ok) {
            state.fixed |= fixer.fix_reorder()?;
            state.reorder_ok = !fixer.status_reorder(ShapeExtendStatus::Fail);
        }
        Ok(state)
    }

    fn fix_small(&mut self, fixer: &ShapeFixWire) -> Result<(), Failure> {
        if !needs_fix(fixer.state.read().fix_small_mode, fixer.topology_mode()) {
            return Ok(());
        }
        let lock_vertex = !fixer.topology_mode() || !self.reorder_ok;
        self.fixed |= fixer.fix_small_all(lock_vertex, fixer.min_tolerance())? != 0;
        self.retry_reorder(fixer)
    }

    fn retry_reorder(&mut self, fixer: &ShapeFixWire) -> Result<(), Failure> {
        if needs_fix(fixer.state.read().fix_reorder_mode, !self.reorder_ok) {
            let _ = fixer.fix_reorder()?;
            self.reorder_ok = !fixer.status_reorder(ShapeExtendStatus::Fail);
        }
        Ok(())
    }

    fn fix_connected(&mut self, fixer: &ShapeFixWire) -> Result<(), Failure> {
        if needs_fix(fixer.state.read().fix_connected_mode, self.reorder_ok) {
            self.fixed |= fixer.fix_connected_all(-1.0)?;
        }
        Ok(())
    }

    fn fix_edge_curves(&mut self, fixer: &ShapeFixWire) -> Result<(), Failure> {
        if needs_fix(fixer.state.read().edge_curves_mode, true) {
            let saved = fixer.state.read().shifted_mode;
            let suppress_default_shift = saved == -1 && !self.reorder_ok;
            if suppress_default_shift {
                fixer.state.write().shifted_mode = 0;
            }
            let result = fixer.fix_edge_curves();
            if suppress_default_shift {
                fixer.state.write().shifted_mode = saved;
            }
            self.fixed |= result?;
        }
        Ok(())
    }

    fn fix_degenerated(&mut self, fixer: &ShapeFixWire) -> Result<(), Failure> {
        if needs_fix(fixer.state.read().fix_degenerated_mode, true) {
            self.fixed |= fixer.fix_degenerated()?;
        }
        Ok(())
    }

    fn fix_self_intersection(&mut self, fixer: &ShapeFixWire) -> Result<(), Failure> {
        if needs_fix(
            fixer.state.read().fix_self_intersection_mode,
            fixer.closed_mode(),
        ) {
            self.fixed |= fixer.perform_self_intersection_stage(self.reorder_ok)?;
        }
        Ok(())
    }

    fn fix_lacking(&mut self, fixer: &ShapeFixWire) -> Result<(), Failure> {
        if needs_fix(fixer.state.read().fix_lacking_mode, self.reorder_ok) {
            self.fixed |= fixer.fix_lacking(false)?;
        }
        Ok(())
    }
}

fn needs_fix(mode: i32, default: bool) -> bool {
    mode < 0 && default || mode > 0
}

fn null_wire_data() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeFix_Wire::Perform requires wire data; expected loaded edge sequence",
        "",
    )
}
