// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_RefsView.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph::RefsView`: read-only reference entries, their per-parent lists and the
//! generic reference queries.
use crate::brep_graph::*;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// `BRepGraph::RefsView`: obtained by [`BRepGraph::refs`].
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// let graph = BRepGraph::new();
/// assert_eq!(graph.refs().faces().nb(), 0);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct RefsView<'g> {
    graph: &'g BRepGraph,
}

impl BRepGraph {
    /// `Refs()`: the reference view.
    pub fn refs(&self) -> RefsView<'_> {
        RefsView { graph: self }
    }
}

macro_rules! ref_ops {
    ($($(#[$doc:meta])* $ops:ident, $accessor:ident, $id:ident, $entry:ident, $get:ident,
       $nb:ident, $nb_active:ident $(, $parent:ident, $relations:ident, $list:ident)?;)*) => {
        impl<'g> RefsView<'g> {$(
            #[doc = concat!("The grouped `", stringify!($ops), "` queries.")]
            pub fn $accessor(self) -> $ops<'g> {
                $ops { graph: self.graph }
            }
        )*}
        $(
            $(#[$doc])*
            #[derive(Clone, Copy, Debug)]
            pub struct $ops<'g> {
                graph: &'g BRepGraph,
            }

            impl<'g> $ops<'g> {
                /// `Nb()`: the entries, soft-removed ones included.
                pub fn nb(self) -> u32 {
                    self.graph.inc_storage().$nb()
                }

                /// `NbActive()`.
                pub fn nb_active(self) -> u32 {
                    self.graph.inc_storage().$nb_active()
                }

                /// `StartId()`.
                pub fn start_id(self) -> $id {
                    $id::start()
                }

                /// `EndId()`.
                pub fn end_id(self) -> $id {
                    $id::new(self.nb())
                }

                /// `Entry(theRefId)`: the reference entry (an id out of range panics).
                pub fn entry(self, id: $id) -> &'g crate::brep_graph_inc::$entry {
                    self.graph.inc_storage().$get(id)
                }

                $(
                    /// `IdsOf(theParent)`: the parent's ordered references (empty for a
                    /// parent out of range).
                    pub fn ids_of(self, parent: $parent) -> &'g [$id] {
                        let storage = self.graph.inc_storage();
                        if !parent.is_valid_for(storage.nb::<<$parent as KindTagOf>::Tag>()) {
                            return &[];
                        }
                        &storage.$relations(parent).$list
                    }
                )?
            }

            impl CountProvider for $ops<'_> {
                fn count(&self) -> u32 {
                    self.nb()
                }
            }
        )*
    };
}

// The kind tag of a typed id alias, so the macro can name the parent's store.
trait KindTagOf {
    type Tag: StoredTag;
}

impl<T: StoredTag> KindTagOf for TypedId<T> {
    type Tag = T;
}

ref_ops! {
    /// `RefsView::ShellOps`.
    ShellOps, shells, ShellRefId, ShellRef, shell_ref, nb_shell_refs, nb_active_shell_refs,
        SolidId, solid_relations, shell_ref_ids;
    /// `RefsView::FaceOps`.
    FaceOps, faces, FaceRefId, FaceRef, face_ref, nb_face_refs, nb_active_face_refs,
        ShellId, shell_relations, face_ref_ids;
    /// `RefsView::WireOps`.
    WireOps, wires, WireRefId, WireRef, wire_ref, nb_wire_refs, nb_active_wire_refs,
        FaceId, face_relations, wire_ref_ids;
    /// `RefsView::VertexOps`.
    VertexOps, vertices, VertexRefId, VertexRef, vertex_ref, nb_vertex_refs,
        nb_active_vertex_refs;
    /// `RefsView::SolidOps`.
    SolidOps, solids, SolidRefId, SolidRef, solid_ref, nb_solid_refs, nb_active_solid_refs,
        CompSolidId, comp_solid_relations, solid_ref_ids;
    /// `RefsView::ChildOps`.
    ChildOps, children, ChildRefId, ChildRef, child_ref, nb_child_refs, nb_active_child_refs,
        CompoundId, compound_relations, child_ref_ids;
    /// `RefsView::OccurrenceOps`.
    OccurrenceOps, occurrences, OccurrenceRefId, OccurrenceRef, occurrence_ref,
        nb_occurrence_refs, nb_active_occurrence_refs, ProductId, product_relations,
        occurrence_ref_ids;
}

