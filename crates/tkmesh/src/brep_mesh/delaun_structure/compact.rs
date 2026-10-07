use super::*;
impl BRepMeshDataStructureOfDelaun {
    /// Compact links before processing the live vertex deletion queue; `mesh.clear_deleted()?`.
    pub fn clear_deleted(&self) -> Result<(), Failure> {
        self.connectivity.write().compact_links()?;
        self.compact_nodes()
    }
    fn compact_nodes(&self) -> Result<(), Failure> {
        let queue = self.nodes.get_list_of_del_nodes();
        let mut last = self.nb_nodes();
        while !queue.read().is_empty() {
            while last > 0 && self.get_node(last)?.movability() == DegreeOfFreedom::Deleted {
                self.nodes.remove_last()?;
                last -= 1;
            }
            let index = *queue.read().first().expect("nonempty deletion queue");
            queue.write().remove_first()?;
            if index > last {
                continue;
            }
            self.move_last_node(index, last)?;
            last -= 1;
        }
        Ok(())
    }
    fn move_last_node(&self, index: i32, last: i32) -> Result<(), Failure> {
        let node = self.get_node(last)?;
        let adjacent = self.links_connected_to(last)?;
        self.nodes.remove_last()?;
        self.nodes.substitute(index, &node)?;
        let mut state = self.connectivity.write();
        state.node_links.change_find(&index)?.assign(&adjacent);
        for edge in adjacent.iter() {
            state.replace_node_in_link(*edge, last, index)?;
        }
        Ok(())
    }
}
impl DelaunConnectivity {
    fn compact_links(&mut self) -> Result<(), Failure> {
        let mut last = self.links.extent() as i32;
        while !self.deleted_links.is_empty() {
            last = self.trim_deleted_links(last)?;
            let index = *self.deleted_links.first().expect("nonempty deleted links");
            self.deleted_links.remove_first()?;
            if index > last {
                continue;
            }
            self.move_last_link(index, last)?;
            last -= 1;
        }
        Ok(())
    }
    fn trim_deleted_links(&mut self, mut last: i32) -> Result<i32, Failure> {
        while last > 0
            && self
                .links
                .find_key(last as usize)?
                .0
                .snapshot()
                .movability()
                == DegreeOfFreedom::Deleted
        {
            self.links.remove_last()?;
            last -= 1;
        }
        Ok(last)
    }
    fn move_last_link(&mut self, index: i32, last: i32) -> Result<(), Failure> {
        let edge = self.links.find_key(last as usize)?.0.snapshot();
        // The native arena keeps the removed pair's bytes alive after RemoveLast.
        let pair = *self.links.find_from_index(last as usize)?;
        self.links.remove_last()?;
        self.replace_link(index, edge, pair)?;
        self.link_domain.remove(last);
        self.link_domain.add(index);
        self.replace_adjacent_link(edge.first_node(), last, index)?;
        self.replace_adjacent_link(edge.last_node(), last, index)?;
        for position in 1..=pair.extent() {
            self.replace_element_link(pair.index(position)?, last, index)?;
        }
        Ok(())
    }
    fn replace_adjacent_link(&mut self, node: i32, old: i32, new: i32) -> Result<(), Failure> {
        for index in self.node_links.change_find(&node)?.iter_mut() {
            if *index == old {
                *index = new;
                break;
            }
        }
        Ok(())
    }
    fn replace_element_link(&mut self, index: i32, old: i32, new: i32) -> Result<(), Failure> {
        let mut element = self.element(index)?;
        for edge in &mut element.edges {
            if *edge == old {
                *edge = new;
                break;
            }
        }
        self.write_native_replacement(index, element)
    }
    fn replace_node_in_link(&mut self, index: i32, old: i32, new: i32) -> Result<(), Failure> {
        let edge = self.links.find_key(index as usize)?.0.snapshot();
        let pair = *self.links.find_from_index(index as usize)?;
        let mut first = edge.first_node();
        let mut last = edge.last_node();
        if first == old {
            first = new;
        } else if last == old {
            last = new;
        }
        self.replace_link(
            index,
            BRepMeshEdge::new(first, last, edge.movability()),
            pair,
        )
    }
}
