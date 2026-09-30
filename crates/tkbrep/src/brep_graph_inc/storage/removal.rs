// Rust translation of the removal and generic-reference members of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Soft removal by generic id (`MarkRemoved`, `MarkRemovedRef`, `SetRemoved` of a use
//! record) and the generic reference base (`BaseRef`, `ChangeBaseRef`).
use super::Storage;
use super::tags::{NodeTag, RefTag};
use crate::brep_graph::*;
use crate::brep_graph_inc::{BaseRef, HasBaseRef};

impl Storage {
    /// `MarkRemoved(theNodeId)`: removes an active node once, then drops its original
    /// binding and the shape binding of that original (when it still names the node).
    pub fn mark_removed(&mut self, node: NodeId) -> bool {
        let removed = node.is_valid() && node.visit(NodeMarker(self));
        if removed {
            self.unbind_removed_original(node);
        }
        removed
    }

    /// `MarkRemovedRef(theRefId)`: removes an active reference once.
    pub fn mark_removed_ref(&mut self, reference: RefId) -> bool {
        reference.is_valid() && reference.visit(RefMarker(self))
    }

    /// `MarkRemoved(theRepId)`: removes an active use record once.
    pub fn mark_rep_removed(&mut self, rep: RepId) -> bool {
        if !rep.is_valid() {
            return false;
        }
        let index = rep.index;
        match rep.kind {
            RepKind::EDGE_CURVE_3D => self.edge_curves3d.mark_removed(index),
            RepKind::EDGE_POLYGON_3D => self.edge_polygons3d.mark_removed(index),
            RepKind::CO_EDGE_CURVE_2D => self.co_edge_curves2d.mark_removed(index),
            RepKind::CO_EDGE_POLYGON_2D => self.co_edge_polygons2d.mark_removed(index),
            RepKind::CO_EDGE_POLYGON_ON_TRI => self.co_edge_polygons_on_tri.mark_removed(index),
            RepKind::FACE_SURFACE => self.face_surfaces.mark_removed(index),
            RepKind::FACE_TRIANGULATION => self.face_triangulations.mark_removed(index),
            _ => false,
        }
    }

    /// `SetRemoved(theRepId, theVal)`.
    pub fn set_rep_removed(&mut self, rep: RepId, value: bool) {
        if !rep.is_valid() {
            return;
        }
        let index = rep.index;
        match rep.kind {
            RepKind::EDGE_CURVE_3D => self.set_removed(EdgeCurve3DRepId::new(index), value),
            RepKind::EDGE_POLYGON_3D => self.set_removed(EdgePolygon3DRepId::new(index), value),
            RepKind::CO_EDGE_CURVE_2D => self.set_removed(CoEdgeCurve2DRepId::new(index), value),
            RepKind::CO_EDGE_POLYGON_2D => {
                self.set_removed(CoEdgePolygon2DRepId::new(index), value)
            }
            RepKind::CO_EDGE_POLYGON_ON_TRI => {
                self.set_removed(CoEdgePolygonOnTriRepId::new(index), value)
            }
            RepKind::FACE_SURFACE => self.set_removed(FaceSurfaceRepId::new(index), value),
            RepKind::FACE_TRIANGULATION => {
                self.set_removed(FaceTriangulationRepId::new(index), value)
            }
            _ => {}
        }
    }

    /// The removed flag of a use record by generic id (the dispatch of
    /// `BRepGraph_RepId::IsRemoved`).
    pub(crate) fn is_rep_removed(&self, rep: RepId) -> bool {
        let index = rep.index;
        match rep.kind {
            RepKind::EDGE_CURVE_3D => self.is_removed(EdgeCurve3DRepId::new(index)),
            RepKind::EDGE_POLYGON_3D => self.is_removed(EdgePolygon3DRepId::new(index)),
            RepKind::CO_EDGE_CURVE_2D => self.is_removed(CoEdgeCurve2DRepId::new(index)),
            RepKind::CO_EDGE_POLYGON_2D => self.is_removed(CoEdgePolygon2DRepId::new(index)),
            RepKind::CO_EDGE_POLYGON_ON_TRI => self.is_removed(CoEdgePolygonOnTriRepId::new(index)),
            RepKind::FACE_SURFACE => self.is_removed(FaceSurfaceRepId::new(index)),
            RepKind::FACE_TRIANGULATION => self.is_removed(FaceTriangulationRepId::new(index)),
            _ => false,
        }
    }

