//! Dedicated fixed-lighting bytecode keeps ordinary Phong shader binaries unchanged.
use super::pipeline_variants::PipelineVariant;
macro_rules! fixed_pair {
    ($stem:literal) => {
        [
            include_bytes!(concat!("../../shaders/", $stem, ".vert.spv")).as_slice(),
            include_bytes!(concat!("../../shaders/", $stem, ".frag.spv")).as_slice(),
        ]
    };
}
pub(super) fn fixed_lighting_pair(kind: PipelineVariant) -> [&'static [u8]; 2] {
    if kind.surface_texture || kind.surface_alpha || kind.mesh_edges {
        return fixed_surface_pair(kind.clipped, kind.surface_texture);
    }
    match (kind.clipped, kind.point, kind.line) {
        (false, true, _) => fixed_pair!("lighting.fixed.point"),
        (true, true, _) => fixed_pair!("lighting.fixed.point.clipped"),
        (false, false, true) => fixed_pair!("lighting.fixed.line"),
        (true, false, true) => fixed_pair!("lighting.fixed.line.clipped"),
        (false, false, false) => fixed_pair!("lighting.fixed"),
        (true, false, false) => fixed_pair!("lighting.fixed.clipped"),
    }
}
fn fixed_surface_pair(clipped: bool, image: bool) -> [&'static [u8]; 2] {
    match (clipped, image) {
        (false, false) => fixed_pair!("surface.fixed"),
        (false, true) => fixed_pair!("surface.fixed.image"),
        (true, false) => fixed_pair!("surface.fixed.clipped"),
        (true, true) => fixed_pair!("surface.fixed.clipped.image"),
    }
}
pub(super) fn fixed_mesh_triple(kind: PipelineVariant) -> [&'static [u8]; 3] {
    let fragment = match (kind.clipped, kind.surface_texture) {
        (false, false) => include_bytes!("../../shaders/surface.edges.fixed.frag.spv").as_slice(),
        (false, true) => {
            include_bytes!("../../shaders/surface.edges.fixed.image.frag.spv").as_slice()
        }
        (true, false) => {
            include_bytes!("../../shaders/surface.edges.fixed.clipped.frag.spv").as_slice()
        }
        (true, true) => {
            include_bytes!("../../shaders/surface.edges.fixed.clipped.image.frag.spv").as_slice()
        }
    };
    let geometry = if kind.clipped {
        include_bytes!("../../shaders/surface.edges.fixed.clipped.geom.spv").as_slice()
    } else {
        include_bytes!("../../shaders/surface.edges.fixed.geom.spv").as_slice()
    };
    [fixed_lighting_pair(kind)[0], fragment, geometry]
}

pub(super) fn fixed_unlit_line_pair(clipped: bool) -> [&'static [u8]; 2] {
    if clipped {
        return fixed_pair!("line.fixed.clipped");
    }
    fixed_pair!("line.fixed")
}
pub(super) fn fixed_unlit_point_pair(clipped: bool) -> [&'static [u8]; 2] {
    if clipped {
        return fixed_pair!("point.fixed.clipped");
    }
    fixed_pair!("point.fixed")
}
