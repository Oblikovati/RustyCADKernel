// Rust translation of BRepGraph_ParentExplorer::advance, backtrackAfterBranchEmission,
// emitNextFromCurrentBranch, prepareCurrentBranch, applyTransition, pushFrame and
// popFrame (ModelingData/TKBRep/BRepGraph/BRepGraph_ParentExplorer.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The upward walk of [`ParentExplorer`](super::ParentExplorer).
use super::steps::next_parent_frame;
use super::{ParentExplorer, StackFrame, TraversalMode};
use crate::brep_graph::explorer_kinds::compose_location;
use crate::brep_graph::{BRepGraph, CoEdgeId, NodeId, NodeKind, RefId, RefKind};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::toploc::Location;

impl ParentExplorer<'_> {
    /// `advance`: the next emitted ancestor.
    pub(super) fn advance(&mut self) -> Result<(), Failure> {
        self.has_more = false;
        self.current = NodeId::default();
        self.current_frame = -1;
        if self.config.mode == TraversalMode::DirectParents {
            return self.advance_direct();
        }
        if self.emit_index >= 1 {
            if self.emit_next_from_current_branch() {
                return Ok(());
            }
            self.backtrack_after_branch_emission();
        }
        while self.stack_top >= 0 {
            let top = self.stack_top as usize;
            if let Some(parent) = next_parent_frame(self.graph, &mut self.stack[top]) {
                let is_avoid = self.matches_avoid(parent.node);
                self.push_frame(parent);
                if is_avoid && self.emit_branch()? {
                    return Ok(());
                }
                continue;
            }
            // The branch reached its root: emit it bottom-up, then drop its top frame.
            if self.stack_top > 0 {
                if self.emit_branch()? {
                    return Ok(());
                }
                continue;
            }
            self.pop_frame();
        }
        Ok(())
    }

    // The branch emission of `advance`: prepare the branch and emit its first frame;
    // when none is emitted, drop the top frame and reset the emission index.
    fn emit_branch(&mut self) -> Result<bool, Failure> {
        self.prepare_current_branch()?;
        self.emit_index = 1;
        if self.emit_next_from_current_branch() {
            return Ok(true);
        }
        self.pop_frame();
        self.emit_index = -1;
        Ok(false)
    }

    // The DirectParents branch of `advance`: one parent frame above the start node.
    fn advance_direct(&mut self) -> Result<(), Failure> {
        if self.stack_top > 0 {
            self.pop_frame();
        }
        while self.stack_top >= 0 {
            let top = self.stack_top as usize;
            let Some(parent) = next_parent_frame(self.graph, &mut self.stack[top]) else {
                self.pop_frame();
                return Ok(());
            };
            self.push_frame(parent);
            self.prepare_current_branch()?;
            let top = self.stack_top as usize;
            if !self.should_emit(self.stack[top].node) {
                self.pop_frame();
                continue;
            }
            self.set_current(self.stack_top);
            return Ok(());
        }
        Ok(())
    }

    /// `backtrackAfterBranchEmission`: pops until a frame has another parent (probed on
    /// a copy, so its cursor does not move).
    fn backtrack_after_branch_emission(&mut self) {
        self.emit_index = -1;
        while self.stack_top >= 0 {
            self.pop_frame();
            if self.stack_top < 0 {
                return;
            }
            let mut probe = self.stack[self.stack_top as usize].clone();
            if next_parent_frame(self.graph, &mut probe).is_some() {
                return;
            }
        }
    }

    /// `emitNextFromCurrentBranch`: the next frame of the branch (from the start node's
    /// parent up to the top) that should be emitted.
    fn emit_next_from_current_branch(&mut self) -> bool {
        while self.emit_index >= 1 && self.emit_index <= self.stack_top {
            let frame = self.emit_index;
            self.emit_index += 1;
            if self.should_emit(self.stack[frame as usize].node) {
                self.set_current(frame);
                return true;
            }
        }
        false
    }

    fn set_current(&mut self, frame: i32) {
        let stored = &self.stack[frame as usize];
        self.current_frame = frame;
        self.current = stored.node;
        self.location = stored.acc_location.clone();
        self.orientation = stored.acc_orientation;
        self.has_more = true;
    }

    /// `prepareCurrentBranch`: identity at the top frame, then each lower frame
    /// composes the transition from the frame above.
    fn prepare_current_branch(&mut self) -> Result<(), Failure> {
        if self.stack_top < 0 {
            return Ok(());
        }
        let top = self.stack_top as usize;
        self.stack[top].acc_location = Location::default();
        self.stack[top].acc_orientation = Orientation::Forward;
        for frame in (0..top).rev() {
            let above = &self.stack[frame + 1];
            let (location, orientation) = apply_transition(
                self.graph,
                above,
                self.stack[frame].node,
                (above.acc_location.clone(), above.acc_orientation),
            )?;
            self.stack[frame].acc_location = location;
            self.stack[frame].acc_orientation = orientation;
        }
        Ok(())
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

    /// `pushFrame`: refused past the node count (the depth guard).
    pub(super) fn push_frame(&mut self, frame: StackFrame) {
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

/// `applyTransition(parent, child, step, ref, location, orientation)`: a stored
/// reference composes its location (and its orientation, except an occurrence
/// reference); a reference step composes `RefAtStep` of the parent; a structural step
/// composes only a CoEdge's orientation onto its edge.
fn apply_transition(
    graph: &BRepGraph,
    parent: &StackFrame,
    _child: NodeId,
    (location, orientation): (Location, Orientation),
) -> Result<(Location, Orientation), Failure> {
    let refs = graph.refs().generic();
    let reference = parent.ref_to_child;
    if reference.is_valid() {
        let location = compose_location(&location, &refs.local_location(reference))?;
        if reference.kind == RefKind::OCCURRENCE {
            return Ok((location, orientation));
        }
        return Ok((
            location,
            top_abs::compose(orientation, refs.orientation(reference)),
        ));
    }
    if parent.step_to_child >= 0 {
        let step_ref: RefId = refs.ref_at_step(parent.node, parent.step_to_child);
        let location = compose_location(&location, &refs.local_location(step_ref))?;
        return Ok((
            location,
            top_abs::compose(orientation, refs.orientation(step_ref)),
        ));
    }
    if parent.node.kind != NodeKind::CO_EDGE {
        // Occurrence to its child and Wire to CoEdge carry no transform; the native
        // debug assertions on the child kind are not compiled in release builds.
        return Ok((location, orientation));
    }
    let co_edge = CoEdgeId::new(parent.node.index);
    let def = graph.inc_storage().co_edge(co_edge);
    if co_edge.is_removed(graph) {
        return Ok((location, orientation));
    }
    Ok((
        location,
        top_abs::compose(orientation, def.orientation.orientation()),
    ))
}
