//! Push-constant variants keep native line parameters separate from sampled surfaces.
use crate::PrimitiveDraw;
use ash::vk;
pub(super) fn encode_transform(draw: &PrimitiveDraw, extent: vk::Extent2D) -> Vec<u8> {
    let mut bytes: Vec<u8> = draw
        .clip_transform
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect();
    append_line_constants(&mut bytes, draw, extent);
    append_sprite_constants(&mut bytes, draw);
    append_surface_constants(&mut bytes, draw);
    crate::mesh_edges::append_mesh_constants(
        &mut bytes,
        draw.mesh_edges,
        [extent.width, extent.height],
    );
    bytes
}
fn append_line_constants(bytes: &mut Vec<u8>, draw: &PrimitiveDraw, extent: vk::Extent2D) {
    let Some(style) = draw
        .line_style
        .or(draw.polygon_edge_style)
        .filter(|_| !uses_surface_constants(draw))
    else {
        return;
    };
    bytes.extend(
        [extent.width as f32, extent.height as f32]
            .iter()
            .flat_map(|value| value.to_ne_bytes()),
    );
    bytes.extend(u32::from(style.pattern).to_ne_bytes());
    bytes.extend(u32::from(style.factor).to_ne_bytes());
}
fn append_sprite_constants(bytes: &mut Vec<u8>, draw: &PrimitiveDraw) {
    append_point_constants(bytes, draw);
    append_glyph_constants(bytes, draw);
}
fn append_point_constants(bytes: &mut Vec<u8>, draw: &PrimitiveDraw) {
    if let Some(style) = &draw.point_style {
        bytes.extend(style.size.to_ne_bytes());
        bytes.extend(
            style
                .image
                .extent
                .into_iter()
                .flat_map(|value| value.to_ne_bytes()),
        );
        bytes.extend((style.color_policy as u32).to_ne_bytes());
        if draw.fixed_function_lighting {
            bytes.extend(style.fixed_material_cutoff.unwrap_or(-1.).to_ne_bytes());
        }
    }
}
fn append_glyph_constants(bytes: &mut Vec<u8>, draw: &PrimitiveDraw) {
    if let Some(style) = &draw.textured_style {
        bytes.extend(
            style
                .image
                .extent
                .iter()
                .flat_map(|value| value.to_ne_bytes()),
        );
        bytes.extend(style.alpha_cutoff.to_ne_bytes());
        bytes.extend(0u32.to_ne_bytes());
    }
}
fn append_surface_constants(bytes: &mut Vec<u8>, draw: &PrimitiveDraw) {
    if !uses_surface_constants(draw) {
        return;
    }
    bytes.extend(draw.surface_cutoff.unwrap_or(0.).to_ne_bytes());
    bytes.extend(surface_constant_flags(draw).to_ne_bytes());
    bytes.extend(
        crate::fixed_surface_environment::fixed_uniform_alpha(draw)
            .unwrap_or(0.)
            .to_ne_bytes(),
    );
    bytes.extend([0_u8; 4]);
}
fn surface_constant_flags(draw: &PrimitiveDraw) -> u32 {
    draw.surface_texture.as_ref().map_or(0, |style| {
        1 + 2 * u32::from(style.vertex_override) + 4 * u32::from(style.back_vertex)
    }) + draw
        .surface_texture_unavailable
        .as_ref()
        .map_or(0, |style| {
            16 + 2 * u32::from(style.vertex_override) + 4 * u32::from(style.back_vertex)
        })
        + 8 * u32::from(draw.surface_cutoff.is_some())
        + 4096 * u32::from(draw.unlit_back_color.is_some())
        + crate::fixed_surface_environment::fixed_surface_environment_flags(draw)
}
fn uses_surface_constants(draw: &PrimitiveDraw) -> bool {
    !draw.surface_texture_binding_only
        && (draw.surface_texture.is_some()
            || draw.surface_cutoff.is_some()
            || draw.surface_texture_unavailable.is_some()
            || draw.mesh_edges.is_some()
            || draw.unlit_back_color.is_some())
}
