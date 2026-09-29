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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_ParentExplorer.hxx and
// .cxx (the configuration, the constructors and the accessors)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_ParentExplorer`: the path-aware upward walk from a node to the graph
//! roots.
//!
//! Every path contributes its own ancestor sequence: Vertex to Edge, Edge to CoEdge,
//! CoEdge to Wire, Wire to Face, Face to Shell, Shell to Solid, Solid to
//! CompSolid/Compound, any node to its compounds and occurrences, Product to its
//! occurrences and Occurrence to its parent product. A branch is emitted from the
//! start node upwards once its root is reached, with the location and orientation
//! accumulated from the branch root down to each frame. The native overloads are
//! named constructors; as in the child explorer, the stack keeps the native layout and
//! refuses a push past the node count.
mod steps;
mod walk;

use crate::brep_graph::{BRepGraph, NodeId, NodeKind, RefId};
use crate::brep_graph_inc::NodeInstance;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// `BRepGraph_ParentExplorer::LinkKind`: how `Current()` owns `CurrentChild()`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LinkKind {
    /// No current branch step.
    #[default]
    None,
    /// Through a reference.
    Reference,
    /// Through a structural link.
    Structural,
}

/// `BRepGraph_ParentExplorer::TraversalMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TraversalMode {
    /// The whole ancestor chain.
    #[default]
    Recursive,
    /// The immediate parents only.
    DirectParents,
}

/// `BRepGraph_ParentExplorer::Config`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Config {
    /// `Mode`.
    pub mode: TraversalMode,
    /// `TargetKind`: emit only this kind (`None` emits every ancestor).
    pub target_kind: Option<NodeKind>,
    /// `AvoidKind`: do not ascend through this kind.
    pub avoid_kind: Option<NodeKind>,
    /// `EmitAvoidKind`: emit avoid-kind ancestors once.
    pub emit_avoid_kind: bool,
}

/// `StackFrame`: one node of the explored branch, with the step to the frame below.
#[derive(Clone, Debug)]
struct StackFrame {
    node: NodeId,
    next_parent_idx: u32,
    step_to_child: i32,
    ref_to_child: RefId,
    acc_location: Location,
    acc_orientation: Orientation,
}

impl Default for StackFrame {
    fn default() -> Self {
        Self {
            node: NodeId::default(),
            next_parent_idx: 0,
            step_to_child: -1,
            ref_to_child: RefId::default(),
            acc_location: Location::default(),
            acc_orientation: Orientation::Forward,
        }
    }
}

impl StackFrame {
    fn start(node: NodeId) -> Self {
        Self {
            node,
            ..Self::default()
        }
    }
}

/// `BRepGraph_ParentExplorer`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::parent_explorer::ParentExplorer;
/// use rustycad_tkbrep::brep_graph::{BRepGraph, FaceId};
/// let graph = BRepGraph::new();
/// assert!(!ParentExplorer::new(&graph, FaceId::new(0).into()).unwrap().more());
/// ```
pub struct ParentExplorer<'g> {
    graph: &'g BRepGraph,
    config: Config,
    stack: Vec<StackFrame>,
    stack_top: i32,
    emit_index: i32,
    current_frame: i32,
    current: NodeId,
    location: Location,
    orientation: Orientation,
    has_more: bool,
}

impl<'g> ParentExplorer<'g> {
    /// `BRepGraph_ParentExplorer(theGraph, theNode, theConfig)`: the avoid kind is
    /// normalised against the target and the start node, then the walk starts.
    pub fn with_config(
        graph: &'g BRepGraph,
        node: NodeId,
        config: &Config,
    ) -> Result<Self, Failure> {
        let mut config = *config;
        config.avoid_kind = normalize_avoid_kind(node, config.target_kind, config.avoid_kind);
        let mut explorer = Self {
            graph,
            config,
            stack: Vec::new(),
            stack_top: -1,
            emit_index: -1,
            current_frame: -1,
            current: NodeId::default(),
            location: Location::default(),
            orientation: Orientation::Forward,
            has_more: false,
        };
        if node.is_valid() {
            explorer.push_frame(StackFrame::start(node));
        }
        explorer.advance()?;
        Ok(explorer)
    }

    /// `BRepGraph_ParentExplorer(theGraph, theNode)`: every ancestor.
    pub fn new(graph: &'g BRepGraph, node: NodeId) -> Result<Self, Failure> {
        Self::with_config(graph, node, &Config::default())
    }

    /// `BRepGraph_ParentExplorer(theGraph, theNode, theMode)`.
    pub fn with_mode(
        graph: &'g BRepGraph,
        node: NodeId,
        mode: TraversalMode,
    ) -> Result<Self, Failure> {
        Self::with_config(
            graph,
            node,
            &Config {
                mode,
                ..Config::default()
            },
        )
    }

    /// `BRepGraph_ParentExplorer(theGraph, theNode, theAvoidKind, theEmitAvoidKind,
    /// theMode)`.
    pub fn with_avoid(
        graph: &'g BRepGraph,
        node: NodeId,
        avoid_kind: Option<NodeKind>,
        emit_avoid_kind: bool,
        mode: TraversalMode,
    ) -> Result<Self, Failure> {
        let config = Config {
            mode,
            target_kind: None,
            avoid_kind,
            emit_avoid_kind,
        };
        Self::with_config(graph, node, &config)
    }

