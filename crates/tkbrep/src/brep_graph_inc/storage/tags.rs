// Rust translation of the TypedStorePlanes specialisations and the flag templates of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TypedStorePlanes<T>`: the store of each kind tag, and the flag members written once
//! over any typed id (`IsRemoved`, `SetRemoved`, `IsOwned`, `SetGuarded`...).
use super::Storage;
use super::stores::{EntityStore, FlagPlane, OwnershipPlanes, RemovalPlanes, RepStore};
use crate::brep_graph::ids::{NodeFamily, RefFamily, RepFamily};
use crate::brep_graph::*;
use crate::brep_graph_inc::definition::*;
use crate::brep_graph_inc::reference::*;
use crate::brep_graph_inc::representation::*;
use crate::brep_graph_inc::{HasBaseDef, HasBaseRef};

/// A kind tag with a store in the storage (`TypedStorePlanes<T>`).
pub trait StoredTag: KindTag {
    /// The store type of the kind.
    type Store: RemovalPlanes;
    /// The store of the kind.
    fn store(storage: &Storage) -> &Self::Store;
    /// The mutable store of the kind.
    fn store_mut(storage: &mut Storage) -> &mut Self::Store;
}

/// A definition-node kind: its store holds `Def` records.
pub trait NodeTag:
    KindTag<Family = NodeFamily> + StoredTag<Store = EntityStore<Self::Def>>
{
    /// The definition record of the kind (`VertexDef`...).
    type Def: HasBaseDef + Default + Clone;
}

/// A reference kind: its store holds `Entry` records.
pub trait RefTag:
    KindTag<Family = RefFamily> + StoredTag<Store = EntityStore<Self::Entry>>
{
    /// The reference entry of the kind (`ShellRef`...).
    type Entry: HasBaseRef + Default + Clone;
}

/// A representation kind: its store holds `Record` records.
pub trait RepTag: KindTag<Family = RepFamily> + StoredTag<Store = RepStore<Self::Record>> {
    /// The use record of the kind (`FaceSurfaceRep`...).
    type Record: Default + Clone;
}

macro_rules! stored_tags {
    ($marker:ident, $assoc:ident, $store:ident; $($tag:ty => $field:ident: $record:ty;)*) => {$(
        impl StoredTag for $tag {
            type Store = $store<$record>;
            fn store(storage: &Storage) -> &Self::Store {
                &storage.$field
            }
            fn store_mut(storage: &mut Storage) -> &mut Self::Store {
                &mut storage.$field
            }
        }
        impl $marker for $tag {
            type $assoc = $record;
        }
    )*};
}

stored_tags! {
    NodeTag, Def, EntityStore;
    VertexTag => vertices: VertexDef;
    EdgeTag => edges: EdgeDef;
    CoEdgeTag => co_edges: CoEdgeDef;
    WireTag => wires: WireDef;
    FaceTag => faces: FaceDef;
    ShellTag => shells: ShellDef;
    SolidTag => solids: SolidDef;
    CompoundTag => compounds: CompoundDef;
    CompSolidTag => comp_solids: CompSolidDef;
    ProductTag => products: ProductDef;
    OccurrenceTag => occurrences: OccurrenceDef;
}

stored_tags! {
    RefTag, Entry, EntityStore;
    ShellRefTag => shell_refs: ShellRef;
    FaceRefTag => face_refs: FaceRef;
    WireRefTag => wire_refs: WireRef;
    VertexRefTag => vertex_refs: VertexRef;
    SolidRefTag => solid_refs: SolidRef;
    ChildRefTag => child_refs: ChildRef;
    OccurrenceRefTag => occurrence_refs: OccurrenceRef;
}

stored_tags! {
    RepTag, Record, RepStore;
    EdgeCurve3DTag => edge_curves3d: EdgeCurve3DRep;
    EdgePolygon3DTag => edge_polygons3d: EdgePolygon3DRep;
    CoEdgeCurve2DTag => co_edge_curves2d: CoEdgeCurve2DRep;
    CoEdgePolygon2DTag => co_edge_polygons2d: CoEdgePolygon2DRep;
    CoEdgePolygonOnTriTag => co_edge_polygons_on_tri: CoEdgePolygonOnTriRep;
    FaceSurfaceTag => face_surfaces: FaceSurfaceRep;
    FaceTriangulationTag => face_triangulations: FaceTriangulationRep;
}

impl Storage {
    // `isInRange(theId)`: inside the store and inside the removed plane.
    fn is_in_range<T: StoredTag>(&self, id: TypedId<T>) -> bool {
        let store = T::store(self);
        id.is_valid_for(store.count()) && store.removed_plane().is_valid_index(id.index)
    }

    /// `IsRemoved(theId)`: the id is in range and soft-removed.
    pub fn is_removed<T: StoredTag>(&self, id: TypedId<T>) -> bool {
        self.is_in_range(id) && T::store(self).removed_plane().test(id.index)
    }

