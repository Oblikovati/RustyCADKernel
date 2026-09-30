// Rust translation of the Append (one and many), Remove* and batch Remove* members
// that ShellOps, SolidOps, CompoundOps and CompSolidOps repeat in
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The four reference containers (shell, solid, compound, compsolid) share one body
//! per operation; [`RefContainer`] names what differs between them.
use super::helpers::{is_active_node, node_item, require_mutable};
use super::removals::detach_ordered_parent_ref;
use crate::brep_graph::*;
use crate::brep_graph_inc::{NodeTag, ParityOrientation, RefTag, Storage};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;

/// What distinguishes one reference container of the editor from another.
pub(crate) trait RefContainer {
    /// The container kind (`ShellTag`...).
    type Parent: NodeTag;
    /// The reference kind (`FaceRefTag`...).
    type Ref: RefTag;
    /// The child id (`FaceId`, or `NodeId` for a compound).
    type Child: Copy + Into<NodeId>;
    /// The kind word of the lock texts (`shell`, `compsolid`...).
    const NAME: &'static str;
    /// `Attach<Child>To<Parent>(parent, child, orientation)`.
    fn attach(
        storage: &mut Storage,
        parent: TypedId<Self::Parent>,
        child: Self::Child,
        orientation: ParityOrientation,
    ) -> TypedId<Self::Ref>;
    /// The parent's ordered references.
    fn ref_ids(storage: &Storage, parent: TypedId<Self::Parent>) -> &[TypedId<Self::Ref>];
    /// The child of a reference.
    fn ref_child(storage: &Storage, reference: TypedId<Self::Ref>) -> NodeId;
    /// The child check of `Append` (active; an active topology node for a compound).
    fn is_usable_child(graph: &BRepGraph, child: Self::Child) -> bool {
        is_active_node(graph, child.into())
    }
}

// The lock checks of every append: the parent is unlocked and unguarded.
fn require_parent<C: RefContainer>(
    graph: &BRepGraph,
    parent: TypedId<C::Parent>,
) -> Result<(), Failure>
where
    TypedId<C::Parent>: Into<NodeId>,
{
    require_mutable(
        graph,
        node_item(parent),
        &format!("BRepGraph::EditorView::Append(): locked {}", C::NAME),
        &format!(
            "BRepGraph::EditorView::Append(): guard active on {}",
            C::NAME
        ),
    )
}

/// `Append(theParent, theChild, theOri)`: a reference under an active parent; invalid
/// when either is inactive; a locked or guarded parent raises Standard_ProgramError.
pub(crate) fn append_one<C: RefContainer>(
    graph: &mut BRepGraph,
    parent: TypedId<C::Parent>,
    child: C::Child,
    orientation: ParityOrientation,
) -> Result<TypedId<C::Ref>, Failure>
where
    TypedId<C::Parent>: Into<NodeId>,
    TypedId<C::Ref>: Into<RefId>,
{
    if !is_active_node(graph, parent) || !C::is_usable_child(graph, child) {
        return Ok(TypedId::default());
    }
    require_parent::<C>(graph, parent)?;
    Ok(link::<C>(graph, parent, child, orientation))
}

// Attach, then the reference's UID and modification.
fn link<C: RefContainer>(
    graph: &mut BRepGraph,
    parent: TypedId<C::Parent>,
    child: C::Child,
    orientation: ParityOrientation,
) -> TypedId<C::Ref>
where
    TypedId<C::Ref>: Into<RefId>,
{
    let reference = C::attach(graph.inc_storage_mut(), parent, child, orientation);
    graph.allocate_ref_uid(reference.into());
    graph.mark_ref_modified(reference.into());
    reference
}

/// `Append(theParent, theChildIds, theOrientations)`: two passes; empty when the
/// orientations are given with another count, the parent is inactive or a child is
/// not usable; a locked or guarded parent raises Standard_ProgramError (after the
/// parent check, before the children are checked).
pub(crate) fn append_many<C: RefContainer>(
    graph: &mut BRepGraph,
    parent: TypedId<C::Parent>,
    children: &[C::Child],
    orientations: &[ParityOrientation],
) -> Result<Vec<TypedId<C::Ref>>, Failure>
where
    TypedId<C::Parent>: Into<NodeId>,
    TypedId<C::Ref>: Into<RefId>,
{
    if !orientations.is_empty() && orientations.len() != children.len() {
        return Ok(Vec::new());
    }
    if !is_active_node(graph, parent) {
        return Ok(Vec::new());
    }
    require_parent::<C>(graph, parent)?;
    if !children
        .iter()
        .all(|child| C::is_usable_child(graph, *child))
    {
        return Ok(Vec::new());
    }
    let orientation = |index: usize| orientations.get(index).copied().unwrap_or_default();
    let linked = children.iter().enumerate();
    Ok(linked
        .map(|(index, child)| link::<C>(graph, parent, *child, orientation(index)))
        .collect())
}

