//! Native edge split criteria and shared parameter ownership at OCCT 3d097a0.
use super::{ShapeUpgradeSplitCurve2d, ShapeUpgradeSplitCurve3d, ShapeUpgradeTool};
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    ncollection::HSequence,
    standard::{Failure, Handle, SharedState, Transient},
};
mod compute;
/// Shared split parameters; `Handle::new(HSequence::<f64>::default())`.
pub type EdgeSplitKnots = Handle<HSequence<f64>>;
/// Native protected edge-division state for specialized criteria.
#[derive(Default)]
pub struct EdgeDivideFields {
    /// Supporting face, retained by Clear.
    pub face: Face,
    /// Whether the last Compute found a pcurve.
    pub has_curve_2d: bool,
    /// Whether the last Compute found a spatial curve.
    pub has_curve_3d: bool,
    /// Shared pcurve splitting parameters.
    pub knots_2d: EdgeSplitKnots,
    /// Shared spatial splitting parameters.
    pub knots_3d: EdgeSplitKnots,
    curve_2d_tool: Handle<ShapeUpgradeSplitCurve2d>,
    curve_3d_tool: Handle<ShapeUpgradeSplitCurve3d>,
}
/// Protected state shared with derived criteria; `EdgeDivideData::default()`.
pub struct EdgeDivideData {
    fields: SharedState<EdgeDivideFields>,
}
impl Default for EdgeDivideData {
    fn default() -> Self {
        Self {
            fields: SharedState::new(EdgeDivideFields {
                curve_2d_tool: Handle::new(ShapeUpgradeSplitCurve2d::new()),
                curve_3d_tool: Handle::new(ShapeUpgradeSplitCurve3d::new()),
                ..EdgeDivideFields::default()
            }),
        }
    }
}
impl EdgeDivideData {
    /// Read native protected fields; `state.with(|fields|fields.has_curve_3d)`.
    pub fn with<R>(&self, read: impl FnOnce(&EdgeDivideFields) -> R) -> R {
        read(&self.fields.read())
    }
    /// Update a derived criterion; `state.with_mut(|fields|fields.has_curve_3d=false)`.
    pub fn with_mut<R>(&self, write: impl FnOnce(&mut EdgeDivideFields) -> R) -> R {
        write(&mut self.fields.write())
    }
    fn clear(&self) {
        self.with_mut(|fields| {
            fields.knots_3d = Handle::default();
            fields.knots_2d = Handle::default();
            fields.has_curve_3d = false;
            fields.has_curve_2d = false;
        });
    }
}
/// Virtual edge criterion and tool selection; `divider.compute(&edge)?`.
pub trait EdgeDivider: Transient {
    /// Shared protected state; `divider.edge_divide_data()`.
    fn edge_divide_data(&self) -> &EdgeDivideData;
    /// Discover curves and compute split parameters; `divider.compute(&edge)?`.
    fn compute(&self, edge: &Edge) -> Result<bool, Failure> {
        self.edge_divide_data().clear();
        let spatial = compute::spatial_knots(self, edge)?;
        let planar = compute::planar_knots(self, edge)?;
        Ok(compute::curve_done(&spatial.cast())? || compute::curve_done(&planar.cast())?)
    }
    /// Virtual pcurve tool selection; `divider.split_curve_2d_tool()`.
    fn split_curve_2d_tool(&self) -> Handle<ShapeUpgradeSplitCurve2d> {
        self.edge_divide_data()
            .with(|fields| fields.curve_2d_tool.clone())
    }
    /// Virtual spatial tool selection; `divider.split_curve_3d_tool()`.
    fn split_curve_3d_tool(&self) -> Handle<ShapeUpgradeSplitCurve3d> {
        self.edge_divide_data()
            .with(|fields| fields.curve_3d_tool.clone())
    }
}
impl_transient_interface!(dyn EdgeDivider, "ShapeUpgrade_EdgeDivide", ShapeUpgradeTool);
/// Base curve-discovery criterion; `ShapeUpgradeEdgeDivide::new()`.
#[derive(Default)]
pub struct ShapeUpgradeEdgeDivide {
    parent: ShapeUpgradeTool,
    state: EdgeDivideData,
}
impl_transient!(ShapeUpgradeEdgeDivide,"ShapeUpgrade_EdgeDivide",ShapeUpgradeTool,parent,interfaces:[dyn EdgeDivider]);
impl EdgeDivider for ShapeUpgradeEdgeDivide {
    fn edge_divide_data(&self) -> &EdgeDivideData {
        &self.state
    }
}
impl std::ops::Deref for ShapeUpgradeEdgeDivide {
    type Target = ShapeUpgradeTool;
    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}
impl ShapeUpgradeEdgeDivide {
    /// Create default spatial and pcurve tools; `ShapeUpgradeEdgeDivide::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Clear discovery flags and knots, retaining face and tools; `divider.clear()`.
    pub fn clear(&self) {
        self.state.clear();
    }
    /// Set the supporting face; `divider.set_face(&face)`.
    pub fn set_face(&self, face: &Face) {
        self.state.with_mut(|fields| fields.face = face.clone());
    }
    /// Whether Compute found a pcurve; `divider.has_curve_2d()`.
    pub fn has_curve_2d(&self) -> bool {
        self.state.with(|fields| fields.has_curve_2d)
    }
    /// Whether Compute found a spatial curve; `divider.has_curve_3d()`.
    pub fn has_curve_3d(&self) -> bool {
        self.state.with(|fields| fields.has_curve_3d)
    }
    /// Shared pcurve parameters; `divider.knots_2d()`.
    pub fn knots_2d(&self) -> EdgeSplitKnots {
        self.state.with(|fields| fields.knots_2d.clone())
    }
    /// Shared spatial parameters; `divider.knots_3d()`.
    pub fn knots_3d(&self) -> EdgeSplitKnots {
        self.state.with(|fields| fields.knots_3d.clone())
    }
    /// Replace the pcurve splitting tool; `divider.set_split_curve_2d_tool(&tool)`.
    pub fn set_split_curve_2d_tool(&self, tool: &Handle<ShapeUpgradeSplitCurve2d>) {
        self.state
            .with_mut(|fields| fields.curve_2d_tool = tool.clone());
    }
    /// Replace the spatial splitting tool; `divider.set_split_curve_3d_tool(&tool)`.
    pub fn set_split_curve_3d_tool(&self, tool: &Handle<ShapeUpgradeSplitCurve3d>) {
        self.state
            .with_mut(|fields| fields.curve_3d_tool = tool.clone());
    }
}
