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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph.hxx and BRepGraph.cxx
// (lifecycle, accessors and generic entity lookup) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph`: the topology-geometry graph over TopoDS / BRep.
use super::data::Data;
use super::hooks::{CacheHooks, LayerHooks, TopoSupplementHooks};
use crate::brep_graph::{
    NodeId, NodeTag, NodeVisitor, ProductId, RefId, RefTag, RefUid, RefVisitor, TypedId, Uid,
};
use crate::brep_graph_inc::{BaseDef, BaseRef, HasBaseDef, HasBaseRef, Storage};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::{Guid, Uuid};
use rustycad_tkmath::math_utils::RandomGenerator;
use std::hash::{BuildHasher, Hasher};

/// `BRepGraph`: flat per-kind entity tables with integer cross-references, the unique
/// owner of its data (not `Clone`, as the native copy is deleted).
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// let mut graph = BRepGraph::new();
/// assert!(graph.is_empty() && graph.is_valid());
/// graph.clear().unwrap();
/// assert_eq!(graph.uids().generation(), 1);
/// ```
#[derive(Debug, Default)]
pub struct BRepGraph {
    data: Data,
}

impl BRepGraph {
    /// `BRepGraph()`: an empty graph (docs/topology-design.md section 11.6 explains why
    /// no layer is registered until the layer registry is attached).
    pub fn new() -> Self {
        Self::default()
    }

    /// `Clear()`: empties the storage and the registries, increments the generation and
    /// draws a new random graph GUID. An active MutGuard raises Standard_ProgramError
    /// (`Standard_ASSERT_RAISE`, active in release builds too).
    pub fn clear(&mut self) -> Result<(), Failure> {
        if self.data.inc_storage.has_any_guard() {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "*** ERROR: ASSERT: BRepGraph::Clear(): guards still active: some MutGuard is alive",
                "",
            ));
        }
        self.data.inc_storage.clear();
        if let Some(caches) = self.data.caches.as_mut() {
            caches.clear_all();
        }
        self.data.inc_storage.increment_generation();
        self.data.inc_storage.set_graph_guid(random_guid());
        if let Some(layers) = self.data.layers.as_mut() {
            layers.clear_all();
        }
        Ok(())
    }

    /// `IsEmpty()`: no node of any kind.
    pub fn is_empty(&self) -> bool {
        self.data.inc_storage.is_empty()
    }

    /// `ValidateRelations()`: the relation tables agree with the record endpoints.
    pub fn validate_relations(&self) -> bool {
        self.data.inc_storage.validate_relations()
    }

    /// `RootProductIds()`: products no active occurrence references.
    pub fn root_product_ids(&self) -> &[ProductId] {
        self.data.inc_storage.root_product_ids()
    }

    /// `IsValid()`: the graph references its data; a Rust graph always does (a moved-from
    /// native graph does not, but a moved Rust value cannot be used).
    pub fn is_valid(&self) -> bool {
        true
    }

    /// `IsNull()`: `!IsValid()`.
    pub fn is_null(&self) -> bool {
        !self.is_valid()
    }

    /// `incStorage()` (private natively): the backend storage.
    pub(crate) fn inc_storage(&self) -> &Storage {
        &self.data.inc_storage
    }

    /// The mutable backend storage.
    pub(crate) fn inc_storage_mut(&mut self) -> &mut Storage {
        &mut self.data.inc_storage
    }

    /// The mutable graph data.
    pub(crate) fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    /// `LayerRegistry().FindLayer<BRepGraph_LayerTopoSupplement>()`: the registered
    /// topology supplement layer, `None` without layer hooks.
    pub(crate) fn topo_supplement(&self) -> Option<&dyn TopoSupplementHooks> {
        self.data
            .layers
            .as_deref()
            .and_then(|layers| layers.topo_supplement())
    }

    /// Installs the layer registry: the layers area of phase 2 calls it from `new`
    /// (docs/topology-design.md section 11.6); a later call replaces the hooks.
    pub fn attach_layer_hooks(&mut self, hooks: Box<dyn LayerHooks>) {
        self.data.layers = Some(hooks);
    }

    /// Installs the cache registry: the caches area of phase 2 calls it from `new`.
    pub fn attach_cache_hooks(&mut self, hooks: Box<dyn CacheHooks>) {
        self.data.caches = Some(hooks);
    }

    /// `allocateUID(theNodeId)` (private natively).
    pub(crate) fn allocate_uid(&mut self, node: NodeId) -> Uid {
        self.data.inc_storage.allocate_node_uid(node)
    }

    /// `allocateRefUID(theRefId)` (private natively).
    pub(crate) fn allocate_ref_uid(&mut self, reference: RefId) -> RefUid {
        self.data.inc_storage.allocate_ref_uid(reference)
    }

    /// `topoEntity(theId)` (private natively): the base of a node in range.
    pub(crate) fn topo_entity(&self, node: NodeId) -> Option<&BaseDef> {
        if !node.is_valid() {
            return None;
        }
        node.visit(EntityFinder(&self.data.inc_storage))
    }

    /// `changeTopoEntity(theId)` (private natively).
    pub(crate) fn change_topo_entity(&mut self, node: NodeId) -> Option<&mut BaseDef> {
        if !node.is_valid() {
            return None;
        }
        node.visit(EntityChanger(&mut self.data.inc_storage))
    }

    /// `refEntity(theId)` (private natively): the base of a reference in range.
    pub(crate) fn ref_entity(&self, reference: RefId) -> Option<&BaseRef> {
        if !reference.is_valid() {
            return None;
        }
        reference.visit(RefFinder(&self.data.inc_storage))
    }
}

