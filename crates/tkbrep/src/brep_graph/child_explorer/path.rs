// Rust translation of the path queries of BRepGraph_ChildExplorer (CurrentParent,
// CurrentLinkKind, CurrentRef, CurrentUsagePath, LocationOf, NodeOf, LocationAt,
// NodeAt, Depth) and childRefIdForStep (ModelingData/TKBRep/BRepGraph/
// BRepGraph_ChildExplorer.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The path queries of [`ChildExplorer`](super::ChildExplorer).
use super::{ChildExplorer, LinkKind, StackFrame};
use crate::brep_graph::{BRepGraph, NodeId, NodeKind, ProductId, RefId, UsagePath, UsageStep};
use rustycad_tkmath::toploc::Location;

impl ChildExplorer<'_> {
    // The current frame, when a match is current and it is not the root.
    fn current_step_frame(&self) -> Option<usize> {
        (self.has_more && self.current_frame > 0).then_some(self.current_frame as usize)
    }

    /// `CurrentParent()`: the node before `Current()` on the path, invalid for the root.
    pub fn current_parent(&self) -> NodeId {
        self.current_step_frame()
            .map_or(NodeId::default(), |frame| self.stack[frame - 1].node)
    }

    /// `CurrentLinkKind()`.
    pub fn current_link_kind(&self) -> LinkKind {
        match self.current_step_frame() {
            None => LinkKind::None,
            Some(frame) if self.stack[frame].step_from_parent >= 0 => LinkKind::Reference,
            Some(_) => LinkKind::Structural,
        }
    }

    /// `CurrentRef()`: the parent-owned reference of the current step, invalid for a
    /// structural step (CoEdge to Edge, Occurrence to its child, Wire to CoEdge).
    pub fn current_ref(&self) -> RefId {
        let Some(frame) = self.current_step_frame() else {
            return RefId::default();
        };
        let cached = self.stack[frame].reference;
        if cached.is_valid() {
            return cached;
        }
        child_ref_id_for_step(
            self.graph,
            self.stack[frame - 1].node,
            self.stack[frame].step_from_parent,
        )
    }

    /// `CurrentUsagePath()`: the steps from the root to `Current()`.
    pub fn current_usage_path(&self) -> UsagePath {
        if !self.has_more || self.current_frame < 0 {
            return UsagePath::new();
        }
        let frames = &self.stack[..=self.current_frame as usize];
        let mut path = UsagePath::with_capacity(frames.len());
        for frame in frames {
            path.append(UsageStep {
                node: frame.node,
                reference: frame.reference,
                step_index: frame.step_from_parent,
            });
        }
        path
    }

    // The frames `LocationOf` and `NodeOf` scan: 1 up to the current frame (or the top).
    fn scanned_frames(&self) -> &[StackFrame] {
        let last = if self.current_frame >= 0 {
            self.current_frame
        } else {
            self.stack_top
        };
        if last < 1 {
            return &[];
        }
        &self.stack[1..=last as usize]
    }

    /// `LocationOf(theKind)`: the accumulated location at the first path step of the
    /// kind (the root excluded), else at the current match of the kind, else identity.
    pub fn location_of(&self, kind: NodeKind) -> Location {
        if let Some(frame) = self
            .scanned_frames()
            .iter()
            .find(|frame| frame.node.kind == kind)
        {
            return frame.acc_location.clone();
        }
        if self.has_more && self.current.kind == kind {
            return self.location.clone();
        }
        Location::default()
    }

    /// `NodeOf(theKind)`: the node of the first path step of the kind, as `LocationOf`.
    pub fn node_of(&self, kind: NodeKind) -> NodeId {
        if let Some(frame) = self
            .scanned_frames()
            .iter()
            .find(|frame| frame.node.kind == kind)
        {
            return frame.node;
        }
        if self.has_more && self.current.kind == kind {
            return self.current;
        }
        NodeId::default()
    }

    // The frame of `LocationAt`/`NodeAt(level)`: `level + 1`, when on the current path.
    fn frame_at_level(&self, level: i32) -> Option<&StackFrame> {
        let frame = level.checked_add(1)?;
        if self.current_frame < 0 || frame < 0 || frame > self.current_frame {
            return None;
        }
        self.stack.get(frame as usize)
    }

    /// `LocationAt(theLevel)`: the accumulated location after step `level` (level 0 is
    /// the first step after the root); identity off the current path.
    pub fn location_at(&self, level: i32) -> Location {
        self.frame_at_level(level)
            .map_or(Location::default(), |frame| frame.acc_location.clone())
    }

    /// `NodeAt(theLevel)`: the node after step `level`; invalid off the current path.
    pub fn node_at(&self, level: i32) -> NodeId {
        self.frame_at_level(level)
            .map_or(NodeId::default(), |frame| frame.node)
    }

    /// `Depth()`: the frames on the stack.
    pub fn depth(&self) -> i32 {
        if self.stack_top < 0 {
            0
        } else {
            self.stack_top + 1
        }
    }
}

// `childRefIdForStep`: `RefAtStep` of the parent, or the `step`-th active occurrence
// reference of a product.
fn child_ref_id_for_step(graph: &BRepGraph, parent: NodeId, step: i32) -> RefId {
    if !parent.is_valid() || step < 0 {
        return RefId::default();
    }
    if parent.kind != NodeKind::PRODUCT {
        return graph.refs().generic().ref_at_step(parent, step);
    }
    let storage = graph.inc_storage();
    storage
        .product_relations(ProductId::from_node_id(parent))
        .occurrence_ref_ids
        .iter()
        .filter(|reference| {
            !reference.is_removed(graph)
                && !storage
                    .occurrence_ref(**reference)
                    .child_occurrence_id
                    .is_removed(graph)
        })
        .nth(step as usize)
        .map_or(RefId::default(), |reference| (*reference).into())
}
