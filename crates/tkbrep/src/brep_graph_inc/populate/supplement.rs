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
// Rust translation of extractEdgeSupplementAttachments, extractOwnerSupplementAttachments,
// attachSupplement, containerSupplementKind and attachNonCoreContainerChild of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Populate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The supplement attachments of population: the TopoDS children the core model does
//! not keep (internal edge vertices, face vertices, non-core container children) go to
//! the registered `BRepGraph_LayerTopoSupplement`, when there is one.
use crate::brep_graph::NodeId;
use crate::brep_graph::NodeKind;
use crate::brep_graph::hooks::{AttachmentKind, TopoSupplementHooks};
use crate::topods::{Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

// The children of a shape as stored (no composed orientation or location).
fn children(shape: &Shape) -> Vec<Shape> {
    ShapeIterator::new(shape, false, false).collect()
}

fn is_vertex_with(child: &Shape, orientation: Orientation) -> Result<bool, Failure> {
    Ok(child.shape_type()? == ShapeEnum::Vertex && child.orientation() == orientation)
}

// `extractEdgeSupplementAttachments`: every non-vertex child, and every vertex child
// other than the last FORWARD and the last REVERSED one (the boundary vertices).
fn edge_attachments(
    layer: &dyn TopoSupplementHooks,
    owner: NodeId,
    edge: &Shape,
) -> Result<(), Failure> {
    let children = children(edge);
    let (mut last_forward, mut last_reversed) = (None, None);
    for (index, child) in children.iter().enumerate() {
        if is_vertex_with(child, Orientation::Forward)? {
            last_forward = Some(index);
        } else if is_vertex_with(child, Orientation::Reversed)? {
            last_reversed = Some(index);
        }
    }
    for (index, child) in children.iter().enumerate() {
        if child.shape_type()? != ShapeEnum::Vertex {
            layer.add_attachment(owner, AttachmentKind::GenericSupplementShape, child);
            continue;
        }
        let boundary = (child.orientation() == Orientation::Forward && last_forward == Some(index))
            || (child.orientation() == Orientation::Reversed && last_reversed == Some(index));
        if !boundary {
            layer.add_attachment(owner, AttachmentKind::EdgeInternalVertex, child);
        }
    }
    Ok(())
}

// The face part of `extractOwnerSupplementAttachments`: every non-wire child.
fn face_attachments(
    layer: &dyn TopoSupplementHooks,
    owner: NodeId,
    face: &Shape,
) -> Result<(), Failure> {
    for child in children(face) {
        let kind = match child.shape_type()? {
            ShapeEnum::Wire => continue,
            ShapeEnum::Vertex => AttachmentKind::FaceDirectVertex,
            _ => AttachmentKind::GenericSupplementShape,
        };
        layer.add_attachment(owner, kind, &child);
    }
    Ok(())
}

/// `attachSupplement(theLayer, theOwner, theOwnerShape)`: the edge and face
/// supplements of a new node, when a supplement layer is registered.
pub(super) fn attach_supplement(
    layer: Option<&dyn TopoSupplementHooks>,
    owner: NodeId,
    owner_shape: &Shape,
) -> Result<(), Failure> {
    let Some(layer) = layer else {
        return Ok(());
    };
    match owner.kind {
        NodeKind::EDGE => edge_attachments(layer, owner, owner_shape),
        NodeKind::FACE => face_attachments(layer, owner, owner_shape),
        _ => Ok(()),
    }
}

/// `containerSupplementKind`: the attachment kind of a non-core container child.
pub(super) fn container_supplement_kind(owner_type: ShapeEnum) -> AttachmentKind {
    match owner_type {
        ShapeEnum::Compound => AttachmentKind::CompoundAuxShape,
        ShapeEnum::CompSolid => AttachmentKind::CompSolidAuxShape,
        ShapeEnum::Solid => AttachmentKind::SolidAuxShape,
        ShapeEnum::Shell => AttachmentKind::ShellAuxShape,
        _ => AttachmentKind::GenericSupplementShape,
    }
}

/// `attachNonCoreContainerChild`: true when the layer took the child.
pub(super) fn attach_non_core_container_child(
    layer: Option<&dyn TopoSupplementHooks>,
    owner: NodeId,
    owner_type: ShapeEnum,
    child: &Shape,
) -> bool {
    layer.is_some_and(|layer| {
        layer.add_attachment(owner, container_supplement_kind(owner_type), child) != 0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_kinds_follow_the_owner() {
        assert_eq!(
            container_supplement_kind(ShapeEnum::Shell),
            AttachmentKind::ShellAuxShape
        );
        assert_eq!(
            container_supplement_kind(ShapeEnum::Face),
            AttachmentKind::GenericSupplementShape
        );
        assert!(!attach_non_core_container_child(
            None,
            NodeId::default(),
            ShapeEnum::Solid,
            &Shape::default()
        ));
        assert!(attach_supplement(None, NodeId::default(), &Shape::default()).is_ok());
    }
}
