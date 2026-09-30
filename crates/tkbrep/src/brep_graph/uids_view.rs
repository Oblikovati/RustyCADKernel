// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_UIDsView.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph::UIDsView`: persistent node and reference identities and version stamps.
use crate::brep_graph::*;
use rustycad_tkernel::standard::Guid;

/// `BRepGraph::UIDsView`: obtained by [`BRepGraph::uids`].
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId};
/// let graph = BRepGraph::new();
/// assert!(!graph.uids().of_node(NodeId::default()).is_valid());
/// assert!(graph.uids().is_stale(&Default::default()));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct UidsView<'g> {
    graph: &'g BRepGraph,
}

impl BRepGraph {
    /// `UIDs()`: the identity view.
    pub fn uids(&self) -> UidsView<'_> {
        UidsView { graph: self }
    }
}

impl UidsView<'_> {
    /// `Of(theNode)`: the UID of an active node in range, invalid otherwise.
    pub fn of_node(self, node: NodeId) -> Uid {
        match self.graph.topo_entity(node) {
            Some(def) if !node.is_removed(self.graph) => Uid::new(node.kind, def.uid),
            _ => Uid::invalid(),
        }
    }

    /// `Of(theRefId)`: the RefUID of an active reference in range, invalid otherwise.
    pub fn of_ref(self, reference: RefId) -> RefUid {
        if !reference.is_valid() || reference.is_removed(self.graph) {
            return RefUid::invalid();
        }
        match self.graph.ref_entity(reference) {
            Some(base) => RefUid::new(reference.kind, base.uid),
            None => RefUid::invalid(),
        }
    }

    /// `Of(theItem)`: the durable UID of a node or reference item.
    pub fn of_item(self, item: ItemId) -> ItemUid {
        if !item.is_valid() {
            return ItemUid::invalid();
        }
        match item.item_domain() {
            ItemDomain::Node => {
                let uid = self.of_node(item.node_id());
                if uid.is_valid() {
                    ItemUid::node(item.node_kind(), uid.counter as usize)
                } else {
                    ItemUid::invalid()
                }
            }
            ItemDomain::Reference => {
                let uid = self.of_ref(item.ref_id());
                if uid.is_valid() {
                    ItemUid::reference(item.ref_kind(), uid.counter as usize)
                } else {
                    ItemUid::invalid()
                }
            }
            ItemDomain::None => ItemUid::invalid(),
        }
    }

    /// `NodeIdFrom(theUID)`: the active node of a UID through the reverse index.
    pub fn node_id_from(self, uid: &Uid) -> NodeId {
        if !uid.is_valid() {
            return NodeId::default();
        }
        let storage = self.graph.inc_storage();
        storage.ensure_uid_reverse_index();
        let found = storage
            .uid_to_node_id
            .read()
            .expect("UID index lock")
            .get(uid)
            .copied();
        match found {
            Some(node)
                if self.graph.topo_entity(node).is_some() && !node.is_removed(self.graph) =>
            {
                node
            }
            _ => NodeId::default(),
        }
    }

    /// `RefIdFrom(theUID)`: the active reference of a RefUID through the reverse index.
    pub fn ref_id_from(self, uid: &RefUid) -> RefId {
        if !uid.is_valid() {
            return RefId::default();
        }
        let storage = self.graph.inc_storage();
        storage.ensure_ref_uid_reverse_index();
        let found = storage
            .ref_uid_to_ref_id
            .read()
            .expect("RefUID index lock")
            .get(uid)
            .copied();
        match found {
            Some(reference) if !reference.is_removed(self.graph) => reference,
            _ => RefId::default(),
        }
    }

    /// `ItemIdFrom(theUID)`: the active item of a durable UID.
    pub fn item_id_from(self, uid: &ItemUid) -> ItemId {
        if !uid.is_valid() {
            return ItemId::default();
        }
        let counter = uid.counter() as u32;
        match uid.item_domain() {
            ItemDomain::Node => {
                ItemId::from(self.node_id_from(&Uid::new(uid.node_kind(), counter)))
            }
            ItemDomain::Reference => {
                ItemId::from(self.ref_id_from(&RefUid::new(uid.ref_kind(), counter)))
            }
            ItemDomain::None => ItemId::default(),
        }
    }

    /// `Has(theUID)`.
    pub fn has_node(self, uid: &Uid) -> bool {
        self.node_id_from(uid).is_valid()
    }

    /// `Has(theRefUID)`.
    pub fn has_ref(self, uid: &RefUid) -> bool {
        self.ref_id_from(uid).is_valid()
    }

    /// `Has(theItemUID)`.
    pub fn has_item(self, uid: &ItemUid) -> bool {
        self.item_id_from(uid).is_valid()
    }

    /// `Generation()`: incremented by each `Clear()`.
    pub fn generation(self) -> u32 {
        self.graph.inc_storage().generation()
    }

    /// `GraphGUID()`: redrawn by each `Clear()`.
    pub fn graph_guid(self) -> Guid {
        *self.graph.inc_storage().graph_guid()
    }

    /// `StampOf(theNode)`: the node UID with its OwnGen and the generation.
    pub fn stamp_of_node(self, node: NodeId) -> VersionStamp {
        let Some(def) = self.graph.topo_entity(node) else {
            return VersionStamp::default();
        };
        if node.is_removed(self.graph) {
            return VersionStamp::default();
        }
        let uid = Uid::new(node.kind, def.uid);
        if !uid.is_valid() {
            return VersionStamp::default();
        }
        VersionStamp::of_node(uid, def.own_gen, self.generation())
    }

    /// `StampOf(theRefId)`: the RefUID with the OwnGen of its active parent definition.
    pub fn stamp_of_ref(self, reference: RefId) -> VersionStamp {
        if !reference.is_valid() {
            return VersionStamp::default();
        }
        let parent = ref_parent_node(self.graph, reference);
        if !parent.is_valid() || reference.is_removed(self.graph) {
            return VersionStamp::default();
        }
        let Some(parent_def) = self.graph.topo_entity(parent) else {
            return VersionStamp::default();
        };
        if parent.is_removed(self.graph) {
            return VersionStamp::default();
        }
        let Some(base) = self.graph.ref_entity(reference) else {
            return VersionStamp::default();
        };
        let uid = RefUid::new(reference.kind, base.uid);
        if !uid.is_valid() {
            return VersionStamp::default();
        }
        VersionStamp::of_ref(uid, parent_def.own_gen, self.generation())
    }

    /// `StampOf(theRepId)`: the owner definition's UID and OwnGen for an active use
    /// record (use records have no identity of their own).
    pub fn stamp_of_rep(self, rep: RepId) -> VersionStamp {
        if !rep.is_valid() || !rep_in_range(self.graph, rep) {
            return VersionStamp::default();
        }
        let parent = rep_owner(self.graph, rep);
        let Some(def) = self.graph.topo_entity(parent) else {
            return VersionStamp::default();
        };
        if parent.is_removed(self.graph) {
            return VersionStamp::default();
        }
        VersionStamp::of_node(self.of_node(parent), def.own_gen, self.generation())
    }

    /// `StampOf(theItem)`.
    pub fn stamp_of_item(self, item: ItemId) -> VersionStamp {
        if !item.is_valid() {
            return VersionStamp::default();
        }
        match item.item_domain() {
            ItemDomain::Node => self.stamp_of_node(item.node_id()),
            ItemDomain::Reference => self.stamp_of_ref(item.ref_id()),
            ItemDomain::None => VersionStamp::default(),
        }
    }

    /// `IsStale(theStamp)`: invalid, from another generation, or the item was removed or
    /// its (parent's) OwnGen changed.
    pub fn is_stale(self, stamp: &VersionStamp) -> bool {
        if !stamp.is_valid() || stamp.generation != self.generation() {
            return true;
        }
        if stamp.is_node_stamp() {
            let node = self.node_id_from(&stamp.node_uid);
            return match self.graph.topo_entity(node) {
                Some(def) if node.is_valid() && !node.is_removed(self.graph) => {
                    def.own_gen != stamp.mutation_gen
                }
                _ => true,
            };
        }
        if stamp.is_ref_stamp() {
            return self.is_ref_stamp_stale(stamp);
        }
        true
    }

    fn is_ref_stamp_stale(self, stamp: &VersionStamp) -> bool {
        let reference = self.ref_id_from(&stamp.ref_uid);
        if !reference.is_valid() {
            return true;
        }
        let parent = ref_parent_node(self.graph, reference);
        if !parent.is_valid() || reference.is_removed(self.graph) {
            return true;
        }
        match self.graph.topo_entity(parent) {
            Some(def) if !parent.is_removed(self.graph) => def.own_gen != stamp.mutation_gen,
            _ => true,
        }
    }
}

