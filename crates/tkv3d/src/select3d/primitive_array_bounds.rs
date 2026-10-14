//! The detection members of `Select3D_SensitivePrimitiveArray`: `Matches`, the element boxes
//! and the overlap and inclusion tests of the points, triangles, nodes and edges.
use super::Select3DBndBox3d;
use super::primitive_array::{
    PrimArrayState, Select3DSensitivePrimitiveArray, point_index, read_position, read_position_2d,
    tri_indices,
};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::Graphic3dTypeOfPrimitiveArray;

impl Select3DSensitivePrimitiveArray {
    /// `applyTransformation` (protected): the box of the eight corners at the initial location.
    /// Example: `array.apply_transformation()`.
    pub fn apply_transformation(&self) -> Select3DBndBox3d {
        let state = self.read();
        if state.init_location.is_identity() {
            return state.bnd_box;
        }
        let (min, max, trsf) = (
            *state.bnd_box.corner_min(),
            *state.bnd_box.corner_max(),
            state.init_location.transformation(),
        );
        let mut bounds = Select3DBndBox3d::default();
        for (x, y, z) in (0..8).map(|i| (i / 4, (i / 2) % 2, i % 2)) {
            let corner = Point3::new(
                if x == 0 { min.x } else { max.x },
                if y == 0 { min.y } else { max.y },
                if z == 0 { min.z } else { max.z },
            );
            let p = corner.transformed(trsf);
            bounds.add(&Vec3::new(p.x(), p.y(), p.z()));
        }
        bounds
    }
    /// The box of a patch of points or triangles.
    pub(super) fn patch_box(
        state: &PrimArrayState,
        elem_idx: i32,
        patch_size: i32,
    ) -> Select3DBndBox3d {
        let mut bounds = Select3DBndBox3d::default();
        let as_f64 = |v: Vec3<f32>| Vec3::new(f64::from(v.x), f64::from(v.y), f64::from(v.z));
        for iter in 0..patch_size {
            match state.prim_type {
                Graphic3dTypeOfPrimitiveArray::Points => {
                    let index = point_index(&state.indices, elem_idx + iter);
                    let p = if state.is_3d {
                        read_position(state, index).unwrap_or_default()
                    } else {
                        let q = read_position_2d(state, index).unwrap_or_default();
                        Vec3::new(q.x, q.y, 0.0)
                    };
                    bounds.add(&as_f64(p));
                }
                Graphic3dTypeOfPrimitiveArray::Triangles => {
                    let nodes = tri_indices(&state.indices, (elem_idx + iter) * 3);
                    let (min, max) = triangle_min_max(state, &nodes);
                    bounds.add(&as_f64(min));
                    bounds.add(&as_f64(max));
                }
                _ => return bounds,
            }
        }
        bounds
    }
}

/// The single-precision component-wise corners of a triangle (z = 0 in 2D).
fn triangle_min_max(state: &PrimArrayState, nodes: &[i32; 3]) -> (Vec3<f32>, Vec3<f32>) {
    if state.is_3d {
        let [a, b, c] = nodes.map(|n| read_position(state, n).unwrap_or_default());
        return (a.cwise_min(&b).cwise_min(&c), a.cwise_max(&b).cwise_max(&c));
    }
    let [a, b, c] = nodes.map(|n| read_position_2d(state, n).unwrap_or_default());
    let (min, max) = (a.cwise_min(&b).cwise_min(&c), a.cwise_max(&b).cwise_max(&c));
    (Vec3::new(min.x, min.y, 0.0), Vec3::new(max.x, max.y, 0.0))
}
