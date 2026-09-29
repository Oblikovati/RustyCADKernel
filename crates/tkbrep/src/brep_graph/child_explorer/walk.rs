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
// Rust translation of BRepGraph_ChildExplorer::startTraversal, advance, pushFrame,
// popFrame and the per-kind child iteration (ModelingData/TKBRep/BRepGraph/
// BRepGraph_ChildExplorer.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The depth-first walk of [`ChildExplorer`](super::ChildExplorer).
use super::{ChildExplorer, Config, StackFrame, TraversalMode};
use crate::brep_graph::explorer_kinds::{compose_location, kind_can_contain};
use crate::brep_graph::refs_iterator::FullOccurrenceRefIterator;
use crate::brep_graph::{
    BRepGraph, CoEdgeId, EdgeId, NodeId, NodeKind, OccurrenceId, ProductId, RefId, TypedId,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::toploc::Location;

/// One child found by the per-kind iteration: the node, the step index (-1 for a
/// structural link), the resolved reference, and the accumulated transforms.
struct ChildStep {
    node: NodeId,
    step: i32,
    reference: RefId,
    location: Location,
    orientation: Orientation,
}

impl ChildStep {
    // A structural child keeping the parent's transforms.
    fn structural(node: NodeId, frame: &StackFrame) -> Self {
        Self {
            node,
            step: -1,
            reference: RefId::default(),
            location: frame.acc_location.clone(),
            orientation: frame.acc_orientation,
        }
    }

    fn into_frame(self) -> StackFrame {
        StackFrame {
            node: self.node,
            next_child_idx: 0,
            step_from_parent: self.step,
            reference: self.reference,
            acc_location: self.location,
            acc_orientation: self.orientation,
        }
    }
}

impl ChildExplorer<'_> {
    /// `startTraversal`: the root itself when it matches the target (or is an emitted
    /// avoid kind), else its first matching descendant.
    pub(super) fn start_traversal(&mut self) -> Result<(), Failure> {
        let root = self.root;
        if !root.is_valid() {
            return Ok(());
        }
        let is_live =
            self.graph.topo().generic().topo_entity(root).is_some() && !root.is_removed(self.graph);
        let root_frame = StackFrame::root(root, &self.config.start_loc, self.config.start_ori);
        if self.matches_avoid(root) {
            if self.config.target_kind.is_some() && self.config.emit_avoid_kind && is_live {
                self.push_frame(root_frame);
                self.set_current_from_frame(self.stack_top);
            }
            return Ok(());
        }
        if !is_live {
            return Ok(());
        }
        self.push_frame(root_frame);
        if self.config.target_kind.is_some() && self.matches_target(root) {
            self.set_current_from_frame(self.stack_top);
            return Ok(());
        }
        self.advance()
    }

    /// `advance`: pops the current frame when it is not descended into, then walks to
    /// the next emitted node.
    pub(super) fn advance(&mut self) -> Result<(), Failure> {
        if self.current_frame >= 0 {
            if self.current_frame == self.stack_top && !self.should_descend_from_current() {
                self.pop_frame();
            }
            self.current_frame = -1;
        }
        self.has_more = false;
        while self.stack_top >= 0 {
            let top = self.stack_top as usize;
            // An invalid child ends the frame even when later siblings exist (a
            // reference naming an invalid node pops its parent), as natively.
            let child = next_child(self.graph, &self.config, &mut self.stack[top])?;
            let Some(child) = child.filter(|child| child.node.is_valid()) else {
                self.pop_frame();
                continue;
            };
            if self.visit_child(child) {
                return Ok(());
            }
        }
        Ok(())
    }

    // The emission and descent decisions for one child; true when it became current.
    fn visit_child(&mut self, child: ChildStep) -> bool {
        let node = child.node;
        let is_live =
            self.graph.topo().generic().topo_entity(node).is_some() && !node.is_removed(self.graph);
        if self.matches_avoid(node) {
            if self.config.emit_avoid_kind && is_live {
                return self.emit(child);
            }
            return false;
        }
        if self.should_emit(node) {
            return is_live && self.emit(child);
        }
        if is_live && self.config.mode == TraversalMode::Recursive && self.can_descend(node.kind) {
            self.push_frame(child.into_frame());
        }
        false
    }

    fn emit(&mut self, child: ChildStep) -> bool {
        self.push_frame(child.into_frame());
        self.set_current_from_frame(self.stack_top);
        true
    }

    // The target or child-bearing test of the descent.
    fn can_descend(&self, kind: NodeKind) -> bool {
        match self.config.target_kind {
            Some(target) => kind_can_contain(kind, target),
            None => kind != NodeKind::VERTEX,
        }
    }

    /// `setCurrentFromFrame`.
    fn set_current_from_frame(&mut self, frame: i32) {
        let stored = &self.stack[frame as usize];
        self.current_frame = frame;
        self.current = stored.node;
        self.location = stored.acc_location.clone();
        self.orientation = stored.acc_orientation;
        self.has_more = true;
    }

    /// `shouldDescendFromCurrent`: recursive mode, not an avoid kind, and a kind that
    /// can contain the target (or any child without a target).
    fn should_descend_from_current(&self) -> bool {
        if self.config.mode != TraversalMode::Recursive || self.current_frame < 0 {
            return false;
        }
        let node = self.stack[self.current_frame as usize].node;
        !self.matches_avoid(node) && self.can_descend(node.kind)
    }

    fn matches_target(&self, node: NodeId) -> bool {
        self.config.target_kind == Some(node.kind)
    }

    fn matches_avoid(&self, node: NodeId) -> bool {
        self.config.avoid_kind == Some(node.kind)
    }

    /// `shouldEmit`: the target (or any kind) and the avoid kind, as configured.
    fn should_emit(&self, node: NodeId) -> bool {
        let is_avoid = self.matches_avoid(node);
        let is_find = self
            .config
            .target_kind
            .is_none_or(|target| node.kind == target);
        if self.config.emit_avoid_kind {
            is_find || is_avoid
        } else {
            is_find && !is_avoid
        }
    }

    /// `pushFrame`: refused past the node count (the cycle guard), else written at the
    /// new top.
    fn push_frame(&mut self, frame: StackFrame) {
        if self.stack_top >= 0 && self.stack_top as u32 >= self.graph.topo().generic().nb_nodes() {
            return;
        }
        self.stack_top += 1;
        let top = self.stack_top as usize;
        if top < self.stack.len() {
            self.stack[top] = frame;
        } else {
            self.stack.push(frame);
        }
    }

    /// `popFrame`.
    fn pop_frame(&mut self) {
        if self.stack_top >= 0 {
            self.stack_top -= 1;
        }
    }
}

