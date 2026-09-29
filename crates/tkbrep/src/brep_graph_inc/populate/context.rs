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
// Rust translation of LocatedNodeBinding, BuildContext, isWireOrderWarning,
// recordWireOrderStatus, definitionShapeKey, shapeWithoutOwnLocation, findExistingNode,
// bindLocatedNode and appendBuildRoot of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Populate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The state of one population pass and its shape-identity bookkeeping.
use super::face::FaceBuildData;
use crate::brep_graph::hooks::TopoSupplementHooks;
use crate::brep_graph::{NodeId, NodeKind};
use crate::brep_graph_inc::{Storage, WireCoEdgeOrderStatus};
use crate::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;
use std::collections::{HashMap, HashSet};

/// `TopologyBuildMode`: keep the containers, or flatten them into appended roots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TopologyBuildMode {
    FullHierarchy,
    Flattened,
}

// `LocatedNodeBinding`: a node built this pass for a TShape at a definition location.
struct LocatedNodeBinding {
    location: Location,
    node: NodeId,
}

/// `BuildContext`: the storage being filled, the build options, the queued faces and
/// the within-build index of located nodes (keyed by TShape address, only searched).
pub(super) struct BuildContext<'b> {
    pub(super) storage: &'b mut Storage,
    pub(super) parallel: bool,
    pub(super) mode: TopologyBuildMode,
    pub(super) supplement: Option<&'b dyn TopoSupplementHooks>,
    pub(super) appended_roots: Option<&'b mut Vec<NodeId>>,
    appended_root_set: HashSet<NodeId>,
    pub(super) pending_faces: Vec<FaceBuildData>,
    located_nodes: Vec<LocatedNodeBinding>,
    located_node_index: HashMap<usize, Vec<usize>>,
    pub(super) has_wire_order_warnings: bool,
}

impl<'b> BuildContext<'b> {
    pub(super) fn new(
        storage: &'b mut Storage,
        parallel: bool,
        mode: TopologyBuildMode,
        supplement: Option<&'b dyn TopoSupplementHooks>,
        appended_roots: Option<&'b mut Vec<NodeId>>,
    ) -> Self {
        Self {
            storage,
            parallel,
            mode,
            supplement,
            appended_roots,
            appended_root_set: HashSet::new(),
            pending_faces: Vec::new(),
            located_nodes: Vec::new(),
            located_node_index: HashMap::new(),
            has_wire_order_warnings: false,
        }
    }

    /// `recordWireOrderStatus`: tolerance, partial or invalid orders are warnings.
    pub(super) fn record_wire_order_status(&mut self, status: WireCoEdgeOrderStatus) {
        use WireCoEdgeOrderStatus as Status;
        let warning = matches!(
            status,
            Status::ToleranceOrdered | Status::Partial | Status::InvalidInput
        );
        self.has_wire_order_warnings = self.has_wire_order_warnings || warning;
    }

    /// `findExistingNode`: the storage binding of the definition key when it has the
    /// expected kind, else a node built this pass for the TShape at an equal location.
    pub(super) fn find_existing_node(
        &self,
        shape: &Shape,
        expected: NodeKind,
        definition_location: &Location,
    ) -> Result<NodeId, Failure> {
        let bound = self
            .storage
            .find_definition_by_shape(&definition_shape_key(shape, definition_location)?);
        if bound.is_valid() && bound.kind == expected {
            return Ok(bound);
        }
        let indices = self.located_node_index.get(&shape.tshape_address());
        let found = indices
            .into_iter()
            .flatten()
            .map(|index| &self.located_nodes[*index])
            .find(|binding| {
                binding.node.kind == expected && binding.location.is_equal(definition_location)
            });
        Ok(found.map_or_else(NodeId::default, |binding| binding.node))
    }

    /// `bindLocatedNode`: records the node for the TShape and location, and binds the
    /// definition key in the storage when asked.
    pub(super) fn bind_located_node(
        &mut self,
        shape: &Shape,
        node: NodeId,
        definition_location: &Location,
        bind_shape: bool,
    ) -> Result<(), Failure> {
        let index = self.located_nodes.len();
        self.located_nodes.push(LocatedNodeBinding {
            location: definition_location.clone(),
            node,
        });
        self.located_node_index
            .entry(shape.tshape_address())
            .or_default()
            .push(index);
        if bind_shape {
            let key = definition_shape_key(shape, definition_location)?;
            self.storage.set_definition_shape_binding(&key, node);
        }
        Ok(())
    }

    /// `appendBuildRoot`: a new valid root of a flattened append, once.
    pub(super) fn append_build_root(&mut self, node: NodeId) {
        let Some(roots) = self.appended_roots.as_deref_mut() else {
            return;
        };
        if node.is_valid() && self.appended_root_set.insert(node) {
            roots.push(node);
        }
    }
}

/// `definitionShapeKey`: the shape at its definition location (orientation kept).
pub(super) fn definition_shape_key(
    shape: &Shape,
    definition_location: &Location,
) -> Result<Shape, Failure> {
    let mut key = shape.clone();
    key.set_location(definition_location, false)?;
    Ok(key)
}

/// `shapeWithoutOwnLocation`: the shape at the identity location.
pub(super) fn shape_without_own_location(shape: &Shape) -> Result<Shape, Failure> {
    definition_shape_key(shape, &Location::default())
}
