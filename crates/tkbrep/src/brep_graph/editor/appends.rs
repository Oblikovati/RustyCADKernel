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
// Rust translation of EditorView::{Shell,Solid,Compound,CompSolid}Ops::Append(one child)
// of ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx and
// EditorView::GenOps::SetChildRefLocalLocation(ref, loc) of BRepGraph_EditorView_Setters.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The container-append operations of the editor used by shape ingestion.
use super::{is_active_node, is_active_topology_node, require_mutable, require_unlocked};
use crate::brep_graph::*;
use crate::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;

macro_rules! editor_ops {
    ($($(#[$doc:meta])* $ops:ident;)*) => {$(
        $(#[$doc])*
        #[derive(Debug)]
        pub struct $ops<'g> {
            pub(super) graph: &'g mut BRepGraph,
        }
    )*};
}

editor_ops! {
    /// `EditorView::ShellOps`.
    ShellOps;
    /// `EditorView::SolidOps`.
    SolidOps;
    /// `EditorView::CompoundOps`.
    CompoundOps;
    /// `EditorView::CompSolidOps`.
    CompSolidOps;
    /// `EditorView::GenOps`.
    GenOps;
}

// The shared tail of the one-child appends: UID of the new reference, then the
// reference modification.
fn finish_append(graph: &mut BRepGraph, reference: RefId) {
    graph.allocate_ref_uid(reference);
    graph.mark_ref_modified(reference);
}

impl ShellOps<'_> {
    /// `Append(theShellEntity, theFaceEntity, theOri)`: a face reference under an active
    /// shell; invalid when either is inactive; a locked or guarded shell raises
    /// Standard_ProgramError.
    pub fn append(
        &mut self,
        shell: ShellId,
        face: FaceId,
        orientation: ParityOrientation,
    ) -> Result<FaceRefId, Failure> {
        if !is_active_node(self.graph, shell.into()) || !is_active_node(self.graph, face.into()) {
            return Ok(FaceRefId::default());
        }
        require_mutable(
            self.graph,
            shell.into(),
            "BRepGraph::EditorView::Append(): locked shell",
            "BRepGraph::EditorView::Append(): guard active on shell",
        )?;
        let reference = self
            .graph
            .inc_storage_mut()
            .attach_face_to_shell(shell, face, orientation);
        finish_append(self.graph, reference.into());
        Ok(reference)
    }
}

impl SolidOps<'_> {
    /// `Append(theSolidEntity, theShellEntity, theOri)`.
    pub fn append(
        &mut self,
        solid: SolidId,
        shell: ShellId,
        orientation: ParityOrientation,
    ) -> Result<ShellRefId, Failure> {
        if !is_active_node(self.graph, solid.into()) || !is_active_node(self.graph, shell.into()) {
            return Ok(ShellRefId::default());
        }
        require_mutable(
            self.graph,
            solid.into(),
            "BRepGraph::EditorView::Append(): locked solid",
            "BRepGraph::EditorView::Append(): guard active on solid",
        )?;
        let reference =
            self.graph
                .inc_storage_mut()
                .attach_shell_to_solid(solid, shell, orientation);
        finish_append(self.graph, reference.into());
        Ok(reference)
    }
}

impl CompoundOps<'_> {
    /// `Append(theCompoundEntity, theChildEntity, theOri)`: a child reference at the
    /// identity location under an active compound; the child must be an active
    /// topology node.
    pub fn append(
        &mut self,
        compound: CompoundId,
        child: NodeId,
        orientation: ParityOrientation,
    ) -> Result<ChildRefId, Failure> {
        if !is_active_node(self.graph, compound.into())
            || !is_active_topology_node(self.graph, child)
        {
            return Ok(ChildRefId::default());
        }
        require_mutable(
            self.graph,
            compound.into(),
            "BRepGraph::EditorView::Append(): locked compound",
            "BRepGraph::EditorView::Append(): guard active on compound",
        )?;
        let storage = self.graph.inc_storage_mut();
        let reference =
            storage.attach_child_to_compound(compound, child, &Location::default(), orientation);
        finish_append(self.graph, reference.into());
        Ok(reference)
    }
}

impl CompSolidOps<'_> {
    /// `Append(theCompSolidEntity, theSolidEntity, theOri)`.
    pub fn append(
        &mut self,
        comp_solid: CompSolidId,
        solid: SolidId,
        orientation: ParityOrientation,
    ) -> Result<SolidRefId, Failure> {
        if !is_active_node(self.graph, comp_solid.into())
            || !is_active_node(self.graph, solid.into())
        {
            return Ok(SolidRefId::default());
        }
        require_mutable(
            self.graph,
            comp_solid.into(),
            "BRepGraph::EditorView::Append(): locked compsolid",
            "BRepGraph::EditorView::Append(): guard active on compsolid",
        )?;
        let storage = self.graph.inc_storage_mut();
        let reference = storage.attach_solid_to_comp_solid(comp_solid, solid, orientation);
        finish_append(self.graph, reference.into());
        Ok(reference)
    }
}

impl GenOps<'_> {
    /// `SetChildRefLocalLocation(theChildRef, theLoc)`: a locked reference raises
    /// Standard_ProgramError; a reference out of range panics (undefined natively).
    pub fn set_child_ref_local_location(
        &mut self,
        child_ref: ChildRefId,
        location: &Location,
    ) -> Result<(), Failure> {
        require_unlocked(
            self.graph,
            RefId::from(child_ref).into(),
            "BRepGraph::EditorView: locked item",
        )?;
        self.graph
            .inc_storage_mut()
            .change_child_ref(child_ref)
            .local_location = location.clone();
        self.graph.mark_ref_modified(child_ref.into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_attach_and_mark() {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        let (shell, face, solid) = (
            storage.append_shell(),
            storage.append_face(),
            storage.append_solid(),
        );
        let (compound, comp_solid) = (storage.append_compound(), storage.append_comp_solid());
        let face_ref = graph
            .editor()
            .shells()
            .append(shell, face, ParityOrientation::FORWARD)
            .unwrap();
        assert_eq!(graph.refs().faces().ids_of(shell), &[face_ref]);
        assert_eq!(graph.inc_storage().shell(shell).base.own_gen, 1);
        let shell_ref = graph
            .editor()
            .solids()
            .append(solid, shell, ParityOrientation::REVERSED)
            .unwrap();
        assert!(shell_ref.is_valid());
        let solid_ref = graph
            .editor()
            .comp_solids()
            .append(comp_solid, solid, ParityOrientation::FORWARD)
            .unwrap();
        assert!(solid_ref.is_valid());
        let child = graph
            .editor()
            .compounds()
            .append(compound, solid.into(), ParityOrientation::FORWARD)
            .unwrap();
        let location = Location::new(rustycad_tkmath::gp::Transform3::default());
        graph
            .editor()
            .generic()
            .set_child_ref_local_location(child, &location)
            .unwrap();
        assert!(
            !graph
                .refs()
                .children()
                .entry(child)
                .local_location
                .is_identity()
        );
        let refused = graph.editor().compounds().append(
            compound,
            NodeId::default(),
            ParityOrientation::FORWARD,
        );
        assert!(!refused.unwrap().is_valid());
    }
}
