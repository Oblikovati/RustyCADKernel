// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_ChildExplorer.hxx and
// .cxx (the configuration, the constructors and the accessors)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_ChildExplorer`: the lazy depth-first downward walk from a root node,
//! composing location and orientation along the path.
//!
//! Every node kind is a distinct step (Compound to children, CompSolid to solids, Solid
//! to shells, Shell to faces, Face to wires, Wire to coedges, CoEdge to edge, Edge to
//! vertices, Product to occurrences, Occurrence to its product or topology root), and a
//! definition reached through several paths is visited once per path. The native
//! constructor overloads are named constructors; the ones taking a root accept any
//! id convertible to `NodeId`, which covers the `ProductId` and typed-id overloads.
//! The stack keeps the native layout: frames above the top stay allocated, and a push
//! past the node count is refused (the cycle guard), after which the current frame is
//! the parent's, as natively.
mod path;
mod walk;

use crate::brep_graph::{BRepGraph, NodeId, NodeKind, RefId};
use crate::brep_graph_inc::NodeInstance;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// `BRepGraph_ChildExplorer::LinkKind`: how `Current()` hangs from `CurrentParent()`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LinkKind {
    /// No incoming link (the root or a self match).
    #[default]
    None,
    /// Through a parent-owned reference.
    Reference,
    /// Through a structural link without a reference.
    Structural,
}

/// `BRepGraph_ChildExplorer::TraversalMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TraversalMode {
    /// The whole subgraph, depth first.
    #[default]
    Recursive,
    /// The immediate children of the root only.
    DirectChildren,
}

/// `BRepGraph_ChildExplorer::Config`: the traversal configuration.
#[derive(Clone, Debug)]
pub struct Config {
    /// `Mode`.
    pub mode: TraversalMode,
    /// `TargetKind`: emit only this kind (`None` emits every node).
    pub target_kind: Option<NodeKind>,
    /// `AvoidKind`: do not descend into this kind.
    pub avoid_kind: Option<NodeKind>,
    /// `EmitAvoidKind`: emit avoid-kind nodes once before skipping their subtree.
    pub emit_avoid_kind: bool,
    /// `AccumulateLocation`.
    pub accumulate_location: bool,
    /// `AccumulateOrientation`.
    pub accumulate_orientation: bool,
    /// `StartLoc`.
    pub start_loc: Location,
    /// `StartOri`.
    pub start_ori: Orientation,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: TraversalMode::Recursive,
            target_kind: None,
            avoid_kind: None,
            emit_avoid_kind: false,
            accumulate_location: true,
            accumulate_orientation: true,
            start_loc: Location::default(),
            start_ori: Orientation::Forward,
        }
    }
}

/// `StackFrame`: one node of the explored path.
#[derive(Clone, Debug)]
struct StackFrame {
    node: NodeId,
    next_child_idx: u32,
    step_from_parent: i32,
    reference: RefId,
    acc_location: Location,
    acc_orientation: Orientation,
}

impl StackFrame {
    fn root(node: NodeId, location: &Location, orientation: Orientation) -> Self {
        Self {
            node,
            next_child_idx: 0,
            step_from_parent: -1,
            reference: RefId::default(),
            acc_location: location.clone(),
            acc_orientation: orientation,
        }
    }
}

/// `BRepGraph_ChildExplorer`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::child_explorer::ChildExplorer;
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeKind, SolidId};
/// let graph = BRepGraph::new();
/// let explorer = ChildExplorer::with_target(&graph, SolidId::new(0), NodeKind::FACE).unwrap();
/// assert!(!explorer.more());
/// ```
pub struct ChildExplorer<'g> {
    graph: &'g BRepGraph,
    root: NodeId,
    config: Config,
    stack: Vec<StackFrame>,
    stack_top: i32,
    current_frame: i32,
    current: NodeId,
    location: Location,
    orientation: Orientation,
    has_more: bool,
}

impl<'g> ChildExplorer<'g> {
    /// `BRepGraph_ChildExplorer(theGraph, theRoot, theConfig)`: the avoid kind is
    /// normalised against the target kind, then the walk starts.
    pub fn with_config(
        graph: &'g BRepGraph,
        root: NodeId,
        config: &Config,
    ) -> Result<Self, Failure> {
        let mut config = config.clone();
        config.avoid_kind = normalize_avoid_kind(config.avoid_kind, config.target_kind);
        let mut explorer = Self {
            graph,
            root,
            location: config.start_loc.clone(),
            orientation: config.start_ori,
            config,
            stack: Vec::new(),
            stack_top: -1,
            current_frame: -1,
            current: NodeId::default(),
            has_more: false,
        };
        explorer.start_traversal()?;
        Ok(explorer)
    }

    /// `BRepGraph_ChildExplorer(theGraph, theRoot)`: every descendant, recursively.
    pub fn new(graph: &'g BRepGraph, root: NodeId) -> Result<Self, Failure> {
        Self::with_config(graph, root, &Config::default())
    }

