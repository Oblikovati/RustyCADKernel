//! Mesh shaders share sampled-surface layouts and insert a genuine geometry stage.
use super::pipeline_variants::PipelineVariant;
macro_rules! edge_binary {
    ($name:literal, $stage:literal) => {
        include_bytes!(concat!(
            "../../shaders/surface.edges",
            $name,
            ".",
            $stage,
            ".spv"
        ))
        .as_slice()
    };
}
pub(super) fn edge_shader_bytes(kind: PipelineVariant) -> [&'static [u8]; 3] {
    let fragment = match (kind.lighting, kind.clipped, kind.surface_texture) {
        (false, false, false) => edge_binary!("", "frag"),
        (false, false, true) => edge_binary!(".image", "frag"),
        (false, true, false) => edge_binary!(".clipped", "frag"),
        (false, true, true) => edge_binary!(".clipped.image", "frag"),
        (true, false, false) => edge_binary!(".lit", "frag"),
        (true, false, true) => edge_binary!(".lit.image", "frag"),
        (true, true, false) => edge_binary!(".lit.clipped", "frag"),
        (true, true, true) => edge_binary!(".lit.clipped.image", "frag"),
    };
    [
        super::surface_texture_pipeline::surface_shaders(kind)[0],
        fragment,
        edge_geometry_bytes(kind),
    ]
}
fn edge_geometry_bytes(kind: PipelineVariant) -> &'static [u8] {
    match (kind.lighting, kind.clipped) {
        (false, false) => edge_binary!("", "geom"),
        (false, true) => edge_binary!(".clipped", "geom"),
        (true, false) => edge_binary!(".lit", "geom"),
        (true, true) => edge_binary!(".lit.clipped", "geom"),
    }
}
