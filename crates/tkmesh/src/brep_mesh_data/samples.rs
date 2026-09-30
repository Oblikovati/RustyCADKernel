//! Parallel sequences mirror native point, parameter, and optional mesh-index lists.
use super::{ArenaVec, MeshAllocator, arena_vector, mesh_index};
use rustycad_tkernel::{ncollection::BaseAllocator, standard::Failure};
pub(super) struct CurveSamples<P> {
    pub(super) points: ArenaVec<P, BaseAllocator>,
    pub(super) parameters: ArenaVec<f64, BaseAllocator>,
    pub(super) indices: Option<ArenaVec<i32, BaseAllocator>>,
}
impl<P: Copy> CurveSamples<P> {
    pub(super) fn new(allocator: &MeshAllocator, indexed: bool) -> Self {
        Self {
            points: arena_vector(allocator),
            parameters: arena_vector(allocator),
            indices: indexed.then(|| arena_vector(allocator)),
        }
    }
    pub(super) fn insert(
        &mut self,
        position: i32,
        point: P,
        parameter: f64,
    ) -> Result<(), Failure> {
        let index = mesh_index(position, self.points.len() + 1, "InsertPoint")?;
        self.points.insert(index, point);
        self.parameters.insert(index, parameter);
        if let Some(indices) = &mut self.indices {
            indices.insert(index, 0);
        }
        Ok(())
    }
    pub(super) fn append(&mut self, point: P, parameter: f64) {
        self.points.push(point);
        self.parameters.push(parameter);
        if let Some(indices) = &mut self.indices {
            indices.push(0);
        }
    }
    pub(super) fn remove(&mut self, index: i32) -> Result<(), Failure> {
        let index = mesh_index(index, self.points.len(), "RemovePoint")?;
        self.points.remove(index);
        self.parameters.remove(index);
        if let Some(indices) = &mut self.indices {
            indices.remove(index);
        }
        Ok(())
    }
    pub(super) fn clear(&mut self, keep: bool) {
        if keep && self.parameters.len() <= 2 {
            return;
        }
        retain_endpoints(&mut self.points, keep);
        retain_endpoints(&mut self.parameters, keep);
        if let Some(indices) = &mut self.indices {
            retain_endpoints(indices, keep);
        }
    }
}
fn retain_endpoints<T>(sequence: &mut ArenaVec<T, BaseAllocator>, keep: bool) {
    if !keep {
        sequence.clear();
        return;
    }
    sequence.drain(1..sequence.len() - 1);
}
