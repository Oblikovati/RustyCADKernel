use super::*;
impl BRepMeshDataStructureOfDelaun {
    /// Count stored links including deletion marks; `mesh.nb_links()`.
    pub fn nb_links(&self) -> i32 {
        self.connectivity.read().links.extent() as i32
    }
    /// Find an undirected, nondeleted link; `mesh.index_of_link(&edge)`.
    pub fn index_of_link(&self, edge: &BRepMeshEdge) -> i32 {
        self.connectivity
            .read()
            .links
            .find_index(&MeshLinkKey(SharedState::new(*edge))) as i32
    }
    /// Read an indexed link; `mesh.get_link(1)?`.
    pub fn get_link(&self, index: i32) -> Result<BRepMeshEdge, Failure> {
        Ok(self
            .connectivity
            .read()
            .links
            .find_key(index as usize)?
            .0
            .snapshot())
    }
    /// Snapshot the domain's native hash iteration order; `mesh.links_of_domain()`.
    pub fn links_of_domain(&self) -> PackedMap<i32> {
        self.connectivity.read().link_domain.clone()
    }
    /// Read the adjacent element pair; `mesh.elements_connected_to(1)?`.
    pub fn elements_connected_to(&self, index: i32) -> Result<PairOfIndex, Failure> {
        Ok(*self
            .connectivity
            .read()
            .links
            .find_from_index(index as usize)?)
    }
    /// Reuse a deleted slot or return the signed orientation of an existing edge; `mesh.add_link(&edge)?`.
    pub fn add_link(&self, edge: &BRepMeshEdge) -> Result<i32, Failure> {
        let mut state = self.connectivity.write();
        let key = MeshLinkKey(SharedState::new(*edge));
        let found = state.links.find_index(&key);
        if found != 0 {
            let same = state
                .links
                .find_key(found)?
                .0
                .snapshot()
                .is_same_orientation(edge);
            return Ok(if same { found as i32 } else { -(found as i32) });
        }
        let index = state.insert_link(key)?;
        state.attach_link(index, edge)?;
        state.link_domain.add(index);
        Ok(index)
    }
    /// Change the movability of a stored link in place, as the native callers do through
    /// `const_cast<BRepMesh_Edge&>(GetLink(i)).SetMovability(...)`: the key is not
    /// rehashed (movability is not part of the link hash) and the element pair and node
    /// link lists are untouched; `mesh.set_link_movability(1, DegreeOfFreedom::Fixed)?`.
    pub fn set_link_movability(
        &self,
        index: i32,
        movability: DegreeOfFreedom,
    ) -> Result<(), Failure> {
        let state = self.connectivity.read();
        state
            .links
            .find_key(index as usize)?
            .0
            .write()
            .set_movability(movability);
        Ok(())
    }
    /// Replace a link while preserving native empty-pair and deleted-slot behavior; `mesh.substitute_link(1,&edge)?`.
    pub fn substitute_link(&self, index: i32, edge: &BRepMeshEdge) -> Result<bool, Failure> {
        self.connectivity.write().substitute_link(index, edge)
    }
}
impl DelaunConnectivity {
    fn substitute_link(&mut self, index: i32, edge: &BRepMeshEdge) -> Result<bool, Failure> {
        let mut old = self.links.find_key(index as usize)?.0.snapshot();
        if old.movability() == DegreeOfFreedom::Deleted {
            self.replace_link(index, *edge, PairOfIndex::default())?;
            return Ok(true);
        }
        if self.links.find_index(&MeshLinkKey(SharedState::new(*edge))) != 0 {
            return Ok(false);
        }
        old.set_movability(DegreeOfFreedom::Deleted);
        self.replace_link(index, old, PairOfIndex::default())?;
        self.clean_link(index, &old)?;
        self.attach_link(index.abs(), edge)?;
        self.replace_link(index, *edge, PairOfIndex::default())?;
        Ok(true)
    }
}
impl BRepMeshDataStructureOfDelaun {
    /// Delete an unattached link, keeping its indexed slot for reuse; `mesh.remove_link(1,false)?`.
    pub fn remove_link(&self, index: i32, force: bool) -> Result<(), Failure> {
        self.connectivity.write().remove_link(index, force)
    }
}
impl DelaunConnectivity {
    fn insert_link(&mut self, key: MeshLinkKey) -> Result<i32, Failure> {
        let Some(&index) = self.deleted_links.first() else {
            return Ok(self.links.add(key, PairOfIndex::default()) as i32);
        };
        self.links
            .substitute(index as usize, key, PairOfIndex::default())?;
        self.deleted_links.remove_first()?;
        Ok(index)
    }
    fn attach_link(&mut self, index: i32, edge: &BRepMeshEdge) -> Result<(), Failure> {
        self.node_links
            .change_find(&edge.first_node())?
            .append(index);
        self.node_links
            .change_find(&edge.last_node())?
            .append(index);
        Ok(())
    }
    fn clean_link(&mut self, index: i32, edge: &BRepMeshEdge) -> Result<(), Failure> {
        self.node_links
            .change_find(&edge.first_node())?
            .remove_value(&index);
        self.node_links
            .change_find(&edge.last_node())?
            .remove_value(&index);
        Ok(())
    }
    pub(super) fn replace_link(
        &mut self,
        index: i32,
        edge: BRepMeshEdge,
        pair: PairOfIndex,
    ) -> Result<(), Failure> {
        self.links
            .substitute(index as usize, MeshLinkKey(SharedState::new(edge)), pair)
    }
    pub(super) fn remove_link(&mut self, index: i32, force: bool) -> Result<(), Failure> {
        let mut edge = self.links.find_key(index as usize)?.0.snapshot();
        if edge.movability() == DegreeOfFreedom::Deleted
            || (!force && edge.movability() != DegreeOfFreedom::Free)
            || self.links.find_from_index(index as usize)?.extent() != 0
        {
            return Ok(());
        }
        self.clean_link(index, &edge)?;
        edge.set_movability(DegreeOfFreedom::Deleted);
        *self.links.find_key(index as usize)?.0.write() = edge;
        self.link_domain.remove(index);
        self.deleted_links.append(index);
        Ok(())
    }
}
