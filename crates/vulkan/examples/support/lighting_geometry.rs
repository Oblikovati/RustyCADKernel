//! Shared clip-space quad geometry keeps lighting acceptance fixtures consistent.
use martensite_vulkan::RenderVertex;

/// Build a front/back quad at explicit depth; `quad_geometry(0., false, [1.; 4])`.
pub fn quad_geometry(z: f32, back: bool, color: [f32; 4]) -> (Vec<RenderVertex>, Vec<u32>) {
    let vertices = [[-1., -1., z], [1., -1., z], [1., 1., z], [-1., 1., z]]
        .map(|point| RenderVertex::new(point, color));
    let indices = if back {
        vec![0, 2, 1, 0, 3, 2]
    } else {
        vec![0, 1, 2, 0, 2, 3]
    };
    (vertices.to_vec(), indices)
}
