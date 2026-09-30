use super::*;
impl BRepMeshVertexTool {
    /// Insert unless coincident, or force a distinct slot; `tool.add(&vertex,false)?`.
    pub fn add(&self, vertex: &BRepMeshVertex, force: bool) -> Result<i32, Failure> {
        let mut state = self.state.write();
        let found = if force { 0 } else { state.find_index(vertex)? };
        if found != 0 {
            return Ok(found);
        }
        let index = state.selector.add(vertex)?;
        let (minimum, maximum) = state.expanded(*vertex.coord());
        state.filter.add_range(&index, &minimum, &maximum);
        Ok(index)
    }
    /// Find the nearest strict-tolerance candidate in its cell; `tool.find_index(&vertex)?`.
    pub fn find_index(&self, vertex: &BRepMeshVertex) -> Result<i32, Failure> {
        self.state.write().find_index(vertex)
    }
    /// Remove cell copies, then append the deleted slot; `tool.delete_vertex(1)?`.
    pub fn delete_vertex(&self, index: i32) -> Result<(), Failure> {
        let mut state = self.state.write();
        let vertex = state.selector.get_vertex(index)?;
        let (minimum, maximum) = state.expanded(*vertex.coord());
        state.filter.remove_range(&index, &minimum, &maximum);
        state.selector.delete(index)
    }
    /// Remove old cell associations before replacing and registering the vertex; `tool.substitute(1,&vertex)?`.
    pub fn substitute(&self, index: i32, vertex: &BRepMeshVertex) -> Result<(), Failure> {
        let mut state = self.state.write();
        let previous = state.selector.get_vertex(index)?;
        let (minimum, maximum) = state.expanded(*previous.coord());
        state.filter.remove_range(&index, &minimum, &maximum);
        state.selector.set_vertex(index, *vertex)?;
        let (minimum, maximum) = state.expanded(*vertex.coord());
        state.filter.add_range(&index, &minimum, &maximum);
        Ok(())
    }
}
impl VertexToolState {
    fn find_index(&mut self, vertex: &BRepMeshVertex) -> Result<i32, Failure> {
        self.selector.set_point(*vertex.coord());
        let selector = &mut self.selector;
        self.filter
            .try_inspect_with(vertex.coord(), |index| selector.inspect(*index))?;
        Ok(selector.get_coincident_point())
    }
    fn expanded(&self, point: Xy) -> (Xy, Xy) {
        (
            Xy::new(point.x() - self.tolerance[0], point.y() - self.tolerance[1]),
            Xy::new(point.x() + self.tolerance[0], point.y() + self.tolerance[1]),
        )
    }
}