// The per-kind child iteration of `advance`: the next child of the frame, advancing
// its `NextChildIdx` as natively, `None` when the frame is exhausted.
fn next_child(
    graph: &BRepGraph,
    config: &Config,
    frame: &mut StackFrame,
) -> Result<Option<ChildStep>, Failure> {
    match frame.node.kind {
        NodeKind::COMPOUND
        | NodeKind::COMP_SOLID
        | NodeKind::SOLID
        | NodeKind::SHELL
        | NodeKind::FACE => next_referenced_child(graph, config, frame),
        NodeKind::EDGE => next_vertex_child(graph, config, frame),
        NodeKind::WIRE => Ok(next_co_edge_child(graph, frame)),
        NodeKind::CO_EDGE => Ok(next_edge_child(graph, config, frame)),
        NodeKind::OCCURRENCE => next_occurrence_child(graph, config, frame),
        NodeKind::PRODUCT => next_product_child(graph, config, frame),
        _ => {
            frame.next_child_idx = 0;
            Ok(None)
        }
    }
}

// The composed transforms of a reference step.
fn referenced_step(
    graph: &BRepGraph,
    config: &Config,
    frame: &StackFrame,
    (reference, step): (RefId, u32),
) -> Result<ChildStep, Failure> {
    let refs = graph.refs().generic();
    let mut child = ChildStep::structural(refs.child_node(reference), frame);
    child.step = step as i32;
    child.reference = reference;
    if config.accumulate_location {
        child.location = compose_location(&frame.acc_location, &refs.local_location(reference))?;
    }
    if config.accumulate_orientation {
        child.orientation = top_abs::compose(frame.acc_orientation, refs.orientation(reference));
    }
    Ok(child)
}

// Compound, CompSolid, Solid, Shell and Face: the next reference of the relation list
// that is not removed.
fn next_referenced_child(
    graph: &BRepGraph,
    config: &Config,
    frame: &mut StackFrame,
) -> Result<Option<ChildStep>, Failure> {
    let mut index = frame.next_child_idx;
    let mut found = None;
    while let Some(reference) = relation_ref_at(graph, frame.node, index) {
        if !graph.refs().generic().is_removed(reference) {
            found = Some((reference, index));
            break;
        }
        index += 1;
    }
    frame.next_child_idx = index + 1;
    found
        .map(|step| referenced_step(graph, config, frame, step))
        .transpose()
}

// The `index`-th reference of a container's relation list, `None` past its end.
fn relation_ref_at(graph: &BRepGraph, node: NodeId, index: u32) -> Option<RefId> {
    let storage = graph.inc_storage();
    let at = index as usize;
    match node.kind {
        NodeKind::COMPOUND => storage
            .compound_relations(TypedId::new(node.index))
            .child_ref_ids
            .get(at)
            .map(|id| (*id).into()),
        NodeKind::COMP_SOLID => storage
            .comp_solid_relations(TypedId::new(node.index))
            .solid_ref_ids
            .get(at)
            .map(|id| (*id).into()),
        NodeKind::SOLID => storage
            .solid_relations(TypedId::new(node.index))
            .shell_ref_ids
            .get(at)
            .map(|id| (*id).into()),
        NodeKind::SHELL => storage
            .shell_relations(TypedId::new(node.index))
            .face_ref_ids
            .get(at)
            .map(|id| (*id).into()),
        _ => storage
            .face_relations(TypedId::new(node.index))
            .wire_ref_ids
            .get(at)
            .map(|id| (*id).into()),
    }
}

