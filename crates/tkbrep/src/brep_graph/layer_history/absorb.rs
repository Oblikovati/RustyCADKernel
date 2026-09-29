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
// Rust translation of BRepGraph_LayerHistory::Absorb (both overloads) and of
// resolveImages/resolveUidImages (ModelingData/TKBRep/BRepGraph/BRepGraph_LayerHistory.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The import of a `BRepTools_History` into the graph-native log.
use super::record::append_unique;
use super::{HistoryKind, LayerHistory};
use crate::brep_graph::layer::AttachedGraph;
use crate::brep_graph::shapes_view::ShapeNodeMap;
use crate::brep_graph::{BRepGraph, NodeId, Uid};
use crate::brep_tools::History;
use crate::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

/// `resolveImages(theImages, theOutputs)`: the distinct output nodes of the images
/// bound in `outputs` (the others were merged into a kept parent and are dropped).
fn resolve_images(images: &[Shape], outputs: &ShapeNodeMap) -> Vec<NodeId> {
    let mut nodes = Vec::new();
    for image in images {
        if let Some(&node) = outputs.seek(image) {
            append_unique(&mut nodes, node);
        }
    }
    nodes
}

/// `resolveUidImages(theOutputGraph, theImages, theOutputs)`: the distinct valid UIDs,
/// in the output graph, of the bound output nodes.
fn resolve_uid_images(
    output_graph: &BRepGraph,
    images: &[Shape],
    outputs: &ShapeNodeMap,
) -> Vec<Uid> {
    let mut uids = Vec::new();
    for image in images {
        let Some(&node) = outputs.seek(image) else {
            continue;
        };
        let uid = output_graph.uids().of_node(node);
        if uid.is_valid() {
            append_unique(&mut uids, uid);
        }
    }
    uids
}

impl LayerHistory {
    /// `Absorb(theInputs, theOutputs, theSource, theOpLabel)`: for every input (in map
    /// order) a Modified and a Generated `Record` of its bound images; the removed
    /// inputs (removal takes precedence) in one final `RecordDeleted`. A null history
    /// or no input records nothing. `graph` is the attached graph whose item UIDs the
    /// records carry.
    pub fn absorb(
        &self,
        graph: &BRepGraph,
        inputs: &ShapeNodeMap,
        outputs: &ShapeNodeMap,
        source: &Handle<History>,
        label: &AsciiString,
    ) -> Result<(), Failure> {
        self.absorb_in(
            self.base.attached_graph(graph),
            inputs,
            outputs,
            source,
            label,
        )
    }

    pub(crate) fn absorb_in(
        &self,
        attached: AttachedGraph<'_>,
        inputs: &ShapeNodeMap,
        outputs: &ShapeNodeMap,
        source: &Handle<History>,
        label: &AsciiString,
    ) -> Result<(), Failure> {
        let Some(source) = source.get() else {
            return Ok(());
        };
        if inputs.is_empty() {
            return Ok(());
        }
        let mut deleted = Vec::new();
        for (shape, &node) in inputs.items() {
            if source.is_removed(shape)? {
                deleted.push(node);
                continue;
            }
            let modified = resolve_images(&source.modified(shape)?, outputs);
            if !modified.is_empty() {
                self.record_in(attached, label, node, &modified, HistoryKind::Modified);
            }
            let generated = resolve_images(&source.generated(shape)?, outputs);
            if !generated.is_empty() {
                self.record_in(attached, label, node, &generated, HistoryKind::Generated);
            }
        }
        if !deleted.is_empty() {
            self.record_deleted_in(attached, label, &deleted);
        }
        Ok(())
    }

    /// `Absorb(theInputGraph, theOutputGraph, theInputs, theOutputs, theSource,
    /// theOpLabel)`: the cross-graph import, keyed by the input UIDs in `input_graph`
    /// and the image UIDs in `output_graph` (`RecordUid`, `RecordDeletedUid`); inputs
    /// without a valid UID are skipped, the others become known inputs.
    pub fn absorb_between(
        &self,
        input_graph: &BRepGraph,
        output_graph: &BRepGraph,
        inputs: &ShapeNodeMap,
        outputs: &ShapeNodeMap,
        source: &Handle<History>,
        label: &AsciiString,
    ) -> Result<(), Failure> {
        let Some(source) = source.get() else {
            return Ok(());
        };
        if inputs.is_empty() {
            return Ok(());
        }
        let mut deleted = Vec::new();
        for (shape, &node) in inputs.items() {
            let input_uid = input_graph.uids().of_node(node);
            if !input_uid.is_valid() {
                continue;
            }
            self.state.write().uids.known_inputs.add(input_uid);
            if source.is_removed(shape)? {
                deleted.push(input_uid);
                continue;
            }
            let modified = resolve_uid_images(output_graph, &source.modified(shape)?, outputs);
            if !modified.is_empty() {
                self.record_uid(label, input_uid, &modified, HistoryKind::Modified);
            }
            let generated = resolve_uid_images(output_graph, &source.generated(shape)?, outputs);
            if !generated.is_empty() {
                self.record_uid(label, input_uid, &generated, HistoryKind::Generated);
            }
        }
        if !deleted.is_empty() {
            self.record_deleted_uid(label, &deleted);
        }
        Ok(())
    }
}
