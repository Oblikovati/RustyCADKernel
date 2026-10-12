//! Apply explicit native lighting policy at the actual live view extraction boundary.
use crate::{PrimitiveDraw, RenderError, RenderLayer, RenderLightingPipeline};
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkservice::graphic3d::*;

pub(crate) fn uses_fixed_view_lighting(view: &dyn Graphic3dCView) -> bool {
    view.structure_manager()
        .graphic_driver()
        .cast::<crate::Graphic3dRenderDriver>()
        .get()
        .is_some_and(|driver| driver.lighting_pipeline == RenderLightingPipeline::FixedFunction)
}
pub(crate) fn apply_fixed_view_lighting(
    layers: &mut [RenderLayer],
    view: &dyn Graphic3dCView,
) -> Result<(), RenderError> {
    let camera = crate::view_render_camera::owned_render_camera(view);
    let matrix = Mat4::<f32>::from_other(&camera.orientation_matrix()).data();
    let mut lights = crate::fixed_light_snapshot::snapshot_fixed_lights(&view.lights(), matrix)?;
    if view.rendering_params().shading_model == Graphic3dTypeOfShadingModel::Unlit {
        lights = crate::RenderLightSet {
            ambient: [1.; 4],
            sources: vec![],
        };
    }
    for draw in layers.iter_mut().flat_map(|layer| &mut layer.draws) {
        prepare_fixed_draw(draw, &lights)?;
    }
    Ok(())
}
fn prepare_fixed_draw(
    draw: &mut PrimitiveDraw,
    lights: &crate::RenderLightSet,
) -> Result<(), RenderError> {
    if draw.textured_style.is_some() || draw.capping_pass.is_mask() {
        return Ok(());
    }
    draw.fixed_function_lighting = true;
    if let Some(style) = draw.point_style.as_mut() {
        crate::fixed_marker_environment::prepare_fixed_marker_image(style)?;
    }
    crate::fixed_line_stipple::restore_fixed_line_strip(draw);
    if let Some(lighting) = draw.lighting.as_mut() {
        lighting.prepare_fixed_lighting()?;
        lighting.lights = lights.clone();
        return Ok(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unlit_current_and_array_colors_remain_raw_in_fixed_attachment() {
        let vertex = crate::RenderVertex::new([0.; 3], [0.25, 0.5, 0.75, 0.5]);
        let mut draw =
            PrimitiveDraw::new(vec![vertex; 3], vec![0, 1, 2], crate::identity_clip()).unwrap();
        draw.source_color_array = true;
        prepare_fixed_draw(&mut draw, &crate::RenderLightSet::default()).unwrap();
        assert_eq!(draw.vertices[0].color, vertex.color);
        draw.source_color_array = false;
        prepare_fixed_draw(&mut draw, &crate::RenderLightSet::default()).unwrap();
        assert_eq!(draw.vertices[0].color, vertex.color);
        assert_eq!(draw.vertices[0].color[3], 0.5);
    }
}
