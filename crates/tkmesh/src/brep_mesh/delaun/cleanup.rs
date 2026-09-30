use super::BRepMeshDelaun;
use crate::brep_mesh::BRepMeshSelectorOfDataStructureOfDelaun;
use rustycad_tkernel::{
    ncollection::{BaseAllocator, DataMap, DefaultHashPolicy},
    standard::Failure,
};
impl BRepMeshDelaun {
    /// Delete triangles attached to auxiliary nodes, then dangling links/nodes; `triangulator.remove_aux_elements()?`.
    pub fn remove_aux_elements(&mut self) -> Result<(), Failure> {
        let selector = BRepMeshSelectorOfDataStructureOfDelaun::new(&self.mesh);
        for index in self.auxiliary.iter() {
            selector.neighbours_of_node(*index)?;
        }
        let arena = BaseAllocator::incremental(crate::imesh_data::MEMORY_BLOCK_SIZE_HUGE);
        let mut boundary = DataMap::with_allocator(DefaultHashPolicy, 10, arena);
        for index in selector.elements().iter() {
            self.delete_triangle(*index, &mut boundary)?;
        }
        self.remove_auxiliary_boundary(&boundary)?;
        for index in self.auxiliary.iter() {
            self.mesh_ref()?.remove_node(*index, false)?;
        }
        Ok(())
    }
    fn delete_triangle(
        &mut self,
        index: i32,
        boundary: &mut DataMap<i32, bool>,
    ) -> Result<(), Failure> {
        if self.initialized_circles {
            self.circles.delete(index)?;
        }
        let element = self.get_triangle(index)?;
        self.mesh_ref()?.remove_element(index)?;
        for i in 0..3 {
            if !boundary.bind(element.edges[i], element.orientations[i]) {
                boundary.unbind(&element.edges[i]);
                self.mesh_ref()?.remove_link(element.edges[i], false)?;
            }
        }
        Ok(())
    }
    fn remove_auxiliary_boundary(&self, boundary: &DataMap<i32, bool>) -> Result<(), Failure> {
        for (index, _) in boundary.items() {
            if self.mesh_ref()?.elements_connected_to(*index)?.is_empty() {
                self.mesh_ref()?.remove_link(*index, false)?;
            }
        }
        Ok(())
    }
}
