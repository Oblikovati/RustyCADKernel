//! Infinite section geometry from OpenGl_CappingPlaneResource at OCCT 3d097a0.
use crate::RenderVertex;
use rustycad_tkernel::ncollection::{Mat4, Vec4};

pub(crate) fn cap_orientation(equation: [f64; 4]) -> Mat4<f32> {
    let [x, y, z, w] = equation;
    let normal = [x as f32, y as f32, z as f32];
    let left = plane_left(x, z, normal);
    let forward = [
        left[2] * normal[1],
        left[0] * normal[2] - left[2] * normal[0],
        -left[0] * normal[1],
    ];
    let mut orientation = Mat4::identity();
    orientation.set_column(0, Vec4::new(left[0], left[1], left[2], 0.));
    orientation.set_column(1, Vec4::new(normal[0], normal[1], normal[2], 0.));
    orientation.set_column(2, Vec4::new(forward[0], forward[1], forward[2], 0.));
    orientation.set_column(
        3,
        Vec4::new((-x * w) as f32, (-y * w) as f32, (-z * w) as f32, 1.),
    );
    orientation
}
fn plane_left(x: f64, z: f64, normal: [f32; 3]) -> [f32; 3] {
    // Native narrows the squared double length before the float square root.
    let length = ((x * x + z * z) as f32).sqrt();
    if length < f32::MIN_POSITIVE {
        return [1., 0., 0.];
    }
    [normal[2] / length, 0., -normal[0] / length]
}
pub(crate) fn cap_vertices(color: [f32; 4]) -> Vec<RenderVertex> {
    let center = [0., 0., 0., 1.];
    [
        center,
        [0., 0., 1., 0.],
        [1., 0., 0., 0.],
        center,
        [-1., 0., 0., 0.],
        [0., 0., 1., 0.],
        center,
        [0., 0., -1., 0.],
        [-1., 0., 0., 0.],
        center,
        [1., 0., 0., 0.],
        [0., 0., -1., 0.],
    ]
    .map(|position| RenderVertex::homogeneous(position, color))
    .to_vec()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn homogeneous_plane_vertices_lie_on_translated_cardinal_and_oblique_planes() {
        for equation in [
            [1., 0., 0., -2.],
            [0., 1., 0., 3.],
            [0., -1., 0., -4.],
            [0., 0., 1., 2.],
            [0.6, 0., 0.8, -5.],
        ] {
            let matrix = cap_orientation(equation);
            let vertices = cap_vertices([1.; 4]);
            assert_eq!(vertices.len(), 12);
            assert_eq!(vertices.iter().filter(|v| v.weight == 0.).count(), 8);
            for v in vertices {
                let p = matrix * Vec4::new(v.position[0], v.position[1], v.position[2], v.weight);
                let distance: f64 = p
                    .data()
                    .iter()
                    .zip(equation)
                    .map(|(p, e)| f64::from(*p) * e)
                    .sum();
                assert!(distance.abs() < 1e-6, "{equation:?}: {distance}");
            }
        }
    }
}
