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
// Rust translation of EditorView::CompoundOps and EditorView::CompSolidOps of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx and
// BRepGraph_EditorView_Mut.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EditorView::CompoundOps` and `EditorView::CompSolidOps`.
use super::guards::mutate_node;
use super::helpers::{is_active_node, is_active_topology_node, node_item, require_mutable};
use super::ref_container::{
    CompSolidSolids, CompoundChildren, append_many, append_one, remove_many, remove_one,
};
use crate::brep_graph::mut_guard::MutGuard;
use crate::brep_graph::*;
use crate::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;

/// `EditorView::CompoundOps`.
#[derive(Debug)]
pub struct CompoundOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

/// `EditorView::CompSolidOps`.
#[derive(Debug)]
pub struct CompSolidOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

impl<'g> CompoundOps<'g> {
    /// `Add(theChildEntities)`: a compound with one FORWARD child reference per child at
    /// the identity location; the invalid id when a child is not an active topology
    /// node.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::BRepGraph;
    /// let mut graph = BRepGraph::new();
    /// let compound = graph.editor().compounds().add(&[]);
    /// assert!(compound.is_valid());
    /// ```
    pub fn add(&mut self, children: &[NodeId]) -> CompoundId {
        if !children
            .iter()
            .all(|child| is_active_topology_node(self.graph, *child))
        {
            return CompoundId::default();
        }
        let compound = self.graph.inc_storage_mut().append_compound();
        self.graph.allocate_uid(compound.into());
        for child in children {
            let storage = self.graph.inc_storage_mut();
            let reference = storage.attach_child_to_compound(
                compound,
                *child,
                &Location::default(),
                ParityOrientation::FORWARD,
            );
            self.graph.allocate_ref_uid(reference.into());
        }
        self.graph.mark_modified(compound.into());
        compound
    }

    /// `Append(theCompoundEntity, theChildEntity, theOri)`: a child reference at the
    /// identity location under an active compound; the child must be an active
    /// topology node.
    pub fn append(
        &mut self,
        compound: CompoundId,
        child: NodeId,
        orientation: ParityOrientation,
    ) -> Result<ChildRefId, Failure> {
        append_one::<CompoundChildren>(self.graph, compound, child, orientation)
    }

    /// `Append(theCompoundEntity, theChildIds, theOrientations)`.
    pub fn append_many(
        &mut self,
        compound: CompoundId,
        children: &[NodeId],
        orientations: &[ParityOrientation],
    ) -> Result<Vec<ChildRefId>, Failure> {
        append_many::<CompoundChildren>(self.graph, compound, children, orientations)
    }

    /// `RemoveChild(theCompoundDefId, theChildRefId)`.
    pub fn remove_child(
        &mut self,
        compound: CompoundId,
        reference: ChildRefId,
    ) -> Result<bool, Failure> {
        remove_one::<CompoundChildren>(self.graph, compound, reference)
    }

    /// `RemoveChildren(theCompoundId, theChildRefs)`: all or nothing.
    pub fn remove_children(
        &mut self,
        compound: CompoundId,
        references: &[ChildRefId],
    ) -> Result<bool, Failure> {
        remove_many::<CompoundChildren>(self.graph, compound, references)
    }

    /// `Mut(theCompound)`.
    pub fn mutate(self, compound: CompoundId) -> Result<MutGuard<'g, CompoundTag>, Failure> {
        mutate_node(self.graph, compound)
    }

    /// `ReplaceChild(theChildRef, theNewChild)`: the lock and guard checks of the parent
    /// compound, then `Gen().SetChildRefChildNodeId`.
    pub fn replace_child(&mut self, reference: ChildRefId, child: NodeId) -> Result<(), Failure> {
        let parent = self
            .graph
            .inc_storage()
            .child_ref(reference)
            .parent_compound_id;
        require_mutable(
            self.graph,
            node_item(parent),
            "BRepGraph::EditorView::ReplaceChild(): locked compound",
            "BRepGraph::EditorView::ReplaceChild(): guard active on compound",
        )?;
        self.graph
            .editor()
            .generic()
            .set_child_ref_child_node_id(reference, child)
    }
}

impl<'g> CompSolidOps<'g> {
    /// `Add(theSolidEntities)`: a compsolid with one FORWARD solid reference per solid;
    /// the invalid id when a solid is not active.
    pub fn add(&mut self, solids: &[SolidId]) -> CompSolidId {
        if !solids
            .iter()
            .all(|solid| is_active_node(self.graph, *solid))
        {
            return CompSolidId::default();
        }
        let comp_solid = self.graph.inc_storage_mut().append_comp_solid();
        self.graph.allocate_uid(comp_solid.into());
        for solid in solids {
            let storage = self.graph.inc_storage_mut();
            let reference =
                storage.attach_solid_to_comp_solid(comp_solid, *solid, ParityOrientation::FORWARD);
            self.graph.allocate_ref_uid(reference.into());
        }
        self.graph.mark_modified(comp_solid.into());
        comp_solid
    }

    /// `Append(theCompSolidEntity, theSolidEntity, theOri)`.
    pub fn append(
        &mut self,
        comp_solid: CompSolidId,
        solid: SolidId,
        orientation: ParityOrientation,
    ) -> Result<SolidRefId, Failure> {
        append_one::<CompSolidSolids>(self.graph, comp_solid, solid, orientation)
    }

    /// `Append(theCompSolidEntity, theSolidIds, theOrientations)`.
    pub fn append_many(
        &mut self,
        comp_solid: CompSolidId,
        solids: &[SolidId],
        orientations: &[ParityOrientation],
    ) -> Result<Vec<SolidRefId>, Failure> {
        append_many::<CompSolidSolids>(self.graph, comp_solid, solids, orientations)
    }

    /// `RemoveSolid(theCompChildSolidId, theSolidRefId)`.
    pub fn remove_solid(
        &mut self,
        comp_solid: CompSolidId,
        reference: SolidRefId,
    ) -> Result<bool, Failure> {
        remove_one::<CompSolidSolids>(self.graph, comp_solid, reference)
    }

    /// `RemoveSolids(theCompSolidId, theSolidRefs)`: all or nothing.
    pub fn remove_solids(
        &mut self,
        comp_solid: CompSolidId,
        references: &[SolidRefId],
    ) -> Result<bool, Failure> {
        remove_many::<CompSolidSolids>(self.graph, comp_solid, references)
    }

    /// `Mut(theCompSolid)`.
    pub fn mutate(self, comp_solid: CompSolidId) -> Result<MutGuard<'g, CompSolidTag>, Failure> {
        mutate_node(self.graph, comp_solid)
    }

    /// `ReplaceSolid(theSolidRef, theNewSolid)`: the lock and guard checks of the parent
    /// compsolid, then `Solids().SetRefChildSolidId`.
    pub fn replace_solid(&mut self, reference: SolidRefId, solid: SolidId) -> Result<(), Failure> {
        let parent = self
            .graph
            .inc_storage()
            .solid_ref(reference)
            .parent_comp_solid_id;
        require_mutable(
            self.graph,
            node_item(parent),
            "BRepGraph::EditorView::ReplaceSolid(): locked compsolid",
            "BRepGraph::EditorView::ReplaceSolid(): guard active on compsolid",
        )?;
        self.graph
            .editor()
            .solids()
            .set_ref_child_solid_id(reference, solid)
    }
}
