//! Public result and tool configuration preserve native no-reset setters.
use super::*;
impl ShapeUpgradeWireDivide {
    /// Return the loaded/resulting wire; `tool.wire()`.
    pub fn wire(&self) -> Wire {
        self.state.with(|fields| fields.wire.clone())
    }
    /// Decode the last Perform status; `tool.status_flag(ShapeExtendStatus::Done1)`.
    pub fn status_flag(&self, status: ShapeExtendStatus) -> bool {
        self.state
            .with(|fields| ShapeExtend::decode_status(fields.status, status))
    }
    /// Configure spatial curve splitting; `tool.set_split_curve_3d_tool(&splitter)`.
    pub fn set_split_curve_3d_tool(&self, tool: &Handle<ShapeUpgradeSplitCurve3d>) {
        self.state.with_mut(|fields| fields.spatial = tool.clone());
    }
    /// Configure pcurve splitting; `tool.set_split_curve_2d_tool(&splitter)`.
    pub fn set_split_curve_2d_tool(&self, tool: &Handle<ShapeUpgradeSplitCurve2d>) {
        self.state.with_mut(|fields| fields.planar = tool.clone());
    }
    /// Configure parameter transfer; `tool.set_transfer_param_tool(&transfer)`.
    pub fn set_transfer_param_tool(&self, tool: &Handle<ShapeAnalysisTransferParameters>) {
        self.state.with_mut(|fields| fields.transfer = tool.clone());
    }
    /// Replace the edge criterion; `tool.set_edge_divide_tool(&criterion)`.
    pub fn set_edge_divide_tool(&self, tool: &Handle<ShapeUpgradeEdgeDivide>) {
        self.state
            .with_mut(|fields| fields.edge_divide = tool.clone());
    }
    /// Select free/shared/all spatial curves, preserving other integer modes; `tool.set_edge_mode(2)`.
    pub fn set_edge_mode(&self, mode: i32) {
        self.state.with_mut(|fields| fields.edge_mode = mode);
    }
    /// Replace small-curve approximation; `tool.set_fix_small_curve_tool(&approximation)`.
    pub fn set_fix_small_curve_tool(&self, tool: &Handle<ShapeUpgradeFixSmallCurves>) {
        self.state.with_mut(|fields| fields.small = tool.clone());
    }
    /// Access the small-curve tool; `tool.fix_small_curve_tool()`.
    pub fn fix_small_curve_tool(&self) -> Handle<ShapeUpgradeFixSmallCurves> {
        self.state.with(|fields| fields.small.clone())
    }
}
