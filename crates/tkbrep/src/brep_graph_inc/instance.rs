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
// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Instance.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraphInc::Instance<T>`: a definition id with the location and orientation of
//! one use, the BRepGraph analogue of a `TopoDS_Shape`.
use crate::brep_graph::ids::hashing::{combine, hash_i32};
use crate::brep_graph::{
    CoEdgeId, CompSolidId, CompoundId, FaceId, NodeId, OccurrenceId, ProductId, ShellId, SolidId,
    VertexId, WireId,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// An id that an [`Instance`] can hold: the typed node ids and `NodeId`.
pub trait InstanceId: Copy {
    /// `DefId.IsValid()`.
    fn is_valid_id(self) -> bool;
    /// `std::hash<TypedIdT>` of the id.
    fn id_hash(self) -> usize;
}

impl InstanceId for NodeId {
    fn is_valid_id(self) -> bool {
        self.is_valid()
    }
    fn id_hash(self) -> usize {
        self.hash_code()
    }
}

macro_rules! typed_instance_ids {
    ($($id:ty),*) => {$(
        impl InstanceId for $id {
            fn is_valid_id(self) -> bool {
                self.is_valid()
            }
            fn id_hash(self) -> usize {
                self.hash_code()
            }
        }
    )*};
}

typed_instance_ids!(
    VertexId,
    CoEdgeId,
    WireId,
    FaceId,
    ShellId,
    SolidId,
    OccurrenceId,
    CompoundId,
    CompSolidId,
    ProductId
);

/// `BRepGraphInc::Instance<TypedIdT>`: the id, the location and the orientation of one
/// use (FORWARD by default).
///
/// ```
/// use rustycad_tkbrep::brep_graph::FaceId;
/// use rustycad_tkbrep::brep_graph_inc::Instance;
/// let instance = Instance::new(FaceId::new(1));
/// assert!(instance.is_valid());
/// assert!(!Instance::<FaceId>::default().is_valid());
/// ```
#[derive(Clone, Debug)]
pub struct Instance<I> {
    /// `DefId`.
    pub def_id: I,
    /// `Location`.
    pub location: Location,
    /// `Orientation`.
    pub orientation: Orientation,
}

impl<I: InstanceId + Default> Default for Instance<I> {
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<I: InstanceId> Instance<I> {
    /// An instance of `def_id` at the identity location, FORWARD.
    pub fn new(def_id: I) -> Self {
        Self {
            def_id,
            location: Location::default(),
            orientation: Orientation::Forward,
        }
    }

    /// `IsValid()`: the id references a definition.
    pub fn is_valid(&self) -> bool {
        self.def_id.is_valid_id()
    }

    /// `std::hash<Instance<T>>`: the id hash, the location hash and the orientation.
    pub fn hash_code(&self) -> usize {
        combine(&[
            self.def_id.id_hash(),
            self.location.hash_code(),
            hash_i32(self.orientation as i32),
        ])
    }
}

/// `BRepGraphInc::VertexInstance`.
pub type VertexInstance = Instance<VertexId>;
/// `BRepGraphInc::CoEdgeInstance`.
pub type CoEdgeInstance = Instance<CoEdgeId>;
/// `BRepGraphInc::WireInstance`.
pub type WireInstance = Instance<WireId>;
/// `BRepGraphInc::FaceInstance`.
pub type FaceInstance = Instance<FaceId>;
/// `BRepGraphInc::ShellInstance`.
pub type ShellInstance = Instance<ShellId>;
/// `BRepGraphInc::SolidInstance`.
pub type SolidInstance = Instance<SolidId>;
/// `BRepGraphInc::OccurrenceInstance`.
pub type OccurrenceInstance = Instance<OccurrenceId>;
/// `BRepGraphInc::CompoundInstance`.
pub type CompoundInstance = Instance<CompoundId>;
/// `BRepGraphInc::CompSolidInstance`.
pub type CompSolidInstance = Instance<CompSolidId>;
/// `BRepGraphInc::ProductInstance`.
pub type ProductInstance = Instance<ProductId>;
/// `BRepGraphInc::NodeInstance`: returned by the explorers with the accumulated
/// transforms.
pub type NodeInstance = Instance<NodeId>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_hash_depends_on_the_orientation() {
        let mut instance = NodeInstance::new(NodeId::from(FaceId::new(2)));
        let forward = instance.hash_code();
        instance.orientation = Orientation::Reversed;
        assert_ne!(forward, instance.hash_code());
        let face = FaceInstance::new(FaceId::new(2));
        assert_eq!(face.hash_code(), forward);
    }
}
