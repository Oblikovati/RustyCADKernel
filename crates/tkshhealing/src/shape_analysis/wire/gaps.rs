// Rust translation of CheckGaps3d/CheckGaps2d and their pair checks from
// ShapeAnalysis/ShapeAnalysis_Wire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Boundary-gap measurements for the current ordered edge sequence.
use super::*;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;

impl ShapeAnalysisWire {
    /// `CheckGaps3d()`: measure every edge-to-edge 3D gap and retain the maximum;
    /// `analyzer.check_gaps_3d()?`.
    pub fn check_gaps_3d(&self) -> Result<bool, Failure> {
        self.reset_gap_status(true);
        if !self.is_loaded() {
            return Ok(false);
        }
        let mut max_distance: f64 = 0.0;
        for index in 1..=self.nb_edges() {
            self.check_gap_3d_at(index)?;
            self.accumulate_gap_status(true);
            if !self.last_check_status(ShapeExtendStatus::Fail1) {
                max_distance = max_distance.max(self.fields.read().min_3d);
            }
        }
        self.fields.write().min_3d = max_distance;
        self.fields.write().max_3d = max_distance;
        Ok(self.status_gaps_3d(ShapeExtendStatus::Done))
    }

    /// `CheckGaps2d()`: measure every p-curve gap using surface resolution;
    /// `analyzer.check_gaps_2d()?`.
    pub fn check_gaps_2d(&self) -> Result<bool, Failure> {
        self.reset_gap_status(false);
        if !self.is_ready() {
            return Ok(false);
        }
        let mut max_distance: f64 = 0.0;
        for index in 1..=self.nb_edges() {
            self.check_gap_2d_at(index)?;
            self.accumulate_gap_status(false);
            if !self.last_check_status(ShapeExtendStatus::Fail1) {
                max_distance = max_distance.max(self.fields.read().min_2d);
            }
        }
        let mut fields = self.fields.write();
        fields.min_2d = max_distance;
        fields.max_2d = max_distance;
        drop(fields);
        Ok(self.status_gaps_2d(ShapeExtendStatus::Done))
    }

    /// `CheckGap3d(num)`: compare the preceding curve end with this curve start.
    pub fn check_gap_3d_at(&self, index: i32) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_loaded() {
            return Ok(false);
        }
        let Some(distance) = self.gap_distance_3d(index)? else {
            return Ok(self.fail_gap());
        };
        self.fields.write().min_3d = distance;
        self.fields.write().max_3d = distance;
        self.record_gap(distance, self.precision());
        Ok(self.last_check_status(ShapeExtendStatus::Done))
    }

    fn gap_distance_3d(&self, index: i32) -> Result<Option<f64>, Failure> {
        let (previous, current) = self.gap_pair(index)?;
        let analysis = ShapeAnalysisEdge::new();
        let (previous_curve, _, previous_last) = analysis.curve_3d(&previous, true)?;
        let (current_curve, current_first, _) = analysis.curve_3d(&current, true)?;
        let (Some(previous_curve), Some(current_curve)) =
            (previous_curve.get(), current_curve.get())
        else {
            return Ok(None);
        };
        let distance = previous_curve
            .value(previous_last)?
            .distance(&current_curve.value(current_first)?);
        Ok(Some(distance))
    }

    /// `CheckGap2d(num)`: compare the preceding p-curve end with this p-curve start.
    pub fn check_gap_2d_at(&self, index: i32) -> Result<bool, Failure> {
        if self.face().is_null() || self.surface().is_null() {
            return Ok(self.fail_gap());
        }
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok(false);
        }
        let Some(distance) = self.gap_distance_2d(index)? else {
            return Ok(self.fail_gap());
        };
        self.fields.write().min_2d = distance;
        self.fields.write().max_2d = distance;
        let threshold = self.surface_resolution_threshold()?;
        self.record_gap(distance, threshold);
        Ok(self.last_check_status(ShapeExtendStatus::Done))
    }

    fn gap_distance_2d(&self, index: i32) -> Result<Option<f64>, Failure> {
        let (previous, current) = self.gap_pair(index)?;
        let face = self.face();
        let analysis = ShapeAnalysisEdge::new();
        let (previous_curve, _, previous_last) = analysis.pcurve_on_face(&previous, &face, true)?;
        let (current_curve, current_first, _) = analysis.pcurve_on_face(&current, &face, true)?;
        let (Some(previous_curve), Some(current_curve)) =
            (previous_curve.get(), current_curve.get())
        else {
            return Ok(None);
        };
        let distance = previous_curve
            .value(previous_last)?
            .distance(current_curve.value(current_first)?);
        Ok(Some(distance))
    }

    /// `StatusGaps3d(status)`: query the accumulated 3D gap flags.
    pub fn status_gaps_3d(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_gaps3d, status)
    }

    /// `StatusGaps2d(status)`: query the accumulated 2D gap flags.
    pub fn status_gaps_2d(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_gaps2d, status)
    }

    /// The largest measured 3D boundary gap.
    pub fn max_distance_3d(&self) -> f64 {
        self.fields.read().max_3d
    }

    /// The last minimum 2D measurement; `analyzer.min_distance_2d()`.
    pub fn min_distance_2d(&self) -> f64 {
        self.fields.read().min_2d
    }

    /// The largest measured 2D boundary gap.
    pub fn max_distance_2d(&self) -> f64 {
        self.fields.read().max_2d
    }

    pub(super) fn gap_pair(&self, index: i32) -> Result<(Edge, Edge), Failure> {
        let count = self.nb_edges();
        let current = if index > 0 { index } else { count };
        let previous = if current > 1 { current - 1 } else { count };
        let wire = self.wire_data();
        Ok((wire.edge(previous)?, wire.edge(current)?))
    }

    fn surface_resolution_threshold(&self) -> Result<f64, Failure> {
        let adaptor = self.surface().adaptor3d();
        let surface = adaptor
            .get()
            .ok_or_else(|| Failure::new("ShapeAnalysis_Wire::CheckGap2d requires a surface"))?;
        Ok(surface
            .u_resolution(self.precision())?
            .max(surface.v_resolution(self.precision())?)
            + rustycad_tkernel::precision::p_confusion())
    }

    fn reset_gap_status(&self, is_3d: bool) {
        let mut fields = self.fields.write();
        if is_3d {
            fields.status_gaps3d = 0;
        } else {
            fields.status_gaps2d = 0;
        }
    }

    fn accumulate_gap_status(&self, is_3d: bool) {
        let mut fields = self.fields.write();
        if is_3d {
            fields.status_gaps3d |= fields.status;
        } else {
            fields.status_gaps2d |= fields.status;
        }
    }

    fn fail_gap(&self) -> bool {
        self.set_last_status(ShapeExtendStatus::Fail1);
        false
    }

    fn record_gap(&self, distance: f64, tolerance: f64) {
        if distance <= tolerance {
            return;
        }
        self.set_last_status(ShapeExtendStatus::Done1);
    }
}
