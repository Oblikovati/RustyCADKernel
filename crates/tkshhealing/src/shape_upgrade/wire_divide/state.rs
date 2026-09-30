//! Tool handles and protected state retain their native sharing and replacement semantics.
use super::*;
/// Native protected state; `tool.wire_divide_data().with(|fields|fields.edge_mode)`.
pub struct WireDivideFields {
    /// Supporting face retained by Load.
    pub face: Face,
    /// Loaded or resulting wire.
    pub wire: Wire,
    /// ShapeExtend status bits for the last Perform.
    pub status: i32,
    /// Zero free edges, one shared edges, two all spatial curves.
    pub edge_mode: i32,
    pub(super) spatial: Handle<ShapeUpgradeSplitCurve3d>,
    pub(super) planar: Handle<ShapeUpgradeSplitCurve2d>,
    pub(super) edge_divide: Handle<ShapeUpgradeEdgeDivide>,
    pub(super) transfer: Handle<ShapeAnalysisTransferParameters>,
    pub(super) small: Handle<ShapeUpgradeFixSmallCurves>,
}
impl Default for WireDivideFields {
    fn default() -> Self {
        Self {
            face: Face::default(),
            wire: Wire::default(),
            status: 0,
            edge_mode: 2,
            spatial: Handle::new(ShapeUpgradeSplitCurve3d::new()),
            planar: Handle::new(ShapeUpgradeSplitCurve2d::new()),
            edge_divide: Handle::new(ShapeUpgradeEdgeDivide::new()),
            transfer: Handle::new(ShapeAnalysisTransferParametersProj::new()).cast(),
            small: Handle::new(ShapeUpgradeFixSmallCurves::new()),
        }
    }
}
/// Shared state for derived wire criteria; `WireDivideData::default()`.
#[derive(Default)]
pub struct WireDivideData {
    fields: SharedState<WireDivideFields>,
}
impl WireDivideData {
    /// Inspect native protected state; `state.with(|fields|fields.status)`.
    pub fn with<R>(&self, read: impl FnOnce(&WireDivideFields) -> R) -> R {
        read(&self.fields.read())
    }
    /// Update derived policy state; `state.with_mut(|fields|fields.edge_mode=2)`.
    pub fn with_mut<R>(&self, write: impl FnOnce(&mut WireDivideFields) -> R) -> R {
        write(&mut self.fields.write())
    }
}
pub(super) fn require<T: rustycad_tkernel::standard::RuntimeTyped + ?Sized>(
    handle: &Handle<T>,
    name: &str,
) -> Result<(), Failure> {
    if handle.is_null() {
        return Err(Failure::with_type(
            "Standard_NullObject",
            &format!("ShapeUpgrade_WireDivide: null {name}; expected initialized tool handle"),
            "",
        ));
    }
    Ok(())
}