impl<'g> ChildOps<'g> {
    /// `IdsReferencing(theChild)`: the compound references that name the node.
    pub fn ids_referencing(self, child: NodeId) -> &'g [ChildRefId] {
        self.graph.inc_storage().compound_refs_of_node(child)
    }
}

impl<'g> OccurrenceOps<'g> {
    /// `IdsReferencing(theChild)`: the occurrence references that reach the node.
    pub fn ids_referencing(self, child: NodeId) -> &'g [OccurrenceRefId] {
        self.graph.inc_storage().occurrence_refs_of_node(child)
    }
}

impl<'g> RefsView<'g> {
    /// `Gen()`: the generic reference queries.
    pub fn generic(self) -> GenOps<'g> {
        GenOps { graph: self.graph }
    }
}

/// `RefsView::GenOps`: generic reference id queries.
#[derive(Clone, Copy, Debug)]
pub struct GenOps<'g> {
    graph: &'g BRepGraph,
}

impl GenOps<'_> {
    /// `Nb(theKind)`: the references of one kind (0 for an unsupported kind).
    pub fn nb(self, kind: RefKind) -> u32 {
        let storage = self.graph.inc_storage();
        match kind {
            RefKind::SHELL => storage.nb_shell_refs(),
            RefKind::FACE => storage.nb_face_refs(),
            RefKind::WIRE => storage.nb_wire_refs(),
            RefKind::VERTEX => storage.nb_vertex_refs(),
            RefKind::SOLID => storage.nb_solid_refs(),
            RefKind::CHILD => storage.nb_child_refs(),
            RefKind::OCCURRENCE => storage.nb_occurrence_refs(),
            _ => 0,
        }
    }

    /// `IsValid(theRef)`: a valid id inside its kind's range.
    pub fn is_valid(self, reference: RefId) -> bool {
        reference.is_valid() && reference.index < self.nb(reference.kind)
    }

    /// `IsActive(theRef)`.
    pub fn is_active(self, reference: RefId) -> bool {
        self.is_valid(reference) && !reference.is_removed(self.graph)
    }

    /// `IsRemoved(theRef)`: invalid or soft-removed.
    pub fn is_removed(self, reference: RefId) -> bool {
        !self.is_valid(reference) || reference.is_removed(self.graph)
    }

    /// `RefAtStep(theParent, theStep)`: the raw reference at a child step of an active
    /// parent (removed references included); the edge's steps are its start and end
    /// vertex references.
    pub fn ref_at_step(self, parent: NodeId, step: i32) -> RefId {
        if !self.graph.topo().generic().is_active(parent) || step < 0 {
            return RefId::default();
        }
        let storage = self.graph.inc_storage();
        let step = step as usize;
        let pick = |ids: Vec<RefId>| ids.get(step).copied().unwrap_or_default();
        match parent.kind {
            NodeKind::COMPOUND => pick(untyped(
                &storage
                    .compound_relations(CompoundId::from_node_id(parent))
                    .child_ref_ids,
            )),
            NodeKind::COMP_SOLID => pick(untyped(
                &storage
                    .comp_solid_relations(CompSolidId::from_node_id(parent))
                    .solid_ref_ids,
            )),
            NodeKind::SOLID => pick(untyped(
                &storage
                    .solid_relations(SolidId::from_node_id(parent))
                    .shell_ref_ids,
            )),
            NodeKind::SHELL => pick(untyped(
                &storage
                    .shell_relations(ShellId::from_node_id(parent))
                    .face_ref_ids,
            )),
            NodeKind::FACE => pick(untyped(
                &storage
                    .face_relations(FaceId::from_node_id(parent))
                    .wire_ref_ids,
            )),
            NodeKind::EDGE => {
                let def = storage.edge(EdgeId::from_node_id(parent));
                pick(vec![
                    def.start_vertex_ref_id.into(),
                    def.end_vertex_ref_id.into(),
                ])
            }
            NodeKind::PRODUCT => pick(untyped(
                &storage
                    .product_relations(ProductId::from_node_id(parent))
                    .occurrence_ref_ids,
            )),
            _ => RefId::default(),
        }
    }

    /// `ChildNode(theRef)`: the child definition of any reference in range.
    pub fn child_node(self, reference: RefId) -> NodeId {
        if !self.is_valid(reference) {
            return NodeId::default();
        }
        let storage = self.graph.inc_storage();
        match reference.kind {
            RefKind::SHELL => storage
                .shell_ref(ShellRefId::from_ref_id(reference))
                .child_shell_id
                .into(),
            RefKind::FACE => storage
                .face_ref(FaceRefId::from_ref_id(reference))
                .child_face_id
                .into(),
            RefKind::WIRE => storage
                .wire_ref(WireRefId::from_ref_id(reference))
                .child_wire_id
                .into(),
            RefKind::VERTEX => storage
                .vertex_ref(VertexRefId::from_ref_id(reference))
                .child_vertex_id
                .into(),
            RefKind::SOLID => storage
                .solid_ref(SolidRefId::from_ref_id(reference))
                .child_solid_id
                .into(),
            RefKind::CHILD => {
                storage
                    .child_ref(ChildRefId::from_ref_id(reference))
                    .child_node_id
            }
            RefKind::OCCURRENCE => storage
                .occurrence_ref(OccurrenceRefId::from_ref_id(reference))
                .child_occurrence_id
                .into(),
            _ => NodeId::default(),
        }
    }

    /// `LocalLocation(theRef)`: the placement of a child or occurrence reference,
    /// identity for the other kinds and for invalid references.
    pub fn local_location(self, reference: RefId) -> Location {
        if !self.is_valid(reference) {
            return Location::default();
        }
        let storage = self.graph.inc_storage();
        match reference.kind {
            RefKind::CHILD => storage
                .child_ref(ChildRefId::from_ref_id(reference))
                .local_location
                .clone(),
            RefKind::OCCURRENCE => storage
                .occurrence_ref(OccurrenceRefId::from_ref_id(reference))
                .local_location
                .clone(),
            _ => Location::default(),
        }
    }

    /// `Orientation(theRef)`: the stored orientation; FORWARD for occurrence and invalid
    /// references.
    pub fn orientation(self, reference: RefId) -> Orientation {
        if !self.is_valid(reference) {
            return Orientation::Forward;
        }
        let storage = self.graph.inc_storage();
        let parity = match reference.kind {
            RefKind::SHELL => {
                storage
                    .shell_ref(ShellRefId::from_ref_id(reference))
                    .orientation
            }
            RefKind::FACE => {
                storage
                    .face_ref(FaceRefId::from_ref_id(reference))
                    .orientation
            }
            RefKind::WIRE => {
                storage
                    .wire_ref(WireRefId::from_ref_id(reference))
                    .orientation
            }
            RefKind::VERTEX => {
                storage
                    .vertex_ref(VertexRefId::from_ref_id(reference))
                    .orientation
            }
            RefKind::SOLID => {
                storage
                    .solid_ref(SolidRefId::from_ref_id(reference))
                    .orientation
            }
            RefKind::CHILD => {
                storage
                    .child_ref(ChildRefId::from_ref_id(reference))
                    .orientation
            }
            _ => return Orientation::Forward,
        };
        parity.orientation()
    }
}

fn untyped<T: KindTag<Family = RefFamily>>(ids: &[TypedId<T>]) -> Vec<RefId> {
    ids.iter().map(|id| RefId::from(*id)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph_inc::ParityOrientation;

    #[test]
    fn generic_reference_queries() {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        let (shell, face) = (storage.append_shell(), storage.append_face());
        let face_ref = storage.attach_face_to_shell(shell, face, ParityOrientation::REVERSED);
        let refs = graph.refs();
        assert_eq!(refs.faces().ids_of(shell), &[face_ref]);
        assert!(refs.faces().ids_of(ShellId::new(4)).is_empty());
        let generic = refs.generic();
        assert_eq!(generic.ref_at_step(shell.into(), 0), RefId::from(face_ref));
        assert!(!generic.ref_at_step(shell.into(), 1).is_valid());
        assert_eq!(generic.child_node(face_ref.into()), NodeId::from(face));
        assert_eq!(generic.orientation(face_ref.into()), Orientation::Reversed);
        assert!(generic.local_location(face_ref.into()).is_identity());
        assert!(generic.is_active(face_ref.into()));
        assert_eq!(generic.nb(RefKind(8)), 0);
        assert_eq!(refs.faces().entry(face_ref).child_face_id, face);
    }
}
