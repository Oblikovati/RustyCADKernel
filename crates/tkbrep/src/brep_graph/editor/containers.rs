// Rust translation of EditorView::ShellOps and EditorView::SolidOps of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx, BRepGraph_EditorView_Mut.cxx
// and BRepGraph_EditorView_Setters.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EditorView::ShellOps` and `EditorView::SolidOps`.
use super::guards::{mutate_node, mutate_ref};
use super::helpers::require_settable;
use super::ref_container::{
    ShellFaces, SolidShells, append_many, append_one, remove_many, remove_one,
};
use crate::brep_graph::mut_guard::MutGuard;
use crate::brep_graph::*;
use crate::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Failure;

/// `EditorView::ShellOps`.
#[derive(Debug)]
pub struct ShellOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

/// `EditorView::SolidOps`.
#[derive(Debug)]
pub struct SolidOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

impl<'g> ShellOps<'g> {
    /// `Add()`: an empty shell.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::BRepGraph;
    /// let mut graph = BRepGraph::new();
    /// assert_eq!(graph.editor().shells().add().index, 0);
    /// ```
    pub fn add(&mut self) -> ShellId {
        let shell = self.graph.inc_storage_mut().append_shell();
        self.graph.allocate_uid(shell.into());
        self.graph.mark_modified(shell.into());
        shell
    }

    /// `Append(theShellEntity, theFaceEntity, theOri)`: a face reference under an active
    /// shell; invalid when either is inactive; a locked or guarded shell raises
    /// Standard_ProgramError.
    pub fn append(
        &mut self,
        shell: ShellId,
        face: FaceId,
        orientation: ParityOrientation,
    ) -> Result<FaceRefId, Failure> {
        append_one::<ShellFaces>(self.graph, shell, face, orientation)
    }

    /// `Append(theShellEntity, theFaceIds, theOrientations)`: every face, or none (an
    /// empty `orientations` means all FORWARD).
    pub fn append_many(
        &mut self,
        shell: ShellId,
        faces: &[FaceId],
        orientations: &[ParityOrientation],
    ) -> Result<Vec<FaceRefId>, Failure> {
        append_many::<ShellFaces>(self.graph, shell, faces, orientations)
    }

    /// `RemoveFace(theChildShellId, theFaceRefId)`.
    pub fn remove_face(&mut self, shell: ShellId, reference: FaceRefId) -> Result<bool, Failure> {
        remove_one::<ShellFaces>(self.graph, shell, reference)
    }

    /// `RemoveFaces(theShellId, theFaceRefs)`: all or nothing.
    pub fn remove_faces(
        &mut self,
        shell: ShellId,
        references: &[FaceRefId],
    ) -> Result<bool, Failure> {
        remove_many::<ShellFaces>(self.graph, shell, references)
    }

    /// `Mut(theShell)`.
    pub fn mutate(self, shell: ShellId) -> Result<MutGuard<'g, ShellTag>, Failure> {
        mutate_node(self.graph, shell)
    }

    /// `MutRef(theShellRef)`.
    pub fn mutate_ref(self, reference: ShellRefId) -> Result<MutGuard<'g, ShellRefTag>, Failure> {
        mutate_ref(self.graph, reference)
    }

    /// `SetRefOrientation(theShellRef, theOrientation)`.
    pub fn set_ref_orientation(
        &mut self,
        reference: ShellRefId,
        orientation: ParityOrientation,
    ) -> Result<(), Failure> {
        require_settable(self.graph, reference)?;
        self.graph
            .inc_storage_mut()
            .change_shell_ref(reference)
            .orientation = orientation;
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefOrientation(theMut, theOrientation)`.
    pub fn set_ref_orientation_guarded(
        guard: &mut MutGuard<'_, ShellRefTag>,
        orientation: ParityOrientation,
    ) {
        guard.internal().orientation = orientation;
    }

    /// `SetRefChildShellId(theShellRef, theShell)`: retargets the reference and rebinds
    /// the shell-to-solid relations.
    pub fn set_ref_child_shell_id(
        &mut self,
        reference: ShellRefId,
        shell: ShellId,
    ) -> Result<(), Failure> {
        require_settable(self.graph, reference)?;
        let storage = self.graph.inc_storage_mut();
        let old = storage.shell_ref(reference).child_shell_id;
        if old == shell {
            return Ok(());
        }
        storage.change_shell_ref(reference).child_shell_id = shell;
        storage.rebind_shell_ref(reference, old, shell);
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefChildShellId(theMut, theShell)`.
    pub fn set_ref_child_shell_id_guarded(guard: &mut MutGuard<'_, ShellRefTag>, shell: ShellId) {
        let old = guard.child_shell_id;
        if old == shell {
            return;
        }
        guard.internal().child_shell_id = shell;
        let reference = guard.id();
        guard
            .graph_mut()
            .inc_storage_mut()
            .rebind_shell_ref(reference, old, shell);
    }
}