    /// `BRepGraph_ParentExplorer(theGraph, theNode, theTargetKind)`: recursive.
    pub fn with_target(
        graph: &'g BRepGraph,
        node: NodeId,
        target_kind: NodeKind,
    ) -> Result<Self, Failure> {
        Self::with_target_mode(graph, node, target_kind, TraversalMode::Recursive)
    }

    /// `BRepGraph_ParentExplorer(theGraph, theNode, theTargetKind, theMode)`.
    pub fn with_target_mode(
        graph: &'g BRepGraph,
        node: NodeId,
        target_kind: NodeKind,
        mode: TraversalMode,
    ) -> Result<Self, Failure> {
        let config = Config {
            mode,
            target_kind: Some(target_kind),
            ..Config::default()
        };
        Self::with_config(graph, node, &config)
    }

    /// `BRepGraph_ParentExplorer(theGraph, theNode, theTargetKind, theAvoidKind,
    /// theEmitAvoidKind, theMode)`.
    pub fn with_target_avoid(
        graph: &'g BRepGraph,
        node: NodeId,
        target_kind: NodeKind,
        avoid_kind: Option<NodeKind>,
        emit_avoid_kind: bool,
        mode: TraversalMode,
    ) -> Result<Self, Failure> {
        let config = Config {
            mode,
            target_kind: Some(target_kind),
            avoid_kind,
            emit_avoid_kind,
        };
        Self::with_config(graph, node, &config)
    }

    /// `GetConfig()`: the configuration, with the normalised avoid kind.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        self.has_more
    }

    /// `Next()`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible; Iterator is implemented too.
    pub fn next(&mut self) -> Result<(), Failure> {
        self.advance()
    }

    /// `Current()`: the current ancestor with its accumulated transforms; the default
    /// instance after the end.
    pub fn current(&self) -> NodeInstance {
        if !self.has_more {
            return NodeInstance::default();
        }
        NodeInstance {
            def_id: self.current,
            location: self.location.clone(),
            orientation: self.orientation,
        }
    }

    // The current frame when an ancestor is current.
    fn current_frame_index(&self) -> Option<usize> {
        (self.has_more && self.current_frame >= 0).then_some(self.current_frame as usize)
    }

    /// `CurrentChild()`: the frame below `Current()` on the branch.
    pub fn current_child(&self) -> NodeId {
        match self.current_frame_index() {
            Some(frame) if frame > 0 => self.stack[frame - 1].node,
            _ => NodeId::default(),
        }
    }

    /// `CurrentLinkKind()`.
    pub fn current_link_kind(&self) -> LinkKind {
        match self.current_frame_index() {
            None => LinkKind::None,
            Some(frame) if self.stack[frame].step_to_child >= 0 => LinkKind::Reference,
            Some(_) => LinkKind::Structural,
        }
    }

    /// `CurrentRef()`: the reference of the current step (the stored one, else
    /// `RefAtStep` of the step), invalid for a structural step.
    pub fn current_ref(&self) -> RefId {
        let Some(frame) = self.current_frame_index().map(|index| &self.stack[index]) else {
            return RefId::default();
        };
        if frame.step_to_child < 0 {
            return RefId::default();
        }
        if frame.ref_to_child.is_valid() {
            return frame.ref_to_child;
        }
        self.graph
            .refs()
            .generic()
            .ref_at_step(frame.node, frame.step_to_child)
    }

    /// `LeafLocation()`: the accumulated location at the start node of the branch.
    pub fn leaf_location(&self) -> Location {
        if self.has_more {
            self.stack[0].acc_location.clone()
        } else {
            Location::default()
        }
    }

    /// `LeafOrientation()`.
    pub fn leaf_orientation(&self) -> Orientation {
        if self.has_more {
            self.stack[0].acc_orientation
        } else {
            Orientation::Forward
        }
    }

    /// `IsCurrentBranchRoot()`: `Current()` is the explicit root of its branch.
    pub fn is_current_branch_root(&self) -> bool {
        self.has_more && self.current_frame == self.branch_root_frame()
    }

    // `branchRootFrame`: the topmost frame, moved down past structural steps.
    fn branch_root_frame(&self) -> i32 {
        if self.stack_top < 0 {
            return -1;
        }
        let mut frame = self.stack_top;
        while frame > 0 && self.stack[frame as usize].step_to_child < 0 {
            frame -= 1;
        }
        frame
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` instance; an error of
/// `next` is yielded once and ends the iteration.
impl Iterator for ParentExplorer<'_> {
    type Item = Result<NodeInstance, Failure>;
    fn next(&mut self) -> Option<Result<NodeInstance, Failure>> {
        if !self.more() {
            return None;
        }
        let current = self.current();
        match ParentExplorer::next(self) {
            Ok(()) => Some(Ok(current)),
            Err(error) => {
                self.has_more = false;
                Some(Err(error))
            }
        }
    }
}

// `normalizeAvoidKind`: with a target, the avoid kind must lie strictly between the
// start node and the target.
fn normalize_avoid_kind(
    node: NodeId,
    target: Option<NodeKind>,
    avoid: Option<NodeKind>,
) -> Option<NodeKind> {
    use crate::brep_graph::explorer_kinds::kind_can_contain;
    let (Some(avoid_kind), Some(target_kind)) = (avoid, target) else {
        return avoid;
    };
    if !node.is_valid() {
        return avoid;
    }
    if avoid_kind == target_kind || !kind_can_contain(target_kind, avoid_kind) {
        return None;
    }
    (node.kind == avoid_kind || kind_can_contain(avoid_kind, node.kind)).then_some(avoid_kind)
}