    /// `BRepGraph_ChildExplorer(theGraph, theRoot, theMode)`.
    pub fn with_mode(
        graph: &'g BRepGraph,
        root: NodeId,
        mode: TraversalMode,
    ) -> Result<Self, Failure> {
        Self::with_config(
            graph,
            root,
            &Config {
                mode,
                ..Config::default()
            },
        )
    }

    /// `BRepGraph_ChildExplorer(theGraph, theRoot, theAvoidKind, theEmitAvoidKind,
    /// theMode)`: every descendant, pruning at the avoid kind.
    pub fn with_avoid(
        graph: &'g BRepGraph,
        root: NodeId,
        avoid_kind: Option<NodeKind>,
        emit_avoid_kind: bool,
        mode: TraversalMode,
    ) -> Result<Self, Failure> {
        let config = Config {
            mode,
            avoid_kind,
            emit_avoid_kind,
            ..Config::default()
        };
        Self::with_config(graph, root, &config)
    }

    /// `BRepGraph_ChildExplorer(theGraph, theRoot, theTargetKind)` (and the `ProductId`
    /// and typed-id overloads): the descendants of the target kind, recursively.
    pub fn with_target(
        graph: &'g BRepGraph,
        root: impl Into<NodeId>,
        target_kind: NodeKind,
    ) -> Result<Self, Failure> {
        Self::with_target_mode(graph, root, target_kind, TraversalMode::Recursive)
    }

    /// `BRepGraph_ChildExplorer(theGraph, theRoot, theTargetKind, theMode)` (and the
    /// `ProductId` and typed-id overloads).
    pub fn with_target_mode(
        graph: &'g BRepGraph,
        root: impl Into<NodeId>,
        target_kind: NodeKind,
        mode: TraversalMode,
    ) -> Result<Self, Failure> {
        let config = Config {
            mode,
            target_kind: Some(target_kind),
            ..Config::default()
        };
        Self::with_config(graph, root.into(), &config)
    }

    /// `BRepGraph_ChildExplorer(theGraph, theRoot, theTargetKind, theAvoidKind,
    /// theEmitAvoidKind, theMode)`.
    pub fn with_target_avoid(
        graph: &'g BRepGraph,
        root: NodeId,
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
            ..Config::default()
        };
        Self::with_config(graph, root, &config)
    }

    /// `BRepGraph_ChildExplorer(theGraph, theRoot, theTargetKind, theCumLoc, theCumOri,
    /// theMode)` (and the `ProductId` and typed-id overloads).
    pub fn with_accumulation(
        graph: &'g BRepGraph,
        root: impl Into<NodeId>,
        target_kind: NodeKind,
        accumulate_location: bool,
        accumulate_orientation: bool,
        mode: TraversalMode,
    ) -> Result<Self, Failure> {
        let config = Config {
            mode,
            target_kind: Some(target_kind),
            accumulate_location,
            accumulate_orientation,
            ..Config::default()
        };
        Self::with_config(graph, root.into(), &config)
    }

    /// `BRepGraph_ChildExplorer(theGraph, theRoot, theTargetKind, theStartLoc,
    /// theStartOri, theMode)`: an explicit initial transform (the native default mode
    /// of this overload is DirectChildren).
    pub fn with_start(
        graph: &'g BRepGraph,
        root: NodeId,
        target_kind: NodeKind,
        start_loc: &Location,
        start_ori: Orientation,
        mode: TraversalMode,
    ) -> Result<Self, Failure> {
        let config = Config {
            mode,
            target_kind: Some(target_kind),
            start_loc: start_loc.clone(),
            start_ori,
            ..Config::default()
        };
        Self::with_config(graph, root, &config)
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

    /// `Current()`: the current node with its accumulated location and orientation (the
    /// last match after the end).
    pub fn current(&self) -> NodeInstance {
        NodeInstance {
            def_id: self.current,
            location: self.location.clone(),
            orientation: self.orientation,
        }
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` instance; an error of
/// `next` is yielded once and ends the iteration.
impl Iterator for ChildExplorer<'_> {
    type Item = Result<NodeInstance, Failure>;
    fn next(&mut self) -> Option<Result<NodeInstance, Failure>> {
        if !self.more() {
            return None;
        }
        let current = self.current();
        match ChildExplorer::next(self) {
            Ok(()) => Some(Ok(current)),
            Err(error) => {
                self.has_more = false;
                Some(Err(error))
            }
        }
    }
}

// `normalizeAvoidKind`: an avoid kind equal to the target, or unable to contain it,
// is dropped.
fn normalize_avoid_kind(avoid: Option<NodeKind>, target: Option<NodeKind>) -> Option<NodeKind> {
    let (Some(avoid_kind), Some(target_kind)) = (avoid, target) else {
        return avoid;
    };
    if avoid_kind == target_kind {
        return None;
    }
    super::explorer_kinds::kind_can_contain(avoid_kind, target_kind).then_some(avoid_kind)
}