impl<'g> SolidOps<'g> {
    /// `Add()`: an empty solid.
    pub fn add(&mut self) -> SolidId {
        let solid = self.graph.inc_storage_mut().append_solid();
        self.graph.allocate_uid(solid.into());
        self.graph.mark_modified(solid.into());
        solid
    }

    /// `Append(theSolidEntity, theShellEntity, theOri)`.
    pub fn append(
        &mut self,
        solid: SolidId,
        shell: ShellId,
        orientation: ParityOrientation,
    ) -> Result<ShellRefId, Failure> {
        append_one::<SolidShells>(self.graph, solid, shell, orientation)
    }

    /// `Append(theSolidEntity, theShellIds, theOrientations)`.
    pub fn append_many(
        &mut self,
        solid: SolidId,
        shells: &[ShellId],
        orientations: &[ParityOrientation],
    ) -> Result<Vec<ShellRefId>, Failure> {
        append_many::<SolidShells>(self.graph, solid, shells, orientations)
    }

    /// `RemoveShell(theChildSolidId, theShellRefId)`.
    pub fn remove_shell(&mut self, solid: SolidId, reference: ShellRefId) -> Result<bool, Failure> {
        remove_one::<SolidShells>(self.graph, solid, reference)
    }

    /// `RemoveShells(theSolidId, theShellRefs)`: all or nothing.
    pub fn remove_shells(
        &mut self,
        solid: SolidId,
        references: &[ShellRefId],
    ) -> Result<bool, Failure> {
        remove_many::<SolidShells>(self.graph, solid, references)
    }

    /// `Mut(theSolid)`.
    pub fn mutate(self, solid: SolidId) -> Result<MutGuard<'g, SolidTag>, Failure> {
        mutate_node(self.graph, solid)
    }

    /// `MutRef(theSolidRef)`.
    pub fn mutate_ref(self, reference: SolidRefId) -> Result<MutGuard<'g, SolidRefTag>, Failure> {
        mutate_ref(self.graph, reference)
    }

    /// `SetRefOrientation(theSolidRef, theOrientation)`.
    pub fn set_ref_orientation(
        &mut self,
        reference: SolidRefId,
        orientation: ParityOrientation,
    ) -> Result<(), Failure> {
        require_settable(self.graph, reference)?;
        self.graph
            .inc_storage_mut()
            .change_solid_ref(reference)
            .orientation = orientation;
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefOrientation(theMut, theOrientation)`.
    pub fn set_ref_orientation_guarded(
        guard: &mut MutGuard<'_, SolidRefTag>,
        orientation: ParityOrientation,
    ) {
        guard.internal().orientation = orientation;
    }

    /// `SetRefChildSolidId(theSolidRef, theSolid)`: retargets the reference and rebinds
    /// the solid-to-compsolid relations.
    pub fn set_ref_child_solid_id(
        &mut self,
        reference: SolidRefId,
        solid: SolidId,
    ) -> Result<(), Failure> {
        require_settable(self.graph, reference)?;
        let storage = self.graph.inc_storage_mut();
        let old = storage.solid_ref(reference).child_solid_id;
        if old == solid {
            return Ok(());
        }
        storage.change_solid_ref(reference).child_solid_id = solid;
        storage.rebind_solid_ref(reference, old, solid);
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefChildSolidId(theMut, theSolid)`.
    pub fn set_ref_child_solid_id_guarded(guard: &mut MutGuard<'_, SolidRefTag>, solid: SolidId) {
        let old = guard.child_solid_id;
        if old == solid {
            return;
        }
        guard.internal().child_solid_id = solid;
        let reference = guard.id();
        guard
            .graph_mut()
            .inc_storage_mut()
            .rebind_solid_ref(reference, old, solid);
    }
}
