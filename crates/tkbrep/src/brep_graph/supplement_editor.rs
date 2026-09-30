// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_SupplementEditor.hxx and
// BRepGraph_SupplementEditor.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_SupplementEditor`: the mutation facade of the runtime supplement
//! attachments.
use super::layer_topo_supplement::{AttachmentKind, LayerTopoSupplement};
use super::*;
use crate::topods::Shape;
use rustycad_tkernel::standard::Failure;

/// `BRepGraph_SupplementEditor`: attaches supplemental shapes to the vertices, edges,
/// faces, shells, solids, compsolids and compounds of one graph.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, supplement_editor::SupplementEditor};
/// use rustycad_tkbrep::topods::Shape;
/// let mut graph = BRepGraph::new();
/// let shell = graph.editor().shells().add();
/// let mut editor = SupplementEditor::new(&mut graph);
/// assert_eq!(editor.attach_to_shell(shell, &Shape::default(), None).unwrap(), 0);
/// ```
#[derive(Debug)]
pub struct SupplementEditor<'g> {
    graph: &'g mut BRepGraph,
}

impl<'g> SupplementEditor<'g> {
    /// `BRepGraph_SupplementEditor(theGraph)`.
    pub fn new(graph: &'g mut BRepGraph) -> Self {
        Self { graph }
    }

    /// `Attach(theOwner, theKind, theShape)`: the attachment uid, 0 when refused (an
    /// owner of an unsupported kind, removed or out of range, or a null shape). The
    /// supplement layer is registered on first use; a null layer handle (another class
    /// under its GUID, a null dereference natively) raises Standard_NullObject.
    pub fn attach(
        &mut self,
        owner: NodeId,
        kind: AttachmentKind,
        shape: &Shape,
    ) -> Result<u64, Failure> {
        if !is_valid_supplement_owner(self.graph, owner) || shape.is_null() {
            return Ok(0);
        }
        let layer = self
            .graph
            .layer_registry()
            .ensure::<LayerTopoSupplement>()?;
        let Some(layer) = layer.get() else {
            return Err(Failure::with_type(
                "Standard_NullObject",
                &format!(
                    "BRepGraph_SupplementEditor::Attach: no LayerTopoSupplement for owner {owner:?}"
                ),
                "",
            ));
        };
        Ok(layer.add_attachment(self.graph, owner, kind, shape))
    }

    /// `AttachToVertex(theVertex, theShape, theKind)`: `kind` defaults to
    /// `VertexSupplementShape`.
    pub fn attach_to_vertex(
        &mut self,
        vertex: VertexId,
        shape: &Shape,
        kind: Option<AttachmentKind>,
    ) -> Result<u64, Failure> {
        self.attach(
            vertex.into(),
            kind.unwrap_or(AttachmentKind::VertexSupplementShape),
            shape,
        )
    }

    /// `AttachToEdge(theEdge, theShape, theKind)`: `kind` defaults to
    /// `EdgeInternalVertex`.
    pub fn attach_to_edge(
        &mut self,
        edge: EdgeId,
        shape: &Shape,
        kind: Option<AttachmentKind>,
    ) -> Result<u64, Failure> {
        self.attach(
            edge.into(),
            kind.unwrap_or(AttachmentKind::EdgeInternalVertex),
            shape,
        )
    }

    /// `AttachToFace(theFace, theShape, theKind)`: `kind` defaults to `FaceDirectVertex`.
    pub fn attach_to_face(
        &mut self,
        face: FaceId,
        shape: &Shape,
        kind: Option<AttachmentKind>,
    ) -> Result<u64, Failure> {
        self.attach(
            face.into(),
            kind.unwrap_or(AttachmentKind::FaceDirectVertex),
            shape,
        )
    }

    /// `AttachToSolid(theSolid, theShape, theKind)`: `kind` defaults to `SolidAuxShape`.
    pub fn attach_to_solid(
        &mut self,
        solid: SolidId,
        shape: &Shape,
        kind: Option<AttachmentKind>,
    ) -> Result<u64, Failure> {
        self.attach(
            solid.into(),
            kind.unwrap_or(AttachmentKind::SolidAuxShape),
            shape,
        )
    }

    /// `AttachToCompSolid(theCompSolid, theShape, theKind)`: `kind` defaults to
    /// `CompSolidAuxShape`.
    pub fn attach_to_comp_solid(
        &mut self,
        comp_solid: CompSolidId,
        shape: &Shape,
        kind: Option<AttachmentKind>,
    ) -> Result<u64, Failure> {
        self.attach(
            comp_solid.into(),
            kind.unwrap_or(AttachmentKind::CompSolidAuxShape),
            shape,
        )
    }

    /// `AttachToShell(theShell, theShape, theKind)`: `kind` defaults to `ShellAuxShape`.
    pub fn attach_to_shell(
        &mut self,
        shell: ShellId,
        shape: &Shape,
        kind: Option<AttachmentKind>,
    ) -> Result<u64, Failure> {
        self.attach(
            shell.into(),
            kind.unwrap_or(AttachmentKind::ShellAuxShape),
            shape,
        )
    }

    /// `AttachToCompound(theCompound, theShape, theKind)`: `kind` defaults to
    /// `CompoundAuxShape`.
    pub fn attach_to_compound(
        &mut self,
        compound: CompoundId,
        shape: &Shape,
        kind: Option<AttachmentKind>,
    ) -> Result<u64, Failure> {
        self.attach(
            compound.into(),
            kind.unwrap_or(AttachmentKind::CompoundAuxShape),
            shape,
        )
    }

    /// `RemoveAttachment(theUid)`: true when the registered supplement layer held the
    /// attachment and removed it.
    pub fn remove_attachment(&mut self, uid: u64) -> bool {
        let layer = self.graph.topo_supplement();
        layer
            .get()
            .is_some_and(|layer| layer.remove_attachment(self.graph, uid))
    }
}

/// `isValidSupplementOwner(theGraph, theOwner)`: a vertex, edge, face, shell, solid,
/// compsolid or compound that `Topo().Gen().IsRemoved` does not report (in range and
/// active).
fn is_valid_supplement_owner(graph: &BRepGraph, owner: NodeId) -> bool {
    if !owner.is_valid() {
        return false;
    }
    let supported = [
        NodeKind::VERTEX,
        NodeKind::EDGE,
        NodeKind::FACE,
        NodeKind::SHELL,
        NodeKind::SOLID,
        NodeKind::COMP_SOLID,
        NodeKind::COMPOUND,
    ];
    supported.contains(&owner.kind) && !graph.topo().generic().is_removed(owner)
}