// `resolveRefParentNodeId`: the parent definition of a reference in range.
fn ref_parent_node(graph: &BRepGraph, reference: RefId) -> NodeId {
    let storage = graph.inc_storage();
    if !graph.refs().generic().is_valid(reference) {
        return NodeId::default();
    }
    match reference.kind {
        RefKind::SHELL => storage
            .shell_ref(ShellRefId::from_ref_id(reference))
            .parent_solid_id
            .into(),
        RefKind::FACE => storage
            .face_ref(FaceRefId::from_ref_id(reference))
            .parent_shell_id
            .into(),
        RefKind::WIRE => storage
            .wire_ref(WireRefId::from_ref_id(reference))
            .parent_face_id
            .into(),
        RefKind::SOLID => storage
            .solid_ref(SolidRefId::from_ref_id(reference))
            .parent_comp_solid_id
            .into(),
        RefKind::CHILD => storage
            .child_ref(ChildRefId::from_ref_id(reference))
            .parent_compound_id
            .into(),
        RefKind::OCCURRENCE => storage
            .occurrence_ref(OccurrenceRefId::from_ref_id(reference))
            .parent_product_id
            .into(),
        RefKind::VERTEX => storage
            .vertex_ref(VertexRefId::from_ref_id(reference))
            .parent_edge_id
            .into(),
        _ => NodeId::default(),
    }
}

