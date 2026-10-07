// Rust translation of ShapeFix/ShapeFix_Wire.cxx FixReorder (both overloads) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeFix_Wire::FixReorder`: analyze the edge order and apply a ShapeAnalysis_WireOrder.
use super::ShapeFixWire;
use crate::shape_analysis::ShapeAnalysisWireOrder;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};

impl ShapeFixWire {
    /// `FixReorder(theModeBoth)`: checks the order of the loaded edges (in both directions
    /// on a bi-periodic surface when `mode_both`), then applies it and sets StatusReorder.
    /// Example: `let reordered = fixer.fix_reorder(false)?;`.
    pub fn fix_reorder(&self, mode_both: bool) -> Result<bool, Failure> {
        self.set_reorder_status(ShapeExtendStatus::Ok);
        if !self.is_loaded() {
            return Ok(false);
        }
        let mut order = ShapeAnalysisWireOrder::new();
        let both = mode_both && self.bi_periodic_surface();
        self.ensure_analyzer()
            .check_order_with(&mut order, self.closed_mode(), true, both)?;
        self.fix_reorder_with(&order)?;
        self.record_reorder_status(order.status());
        Ok(self.last_fix_status(ShapeExtendStatus::Done))
    }

    /// `FixReorder(wi)`: reorders the wire data by the signed indices of `order`; the
    /// outcome is in LastFixStatus (FAIL1 bad order, FAIL2 count mismatch, FAIL3 hole).
    /// Example: `fixer.fix_reorder_with(&order)?`.
    pub fn fix_reorder_with(&self, order: &ShapeAnalysisWireOrder) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        if !self.is_loaded() || order.status() == 0 {
            return Ok(false);
        }
        if order.status() <= -10 {
            self.or_last_fix_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        }
        let wire_data = self.wire_data()?;
        if wire_data.nb_edges() != order.nb_edges() {
            self.or_last_fix_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        self.apply_order(&wire_data, order)
    }

    fn bi_periodic_surface(&self) -> bool {
        let surface = self.ensure_analyzer().surface();
        let Some(analysis) = surface.get() else {
            return false;
        };
        let geometry = analysis.surface();
        // Native dereferences the surface unguarded; a null one cannot be bi-periodic.
        !geometry.is_null() && geometry.is_u_periodic() && geometry.is_v_periodic()
    }

    // Native checks every rank for a hole before it moves any edge.
    fn apply_order(
        &self,
        wire_data: &Handle<ShapeExtendWireData>,
        order: &ShapeAnalysisWireOrder,
    ) -> Result<bool, Failure> {
        let count = wire_data.nb_edges();
        let ranks = (1..=count)
            .map(|rank| order.ordered(rank))
            .collect::<Result<Vec<i32>, _>>()?;
        if ranks.contains(&0) {
            self.or_last_fix_status(ShapeExtendStatus::Fail3);
            return Ok(false);
        }
        let edges = ranks
            .iter()
            .map(|&rank| wire_data.edge(rank))
            .collect::<Result<Vec<Edge>, _>>()?;
        for (index, edge) in edges.iter().enumerate() {
            wire_data.set(edge, index as i32 + 1)?;
        }
        self.or_last_fix_status(ShapeExtendStatus::Done1);
        Ok(true)
    }

    fn record_reorder_status(&self, order_status: i32) {
        if self.last_fix_status(ShapeExtendStatus::Fail) {
            let fail = if self.last_fix_status(ShapeExtendStatus::Fail1) {
                ShapeExtendStatus::Fail1
            } else {
                ShapeExtendStatus::Fail2
            };
            self.or_reorder_status(fail);
        }
        if !self.last_fix_status(ShapeExtendStatus::Done) {
            return;
        }
        self.or_reorder_status(ShapeExtendStatus::Done1);
        if matches!(order_status, 2 | -2) {
            self.or_reorder_status(ShapeExtendStatus::Done2);
        }
        if order_status < 0 {
            self.or_reorder_status(ShapeExtendStatus::Done3);
        }
        if order_status == 3 {
            self.or_reorder_status(ShapeExtendStatus::Done5);
        }
    }

    fn set_reorder_status(&self, status: ShapeExtendStatus) {
        self.state.write().reorder_status = ShapeExtend::encode_status(status);
    }

    fn or_reorder_status(&self, status: ShapeExtendStatus) {
        self.state.write().reorder_status |= ShapeExtend::encode_status(status);
    }
}
