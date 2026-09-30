// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_Zone.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT_Zone`: the frontier of the zone of proximity of a basic element, the arcs
//! around it.
use super::node::arc_ref;
use super::{MatArc, MatBasicElt, MatNode, MatSide};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::errors;

struct ZoneState {
    frontier: Vec<Handle<MatArc>>,
    limited: bool,
}

/// `MAT_Zone`: the arcs of a [`super::MatGraph`] that bound the zone of a basic
/// element, found by turning on the left from its end arc, then (when an infinite
/// node is reached) on the right from its start arc.
/// ```
/// use rustycad_tktopalgo::mat::MatZone;
/// let zone = MatZone::new();
/// assert!(!zone.no_empty_zone() && zone.limited());
/// ```
pub struct MatZone {
    parent: StandardTransient,
    state: SharedState<ZoneState>,
}
impl_transient!(MatZone, "MAT_Zone", StandardTransient, parent);

impl Default for MatZone {
    fn default() -> Self {
        Self::new()
    }
}

// The node a handle holds; a null one is a native null dereference.
fn node_ref(node: &Handle<MatNode>) -> Result<&MatNode, Failure> {
    node.get().ok_or_else(|| {
        errors::null_object("MAT_Zone: a null node is dereferenced; expected the ends of the arcs")
    })
}

impl MatZone {
    /// `MAT_Zone()`: an empty, limited zone; `MatZone::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(ZoneState {
                frontier: Vec::new(),
                limited: true,
            }),
        }
    }
    /// `MAT_Zone(aBasicElt)`: [`MatZone::perform`] on a new zone;
    /// `MatZone::with_basic_elt(&elt)?`.
    pub fn with_basic_elt(a_basic_elt: &Handle<MatBasicElt>) -> Result<Self, Failure> {
        let zone: Self = Self::new();
        zone.perform(a_basic_elt)?;
        Ok(zone)
    }
    /// `Perform(aBasicElt)`: computes the frontier of the zone of `a_basic_elt`;
    /// empty when the element has no end arc; `zone.perform(&elt)?`.
    pub fn perform(&self, a_basic_elt: &Handle<MatBasicElt>) -> Result<(), Failure> {
        {
            let mut state = self.state.write();
            state.limited = true;
            state.frontier.clear();
        }
        let current_arc: Handle<MatArc> = a_basic_elt.end_arc();
        if current_arc.is_null() {
            return Ok(());
        }
        let (frontier, limited) = zone_frontier(a_basic_elt, current_arc)?;
        let mut state = self.state.write();
        state.frontier = frontier;
        state.limited = limited;
        Ok(())
    }
    /// `NumberOfArcs()`: the number of arcs on the frontier; `zone.number_of_arcs()`.
    pub fn number_of_arcs(&self) -> i32 {
        self.state.read().frontier.len() as i32
    }
    /// `ArcOnFrontier(Index)`: the arc `index` of the frontier; Standard_OutOfRange
    /// outside `1..=NumberOfArcs()`; `zone.arc_on_frontier(1)?`.
    pub fn arc_on_frontier(&self, index: i32) -> Result<Handle<MatArc>, Failure> {
        let state = self.state.read();
        usize::try_from(index - 1)
            .ok()
            .and_then(|offset| state.frontier.get(offset).cloned())
            .ok_or_else(|| {
                errors::out_of_range(&format!(
                    "MAT_Zone::ArcOnFrontier: index {index}; expected 1..={}",
                    state.frontier.len()
                ))
            })
    }
    /// `NoEmptyZone()`: true when the frontier has arcs; `zone.no_empty_zone()`.
    pub fn no_empty_zone(&self) -> bool {
        !self.state.read().frontier.is_empty()
    }
    /// `Limited()`: false when the zone reaches an infinite node; `zone.limited()`.
    pub fn limited(&self) -> bool {
        self.state.read().limited
    }
}

// The frontier from the end arc: the arcs on the left up to a pending node or back
// to the start node, then, from an infinite node, the arcs on the right from the start
// arc up to the other infinite node.
fn zone_frontier(
    elt: &Handle<MatBasicElt>,
    mut current_arc: Handle<MatArc>,
) -> Result<(Vec<Handle<MatArc>>, bool), Failure> {
    let mut frontier: Vec<Handle<MatArc>> = vec![current_arc.clone()];
    let mut next_node: Handle<MatNode> = node_for_turn(&current_arc, elt, MatSide::Left)?;
    let start_node: Handle<MatNode> = current_arc.the_other_node(&next_node)?;
    while !node_ref(&next_node)?.pending_node()? && next_node != start_node {
        current_arc = arc_ref(&current_arc)?.neighbour(&next_node, MatSide::Left)?;
        frontier.push(current_arc.clone());
        next_node = arc_ref(&current_arc)?.the_other_node(&next_node)?;
    }
    if !node_ref(&next_node)?.infinite() {
        return Ok((frontier, true));
    }
    current_arc = elt.start_arc();
    frontier.push(current_arc.clone());
    next_node = node_for_turn(&current_arc, elt, MatSide::Right)?;
    while !node_ref(&next_node)?.infinite() {
        current_arc = arc_ref(&current_arc)?.neighbour(&next_node, MatSide::Right)?;
        frontier.push(current_arc.clone());
        next_node = arc_ref(&current_arc)?.the_other_node(&next_node)?;
    }
    Ok((frontier, false))
}

// `NodeForTurn(anArc, aBasicElt, aSide)`: the end of the arc from which the zone is
// followed on `a_side`.
fn node_for_turn(
    an_arc: &Handle<MatArc>,
    a_be: &Handle<MatBasicElt>,
    a_side: MatSide,
) -> Result<Handle<MatNode>, Failure> {
    let arc: &MatArc = arc_ref(an_arc)?;
    let mut node_sol: Handle<MatNode> = arc.first_node();
    let mut neighbour_arc: Handle<MatArc> = arc.neighbour(&node_sol, a_side)?;
    if neighbour_arc.is_null() {
        node_sol = arc.second_node();
        neighbour_arc = arc.neighbour(&node_sol, a_side)?;
    }
    let Some(neighbour) = neighbour_arc.get() else {
        return Ok(node_sol);
    };
    if neighbour.first_element() == *a_be || neighbour.second_element() == *a_be {
        return Ok(node_sol);
    }
    arc.the_other_node(&node_sol)
}