// Whether an active use record exists at the id (the per-kind guard of `StampOf`).
fn rep_in_range(graph: &BRepGraph, rep: RepId) -> bool {
    let storage = graph.inc_storage();
    let count = match rep.kind {
        RepKind::EDGE_CURVE_3D => storage.nb_edge_curves3d(),
        RepKind::EDGE_POLYGON_3D => storage.nb_edge_polygons3d(),
        RepKind::CO_EDGE_CURVE_2D => storage.nb_co_edge_curves2d(),
        RepKind::CO_EDGE_POLYGON_2D => storage.nb_co_edge_polygons2d(),
        RepKind::CO_EDGE_POLYGON_ON_TRI => storage.nb_co_edge_polygons_on_tri(),
        RepKind::FACE_SURFACE => storage.nb_face_surfaces(),
        RepKind::FACE_TRIANGULATION => storage.nb_face_triangulations(),
        _ => 0,
    };
    rep.index < count && !rep.is_removed(graph)
}

// The owner definition of an active use record, invalid when the record is out of
// range or removed.
fn rep_owner(graph: &BRepGraph, rep: RepId) -> NodeId {
    if !rep_in_range(graph, rep) {
        return NodeId::default();
    }
    let storage = graph.inc_storage();
    let index = rep.index;
    match rep.kind {
        RepKind::EDGE_CURVE_3D => storage
            .edge_curve3d_rep(EdgeCurve3DRepId::new(index))
            .parent_edge_id
            .into(),
        RepKind::EDGE_POLYGON_3D => storage
            .edge_polygon3d_rep(EdgePolygon3DRepId::new(index))
            .parent_edge_id
            .into(),
        RepKind::CO_EDGE_CURVE_2D => storage
            .co_edge_curve2d_rep(CoEdgeCurve2DRepId::new(index))
            .parent_co_edge_id
            .into(),
        RepKind::CO_EDGE_POLYGON_2D => storage
            .co_edge_polygon2d_rep(CoEdgePolygon2DRepId::new(index))
            .parent_co_edge_id
            .into(),
        RepKind::CO_EDGE_POLYGON_ON_TRI => storage
            .co_edge_polygon_on_tri_rep(CoEdgePolygonOnTriRepId::new(index))
            .parent_co_edge_id
            .into(),
        RepKind::FACE_SURFACE => storage
            .face_surface_rep(FaceSurfaceRepId::new(index))
            .parent_face_id
            .into(),
        RepKind::FACE_TRIANGULATION => storage
            .face_triangulation_rep(FaceTriangulationRepId::new(index))
            .parent_face_id
            .into(),
        _ => NodeId::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uids_and_stamps_follow_the_graph() {
        let mut graph = BRepGraph::new();
        let face = graph.inc_storage_mut().append_face();
        let uid = graph.allocate_uid(face.into());
        let surface = graph.inc_storage_mut().append_face_surface_rep();
        graph
            .inc_storage_mut()
            .change_face_surface_rep(surface)
            .parent_face_id = face;
        let uids = graph.uids();
        assert_eq!(uids.of_node(face.into()), uid);
        assert_eq!(uids.node_id_from(&uid), NodeId::from(face));
        assert!(uids.has_item(&uids.of_item(ItemId::from(NodeId::from(face)))));
        let stamp = uids.stamp_of_node(face.into());
        assert!(!uids.is_stale(&stamp));
        assert_eq!(uids.stamp_of_rep(surface.into()), stamp);
        assert!(
            !uids
                .stamp_of_rep(FaceSurfaceRepId::new(3).into())
                .is_valid()
        );
        graph.mark_modified(face.into());
        assert!(graph.uids().is_stale(&stamp));
    }
}
