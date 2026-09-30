// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_MutGuard.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_MutGuard<T>`: the scope token that batches the modification notice of
//! one definition or reference entry (docs/topology-design.md section 11.4).
use super::{
    BRepGraph, ChildRefTag, CoEdgeTag, CompSolidTag, CompoundTag, EdgeTag, FaceRefTag, FaceTag,
    OccurrenceRefTag, OccurrenceTag, ProductTag, ShellRefTag, ShellTag, SolidRefTag, SolidTag,
    TypedId, VertexRefTag, VertexTag, WireRefTag, WireTag,
};
use crate::brep_graph_inc::{NodeTag, OwnershipPlanes, RefTag, Storage, StoredTag};
use rustycad_tkernel::standard::Failure;
use std::ops::Deref;

/// The kinds a guard can hold: the definition kinds (`BaseDef`, notified with
/// `markModified`) and the reference kinds (`BaseRef`, notified with
/// `markRefModified`), the compile-time dispatch of the native template.
pub trait GuardTag: StoredTag<Store: OwnershipPlanes> {
    /// The guarded record (`VertexDef`, `ShellRef`...).
    type Target;
    /// The record of `id` (panics out of range, undefined natively).
    fn target(storage: &Storage, id: TypedId<Self>) -> &Self::Target;
    /// The mutable record of `id`.
    fn target_mut(storage: &mut Storage, id: TypedId<Self>) -> &mut Self::Target;
    /// `notify()`: `markModified` for a definition, `markRefModified` for a reference.
    fn notify(graph: &mut BRepGraph, id: TypedId<Self>);
}

macro_rules! guard_tags {
    ($family:ident, $record:ident, $getter:ident, $changer:ident, $mark:ident;
     $($tag:ty),* $(,)?) => {$(
        impl GuardTag for $tag {
            type Target = <$tag as $family>::$record;
            fn target(storage: &Storage, id: TypedId<Self>) -> &Self::Target {
                storage.$getter(id)
            }
            fn target_mut(storage: &mut Storage, id: TypedId<Self>) -> &mut Self::Target {
                storage.$changer(id)
            }
            fn notify(graph: &mut BRepGraph, id: TypedId<Self>) {
                graph.$mark(id.into());
            }
        }
    )*};
}

guard_tags! {
    NodeTag, Def, def, change_def, mark_modified;
    VertexTag, EdgeTag, CoEdgeTag, WireTag, FaceTag, ShellTag, SolidTag, CompoundTag,
    CompSolidTag, ProductTag, OccurrenceTag,
}

guard_tags! {
    RefTag, Entry, entry, change_entry, mark_ref_modified;
    VertexRefTag, WireRefTag, FaceRefTag, ShellRefTag, SolidRefTag, ChildRefTag,
    OccurrenceRefTag,
}

/// `BRepGraph_MutGuard<T>`: registered on its item (the guard plane) for its whole
/// life; the typed setters of the editor write through [`internal`](Self::internal),
/// which flags it dirty, and releasing a dirty guard (dropping it, or assigning over
/// it with [`move_assign`](Self::move_assign)) fires one `markModified` or
/// `markRefModified`, after the guard plane is cleared.
///
/// The guard holds the graph's only mutable borrow, so no second guard can be taken
/// from outside it; [`graph_mut`](Self::graph_mut) reaches the editor from inside,
/// where a second guard on the same item raises Standard_ProgramError as natively.
/// The record is read through the storage by id, never through a held reference, so
/// no `unsafe` is needed. The C++ move constructor is [`move_from`](Self::move_from):
/// it leaves the source inert (it owns nothing and releases nothing), as natively.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::mut_guard::MutGuard;
/// use rustycad_tkmath::gp::Point3;
/// let mut graph = BRepGraph::new();
/// let vertex = graph.editor().vertices().add(Point3::new(0.0, 0.0, 0.0), 1.0e-7);
/// let mut guard = graph.editor().vertices().mutate(vertex).unwrap();
/// assert!(!guard.is_dirty());
/// guard.internal().tolerance = 0.5;
/// assert!(guard.is_dirty() && guard.tolerance == 0.5);
/// let moved = MutGuard::move_from(&mut guard);
/// assert!(!guard.is_active() && moved.is_active());
/// drop((guard, moved));
/// assert_eq!(graph.topo().vertices().definition(vertex).base.own_gen, 2);
/// ```
#[derive(Debug)]
pub struct MutGuard<'g, T: GuardTag> {
    graph: Option<&'g mut BRepGraph>,
    id: TypedId<T>,
    dirty: bool,
}

