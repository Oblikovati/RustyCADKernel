// Rust translation of the relation-list helpers, the sparse incoming maps, the parent
// flags and the relation setters of ModelingData/TKBRep/BRepGraphInc/
// BRepGraphInc_Storage.cxx and .lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The relation-list helpers and the members that replace or reverse a whole list, the
//! sparse compound and occurrence incoming maps, and the generic parent and guard flags.
use super::Storage;
use super::tags::{NodeTag, RefTag};
use crate::brep_graph::*;

/// `appendUniqueRelationId`: appends the id when the list does not hold it.
pub(crate) fn append_unique<I: PartialEq + Copy>(ids: &mut Vec<I>, id: I) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

/// `eraseRelationId`: removes the first occurrence of the id; true when found.
pub(crate) fn erase_first<I: PartialEq + Copy>(ids: &mut Vec<I>, id: I) -> bool {
    match ids.iter().position(|item| *item == id) {
        Some(position) => {
            ids.remove(position);
            true
        }
        None => false,
    }
}

impl Storage {
    /// `CompoundRefsOfNode(theNode)`: the compound child references naming the node.
    pub fn compound_refs_of_node(&self, node: NodeId) -> &[ChildRefId] {
        self.node_to_compounds.get(&node).map_or(&[], Vec::as_slice)
    }

    /// `OccurrenceRefsOfNode(theNode)`: the occurrence references whose occurrence names
    /// the node.
    pub fn occurrence_refs_of_node(&self, node: NodeId) -> &[OccurrenceRefId] {
        self.node_to_occurrences
            .get(&node)
            .map_or(&[], Vec::as_slice)
    }

    /// `ChangeCompoundRefsOfNodeInternal(theNode)`: bound on demand.
    pub(crate) fn change_compound_refs_of_node_internal(
        &mut self,
        node: NodeId,
    ) -> &mut Vec<ChildRefId> {
        self.node_to_compounds.entry(node).or_default()
    }

    /// `ChangeOccurrenceRefsOfNodeInternal(theNode)`: bound on demand.
    pub(crate) fn change_occurrence_refs_of_node_internal(
        &mut self,
        node: NodeId,
    ) -> &mut Vec<OccurrenceRefId> {
        self.node_to_occurrences.entry(node).or_default()
    }

    /// `HasCompoundParent(theNode)`: the node's has-compound-parent bit.
    pub fn has_compound_parent(&self, node: NodeId) -> bool {
        node.is_valid() && node.visit(ParentFlagReader(self, true))
    }

    /// `HasOccurrenceParent(theNode)`: the node's has-occurrence-parent bit.
    pub fn has_occurrence_parent(&self, node: NodeId) -> bool {
        node.is_valid() && node.visit(ParentFlagReader(self, false))
    }

    /// `SetHasCompoundParent(theNode, theVal)` (private natively).
    pub(crate) fn set_has_compound_parent(&mut self, node: NodeId, value: bool) {
        if node.is_valid() {
            node.visit(ParentFlagWriter(self, true, value));
        }
    }

    /// `SetHasOccurrenceParent(theNode, theVal)` (private natively).
    pub(crate) fn set_has_occurrence_parent(&mut self, node: NodeId, value: bool) {
        if node.is_valid() {
            node.visit(ParentFlagWriter(self, false, value));
        }
    }

    /// `IsGuarded(const BRepGraph_ItemId&)`: the guard bit of a node or a reference.
    pub fn is_item_guarded(&self, item: ItemId) -> bool {
        match item.item_domain() {
            ItemDomain::Node => item.node_id().visit(NodeGuard(self)),
            ItemDomain::Reference => item.ref_id().visit(RefGuard(self)),
            ItemDomain::None => false,
        }
    }

    /// `SetGuarded(const BRepGraph_ItemId&)`: an invalid item is ignored (the assertion
    /// is debug-only natively).
    pub fn set_item_guarded(&mut self, item: ItemId) {
        self.write_item_guard(item, true);
    }

    /// `ClearGuarded(const BRepGraph_ItemId&)`.
    pub fn clear_item_guarded(&mut self, item: ItemId) {
        self.write_item_guard(item, false);
    }

    fn write_item_guard(&mut self, item: ItemId, value: bool) {
        match item.item_domain() {
            ItemDomain::Node => item.node_id().visit(NodeGuardWriter(self, value)),
            ItemDomain::Reference => item.ref_id().visit(RefGuardWriter(self, value)),
            ItemDomain::None => {}
        }
    }

    /// `ReverseWireCoEdges(theWireId)`.
    pub fn reverse_wire_co_edges(&mut self, wire: WireId) {
        if wire.is_valid_for(self.nb_wires()) {
            self.change_wire_relations_internal(wire)
                .co_edge_ids
                .reverse();
        }
    }

    /// `SetWireCoEdges(theWireId, theCoEdgeIds)`.
    pub fn set_wire_co_edges(&mut self, wire: WireId, co_edges: &[CoEdgeId]) {
        if wire.is_valid_for(self.nb_wires()) {
            self.change_wire_relations_internal(wire).co_edge_ids = co_edges.to_vec();
        }
    }

    /// `SetFaceWireRefs(theFaceId, theWireRefIds)`.
    pub fn set_face_wire_refs(&mut self, face: FaceId, refs: &[WireRefId]) {
        if face.is_valid_for(self.nb_faces()) {
            self.change_face_relations_internal(face).wire_ref_ids = refs.to_vec();
        }
    }

