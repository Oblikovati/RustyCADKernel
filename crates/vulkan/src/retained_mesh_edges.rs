//! Snapshot native Graphic3d solid edges independently of image and lighting revisions.
use crate::{
    LineStyle, PrimitiveDraw, PrimitiveTopology, RenderError, RenderMeshEdgePipeline,
    RenderMeshEdges,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
pub(crate) fn driver_mesh_edge_pipeline(
    structure: &Handle<dyn Graphic3dStructureSource>,
) -> RenderMeshEdgePipeline {
    structure
        .structure_manager()
        .graphic_driver()
        .cast::<crate::Graphic3dRenderDriver>()
        .get()
        .map_or_else(RenderMeshEdgePipeline::default, |driver| {
            driver.mesh_edge_pipeline
        })
}
pub(crate) fn apply_retained_mesh_edges(
    draws: &mut Vec<PrimitiveDraw>,
    aspect: &dyn Graphic3dAspectsSource,
    pipeline: RenderMeshEdgePipeline,
) -> Result<(), RenderError> {
    if !retained_mesh_requested(draws, aspect) {
        return Ok(());
    }
    let rgba = mesh_edge_rgba(aspect);
    apply_edge_alpha_cutoff(draws, rgba[3]);
    if aspect.line_type() != AspectTypeOfLine::Solid
        || pipeline == RenderMeshEdgePipeline::PolygonCompatibility
    {
        return append_native_polygon_edges(draws, aspect);
    }
    snapshot_native_solid_edges(draws, aspect, rgba)
}
fn snapshot_native_solid_edges(
    draws: &mut [PrimitiveDraw],
    aspect: &dyn Graphic3dAspectsSource,
    rgba: [f32; 4],
) -> Result<(), RenderError> {
    let edges = RenderMeshEdges::new(rgba, aspect.line_width(), aspect.to_skip_first_edge())?;
    for draw in draws {
        draw.mesh_edges = Some(edges);
    }
    Ok(())
}

fn retained_mesh_requested(draws: &[PrimitiveDraw], aspect: &dyn Graphic3dAspectsSource) -> bool {
    aspect.to_draw_edges()
        && draws
            .iter()
            .any(|draw| draw.topology() == PrimitiveTopology::Triangles)
}
fn mesh_edge_rgba(aspect: &dyn Graphic3dAspectsSource) -> [f32; 4] {
    let color = aspect.edge_color_rgba();
    [
        color.rgb().red() as f32,
        color.rgb().green() as f32,
        color.rgb().blue() as f32,
        color.alpha(),
    ]
}
fn apply_edge_alpha_cutoff(draws: &mut [PrimitiveDraw], alpha: f32) {
    if alpha >= 1. {
        return;
    }
    // OCCT3d097a0 SetShadingMaterial applies this to solid faces before either edge path.
    for draw in draws {
        draw.surface_cutoff = Some(0.285);
    }
}
fn append_native_polygon_edges(
    draws: &mut Vec<PrimitiveDraw>,
    aspect: &dyn Graphic3dAspectsSource,
) -> Result<(), RenderError> {
    let style = LineStyle::new(
        aspect.line_width(),
        aspect.line_pattern(),
        aspect.line_stipple_factor(),
    )?;
    let rgba = mesh_edge_rgba(aspect);
    let edges: Vec<PrimitiveDraw> = draws
        .iter()
        .map(|face| {
            let mut edge = polygon_edge_draw(face, style, rgba);
            edge.edge_background_fallback = rgba[3] < 0.1;
            edge
        })
        .collect();
    draws.extend(edges);
    Ok(())
}
fn polygon_edge_draw(face: &PrimitiveDraw, style: LineStyle, color: [f32; 4]) -> PrimitiveDraw {
    let mut edge = face.clone();
    for vertex in &mut edge.vertices {
        vertex.color = color;
    }
    edge.mesh_edges = None;
    edge.lighting = None;
    edge.surface_cutoff = None;
    edge.surface_forced_blend = false;
    edge.surface_texture_binding_only = true;
    edge.polygon_edge_style = Some(style);
    edge
}
pub(crate) fn resolve_edge_background(draws: &mut [PrimitiveDraw], clear: [f32; 4]) {
    for draw in draws
        .iter_mut()
        .filter(|draw| draw.edge_background_fallback)
    {
        for vertex in &mut draw.vertices {
            vertex.color = clear;
        }
        draw.edge_background_fallback = false;
    }
}
pub(crate) fn apply_graphic3d_raster_widths(
    draws: &mut [PrimitiveDraw],
    params: &Graphic3dRenderingParams,
) -> Result<(), RenderError> {
    let scale = native_line_width_scale(params.resolution_ratio());
    for draw in draws {
        for line in [&mut draw.line_style, &mut draw.polygon_edge_style]
            .into_iter()
            .flatten()
        {
            line.width *= scale;
        }
        if let Some(edges) = draw.mesh_edges {
            draw.mesh_edges = Some(
                RenderMeshEdges::new(edges.color, edges.width * scale, edges.skip_first)?
                    .with_feather_width(params.line_feather * scale)?,
            );
        }
    }
    Ok(())
}
fn native_line_width_scale(ratio: f32) -> f32 {
    // OpenGl_Context::SetResolutionRatio rounds the scale, not each aspect width.
    (ratio + 0.5).floor().max(1.)
}
#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::quantity::ColorRgba;
    #[test]
    fn native_polygon_edges_follow_all_face_bounds_and_keep_original_winding() {
        let aspect = Graphic3dAspects::default();
        aspect.set_draw_edges(true);
        aspect.set_line_type(AspectTypeOfLine::Dash);
        let first = bound_draw(1.);
        let second = bound_draw(2.);
        let mut draws = vec![first.clone(), second.clone()];
        apply_retained_mesh_edges(&mut draws, &aspect, RenderMeshEdgePipeline::default()).unwrap();
        assert_eq!(
            draws
                .iter()
                .map(|draw| draw.vertices[0].position[0])
                .collect::<Vec<_>>(),
            [1., 2., 1., 2.]
        );
        assert!(
            draws[..2]
                .iter()
                .all(|draw| draw.polygon_edge_style.is_none())
        );
        assert!(
            draws[2..]
                .iter()
                .all(|draw| draw.surface_texture_binding_only)
        );
        assert_eq!(draws[2].indices, first.indices);
        assert_eq!(draws[3].indices, second.indices);
    }
    fn bound_draw(position: f32) -> PrimitiveDraw {
        let vertices = vec![crate::RenderVertex::new([position, 0., 0.], [1.; 4]); 3];
        PrimitiveDraw::new(vertices, vec![2, 0, 1], crate::identity_clip()).unwrap()
    }
    #[test]
    fn native_small_edge_alpha_uses_view_background_without_changing_face_colors() {
        let aspect = Graphic3dAspects::default();
        aspect.set_draw_edges(true);
        aspect.set_edge_color_rgba(ColorRgba::new(0., 1., 1., 0.05).unwrap());
        let mut draws = vec![bound_draw(1.)];
        apply_retained_mesh_edges(
            &mut draws,
            &aspect,
            RenderMeshEdgePipeline::PolygonCompatibility,
        )
        .unwrap();
        resolve_edge_background(&mut draws, [0., 0., 1., 1.]);
        assert_eq!(draws[0].vertices[0].color, [1.; 4]);
        assert_eq!(draws[1].vertices[0].color, [0., 0., 1., 1.]);
        assert!(!draws[1].edge_background_fallback);
        assert_eq!(draws[0].surface_cutoff, Some(0.285));
    }
    #[test]
    fn native_resolution_rounding_scales_width_and_feather_independently() {
        assert_eq!(
            [0.5, 1., 1.49, 1.5, 2.49].map(native_line_width_scale),
            [1., 1., 1., 2., 2.]
        );
        let mut draw = bound_draw(1.);
        draw.mesh_edges = Some(RenderMeshEdges::new([1.; 4], 3., true).unwrap());
        draw.polygon_edge_style = Some(LineStyle::new(3., 0xffff, 1).unwrap());
        let mut params = Graphic3dRenderingParams::new().unwrap();
        params.resolution = 144;
        params.line_feather = 2.;
        let mut draws = vec![draw];
        apply_graphic3d_raster_widths(&mut draws, &params).unwrap();
        assert_eq!(draws[0].mesh_edges.unwrap().width, 6.);
        assert_eq!(draws[0].mesh_edges.unwrap().feather, 4.);
        assert_eq!(draws[0].polygon_edge_style.unwrap().width, 6.);
    }
    #[test]
    fn retained_edge_color_revision_keeps_surface_alpha_classification() {
        let aspect = Graphic3dAspects::default();
        aspect.set_draw_edges(true);
        aspect.set_edge_color_rgba(ColorRgba::new(0., 1., 1., 0.25).unwrap());
        let mut draws = vec![PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap()];
        apply_retained_mesh_edges(&mut draws, &aspect, RenderMeshEdgePipeline::default()).unwrap();
        assert_eq!(draws[0].surface_cutoff, Some(0.285));
        assert!(!draws[0].surface_blend);
        assert_eq!(draws[0].mesh_edges.unwrap().color, [0., 1., 1., 0.25]);
        aspect.set_line_type(AspectTypeOfLine::Dash);
        assert!(
            apply_retained_mesh_edges(&mut draws, &aspect, RenderMeshEdgePipeline::default())
                .is_ok()
        );
        assert_eq!(draws.len(), 2);
        assert!(draws[1].polygon_edge_style.is_some());
        aspect.set_draw_edges(false);
        assert!(
            apply_retained_mesh_edges(&mut draws, &aspect, RenderMeshEdgePipeline::default())
                .is_ok()
        );
    }
}
