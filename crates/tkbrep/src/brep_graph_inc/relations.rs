// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Relations.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The relation tables: ordered child uses and incoming parent uses of each definition,
//! kept outside the definition records.
use crate::brep_graph::{
    ChildRefId, CoEdgeId, EdgeId, FaceRefId, OccurrenceRefId, ShellRefId, SolidRefId, WireRefId,
};

/// `BRepGraphInc::FaceRelations`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FaceRelations {
    /// `WireRefIds`: the wire references owned by the face.
    pub wire_ref_ids: Vec<WireRefId>,
    /// `ParentFaceRefIds`: the face references that use the face.
    pub parent_face_ref_ids: Vec<FaceRefId>,
}

/// `BRepGraphInc::WireRelations`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WireRelations {
    /// `CoEdgeIds`: the ordered coedges of the wire.
    pub co_edge_ids: Vec<CoEdgeId>,
    /// `ParentWireRefIds`: the wire references that use the wire.
    pub parent_wire_ref_ids: Vec<WireRefId>,
}

/// `BRepGraphInc::EdgeRelations`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EdgeRelations {
    /// `CoEdgeIds`: the coedges that use the edge.
    pub co_edge_ids: Vec<CoEdgeId>,
}

/// `BRepGraphInc::ShellRelations`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShellRelations {
    /// `FaceRefIds`.
    pub face_ref_ids: Vec<FaceRefId>,
    /// `ParentShellRefIds`.
    pub parent_shell_ref_ids: Vec<ShellRefId>,
}

/// `BRepGraphInc::SolidRelations`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SolidRelations {
    /// `ShellRefIds`.
    pub shell_ref_ids: Vec<ShellRefId>,
    /// `ParentSolidRefIds`.
    pub parent_solid_ref_ids: Vec<SolidRefId>,
}

/// `BRepGraphInc::CompoundRelations`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompoundRelations {
    /// `ChildRefIds`.
    pub child_ref_ids: Vec<ChildRefId>,
}

/// `BRepGraphInc::CompSolidRelations`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompSolidRelations {
    /// `SolidRefIds`.
    pub solid_ref_ids: Vec<SolidRefId>,
}

/// `BRepGraphInc::VertexRelations`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VertexRelations {
    /// `EdgeIds`: the edges that share the vertex.
    pub edge_ids: Vec<EdgeId>,
}

/// `BRepGraphInc::ProductRelations`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProductRelations {
    /// `OccurrenceRefIds`.
    pub occurrence_ref_ids: Vec<OccurrenceRefId>,
}

/// `BRepGraphInc::OccurrenceRelations`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OccurrenceRelations {
    /// `ParentOccurrenceRefIds`.
    pub parent_occurrence_ref_ids: Vec<OccurrenceRefId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relations_start_empty() {
        assert!(FaceRelations::default().wire_ref_ids.is_empty());
        assert!(WireRelations::default().co_edge_ids.is_empty());
        assert!(EdgeRelations::default().co_edge_ids.is_empty());
        assert!(ShellRelations::default().face_ref_ids.is_empty());
        assert!(SolidRelations::default().shell_ref_ids.is_empty());
        assert!(CompoundRelations::default().child_ref_ids.is_empty());
        assert!(CompSolidRelations::default().solid_ref_ids.is_empty());
        assert!(VertexRelations::default().edge_ids.is_empty());
        assert!(ProductRelations::default().occurrence_ref_ids.is_empty());
        assert!(
            OccurrenceRelations::default()
                .parent_occurrence_ref_ids
                .is_empty()
        );
    }
}
