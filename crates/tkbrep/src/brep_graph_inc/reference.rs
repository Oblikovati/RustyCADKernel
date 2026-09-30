// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Reference.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reference entries (`BRepGraphInc::*Ref`): the typed parent-child uses.
use super::parity_orientation::ParityOrientation;
use crate::brep_graph::{
    CompSolidId, CompoundId, EdgeId, FaceId, NodeId, OccurrenceId, ProductId, ShellId, SolidId,
    VertexId, WireId,
};
use rustycad_tkmath::toploc::Location;

/// `BRepGraphInc::BaseRef`: the fields shared by every reference entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BaseRef {
    /// `UID`: the per-kind reference UID counter (0 until allocated).
    pub uid: u32,
}

/// Access to the [`BaseRef`] part of a reference entry.
pub trait HasBaseRef {
    /// The base fields.
    fn base(&self) -> &BaseRef;
    /// The mutable base fields.
    fn base_mut(&mut self) -> &mut BaseRef;
}

impl HasBaseRef for BaseRef {
    fn base(&self) -> &BaseRef {
        self
    }
    fn base_mut(&mut self) -> &mut BaseRef {
        self
    }
}

macro_rules! references {
    ($($(#[$doc:meta])* $type:ident {
        $($(#[$field_doc:meta])* $field:ident: $field_type:ty),* $(,)?
    })*) => {$(
        $(#[$doc])*
        #[derive(Clone, Debug, Default)]
        pub struct $type {
            /// The base fields.
            pub base: BaseRef,
            $($(#[$field_doc])* pub $field: $field_type,)*
        }
        impl HasBaseRef for $type {
            fn base(&self) -> &BaseRef {
                &self.base
            }
            fn base_mut(&mut self) -> &mut BaseRef {
                &mut self.base
            }
        }
    )*};
}

references! {
    /// `BRepGraphInc::ShellRef`.
    ShellRef {
        /// `ParentSolidId`.
        parent_solid_id: SolidId,
        /// `ChildShellId`.
        child_shell_id: ShellId,
        /// `Orientation`.
        orientation: ParityOrientation,
    }
    /// `BRepGraphInc::FaceRef`.
    FaceRef {
        /// `ParentShellId`.
        parent_shell_id: ShellId,
        /// `ChildFaceId`.
        child_face_id: FaceId,
        /// `Orientation`.
        orientation: ParityOrientation,
    }
    /// `BRepGraphInc::WireRef`.
    WireRef {
        /// `ParentFaceId`.
        parent_face_id: FaceId,
        /// `ChildWireId`.
        child_wire_id: WireId,
        /// `Orientation`.
        orientation: ParityOrientation,
    }
    /// `BRepGraphInc::VertexRef`.
    VertexRef {
        /// `ChildVertexId`.
        child_vertex_id: VertexId,
        /// `ParentEdgeId`: the edge that owns the reference.
        parent_edge_id: EdgeId,
        /// `Orientation`.
        orientation: ParityOrientation,
    }
    /// `BRepGraphInc::SolidRef`.
    SolidRef {
        /// `ParentCompSolidId`.
        parent_comp_solid_id: CompSolidId,
        /// `ChildSolidId`.
        child_solid_id: SolidId,
        /// `Orientation`.
        orientation: ParityOrientation,
    }
    /// `BRepGraphInc::ChildRef`: a compound child of any kind, with a placement.
    ChildRef {
        /// `ParentCompoundId`.
        parent_compound_id: CompoundId,
        /// `ChildNodeId`.
        child_node_id: NodeId,
        /// `Orientation`.
        orientation: ParityOrientation,
        /// `LocalLocation`: the placement relative to the parent.
        local_location: Location,
    }
    /// `BRepGraphInc::OccurrenceRef`: an occurrence of a product, with a placement.
    OccurrenceRef {
        /// `ParentProductId`.
        parent_product_id: ProductId,
        /// `ChildOccurrenceId`.
        child_occurrence_id: OccurrenceId,
        /// `LocalLocation`: the placement relative to the parent product.
        local_location: Location,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_default_to_forward_and_identity() {
        let child = ChildRef::default();
        assert!(child.local_location.is_identity());
        assert!(!child.orientation.is_reversed);
        let mut occurrence = OccurrenceRef::default();
        occurrence.base_mut().uid = 3;
        assert_eq!(occurrence.base().uid, 3);
        assert!(!VertexRef::default().child_vertex_id.is_valid());
    }
}
