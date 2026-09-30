// Rust translation of the array views of FoundationClasses/TKMath/Poly/Poly_Triangulation.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Map*Array` return one-based copies instead of views that must not outlive the mesh;
//! `Internal*` and the deprecated `Change*` members lend their array to a closure under the
//! write lock, because a reference cannot outlive the lock.
use super::{ArrayOfNodes, ArrayOfUvNodes, Triangle, Triangulation};
use crate::gp::{Point2, Point3};
use rustycad_tkernel::{
    ncollection::{Array1, Vec3},
    standard::Failure,
};
impl Triangulation {
    /// One-based copy of the nodes, or `None` without nodes; `triangulation.map_node_array()`.
    pub fn map_node_array(&self) -> Option<Array1<'static, Point3>> {
        let fields = self.fields.read();
        one_based(fields.nodes.len(), |offset| fields.nodes.value(offset).ok())
    }
    /// One-based copy of the UV nodes, or `None`; `triangulation.map_uv_node_array()`.
    pub fn map_uv_node_array(&self) -> Option<Array1<'static, Point2>> {
        let fields = self.fields.read();
        one_based(fields.uv_nodes.len(), |offset| {
            fields.uv_nodes.value(offset).ok()
        })
    }
    /// One-based copy of the triangles, or `None`; `triangulation.map_triangle_array()`.
    pub fn map_triangle_array(&self) -> Option<Array1<'static, Triangle>> {
        let fields = self.fields.read();
        let values = fields.triangles.as_slice();
        one_based(values.len(), |offset| values.get(offset).copied())
    }
    /// Normals flattened to `x, y, z` floats at `1..=3*NbNodes`, or `None`; `triangulation.map_normal_array()`.
    pub fn map_normal_array(&self) -> Option<Array1<'static, f32>> {
        let fields = self.fields.read();
        let values: Vec<f32> = fields
            .normals
            .iter()
            .flat_map(|n| [n.x(), n.y(), n.z()])
            .collect();
        one_based(values.len(), |offset| values.get(offset).copied())
    }
    /// Deprecated `SetNormals`: requires exactly `3 * NbNodes` values, else `Standard_DomainError`;
    /// `triangulation.set_normals(Some(&flat))?`.
    pub fn set_normals(&self, normals: Option<&Array1<'_, f32>>) -> Result<(), Failure> {
        let expected = 3 * self.nb_nodes();
        let Some(normals) = normals.filter(|n| n.len() as i32 == expected) else {
            let found = normals.map_or(-1, |n| n.len() as i32);
            return Err(Failure::with_type(
                "Standard_DomainError",
                &format!(
                    "Poly_Triangulation::SetNormals : wrong length {found}; expected {expected}"
                ),
                "",
            ));
        };
        self.add_normals()?;
        for (node, xyz) in normals.as_slice().as_chunks::<3>().0.iter().enumerate() {
            self.set_normal_vec3(node as i32 + 1, Vec3::new(xyz[0], xyz[1], xyz[2]))?;
        }
        Ok(())
    }
    /// Deprecated `Triangles()`: a copy of the triangle array with its bounds; `triangulation.triangles()`.
    pub fn triangles(&self) -> Array1<'static, Triangle> {
        self.fields.read().triangles.clone()
    }
    /// Deprecated `ChangeTriangles()` / `InternalTriangles()`; `triangulation.change_triangles(|t| t.len())`.
    pub fn change_triangles<R>(&self, edit: impl FnOnce(&mut Array1<'static, Triangle>) -> R) -> R {
        edit(&mut self.fields.write().triangles)
    }
    /// Deprecated `ChangeTriangle(i)`; `triangulation.change_triangle(1, |t| t.set(1, 2, 3))?`.
    pub fn change_triangle<R>(
        &self,
        index: i32,
        edit: impl FnOnce(&mut Triangle) -> R,
    ) -> Result<R, Failure> {
        let mut fields = self.fields.write();
        let triangle = fields
            .triangles
            .change_value(index)
            .map_err(|e| e.into_failure(&format!("Poly_Triangulation::ChangeTriangle({index})")))?;
        Ok(edit(triangle))
    }
    /// `InternalNodes()`; `triangulation.change_internal_nodes(|nodes| nodes.len())`.
    pub fn change_internal_nodes<R>(
        &self,
        edit: impl FnOnce(&mut ArrayOfNodes<'static>) -> R,
    ) -> R {
        edit(&mut self.fields.write().nodes)
    }
    /// `InternalUVNodes()`; `triangulation.change_internal_uv_nodes(|nodes| nodes.len())`.
    pub fn change_internal_uv_nodes<R>(
        &self,
        edit: impl FnOnce(&mut ArrayOfUvNodes<'static>) -> R,
    ) -> R {
        edit(&mut self.fields.write().uv_nodes)
    }
    /// `InternalNormals()`, zero-based; `triangulation.change_internal_normals(|normals| normals.len())`.
    pub fn change_internal_normals<R>(
        &self,
        edit: impl FnOnce(&mut Array1<'static, Vec3<f32>>) -> R,
    ) -> R {
        edit(&mut self.fields.write().normals)
    }
}
fn one_based<T: Default>(
    length: usize,
    value: impl Fn(usize) -> Option<T>,
) -> Option<Array1<'static, T>> {
    if length == 0 {
        return None;
    }
    let mut array = Array1::with_bounds(1, length as i32).ok()?;
    for (offset, slot) in array.iter_mut().enumerate() {
        *slot = value(offset)?;
    }
    Some(array)
}
