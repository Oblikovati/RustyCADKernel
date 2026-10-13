//! Resolve visible Graphic3d layers into the same immutable submissions used by offscreen tests.
use crate::{
    PrimitiveDraw, RenderError, RenderFrame, RenderLayer, driver_structure::RenderStructure,
    error::rejected,
};
use rustycad_tkernel::standard::{Handle, HandleObserver, HandleReference};
use rustycad_tkservice::graphic3d::*;
pub(crate) fn frame_for_view(
    view: &dyn Graphic3dCView,
    extent: [u32; 2],
    clipping: &dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource>,
) -> Result<RenderFrame, RenderError> {
    validate_view(view)?;
    crate::retained_texture_images::prune_view_texture_snapshots(view)?;
    let camera = crate::view_render_camera::owned_render_camera(view);
    let (mut passes, rendering) = view_layer_passes(view, extent, &camera, clipping)?;
    apply_view_culling(&mut passes, view.backfacing_model());
    crate::retained_lighting::apply_view_lights(&mut passes, view)?;
    let mut frame = RenderFrame::with_layers(extent[0], extent[1], view_clear_color(view), passes)?;
    frame.raster = crate::retained_transparency::view_raster_settings(view, &rendering)?;
    crate::compatibility_text_color::capture_text_color_target(&mut frame);
    Ok(frame)
}
struct ViewLayerInputs<'a> {
    camera: &'a Graphic3dCamera,
    clipping: HandleObserver<Graphic3dSequenceOfHClipPlane>,
    rendering: Graphic3dRenderingParams,
    extent: [u32; 2],
    view_id: i32,
}
impl<'a> ViewLayerInputs<'a> {
    fn new(
        view: &dyn Graphic3dCView,
        extent: [u32; 2],
        camera: &'a Graphic3dCamera,
        clipping: &dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource>,
    ) -> Self {
        // OCCT UpdateCulling precedes Reset(myClipPlanes); capture this actual owned field afterward.
        Self {
            camera,
            clipping: clipping
                .observe()
                .observe_as::<Graphic3dSequenceOfHClipPlane>(),
            rendering: view.rendering_params(),
            extent,
            view_id: view.identification(),
        }
    }
    fn layer(&self, layer: &Graphic3dLayer) -> Result<RenderLayer, RenderError> {
        layer_pass(
            layer,
            self.camera,
            self.view_id,
            self.extent,
            &self.rendering,
            &self.clipping,
        )
    }
}
fn view_layer_passes(
    view: &dyn Graphic3dCView,
    extent: [u32; 2],
    camera: &Graphic3dCamera,
    clipping: &dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource>,
) -> Result<(Vec<RenderLayer>, Graphic3dRenderingParams), RenderError> {
    let mut layers = view.layers();
    // OpenGl_FilteredIndexedLayerIterator renders normal layers before immediate ones.
    layers.sort_by_key(|layer| layer.is_immediate());
    update_view_culling(view, &layers, extent)?;
    let inputs = ViewLayerInputs::new(view, extent, camera, clipping);
    let passes = layers
        .iter()
        .filter(|layer| layer_selected(view, layer.layer_id()))
        .map(|layer| inputs.layer(layer))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((passes, inputs.rendering))
}
fn view_clear_color(view: &dyn Graphic3dCView) -> [f32; 4] {
    let color = view.cview_base().fields.read().bg_color;
    [
        color.rgb().red() as f32,
        color.rgb().green() as f32,
        color.rgb().blue() as f32,
        color.alpha(),
    ]
}

fn update_view_culling(
    view: &dyn Graphic3dCView,
    layers: &[Handle<Graphic3dLayer>],
    extent: [u32; 2],
) -> Result<(), RenderError> {
    let params = view.rendering_params();
    let selector = view_culling_selector(view, extent);
    for layer in layers {
        layer
            .update_culling(
                view.identification(),
                &selector,
                params.ray_tracing.frustum_culling_state,
            )
            .map_err(|error| rejected(error, "a valid native layer culling traversal"))?;
    }
    Ok(())
}
fn view_culling_selector(view: &dyn Graphic3dCView, extent: [u32; 2]) -> Graphic3dCullingTool {
    let mut selector = Graphic3dCullingTool::default();
    selector.set_view_volume(
        &crate::view_render_camera::owned_render_camera(view),
        &rustycad_tkernel::ncollection::Mat4::identity(),
    );
    selector.set_viewport_size(
        extent[0] as i32,
        extent[1] as i32,
        f64::from(view.rendering_params().resolution_ratio()),
    );
    selector.cache_clip_pts_projections();
    selector
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
    clipping: &Handle<Graphic3dSequenceOfHClipPlane>,
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
                clipping,
            )?);
        }
    }
    let offset = settings.polygon_offset();
    Ok(RenderLayer::new(
        draws,
        settings.to_enable_depth_test(),
        settings.to_enable_depth_write(),
        settings.to_clear_depth(),
    )
    .with_polygon_offset(crate::RenderPolygonOffset::new(
        offset.mode.0,
        offset.factor,
        offset.units,
    )?))
}
fn structure_draws(
    structure: &Handle<dyn Graphic3dCStructure>,
    camera: &Graphic3dCamera,
    view_id: i32,
    extent: [u32; 2],
    rendering: &Graphic3dRenderingParams,
    clipping: &Handle<Graphic3dSequenceOfHClipPlane>,
) -> Result<Vec<PrimitiveDraw>, RenderError> {
    if structure.is_null() {
        return Ok(vec![]);
    }
    let retained = structure.cast::<RenderStructure>();
    retained
        .get()
        .ok_or_else(|| rejected(structure.object_address(), "a Martensite driver structure"))?
        .draws(camera, view_id, extent, rendering, clipping)
}
fn validate_raster_params(view: &dyn Graphic3dCView) -> Result<(), RenderError> {
    let params = view.rendering_params();
    if params.method != Graphic3dRenderingMode::Rasterization
        || params.render_resolution_scale != 1.0
    {
        return Err(rejected(
            (
                params.method,
                params.nb_msaa_samples,
                params.render_resolution_scale,
                params.transparency_method,
            ),
            "rasterization at scale 1",
        ));
    }
    Ok(())
}
fn validate_view(view: &dyn Graphic3dCView) -> Result<(), RenderError> {
    validate_raster_params(view)?;
    if crate::view_render_camera::owned_render_camera(view).is_stereo() || view.computed_mode() {
        return Err(rejected(
            "stereo or computed rendering",
            "ordinary camera rendering in this slice",
        ));
    }
    if view.cview_base().fields.read().background_type != Graphic3dTypeOfBackground::None {
        return Err(rejected(
            view.cview_base().fields.read().background_type,
            "a solid background in this renderer slice",
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

fn apply_view_culling(layers: &mut [RenderLayer], mode: Graphic3dTypeOfBackfacingModel) {
    if mode == Graphic3dTypeOfBackfacingModel::Auto {
        return;
    }
    for draw in layers.iter_mut().flat_map(|layer| &mut layer.draws) {
        draw.face_culling = crate::face_culling::resolve_culling(mode, false);
    }
}
