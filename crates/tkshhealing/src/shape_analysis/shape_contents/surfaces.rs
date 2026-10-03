use super::*;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomElementarySurface, GeomOffsetSurface,
    GeomRectangularTrimmedSurface, GeomSurface,
};
impl ShapeAnalysisShapeContents {
    pub(super) fn analyze_surface(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        face: &Face,
    ) -> Result<(), Failure> {
        let surface = self.unwrap_trimmed_surface(surface);
        if !surface.is_null() && !(surface.is_cn_u(1)? && surface.is_cn_v(1)?) {
            self.counts.c0_surfaces += 1;
        }
        self.record_bspline_surface(&surface, face);
        self.record_indirect_surface(&surface, face)?;
        if !surface.cast::<GeomOffsetSurface>().is_null() {
            self.counts.offset_surfaces += 1;
            self.recordings.offset_surface.append(face);
        } else if !surface.cast::<GeomBezierSurface>().is_null() {
            self.counts.bezier_surfaces += 1;
        }
        Ok(())
    }
    fn unwrap_trimmed_surface(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
    ) -> Handle<dyn GeomSurface> {
        if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
            self.counts.trimmed_surfaces += 1;
            return trimmed.basis_surface();
        }
        surface.clone()
    }
    fn record_bspline_surface(&mut self, surface: &Handle<dyn GeomSurface>, face: &Face) {
        let handle = surface.cast::<GeomBSplineSurface>();
        let Some(spline) = handle.get() else {
            return;
        };
        self.counts.bspline_surfaces += 1;
        if spline.nb_u_poles() * spline.nb_v_poles() > 8192 {
            self.counts.big_splines += 1;
            self.recordings.big_spline.append(face);
        }
    }
    fn record_indirect_surface(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        face: &Face,
    ) -> Result<(), Failure> {
        let handle = surface.cast::<dyn GeomElementarySurface>();
        let Some(elementary) = handle.get() else {
            return Ok(());
        };
        if !elementary.position().direct()? {
            self.counts.indirect_surfaces += 1;
            self.recordings.indirect.append(face);
        }
        Ok(())
    }
}