// Edge: the start then end vertex reference that is valid and not removed.
fn next_vertex_child(
    graph: &BRepGraph,
    config: &Config,
    frame: &mut StackFrame,
) -> Result<Option<ChildStep>, Failure> {
    let def = graph.inc_storage().edge(EdgeId::new(frame.node.index));
    let mut index = frame.next_child_idx;
    let mut found = None;
    while index < 2 {
        let reference = if index == 0 {
            def.start_vertex_ref_id
        } else {
            def.end_vertex_ref_id
        };
        if reference.is_valid() && !graph.refs().generic().is_removed(reference.into()) {
            found = Some((reference.into(), index));
            break;
        }
        index += 1;
    }
    frame.next_child_idx = index + 1;
    found
        .map(|step| referenced_step(graph, config, frame, step))
        .transpose()
}

// Wire: the next coedge in range and not removed (a structural step).
fn next_co_edge_child(graph: &BRepGraph, frame: &mut StackFrame) -> Option<ChildStep> {
    let storage = graph.inc_storage();
    let co_edges = &storage
        .wire_relations(TypedId::new(frame.node.index))
        .co_edge_ids;
    let mut index = frame.next_child_idx as usize;
    let mut found = None;
    while let Some(co_edge) = co_edges.get(index) {
        if co_edge.is_valid_for(storage.nb_co_edges()) && !co_edge.is_removed(graph) {
            found = Some(*co_edge);
            break;
        }
        index += 1;
    }
    frame.next_child_idx = index as u32 + 1;
    found.map(|co_edge: CoEdgeId| ChildStep::structural(co_edge.into(), frame))
}

// CoEdge: its edge once, with the coedge orientation composed.
fn next_edge_child(
    graph: &BRepGraph,
    config: &Config,
    frame: &mut StackFrame,
) -> Option<ChildStep> {
    let first_visit = frame.next_child_idx == 0;
    frame.next_child_idx = 1;
    let co_edge = CoEdgeId::new(frame.node.index);
    let def = graph.inc_storage().co_edge(co_edge);
    if !first_visit || co_edge.is_removed(graph) || !def.child_edge_id.is_valid() {
        return None;
    }
    let mut child = ChildStep::structural(def.child_edge_id.into(), frame);
    if config.accumulate_orientation {
        child.orientation = top_abs::compose(frame.acc_orientation, def.orientation.orientation());
    }
    Some(child)
}

// Occurrence: its child once; without an incoming reference, the location of the
// first live reference to the occurrence in id order.
fn next_occurrence_child(
    graph: &BRepGraph,
    config: &Config,
    frame: &mut StackFrame,
) -> Result<Option<ChildStep>, Failure> {
    let first_visit = frame.next_child_idx == 0;
    frame.next_child_idx = 1;
    let occurrence = OccurrenceId::new(frame.node.index);
    let def = graph.inc_storage().occurrence(occurrence);
    if !first_visit || frame.node.is_removed(graph) || !def.child_node_id.is_valid() {
        return Ok(None);
    }
    let mut child = ChildStep::structural(def.child_node_id, frame);
    if config.accumulate_location && !frame.reference.is_valid() {
        let mut refs = FullOccurrenceRefIterator::new(graph);
        while refs.more() {
            let entry = refs.current();
            if !refs.current_id().is_removed(graph) && entry.child_occurrence_id == occurrence {
                child.location = compose_location(&frame.acc_location, &entry.local_location)?;
                break;
            }
            refs.next();
        }
    }
    Ok(Some(child))
}

// Product: the `NextChildIdx`-th live occurrence reference with a live occurrence.
fn next_product_child(
    graph: &BRepGraph,
    config: &Config,
    frame: &mut StackFrame,
) -> Result<Option<ChildStep>, Failure> {
    let wanted = frame.next_child_idx;
    frame.next_child_idx = wanted + 1;
    let storage = graph.inc_storage();
    let product = ProductId::new(frame.node.index);
    let found = storage
        .product_relations(product)
        .occurrence_ref_ids
        .iter()
        .filter(|reference| {
            !reference.is_removed(graph)
                && !storage
                    .occurrence_ref(**reference)
                    .child_occurrence_id
                    .is_removed(graph)
        })
        .nth(wanted as usize);
    let Some(reference) = found else {
        return Ok(None);
    };
    let entry = storage.occurrence_ref(*reference);
    let mut child = ChildStep::structural(entry.child_occurrence_id.into(), frame);
    child.step = wanted as i32;
    child.reference = (*reference).into();
    if config.accumulate_location {
        child.location = compose_location(&frame.acc_location, &entry.local_location)?;
    }
    Ok(Some(child))
}
