//! Native 45-point comparison of edge curves and their surface representations.
// ShapeAnalysis_Wire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface},
    geom::GeomCurve,
    geom_adaptor::GeomAdaptorSurface,
};

impl ShapeAnalysisWire {
    /// Compare every edge's representations; `analyzer.check_curve_gaps()?`.
    pub fn check_curve_gaps(&self) -> Result<bool, Failure> {
        self.fields.write().status_curve_gaps = 0;
        if !self.is_ready() {
            return Ok(false);
        }
        let mut maximum = 0.0;
        for index in 1..=self.nb_edges() {
            self.check_curve_gap_at(index)?;
            maximum = self.accumulate_curve_gap(maximum);
        }
        self.store_curve_gap(maximum);
        Ok(self.status_curve_gaps(ShapeExtendStatus::Done))
    }

    /// Compare one edge, with zero selecting the last; `analyzer.check_curve_gap_at(0)?`.
    pub fn check_curve_gap_at(&self, index: i32) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_loaded() {
            return Ok(false);
        }
        let selected = if index > 0 { index } else { self.nb_edges() };
        let Some(distance) = self.measure_curve_gap(selected)? else {
            self.set_last_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        };
        self.store_curve_gap(distance);
        if distance > self.precision() {
            self.set_last_status(ShapeExtendStatus::Done1);
        }
        Ok(self.last_check_status(ShapeExtendStatus::Done))
    }

    /// Query the aggregate flags; `analyzer.status_curve_gaps(ShapeExtendStatus::Fail)`.
    pub fn status_curve_gaps(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_curve_gaps, status)
    }

    fn measure_curve_gap(&self, index: i32) -> Result<Option<f64>, Failure> {
        let edge = self.wire_data().edge(index)?;
        let analysis = ShapeAnalysisEdge::new();
        let (curve, first, last) = analysis.curve_3d(&edge, false)?;
        let Some(curve) = curve.get() else {
            return Ok(None);
        };
        let (pcurve, pfirst, plast) = analysis.pcurve_on_face(&edge, &self.face(), false)?;
        if pcurve.is_null() {
            return Ok(None);
        }
        let adapted = Handle::new(Geom2dAdaptorCurve::with_range(&pcurve, pfirst, plast)?);
        let surface = Handle::new(GeomAdaptorSurface::with_surface(&self.surface().surface())?);
        let on_surface = Adaptor3dCurveOnSurface::with_curve(&adapted.cast(), &surface.cast())?;
        sample_curve_gap(curve, (first, last), &on_surface, (pfirst, plast)).map(Some)
    }

    fn accumulate_curve_gap(&self, previous: f64) -> f64 {
        let mut fields = self.fields.write();
        fields.status_curve_gaps |= fields.status;
        if !ShapeExtend::decode_status(fields.status, ShapeExtendStatus::Fail1)
            && previous < fields.min_3d
        {
            return fields.min_3d;
        }
        previous
    }

    fn store_curve_gap(&self, distance: f64) {
        let mut fields = self.fields.write();
        fields.min_3d = distance;
        fields.max_3d = distance;
    }
}

fn sample_curve_gap(
    curve: &dyn GeomCurve,
    range: (f64, f64),
    on_surface: &Adaptor3dCurveOnSurface,
    surface_range: (f64, f64),
) -> Result<f64, Failure> {
    let mut maximum = 0.0;
    for index in 0..45 {
        let parameter = range.0 + f64::from(index) * (range.1 - range.0) / 44.;
        let uv_parameter =
            surface_range.0 + f64::from(index) * (surface_range.1 - surface_range.0) / 44.;
        let distance = curve
            .value(parameter)?
            .square_distance(&on_surface.value(uv_parameter)?);
        if maximum < distance {
            maximum = distance;
        }
    }
    Ok(maximum.sqrt())
}
