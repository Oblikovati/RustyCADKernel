//! Snapshot the actual view parameters and its injected legacy lighting policy once per frame.
use crate::{
    Graphic3dRenderDriver, RenderError, RenderLightingPipeline, transparency::RenderRasterSettings,
};
use rustycad_tkservice::graphic3d::{Graphic3dCView, Graphic3dRenderingParams};

pub(crate) fn view_raster_settings(
    view: &dyn Graphic3dCView,
    params: &Graphic3dRenderingParams,
) -> Result<RenderRasterSettings, RenderError> {
    let manager = view.cview_base().fields.read().structure_manager.clone();
    let fixed = manager
        .get()
        .and_then(|manager| {
            manager
                .graphic_driver()
                .cast::<Graphic3dRenderDriver>()
                .get()
                .map(|driver| driver.lighting_pipeline == RenderLightingPipeline::FixedFunction)
        })
        .unwrap_or(false);
    RenderRasterSettings::from_parameters(params, fixed)
}
