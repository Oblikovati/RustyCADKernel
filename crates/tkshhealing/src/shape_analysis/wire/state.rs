//! Snapshot access to native wire analyzer state.
use super::*;
impl ShapeAnalysisWire {
    /// Reset the implemented check status fields; `analyzer.clear_statuses()`.
    pub fn clear_statuses(&self) {
        let mut fields = self.fields.write();
        fields.status = 0;
        fields.status_order = 0;
        fields.status_connected = 0;
        fields.status_gaps3d = 0;
        fields.status_gaps2d = 0;
        fields.status_curve_gaps = 0;
        fields.status_degenerated = 0;
        fields.status_closed = 0;
        fields.status_lacking = 0;
        fields.status_small = 0;
        fields.status_self_intersection = 0;
        fields.status_edge_curves = 0;
        fields.status_loop = 0;
        fields.min_3d = 0.;
        fields.max_3d = 0.;
        fields.min_2d = 0.;
        fields.max_2d = 0.;
    }
    /// Assign precision without normalization; `analyzer.set_precision(-1.0)`.
    pub fn set_precision(&self, value: f64) {
        self.fields.write().precision = value;
    }
    /// Stored precision; `analyzer.precision()`.
    pub fn precision(&self) -> f64 {
        self.fields.read().precision
    }
    /// Shared edge list; `analyzer.wire_data()`.
    pub fn wire_data(&self) -> Handle<ShapeExtendWireData> {
        self.fields.read().wire.clone()
    }
    /// Number of loaded manifold edges, zero for a null list; `analyzer.nb_edges()`.
    pub fn nb_edges(&self) -> i32 {
        self.wire_data()
            .get()
            .map_or(0, ShapeExtendWireData::nb_edges)
    }
    /// True for a nonempty loaded wire; `analyzer.is_loaded()`.
    pub fn is_loaded(&self) -> bool {
        self.nb_edges() > 0
    }
    /// True when both wire and face are supplied; `analyzer.is_ready()`.
    pub fn is_ready(&self) -> bool {
        self.is_loaded() && !self.face().is_null()
    }
    /// Face snapshot; `analyzer.face()`.
    pub fn face(&self) -> Face {
        self.fields.read().face.clone()
    }
    /// Shared surface analyzer; `analyzer.surface()`.
    pub fn surface(&self) -> Handle<ShapeAnalysisSurface> {
        self.fields.read().surface.clone()
    }
    /// Result of the public ordering check; `analyzer.status_order(ShapeExtendStatus::Done)`.
    pub fn status_order(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_order, status)
    }
    /// Result of the most recent advanced check; `analyzer.last_check_status(ShapeExtendStatus::Fail)`.
    pub fn last_check_status(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status, status)
    }
    pub(super) fn set_last_status(&self, status: ShapeExtendStatus) {
        self.fields.write().status = ShapeExtend::encode_status(status);
    }
}
