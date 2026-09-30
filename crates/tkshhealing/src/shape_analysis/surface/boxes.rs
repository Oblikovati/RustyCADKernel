use super::*;
use rustycad_tkernel::precision;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::bnd_lib::Add3dCurve;

impl ShapeAnalysisSurface {
    /// Computes boundary boxes once; `analysis.compute_boxes()?`.
    pub fn compute_boxes(&self) -> Result<(), Failure> {
        if self.fields.read().boxes_computed {
            return Ok(());
        }
        self.fields.write().boxes_computed = true;
        self.compute_bound_isos();
        let isos = self.fields.read().bound_isos.clone().unwrap();
        for (index, iso) in isos.iter().enumerate().filter(|(_, iso)| !iso.is_null()) {
            let mut bounds = self.fields.read().boxes[index].clone();
            let result = Add3dCurve::add(
                &GeomAdaptorCurve::with_curve(iso)?,
                precision::confusion(),
                &mut bounds,
            );
            self.fields.write().boxes[index] = bounds;
            result?;
        }
        Ok(())
    }

    /// Returns the first U boundary box; `analysis.get_box_uf()?`.
    pub fn get_box_uf(&self) -> Result<BoundingBox3, Failure> {
        self.boundary_box(0)
    }
    /// Returns the last U boundary box; `analysis.get_box_ul()?`.
    pub fn get_box_ul(&self) -> Result<BoundingBox3, Failure> {
        self.boundary_box(1)
    }
    /// Returns the first V boundary box; `analysis.get_box_vf()?`.
    pub fn get_box_vf(&self) -> Result<BoundingBox3, Failure> {
        self.boundary_box(2)
    }
    /// Returns the last V boundary box; `analysis.get_box_vl()?`.
    pub fn get_box_vl(&self) -> Result<BoundingBox3, Failure> {
        self.boundary_box(3)
    }

    fn boundary_box(&self, index: usize) -> Result<BoundingBox3, Failure> {
        self.compute_boxes()?;
        Ok(self.fields.read().boxes[index].clone())
    }
}