    /// `SetRemoved(theId, theVal)`: flips the removed bit and the active count; an out of
    /// range id is ignored.
    pub fn set_removed<T: StoredTag>(&mut self, id: TypedId<T>, value: bool) {
        if !self.is_in_range(id) {
            return;
        }
        let (plane, nb_active) = T::store_mut(self).removal_mut();
        if plane.test(id.index) == value {
            return;
        }
        if value {
            plane.set(id.index);
            *nb_active = nb_active.saturating_sub(1);
        } else {
            plane.clear(id.index);
            *nb_active += 1;
        }
    }

    // The test of one non-removal plane for an in-range id.
    fn test_plane<T: StoredTag>(&self, id: TypedId<T>, plane: FlagPlane) -> bool
    where
        T::Store: OwnershipPlanes,
    {
        self.is_in_range(id) && T::store(self).plane(plane).test(id.index)
    }

    // The update of one non-removal plane for an in-range id.
    fn update_plane<T: StoredTag>(&mut self, id: TypedId<T>, plane: FlagPlane, value: bool)
    where
        T::Store: OwnershipPlanes,
    {
        if !self.is_in_range(id) {
            return;
        }
        let bits = T::store_mut(self).plane_mut(plane);
        if value {
            bits.set(id.index);
        } else {
            bits.clear(id.index);
        }
    }

    /// `IsOwned(theId)`.
    pub fn is_owned<T: StoredTag>(&self, id: TypedId<T>) -> bool
    where
        T::Store: OwnershipPlanes,
    {
        self.is_in_range(id) && T::store(self).owned_plane().test(id.index)
    }

    /// `SetOwned(theId, theVal)`: takes `&self`, since `BRepGraph_LayerLock` writes the
    /// plane through the shared graph it is attached to (docs/topology-design.md 11.10).
    pub fn set_owned<T: StoredTag>(&self, id: TypedId<T>, value: bool)
    where
        T::Store: OwnershipPlanes,
    {
        if !self.is_in_range(id) {
            return;
        }
        let plane = T::store(self).owned_plane();
        if value {
            plane.set(id.index);
        } else {
            plane.clear(id.index);
        }
    }

    /// `IsGuarded(theId)`: an active MutGuard.
    pub fn is_guarded<T: StoredTag>(&self, id: TypedId<T>) -> bool
    where
        T::Store: OwnershipPlanes,
    {
        self.test_plane(id, FlagPlane::Guard)
    }

    /// `SetGuarded(theId)`.
    pub fn set_guarded<T: StoredTag>(&mut self, id: TypedId<T>)
    where
        T::Store: OwnershipPlanes,
    {
        self.update_plane(id, FlagPlane::Guard, true);
    }

    /// `ClearGuarded(theId)`.
    pub fn clear_guarded<T: StoredTag>(&mut self, id: TypedId<T>)
    where
        T::Store: OwnershipPlanes,
    {
        self.update_plane(id, FlagPlane::Guard, false);
    }

    /// `HasCompoundParentTyped(theId)` (private natively).
    pub(crate) fn has_compound_parent_typed<T: StoredTag>(&self, id: TypedId<T>) -> bool
    where
        T::Store: OwnershipPlanes,
    {
        self.test_plane(id, FlagPlane::HasCompoundParent)
    }

    /// `HasOccurrenceParentTyped(theId)` (private natively).
    pub(crate) fn has_occurrence_parent_typed<T: StoredTag>(&self, id: TypedId<T>) -> bool
    where
        T::Store: OwnershipPlanes,
    {
        self.test_plane(id, FlagPlane::HasOccurrenceParent)
    }

    /// `SetHasCompoundParentTyped(theId, theVal)` (private natively).
    pub(crate) fn set_has_compound_parent_typed<T: StoredTag>(
        &mut self,
        id: TypedId<T>,
        value: bool,
    ) where
        T::Store: OwnershipPlanes,
    {
        self.update_plane(id, FlagPlane::HasCompoundParent, value);
    }

    /// `SetHasOccurrenceParentTyped(theId, theVal)` (private natively).
    pub(crate) fn set_has_occurrence_parent_typed<T: StoredTag>(
        &mut self,
        id: TypedId<T>,
        value: bool,
    ) where
        T::Store: OwnershipPlanes,
    {
        self.update_plane(id, FlagPlane::HasOccurrenceParent, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_planes_follow_the_native_templates() {
        let mut storage = Storage::new();
        let face = storage.append_face();
        storage.set_removed(face, true);
        assert!(storage.is_removed(face));
        assert_eq!(storage.nb_active_faces(), 0);
        storage.set_removed(face, false);
        assert_eq!(storage.nb_active_faces(), 1);
        storage.set_owned(face, true);
        assert!(storage.is_owned(face));
        storage.set_guarded(face);
        assert!(storage.is_guarded(face) && storage.has_any_guard());
        storage.clear_guarded(face);
        assert!(!storage.is_guarded(face));
        storage.set_has_compound_parent_typed(face, true);
        assert!(storage.has_compound_parent_typed(face));
        assert!(!storage.has_occurrence_parent_typed(face));
        storage.set_removed(FaceId::new(9), true);
        assert!(!storage.is_removed(FaceId::new(9)));
    }
}
