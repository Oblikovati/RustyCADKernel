use super::ShapeFixWire;
use crate::shape_analysis::ShapeAnalysisWire;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

impl ShapeFixWire {
    /// Removes one small edge and repairs its adjacent connection when needed.
    /// Example: `fixer.fix_small_at(1, false, 1.0e-7)?`.
    pub fn fix_small_at(
        &self,
        index: i32,
        lock_vertex: bool,
        small_tolerance: f64,
    ) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        let Some(analyzer) = self.ready_small_analyzer()? else {
            return Ok(false);
        };
        let selected = select_edge_index(index, analyzer.nb_edges());
        analyzer.check_small_at(selected, small_tolerance)?;
        self.copy_small_failure(&analyzer);
        if !analyzer.last_check_status(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        self.remove_small_edge(&analyzer, selected, lock_vertex, small_tolerance)
    }

    /// Removes small edges in reverse order and accumulates native status flags.
    /// Example: `fixer.fix_small_all(false, 1.0e-7)?`.
    pub fn fix_small_all(&self, lock_vertex: bool, small_tolerance: f64) -> Result<i32, Failure> {
        self.state.write().small_status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
        let Some(analyzer) = self.loaded_wire_analyzer()? else {
            return Ok(0);
        };
        for index in (1..=analyzer.nb_edges()).rev() {
            self.fix_small_at(index, lock_vertex, small_tolerance)?;
            self.accumulate_small_status();
        }
        Ok(i32::from(self.status_small(ShapeExtendStatus::Done)))
    }

    fn ready_small_analyzer(&self) -> Result<Option<Handle<ShapeAnalysisWire>>, Failure> {
        let Some(analyzer) = self.loaded_wire_analyzer()? else {
            return Ok(None);
        };
        if analyzer.nb_edges() <= 1 {
            return Ok(None);
        }
        Ok(Some(analyzer))
    }

    pub(super) fn loaded_wire_analyzer(
        &self,
    ) -> Result<Option<Handle<ShapeAnalysisWire>>, Failure> {
        let analyzer = self.state.read().analyzer.clone();
        if analyzer.is_null() || !analyzer.is_loaded() || analyzer.nb_edges() <= 0 {
            return Ok(None);
        }
        Ok(Some(analyzer))
    }

    fn remove_small_edge(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        lock_vertex: bool,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let done2 = analyzer.last_check_status(ShapeExtendStatus::Done2);
        if done2 && (lock_vertex || !self.topology_mode()) {
            self.or_last_fix_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        let wire_data = analyzer.wire_data();
        let edge = wire_data.edge(index)?;
        self.remove_edge_from_context(&edge)?;
        self.warn_small_edge(&edge);
        wire_data.remove(index)?;
        self.or_last_fix_status(if done2 {
            ShapeExtendStatus::Done2
        } else {
            ShapeExtendStatus::Done1
        });
        if done2 {
            self.fix_connected_after_removal(index, tolerance)?;
        }
        Ok(true)
    }

    fn remove_edge_from_context(&self, edge: &Edge) -> Result<(), Failure> {
        if let Some(context) = self.context().get() {
            ReShape::remove(context, &edge.clone().into_shape())?;
        }
        Ok(())
    }

    fn warn_small_edge(&self, edge: &Edge) {
        let key = AsciiString::from("FixAdvWire.FixSmall.MSG0");
        self.send_warning(&edge.clone().into_shape(), &Msg::from_key(&key));
    }

    fn copy_small_failure(&self, analyzer: &ShapeAnalysisWire) {
        if analyzer.last_check_status(ShapeExtendStatus::Fail) {
            self.set_last_fix_status(ShapeExtendStatus::Fail1);
        }
    }

    fn accumulate_small_status(&self) {
        let last_status = self.state.read().last_fix_status;
        self.state.write().small_status |= last_status;
    }
}

pub(super) fn select_edge_index(index: i32, count: i32) -> i32 {
    if index > 0 { index } else { count }
}

#[cfg(test)]
mod tests {
    use super::super::wire_connected::connected_tolerance;
    use super::select_edge_index;
    use crate::shape_fix::FixRoot;

    #[test]
    fn nonpositive_native_indices_select_the_last_edge() {
        assert_eq!(select_edge_index(0, 4), 4);
        assert_eq!(select_edge_index(-1, 4), 4);
        assert_eq!(select_edge_index(2, 4), 2);
    }

    #[test]
    fn negative_connected_precision_uses_root_maximum() {
        let fixer = super::ShapeFixWire::default();
        fixer.set_max_tolerance(0.75);
        assert_eq!(connected_tolerance(&fixer, -1.0), 0.75);
        assert_eq!(connected_tolerance(&fixer, 0.25), 0.25);
    }
}
