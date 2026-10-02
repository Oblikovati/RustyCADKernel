use super::BRepMeshDelaun;
use super::triangle_build::loop_edges;
use crate::brep_mesh::BRepMeshSelectorOfDataStructureOfDelaun;
use rustycad_tkernel::{ncollection::DataMap, standard::Failure};
impl BRepMeshDelaun {
    /// Delete triangles attached to auxiliary nodes, then dangling links/nodes; `triangulator.remove_aux_elements()?`.
    pub fn remove_aux_elements(&mut self) -> Result<(), Failure> {
        let selector = BRepMeshSelectorOfDataStructureOfDelaun::new(&self.mesh);
        for index in self.auxiliary.iter() {
            selector.neighbours_of_node(*index)?;
        }
        let mut boundary = loop_edges();
        for index in selector.elements().iter() {
            self.delete_triangle(*index, &mut boundary)?;
        }
        self.remove_auxiliary_boundary(&boundary)?;
        for index in self.auxiliary.iter() {
            self.mesh_ref()?.remove_node(*index, false)?;
        }
        Ok(())
    }
    pub(super) fn delete_triangle(
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
    pub(super) fn remove_auxiliary_boundary(
        &self,
        boundary: &DataMap<i32, bool>,
    ) -> Result<(), Failure> {
        for (index, _) in boundary.items() {
            if self.mesh_ref()?.elements_connected_to(*index)?.is_empty() {
                self.mesh_ref()?.remove_link(*index, false)?;
            }
        }
        Ok(())
    }
}