    /// `SetShellFaceRefs(theShellId, theFaceRefIds)`.
    pub fn set_shell_face_refs(&mut self, shell: ShellId, refs: &[FaceRefId]) {
        if shell.is_valid_for(self.nb_shells()) {
            self.change_shell_relations_internal(shell).face_ref_ids = refs.to_vec();
        }
    }

    /// `SetSolidShellRefs(theSolidId, theShellRefIds)`.
    pub fn set_solid_shell_refs(&mut self, solid: SolidId, refs: &[ShellRefId]) {
        if solid.is_valid_for(self.nb_solids()) {
            self.change_solid_relations_internal(solid).shell_ref_ids = refs.to_vec();
        }
    }

    /// `SetCompSolidSolidRefs(theCompSolidId, theSolidRefIds)`.
    pub fn set_comp_solid_solid_refs(&mut self, comp_solid: CompSolidId, refs: &[SolidRefId]) {
        if comp_solid.is_valid_for(self.nb_comp_solids()) {
            self.change_comp_solid_relations_internal(comp_solid)
                .solid_ref_ids = refs.to_vec();
        }
    }

    /// `SetCompoundChildRefs(theCompoundId, theChildRefIds)`.
    pub fn set_compound_child_refs(&mut self, compound: CompoundId, refs: &[ChildRefId]) {
        if compound.is_valid_for(self.nb_compounds()) {
            self.change_compound_relations_internal(compound)
                .child_ref_ids = refs.to_vec();
        }
    }

    /// `SetProductOccurrenceRefs(theProductId, theOccurrenceRefIds)`.
    pub fn set_product_occurrence_refs(&mut self, product: ProductId, refs: &[OccurrenceRefId]) {
        if product.is_valid_for(self.nb_products()) {
            self.change_product_relations_internal(product)
                .occurrence_ref_ids = refs.to_vec();
        }
    }
}

// Reads the compound (true) or occurrence (false) parent bit of a node.
struct ParentFlagReader<'s>(&'s Storage, bool);

impl NodeVisitor for ParentFlagReader<'_> {
    type Output = bool;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> bool {
        if self.1 {
            self.0.has_compound_parent_typed(id)
        } else {
            self.0.has_occurrence_parent_typed(id)
        }
    }
}

// Writes the compound (true) or occurrence (false) parent bit of a node.
struct ParentFlagWriter<'s>(&'s mut Storage, bool, bool);

impl NodeVisitor for ParentFlagWriter<'_> {
    type Output = ();
    fn visit<T: NodeTag>(self, id: TypedId<T>) {
        if self.1 {
            self.0.set_has_compound_parent_typed(id, self.2);
        } else {
            self.0.set_has_occurrence_parent_typed(id, self.2);
        }
    }
}

struct NodeGuard<'s>(&'s Storage);

impl NodeVisitor for NodeGuard<'_> {
    type Output = bool;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> bool {
        self.0.is_guarded(id)
    }
}

struct RefGuard<'s>(&'s Storage);

impl RefVisitor for RefGuard<'_> {
    type Output = bool;
    fn visit<T: RefTag>(self, id: TypedId<T>) -> bool {
        self.0.is_guarded(id)
    }
}

struct NodeGuardWriter<'s>(&'s mut Storage, bool);

impl NodeVisitor for NodeGuardWriter<'_> {
    type Output = ();
    fn visit<T: NodeTag>(self, id: TypedId<T>) {
        if self.1 {
            self.0.set_guarded(id);
        } else {
            self.0.clear_guarded(id);
        }
    }
}

struct RefGuardWriter<'s>(&'s mut Storage, bool);

impl RefVisitor for RefGuardWriter<'_> {
    type Output = ();
    fn visit<T: RefTag>(self, id: TypedId<T>) {
        if self.1 {
            self.0.set_guarded(id);
        } else {
            self.0.clear_guarded(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_helpers_keep_order() {
        let mut ids = vec![1, 2];
        append_unique(&mut ids, 2);
        append_unique(&mut ids, 3);
        assert_eq!(ids, vec![1, 2, 3]);
        assert!(erase_first(&mut ids, 2));
        assert!(!erase_first(&mut ids, 9));
        assert_eq!(ids, vec![1, 3]);
    }

    #[test]
    fn setters_replace_whole_lists() {
        let mut storage = Storage::new();
        let wire = storage.append_wire();
        storage.set_wire_co_edges(wire, &[CoEdgeId::new(1), CoEdgeId::new(2)]);
        storage.reverse_wire_co_edges(wire);
        assert_eq!(
            storage.wire_relations(wire).co_edge_ids[0],
            CoEdgeId::new(2)
        );
        let face = storage.append_face();
        storage.set_face_wire_refs(face, &[WireRefId::new(4)]);
        assert_eq!(storage.face_relations(face).wire_ref_ids.len(), 1);
        storage.set_wire_co_edges(WireId::new(8), &[]);
        let item = ItemId::from(NodeId::from(face));
        storage.set_item_guarded(item);
        assert!(storage.is_item_guarded(item));
        storage.clear_item_guarded(item);
        assert!(!storage.is_item_guarded(item));
        storage.set_has_occurrence_parent(face.into(), true);
        assert!(storage.has_occurrence_parent(face.into()));
    }
}
