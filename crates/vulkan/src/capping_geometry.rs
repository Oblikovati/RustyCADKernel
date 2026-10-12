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
/// Preserve the actual section primitive's homogeneous UVs; `cap_texture_uv(&vertices)`.
pub(crate) fn cap_texture_uv(vertices: &[RenderVertex]) -> Vec<[f32; 4]> {
    vertices
        .iter()
        .map(|vertex| {
            // OCCT3d097a0 THE_CAPPING_PLN_VERTS uses local X/-Z and the position's infinity weight.
            let t = if vertex.position[2] == 0. {
                0.
            } else {
                -vertex.position[2]
            };
            [vertex.position[0], t, 0., vertex.weight]
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_native_section_buffer_preserves_position_normal_and_projective_uv() {
        let bytes = include_bytes!("../tests/fixtures/texture-texgen-capping/native-cap.vertices");
        let native: Vec<f32> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect();
        let vertices = cap_vertices([1.; 4]);
        let uv = cap_texture_uv(&vertices);
        for (rank, values) in native.as_chunks::<12>().0.iter().enumerate() {
            let v = vertices[rank];
            assert_eq!(
                values[..4],
                [v.position[0], v.position[1], v.position[2], v.weight]
            );
            assert_eq!(values[4..8], [0., -1., 0., 0.]);
            assert_eq!(values[8..], uv[rank]);
        }
    }
    #[test]
    fn actual_native_oblique_orientation_matches_the_section_model_replacement() {
        let bytes =
            include_bytes!("../tests/fixtures/texture-texgen-capping/native-cap.orientation");
        let native: Vec<f32> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect();
        assert_eq!(
            cap_orientation([0.6, 0., -0.8, -2.]).data().as_slice(),
            native
        );
    }
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