const INERT_GUARD: &str = "BRepGraph_MutGuard: guard is empty or moved-from";

impl<'g, T: GuardTag> MutGuard<'g, T> {
    /// `BRepGraph_MutGuard(theGraph, theStorage, theEntity, theId)`: registers the
    /// guard on the item; an item already guarded raises Standard_ProgramError.
    pub fn new(graph: &'g mut BRepGraph, id: TypedId<T>) -> Result<Self, Failure> {
        if graph.inc_storage().is_guarded(id) {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                &format!(
                    "BRepGraph_MutGuard: guard already active on this item ({} {})",
                    T::NAME,
                    id.index
                ),
                "",
            ));
        }
        graph.inc_storage_mut().set_guarded(id);
        Ok(Self {
            graph: Some(graph),
            id,
            dirty: false,
        })
    }

    /// `BRepGraph_MutGuard(BRepGraph_MutGuard&&)`: takes over the source's item, scope
    /// and dirty flag; the source becomes inert.
    pub fn move_from(source: &mut Self) -> Self {
        let dirty = std::mem::replace(&mut source.dirty, false);
        Self {
            graph: source.graph.take(),
            id: source.id,
            dirty,
        }
    }

    /// `operator=(BRepGraph_MutGuard&&)`: releases this guard's item (notifying it when
    /// dirty), then takes over the source's; the source becomes inert.
    pub fn move_assign(&mut self, source: &mut Self) {
        self.release();
        *self = Self::move_from(source);
    }

    /// `operator bool()`: the guard still owns its item (false once moved from).
    pub fn is_active(&self) -> bool {
        self.graph.is_some()
    }

    /// `operator->()` / `operator*()`: the record, or Standard_ProgramError for an
    /// inert guard. `Deref` is the unchecked form, which panics with the same text.
    pub fn try_get(&self) -> Result<&T::Target, Failure> {
        match &self.graph {
            Some(graph) => Ok(T::target(graph.inc_storage(), self.id)),
            None => Err(Failure::with_type(
                "Standard_ProgramError",
                &format!("{INERT_GUARD} ({} {})", T::NAME, self.id.index),
                "",
            )),
        }
    }

    /// `Id()`: the guarded item.
    pub fn id(&self) -> TypedId<T> {
        self.id
    }

    /// `Graph()`: the owning graph, for reads (an inert guard has none: a panic, a null
    /// dereference natively).
    pub fn graph(&self) -> &BRepGraph {
        self.graph.as_deref().expect(INERT_GUARD)
    }

    /// `Graph()`: the owning graph, for the editor calls made inside the scope.
    pub fn graph_mut(&mut self) -> &mut BRepGraph {
        self.graph.as_deref_mut().expect(INERT_GUARD)
    }

    /// `MarkDirty()`: flags a modification the guard did not see (an in-place change of
    /// a shared geometry handle).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// `IsDirty()`: `internal` or `mark_dirty` flagged the item.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// `Internal()`: the mutable record, flagging the guard dirty. External code goes
    /// through the editor's typed setters.
    pub fn internal(&mut self) -> &mut T::Target {
        self.dirty = true;
        let id = self.id;
        T::target_mut(self.graph_mut().inc_storage_mut(), id)
    }

    // The release of `~BRepGraph_MutGuard` and `operator=`: the guard plane cleared,
    // then one notification when dirty; nothing for an inert guard.
    fn release(&mut self) {
        let Some(graph) = self.graph.take() else {
            return;
        };
        graph.inc_storage_mut().clear_guarded(self.id);
        if std::mem::replace(&mut self.dirty, false) {
            T::notify(graph, self.id);
        }
    }
}

impl<T: GuardTag> Deref for MutGuard<'_, T> {
    type Target = T::Target;

    /// `operator->()` / `operator*()` of an active guard (see
    /// [`try_get`](MutGuard::try_get)).
    fn deref(&self) -> &T::Target {
        self.try_get().expect(INERT_GUARD)
    }
}

impl<T: GuardTag> Drop for MutGuard<'_, T> {
    /// `~BRepGraph_MutGuard()`: clears the guard plane, then notifies once if dirty,
    /// so the propagation does not see its own registration.
    fn drop(&mut self) {
        self.release();
    }
}
