//! Resolve visible Graphic3d layers into the same immutable submissions used by offscreen tests.
use crate::{
    PrimitiveDraw, RenderError, RenderFrame, RenderLayer, driver_structure::RenderStructure,
    error::rejected,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectWindowMembers, graphic3d::*};
pub(crate) fn frame_for_view(
    view: &dyn Graphic3dCView,
    extent: [u32; 2],
) -> Result<RenderFrame, RenderError> {
    validate_view(view)?;
    let camera = view.camera();
    let mut layers = view.layers();
    // OpenGl_FilteredIndexedLayerIterator at the pinned baseline renders normal layers before immediate ones.
    layers.sort_by_key(|layer| layer.is_immediate());
    let rendering = view.rendering_params();
    let selected = layers
        .iter()
        .filter(|layer| layer_selected(view, layer.layer_id()));
    let passes = selected
        .map(|layer| layer_pass(layer, &camera, view.identification(), extent, &rendering))
        .collect::<Result<_, _>>()?;
    let color = view.cview_base().fields.read().bg_color;
    let clear = [
        color.rgb().red() as f32,
        color.rgb().green() as f32,
        color.rgb().blue() as f32,
        color.alpha(),
    ];
    RenderFrame::with_layers(extent[0], extent[1], clear, passes)
}
fn layer_selected(view: &dyn Graphic3dCView, id: Graphic3dZLayerId) -> bool {
    if view.z_layer_redraw_mode() {
        return id == view.z_layer_target();
    }
    id >= view.z_layer_target()
}
fn layer_pass(
    layer: &Graphic3dLayer,
    camera: &Graphic3dCamera,
    view_id: i32,
    extent: [u32; 2],
    rendering: &Graphic3dRenderingParams,
) -> Result<RenderLayer, RenderError> {
    let settings = layer.layer_settings();
    validate_layer(layer.layer_id(), &settings)?;
    let mut draws = Vec::new();
    for priority in layer.array_of_structures() {
        for weak in priority.iter() {
            draws.extend(structure_draws(
                &weak.upgrade(),
                camera,
                view_id,
                extent,
                rendering,
            )?);
        }
    }
    Ok(RenderLayer::new(
        draws,
        settings.to_enable_depth_test(),
        settings.to_enable_depth_write(),
        settings.to_clear_depth(),
    ))
}
fn structure_draws(
    structure: &Handle<dyn Graphic3dCStructure>,
    camera: &Graphic3dCamera,
    view_id: i32,
    extent: [u32; 2],
    rendering: &Graphic3dRenderingParams,
) -> Result<Vec<PrimitiveDraw>, RenderError> {
    if structure.is_null() {
        return Ok(vec![]);
    }
    let retained = structure.cast::<RenderStructure>();
    retained
        .get()
        .ok_or_else(|| rejected(structure.object_address(), "a Martensite driver structure"))?
        .draws(camera, view_id, extent, rendering)
}
fn validate_raster_params(view: &dyn Graphic3dCView) -> Result<(), RenderError> {
    let params = view.rendering_params();
    if params.method != Graphic3dRenderingMode::Rasterization
        || params.nb_msaa_samples != 0
        || params.render_resolution_scale != 1.0
    {
        return Err(rejected(
            (
                params.method,
                params.nb_msaa_samples,
                params.render_resolution_scale,
            ),
            "single-sample rasterization at scale 1",
        ));
    }
    Ok(())
}
fn validate_view(view: &dyn Graphic3dCView) -> Result<(), RenderError> {
    validate_raster_params(view)?;
    if view.camera().is_stereo() || !view.clip_planes().is_null() || view.computed_mode() {
        return Err(rejected(
            "stereo, clip planes or computed rendering",
            "ordinary unclipped camera rendering in this slice",
        ));
    }
    if view.cview_base().fields.read().background_type != Graphic3dTypeOfBackground::None {
        return Err(rejected(
            view.cview_base().fields.read().background_type,
            "a solid background in this renderer slice",
        ));
    }
    if view.fbo().is_null() && !view.window().is_virtual() {
        return Err(rejected(
            "onscreen window",
            "a virtual window until Vulkan presentation is implemented",
        ));
    }
    Ok(())
}

fn validate_layer(
    id: Graphic3dZLayerId,
    settings: &Graphic3dZLayerSettings,
) -> Result<(), RenderError> {
    if !settings.origin_transformation().is_null()
        || settings.has_culling_distance()
        || settings.has_culling_size()
    {
        return Err(rejected(
            id,
            "zero layer origin and no distance/size culling in this renderer slice",
        ));
    }
    Ok(())
}