struct EntityFinder<'s>(&'s Storage);

impl<'s> NodeVisitor for EntityFinder<'s> {
    type Output = Option<&'s BaseDef>;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> Self::Output {
        T::store(self.0)
            .items
            .get(id.index as usize)
            .map(HasBaseDef::base)
    }
}

struct EntityChanger<'s>(&'s mut Storage);

impl<'s> NodeVisitor for EntityChanger<'s> {
    type Output = Option<&'s mut BaseDef>;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> Self::Output {
        T::store_mut(self.0)
            .items
            .get_mut(id.index as usize)
            .map(HasBaseDef::base_mut)
    }
}

struct RefFinder<'s>(&'s Storage);

impl<'s> RefVisitor for RefFinder<'s> {
    type Output = Option<&'s BaseRef>;
    fn visit<T: RefTag>(self, id: TypedId<T>) -> Self::Output {
        T::store(self.0)
            .items
            .get(id.index as usize)
            .map(HasBaseRef::base)
    }
}

// `generateRandomGUID`: two draws of the MathUtils generator seeded from a random
// source (`std::random_device` natively; the process-random hasher keys here).
fn random_guid() -> Guid {
    let seed = std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish() as u32;
    let mut generator = RandomGenerator::new(u64::from(seed));
    let (first, second) = (generator.next_int(), generator.next_int());
    let mut data4 = [0_u8; 8];
    for (index, byte) in data4.iter_mut().enumerate() {
        *byte = (second >> (index * 8)) as u8;
    }
    Guid::from_uuid(Uuid {
        data1: first as u32,
        data2: (first >> 32) as u16,
        data3: (first >> 48) as u16,
        data4,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::{FaceId, FaceRefId};

    #[test]
    fn generic_lookups_stop_at_the_store_end() {
        let mut graph = BRepGraph::new();
        let face = graph.inc_storage_mut().append_face();
        assert!(graph.topo_entity(face.into()).is_some());
        assert!(graph.topo_entity(FaceId::new(1).into()).is_none());
        graph.change_topo_entity(face.into()).unwrap().own_gen = 3;
        assert_eq!(graph.topo_entity(face.into()).unwrap().own_gen, 3);
        assert!(graph.ref_entity(FaceRefId::new(0).into()).is_none());
        assert_eq!(graph.allocate_uid(face.into()).counter, 1);
        assert!(!graph.allocate_ref_uid(FaceRefId::new(0).into()).is_valid());
    }

    #[test]
    fn clear_refuses_active_guards() {
        let mut graph = BRepGraph::new();
        let face = graph.inc_storage_mut().append_face();
        graph.inc_storage_mut().set_guarded(face);
        let error = graph.clear().unwrap_err();
        assert_eq!(error.exception_type(), "Standard_ProgramError");
        graph.inc_storage_mut().clear_guarded(face);
        let before = *graph.inc_storage().graph_guid();
        graph.clear().unwrap();
        assert_ne!(*graph.inc_storage().graph_guid(), before);
        assert!(graph.is_empty() && !graph.is_null());
    }
}
