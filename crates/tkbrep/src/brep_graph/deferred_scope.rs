// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_DeferredScope.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_DeferredScope`: the scope of a batch of mutations under deferred
//! invalidation.
use super::BRepGraph;
use std::ops::{Deref, DerefMut};

/// `BRepGraph_DeferredScope`: begins deferred invalidation unless it is already active
/// (a nested scope is then a no-op) and, when it began it, ends it and commits the
/// mutation when dropped, on every exit path. It holds the graph's mutable borrow and
/// dereferences to the graph, so the batch is edited through it.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, deferred_scope::DeferredScope};
/// let mut graph = BRepGraph::new();
/// {
///     let mut scope = DeferredScope::new(&mut graph);
///     assert!(scope.editor_state().is_deferred_mode());
///     scope.editor().shells().add();
/// }
/// assert!(!graph.editor_state().is_deferred_mode());
/// ```
#[derive(Debug)]
pub struct DeferredScope<'g> {
    graph: &'g mut BRepGraph,
    owns_scope: bool,
}

impl<'g> DeferredScope<'g> {
    /// `BRepGraph_DeferredScope(theGraph)`.
    pub fn new(graph: &'g mut BRepGraph) -> Self {
        let owns_scope = !graph.editor_state().is_deferred_mode();
        if owns_scope {
            graph.editor().begin_deferred_invalidation();
        }
        Self { graph, owns_scope }
    }
}

impl Deref for DeferredScope<'_> {
    type Target = BRepGraph;

    fn deref(&self) -> &BRepGraph {
        self.graph
    }
}

impl DerefMut for DeferredScope<'_> {
    fn deref_mut(&mut self) -> &mut BRepGraph {
        self.graph
    }
}

impl Drop for DeferredScope<'_> {
    /// `~BRepGraph_DeferredScope()`: `EndDeferredInvalidation` then `CommitMutation`
    /// when this scope began the mode.
    fn drop(&mut self) {
        if self.owns_scope {
            let mut editor = self.graph.editor();
            editor.end_deferred_invalidation();
            editor.commit_mutation();
        }
    }
}