    /// `isNodeRemoved` of the storage: true for an invalid id, else the removed flag.
    pub(crate) fn is_node_removed(&self, node: NodeId) -> bool {
        !node.is_valid() || node.visit(NodeRemovedFlag(self))
    }

    /// `BaseRef(theRefId)`: the base of any reference entry, `None` when the id is
    /// invalid or out of range (natively a static invalid base).
    pub fn base_ref(&self, reference: RefId) -> Option<&BaseRef> {
        if !reference.is_valid() {
            return None;
        }
        reference.visit(BaseRefFinder(self))
    }

    /// `ChangeBaseRef(theRefId)`: the mutable base, `None` when invalid or out of range.
    pub fn change_base_ref(&mut self, reference: RefId) -> Option<&mut BaseRef> {
        if !reference.is_valid() {
            return None;
        }
        reference.visit(BaseRefChanger(self))
    }

    // The original-binding cleanup of `MarkRemoved(theNodeId)`.
    fn unbind_removed_original(&mut self, node: NodeId) {
        let original = self.bindings_mut().originals.remove(&node);
        if let Some(shape) = original
            && !shape.is_null()
        {
            self.remove_definition_shape_binding(&shape, node);
        }
    }
}

struct NodeMarker<'s>(&'s mut Storage);

impl NodeVisitor for NodeMarker<'_> {
    type Output = bool;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> bool {
        T::store_mut(self.0).mark_removed(id.index)
    }
}

struct RefMarker<'s>(&'s mut Storage);

impl RefVisitor for RefMarker<'_> {
    type Output = bool;
    fn visit<T: RefTag>(self, id: TypedId<T>) -> bool {
        T::store_mut(self.0).mark_removed(id.index)
    }
}

struct NodeRemovedFlag<'s>(&'s Storage);

impl NodeVisitor for NodeRemovedFlag<'_> {
    type Output = bool;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> bool {
        self.0.is_removed(id)
    }
}

struct BaseRefFinder<'s>(&'s Storage);

impl<'s> RefVisitor for BaseRefFinder<'s> {
    type Output = Option<&'s BaseRef>;
    fn visit<T: RefTag>(self, id: TypedId<T>) -> Option<&'s BaseRef> {
        let store = T::store(self.0);
        store.items.get(id.index as usize).map(HasBaseRef::base)
    }
}

struct BaseRefChanger<'s>(&'s mut Storage);

impl<'s> RefVisitor for BaseRefChanger<'s> {
    type Output = Option<&'s mut BaseRef>;
    fn visit<T: RefTag>(self, id: TypedId<T>) -> Option<&'s mut BaseRef> {
        let store = T::store_mut(self.0);
        store
            .items
            .get_mut(id.index as usize)
            .map(HasBaseRef::base_mut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removal_counts_once() {
        let mut storage = Storage::new();
        let edge = storage.append_edge();
        assert!(storage.mark_removed(edge.into()));
        assert!(!storage.mark_removed(edge.into()));
        assert!(storage.is_node_removed(edge.into()));
        assert!(storage.is_node_removed(NodeId::default()));
        let reference = storage.append_face_ref();
        assert!(storage.mark_removed_ref(reference.into()));
        let curve = storage.append_edge_curve3d_rep();
        assert!(storage.mark_rep_removed(curve.into()));
        assert!(storage.is_rep_removed(curve.into()));
        storage.set_rep_removed(curve.into(), false);
        assert_eq!(storage.nb_active_edge_curves3d(), 1);
        assert!(storage.base_ref(FaceRefId::new(3).into()).is_none());
        storage.change_base_ref(reference.into()).unwrap().uid = 4;
        assert_eq!(storage.base_ref(reference.into()).unwrap().uid, 4);
    }
}