/// `Remove<Child>(theParent, theRefId)`: detaches one active use listed by an active
/// parent whose child is active (removing the child subgraph when it has no other
/// use); false otherwise, and for ids out of range (the release behaviour of the
/// native assertions).
pub(crate) fn remove_one<C: RefContainer>(
    graph: &mut BRepGraph,
    parent: TypedId<C::Parent>,
    reference: TypedId<C::Ref>,
) -> Result<bool, Failure>
where
    TypedId<C::Parent>: Into<NodeId>,
    TypedId<C::Ref>: Into<RefId>,
{
    let storage = graph.inc_storage();
    if !parent.is_valid_for(storage.nb::<C::Parent>())
        || !reference.is_valid_for(storage.nb::<C::Ref>())
        || !is_active_node(graph, parent)
    {
        return Ok(false);
    }
    let child = C::ref_child(storage, reference);
    let listed = C::ref_ids(storage, parent).contains(&reference);
    if storage.is_removed(reference)
        || !listed
        || !child.is_valid()
        || !is_active_node(graph, child)
    {
        return Ok(false);
    }
    if !detach_ordered_parent_ref(graph, reference.into(), listed, child, true)? {
        return Ok(false);
    }
    graph.mark_modified(parent.into());
    Ok(true)
}

/// `Remove<Children>(theParent, theRefs)`: all or nothing; every reference is checked
/// (in range, active, listed by the active parent) before the first removal, then
/// each is removed in order (false at the first refusal).
pub(crate) fn remove_many<C: RefContainer>(
    graph: &mut BRepGraph,
    parent: TypedId<C::Parent>,
    references: &[TypedId<C::Ref>],
) -> Result<bool, Failure>
where
    TypedId<C::Parent>: Into<NodeId>,
    TypedId<C::Ref>: Into<RefId>,
{
    if !is_active_node(graph, parent) {
        return Ok(false);
    }
    let storage = graph.inc_storage();
    let valid = |reference: &TypedId<C::Ref>| {
        reference.is_valid_for(storage.nb::<C::Ref>())
            && !storage.is_removed(*reference)
            && C::ref_ids(storage, parent).contains(reference)
    };
    if !references.iter().all(valid) {
        return Ok(false);
    }
    for reference in references {
        if !remove_one::<C>(graph, parent, *reference)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Shell faces.
pub(crate) struct ShellFaces;
/// Solid shells.
pub(crate) struct SolidShells;
/// Compound children.
pub(crate) struct CompoundChildren;
/// CompSolid solids.
pub(crate) struct CompSolidSolids;

impl RefContainer for ShellFaces {
    type Parent = ShellTag;
    type Ref = FaceRefTag;
    type Child = FaceId;
    const NAME: &'static str = "shell";
    fn attach(
        storage: &mut Storage,
        parent: ShellId,
        child: FaceId,
        orientation: ParityOrientation,
    ) -> FaceRefId {
        storage.attach_face_to_shell(parent, child, orientation)
    }
    fn ref_ids(storage: &Storage, parent: ShellId) -> &[FaceRefId] {
        &storage.shell_relations(parent).face_ref_ids
    }
    fn ref_child(storage: &Storage, reference: FaceRefId) -> NodeId {
        storage.face_ref(reference).child_face_id.into()
    }
}

impl RefContainer for SolidShells {
    type Parent = SolidTag;
    type Ref = ShellRefTag;
    type Child = ShellId;
    const NAME: &'static str = "solid";
    fn attach(
        storage: &mut Storage,
        parent: SolidId,
        child: ShellId,
        orientation: ParityOrientation,
    ) -> ShellRefId {
        storage.attach_shell_to_solid(parent, child, orientation)
    }
    fn ref_ids(storage: &Storage, parent: SolidId) -> &[ShellRefId] {
        &storage.solid_relations(parent).shell_ref_ids
    }
    fn ref_child(storage: &Storage, reference: ShellRefId) -> NodeId {
        storage.shell_ref(reference).child_shell_id.into()
    }
}

impl RefContainer for CompoundChildren {
    type Parent = CompoundTag;
    type Ref = ChildRefTag;
    type Child = NodeId;
    const NAME: &'static str = "compound";
    fn attach(
        storage: &mut Storage,
        parent: CompoundId,
        child: NodeId,
        orientation: ParityOrientation,
    ) -> ChildRefId {
        storage.attach_child_to_compound(parent, child, &Location::default(), orientation)
    }
    fn ref_ids(storage: &Storage, parent: CompoundId) -> &[ChildRefId] {
        &storage.compound_relations(parent).child_ref_ids
    }
    fn ref_child(storage: &Storage, reference: ChildRefId) -> NodeId {
        storage.child_ref(reference).child_node_id
    }
    fn is_usable_child(graph: &BRepGraph, child: NodeId) -> bool {
        super::helpers::is_active_topology_node(graph, child)
    }
}

impl RefContainer for CompSolidSolids {
    type Parent = CompSolidTag;
    type Ref = SolidRefTag;
    type Child = SolidId;
    const NAME: &'static str = "compsolid";
    fn attach(
        storage: &mut Storage,
        parent: CompSolidId,
        child: SolidId,
        orientation: ParityOrientation,
    ) -> SolidRefId {
        storage.attach_solid_to_comp_solid(parent, child, orientation)
    }
    fn ref_ids(storage: &Storage, parent: CompSolidId) -> &[SolidRefId] {
        &storage.comp_solid_relations(parent).solid_ref_ids
    }
    fn ref_child(storage: &Storage, reference: SolidRefId) -> NodeId {
        storage.solid_ref(reference).child_solid_id.into()
    }
}
