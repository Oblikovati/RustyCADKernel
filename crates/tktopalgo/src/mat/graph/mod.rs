// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_Graph.hxx/.cxx (the maps, the accessors and the compactions) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT_Graph`: the graph of the bisecting locus, arcs, nodes and basic elements
//! indexed from 1.
mod build;
mod fusion;

pub use fusion::MatFusion;

use super::{MatArc, MatBasicElt, MatNode};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::errors;

/// The maps and counts of a graph.
#[derive(Default)]
pub(super) struct GraphState {
    pub arcs: DataMap<i32, Handle<MatArc>>,
    pub basic_elts: DataMap<i32, Handle<MatBasicElt>>,
    pub nodes: DataMap<i32, Handle<MatNode>>,
    pub number_of_arcs: i32,
    pub number_of_nodes: i32,
    pub number_of_basic_elts: i32,
    pub number_of_infinite_nodes: i32,
}

/// `MAT_Graph`: a transient graph built from the roots of the bisector trees of the
/// MAT algorithm ([`MatGraph::perform`]), then edited by the fusion of the basic
/// elements that the contour repeats. A missing index raises Standard_NoSuchObject.
/// ```
/// use rustycad_tktopalgo::mat::MatGraph;
/// let graph = MatGraph::new();
/// assert_eq!(graph.number_of_arcs(), 0);
/// assert!(graph.arc(1).is_err());
/// ```
pub struct MatGraph {
    parent: StandardTransient,
    state: SharedState<GraphState>,
}
impl_transient!(MatGraph, "MAT_Graph", StandardTransient, parent);

impl Default for MatGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl MatGraph {
    /// `MAT_Graph()`: an empty graph; `MatGraph::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(GraphState::default()),
        }
    }
    /// `Arc(Index)`; `graph.arc(1)?`.
    pub fn arc(&self, index: i32) -> Result<Handle<MatArc>, Failure> {
        self.state.read().arcs.find(&index).cloned()
    }
    /// `BasicElt(Index)`; `graph.basic_elt(1)?`.
    pub fn basic_elt(&self, index: i32) -> Result<Handle<MatBasicElt>, Failure> {
        self.state.read().basic_elts.find(&index).cloned()
    }
    /// `Node(Index)`; `graph.node(1)?`.
    pub fn node(&self, index: i32) -> Result<Handle<MatNode>, Failure> {
        self.state.read().nodes.find(&index).cloned()
    }
    /// `NumberOfArcs()`; `graph.number_of_arcs()`.
    pub fn number_of_arcs(&self) -> i32 {
        self.state.read().number_of_arcs
    }
    /// `NumberOfNodes()`; `graph.number_of_nodes()`.
    pub fn number_of_nodes(&self) -> i32 {
        self.state.read().number_of_nodes
    }
    /// `NumberOfBasicElts()`; `graph.number_of_basic_elts()`.
    pub fn number_of_basic_elts(&self) -> i32 {
        self.state.read().number_of_basic_elts
    }
    /// `NumberOfInfiniteNodes()`; `graph.number_of_infinite_nodes()`.
    pub fn number_of_infinite_nodes(&self) -> i32 {
        self.state.read().number_of_infinite_nodes
    }
    /// `CompactArcs()`: renumbers the arcs from 1 to fill the holes the fusions left;
    /// `graph.compact_arcs()?`.
    pub fn compact_arcs(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        let count: i32 = state.number_of_arcs;
        compact(&mut state.arcs, count, |arc, index| arc.set_index(index))
    }
    /// `CompactNodes()`: renumbers the nodes from 1; `graph.compact_nodes()?`.
    pub fn compact_nodes(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        let count: i32 = state.number_of_nodes;
        compact(&mut state.nodes, count, |node, index| node.set_index(index))
    }
    /// `ChangeBasicElts(NewMap)`: replaces the basic elements; each takes its key as
    /// index; `graph.change_basic_elts(&map)`.
    pub fn change_basic_elts(&self, new_map: &DataMap<i32, Handle<MatBasicElt>>) {
        let mut state = self.state.write();
        state.basic_elts.assign(new_map);
        for (key, value) in state.basic_elts.items() {
            value.set_index(*key);
        }
    }
    /// `ChangeBasicElt(Index)`; `graph.change_basic_elt(1)?`.
    pub fn change_basic_elt(&self, index: i32) -> Result<Handle<MatBasicElt>, Failure> {
        self.basic_elt(index)
    }
}

// `CompactArcs` and `CompactNodes`: from index 1, each bound item after a hole moves
// to the next free index (`set_index` renumbers it). Natively the scan never ends
// when fewer items than `count` are bound; the port stops at the largest key.
fn compact<T>(
    map: &mut DataMap<i32, Handle<T>>,
    count: i32,
    set_index: impl Fn(&T, i32),
) -> Result<(), Failure>
where
    T: rustycad_tkernel::standard::RuntimeTyped,
{
    let largest: i32 = map.items().map(|(key, _)| *key).max().unwrap_or(0);
    let (mut i_find, mut i, mut ya_decalage) = (0, 1, false);
    while i_find < count && i <= largest {
        if !map.is_bound(&i) {
            ya_decalage = true;
        } else {
            i_find += 1;
            if ya_decalage {
                let item: Handle<T> = map.find(&i)?.clone();
                let bound: &T = item.get().ok_or_else(|| {
                    errors::null_object(&format!(
                        "MAT_Graph: the item {i} is null; expected an arc or a node"
                    ))
                })?;
                set_index(bound, i_find);
                map.bind(i_find, item);
                map.unbind(&i);
            }
        }
        i += 1;
    }
    Ok(())
}
