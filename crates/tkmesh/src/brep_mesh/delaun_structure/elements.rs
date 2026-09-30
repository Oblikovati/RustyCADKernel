use super::*;
impl BRepMeshDataStructureOfDelaun {
    /// Count all appended element slots; `mesh.nb_elements()`.
    pub fn nb_elements(&self) -> i32 {
        self.connectivity.read().elements.length()
    }
    /// Read an element by its one-based public index; `mesh.get_element(1)?`.
    pub fn get_element(&self, index: i32) -> Result<BRepMeshTriangle, Failure> {
        self.connectivity.read().element(index)
    }
    /// Snapshot domain indices in native hash order; `mesh.elements_of_domain()`.
    pub fn elements_of_domain(&self) -> Map<i32> {
        self.connectivity.read().element_domain.clone()
    }
    /// Append before registering adjacent pairs, retaining partial changes on failure; `mesh.add_element(&triangle)?`.
    pub fn add_element(&self, element: &BRepMeshTriangle) -> Result<i32, Failure> {
        let mut state = self.connectivity.write();
        state.elements.append(*element);
        let index = state.elements.length();
        state.element_domain.add(index);
        state.attach_element(index, element)?;
        Ok(index)
    }
    /// Preserve OCCT's zero-based replacement offset; `mesh.substitute_element(1,&triangle)?`.
    pub fn substitute_element(
        &self,
        index: i32,
        element: &BRepMeshTriangle,
    ) -> Result<bool, Failure> {
        let mut state = self.connectivity.write();
        let old = state.element(index)?;
        if old.movability == DegreeOfFreedom::Deleted {
            state.write_native_replacement(index, *element)?;
            return Ok(true);
        }
        state.clean_element(index, &old)?;
        state.write_native_replacement(index, *element)?;
        state.attach_element(index, element)?;
        Ok(true)
    }
    /// Unregister free elements before marking the stored triangle; `mesh.remove_element(1)?`.
    pub fn remove_element(&self, index: i32) -> Result<(), Failure> {
        let mut state = self.connectivity.write();
        let element = state.element(index)?;
        if element.movability == DegreeOfFreedom::Deleted {
            return Ok(());
        }
        state.clean_element(index, &element)?;
        state.mark_element_deleted(index)?;
        state.element_domain.remove(&index);
        Ok(())
    }
    /// Derive nodes solely from the first and third oriented edges; `mesh.element_nodes(&triangle)?`.
    pub fn element_nodes(&self, element: &BRepMeshTriangle) -> Result<[i32; 3], Failure> {
        let first = self.get_link(element.edges[0])?;
        let third = self.get_link(element.edges[2])?;
        let mut nodes = [first.first_node(), first.last_node(), third.last_node()];
        if !element.orientations[0] {
            nodes.swap(0, 1);
        }
        if element.orientations[2] {
            nodes[2] = third.first_node();
        }
        Ok(nodes)
    }
    /// Clear active elements and then remove their unattached free links; `mesh.clear_domain()?`.
    pub fn clear_domain(&self) -> Result<(), Failure> {
        let mut state = self.connectivity.write();
        let mut free_edges = Map::<i32>::default();
        let indices: Vec<i32> = state.element_domain.iter().copied().collect();
        for index in indices {
            let element = state.element(index)?;
            for edge in element.edges {
                free_edges.add(edge);
            }
            state.clean_element(index, &element)?;
            state.mark_element_deleted(index)?;
        }
        state.element_domain.clear(false);
        for edge in free_edges.iter() {
            state.remove_link(*edge, false)?;
        }
        Ok(())
    }
}
impl DelaunConnectivity {
    pub(super) fn element(&self, index: i32) -> Result<BRepMeshTriangle, Failure> {
        self.elements
            .value(index.wrapping_sub(1))
            .copied()
            .map_err(|e| e.into_failure("BRepMesh_DataStructureOfDelaun"))
    }
    fn mark_element_deleted(&mut self, index: i32) -> Result<(), Failure> {
        self.elements
            .change_value(index.wrapping_sub(1))
            .map_err(|e| e.into_failure("BRepMesh_DataStructureOfDelaun"))?
            .movability = DegreeOfFreedom::Deleted;
        Ok(())
    }
    pub(super) fn write_native_replacement(
        &mut self,
        index: i32,
        element: BRepMeshTriangle,
    ) -> Result<(), Failure> {
        // OCCT 3d097a0328e71 indexes myElements(theIndex), unlike GetElement(index-1).
        *self
            .elements
            .change_value(index)
            .map_err(|e| e.into_failure("BRepMesh_DataStructureOfDelaun replacement"))? = element;
        Ok(())
    }
    fn attach_element(&mut self, index: i32, element: &BRepMeshTriangle) -> Result<(), Failure> {
        for edge in element.edges {
            self.links.change_from_index(edge as usize)?.append(index)?;
        }
        Ok(())
    }
    fn clean_element(&mut self, index: i32, element: &BRepMeshTriangle) -> Result<(), Failure> {
        if element.movability != DegreeOfFreedom::Free {
            return Ok(());
        }
        for edge in element.edges {
            remove_element_index(index, self.links.change_from_index(edge as usize)?)?;
        }
        Ok(())
    }
}
fn remove_element_index(index: i32, pair: &mut PairOfIndex) -> Result<(), Failure> {
    for position in 1..=pair.extent() {
        if pair.index(position)? == index {
            pair.remove_index(position)?;
            break;
        }
    }
    Ok(())
}
