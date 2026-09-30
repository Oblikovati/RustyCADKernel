//! Closed-curve splitting and the native four-line plane/cone reduction.
use super::*;
impl FaceFace {
    /// Splits closed curves and reduces the native plane/cone case; `ff.prepare_lines_3d(true)?`.
    pub fn prepare_lines_3d(&mut self, split: bool) -> Result<(), Failure> {
        let mut curves = self.prepared_curves(split)?;
        let first = self.surfaces[0].get_type()?;
        let second = self.surfaces[1].get_type()?;
        let plane_cone = (first == SurfaceType::Plane && second == SurfaceType::Cone)
            || (second == SurfaceType::Plane && first == SurfaceType::Cone);
        if plane_cone && curves.len() == 4 && curves[0].curve_type()? == CurveType::Line {
            let mut output = Vec::new();
            Tools::reject_lines(&mut curves, &mut output)?;
            curves = output;
        }
        // Failure leaves the original sequence intact, as the native temporary does.
        self.curves = curves;
        Ok(())
    }
    fn prepared_curves(&self, split: bool) -> Result<Vec<Curve>, Failure> {
        let mut curves = Vec::new();
        for curve in &self.curves {
            if !split || Tools::split_curve(curve, &mut curves)? == 0 {
                curves.push(curve.clone());
            }
        }
        Ok(curves)
    }
}
