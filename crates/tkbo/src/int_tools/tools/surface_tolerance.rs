use super::Tools;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::{geom2d::Geom2dCurve, geom2d_adaptor::Geom2dAdaptorCurve};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurveOnSurface,
    geom::{GeomCurve, GeomSurface},
    geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface},
};
use rustycad_tkgeombase::geom_lib::CheckCurveOnSurface;
impl Tools {
    /// Measures spatial/pcurve discrepancy and adds the native numerical margin; `Tools::compute_tolerance(&c3, &c2, &surface, 0., 1., &mut d, &mut t, 1e-9, false)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn compute_tolerance(
        curve: &Handle<dyn GeomCurve>,
        pcurve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        first: f64,
        last: f64,
        distance: &mut f64,
        parameter: &mut f64,
        range_tolerance: f64,
        parallel: bool,
    ) -> Result<bool, Failure> {
        let spatial = Handle::new(GeomAdaptorCurve::with_range(curve, first, last)?).into_cast();
        let planar = Handle::new(Geom2dAdaptorCurve::with_range(pcurve, first, last)?).into_cast();
        let surface = Handle::new(GeomAdaptorSurface::with_surface(surface)?).into_cast();
        let on_surface = Handle::new(Adaptor3dCurveOnSurface::with_curve(&planar, &surface)?);
        let mut check = CheckCurveOnSurface::with_curve(&spatial, range_tolerance);
        check.set_parallel(parallel);
        check.perform(&on_surface);
        if !check.is_done() {
            return Ok(false);
        }
        *distance = (1.0 + 1.0e-5) * check.max_distance();
        *parameter = check.max_parameter();
        Ok(true)
    }
}
