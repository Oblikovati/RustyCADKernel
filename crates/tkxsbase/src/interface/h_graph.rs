// Rust port of DataExchange/TKXSBase/Interface/Interface_HGraph (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_HGraph`: an `Interface_Graph` shared through a handle.
use super::{
    InterfaceGTool, InterfaceGeneralLib, InterfaceGraph, InterfaceInterfaceModel, InterfaceProtocol,
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// `Interface_HGraph`; the graph is lent through its lock (`Graph`, `CGraph`).
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceHGraph;
/// fn size(hgraph: &InterfaceHGraph) -> i32 { hgraph.graph().size() }
/// ```
pub struct InterfaceHGraph {
    transient: StandardTransient,
    thegraph: SharedState<InterfaceGraph>,
}
impl_transient!(
    InterfaceHGraph,
    "Interface_HGraph",
    StandardTransient,
    transient
);

impl InterfaceHGraph {
    /// `Interface_HGraph(agraph)`: a copy of the graph.
    pub fn from_graph(agraph: &InterfaceGraph) -> Self {
        Self::wrap(agraph.clone())
    }
    /// `Interface_HGraph(amodel, lib, theModeStats = true)`.
    pub fn with_lib(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        lib: &InterfaceGeneralLib,
        mode_stats: bool,
    ) -> Result<Self, Failure> {
        Ok(Self::wrap(InterfaceGraph::with_lib(
            amodel, lib, mode_stats,
        )?))
    }
    /// `Interface_HGraph(amodel, protocol, theModeStats = true)`.
    pub fn with_protocol(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
        mode_stats: bool,
    ) -> Result<Self, Failure> {
        Ok(Self::wrap(InterfaceGraph::with_protocol(
            amodel, protocol, mode_stats,
        )?))
    }
    /// `Interface_HGraph(amodel, gtool, theModeStats = true)`.
    pub fn with_gtool(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        gtool: &Handle<InterfaceGTool>,
        mode_stats: bool,
    ) -> Result<Self, Failure> {
        Ok(Self::wrap(InterfaceGraph::with_gtool(
            amodel, gtool, mode_stats,
        )?))
    }
    /// `Interface_HGraph(amodel, theModeStats = true)`.
    pub fn new(
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        mode_stats: bool,
    ) -> Result<Self, Failure> {
        Ok(Self::wrap(InterfaceGraph::new(amodel, mode_stats)?))
    }
    fn wrap(graph: InterfaceGraph) -> Self {
        Self {
            transient: StandardTransient::default(),
            thegraph: SharedState::new(graph),
        }
    }
    /// `Graph`: the graph, read only (a guard to drop before editing it).
    pub fn graph(&self) -> RwLockReadGuard<'_, InterfaceGraph> {
        self.thegraph.read()
    }
    /// `CGraph`: the graph, modifiable.
    pub fn c_graph(&self) -> RwLockWriteGuard<'_, InterfaceGraph> {
        self.thegraph.write()
    }
}
