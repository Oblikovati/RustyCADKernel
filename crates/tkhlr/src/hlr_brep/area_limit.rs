//! HLRBRep_AreaLimit at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::hlr_algo::Intersection;
use rustycad_tkernel::{
    impl_transient,
    standard::{Handle, StandardTransient},
};
use rustycad_tkg3d::top_abs::State;
use std::sync::{Mutex, MutexGuard};

/// An edge vertex with states on either side and independently assigned neighbor links.
/// Example: `limit.clear()` releases its links without changing the vertex or states.
pub struct AreaLimit {
    parent: StandardTransient,
    vertex: Intersection,
    boundary: bool,
    interference: bool,
    relations: Mutex<LimitRelations>,
}

struct LimitRelations {
    states: [State; 4],
    previous: Handle<AreaLimit>,
    next: Handle<AreaLimit>,
}

impl_transient!(AreaLimit, "HLRBRep_AreaLimit", StandardTransient, parent);

macro_rules! state_accessors {
    ($get:ident, $set:ident, $index:expr, $doc:literal) => {
        #[doc = $doc]
        pub fn $get(&self) -> State {
            self.relations().states[$index]
        }
        #[doc = concat!("Updates this state through shared handles; `limit.", stringify!($set), "(State::On)`.")]
        pub fn $set(&self, state: State) {
            self.relations().states[$index] = state;
        }
    };
}

impl AreaLimit {
    /// Copies the vertex and initializes null links; `AreaLimit::new(&v, true, false, a, b, c, d)`.
    pub fn new(
        vertex: &Intersection,
        boundary: bool,
        interference: bool,
        state_before: State,
        state_after: State,
        edge_before: State,
        edge_after: State,
    ) -> Self {
        Self {
            parent: StandardTransient::default(),
            vertex: *vertex,
            boundary,
            interference,
            relations: Mutex::new(LimitRelations {
                states: [state_before, state_after, edge_before, edge_after],
                previous: Handle::default(),
                next: Handle::default(),
            }),
        }
    }

    fn relations(&self) -> MutexGuard<'_, LimitRelations> {
        self.relations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Borrows the constructor's vertex copy; `let v = limit.vertex()`.
    pub fn vertex(&self) -> &Intersection {
        &self.vertex
    }
    /// Reports whether the vertex belongs to the edge boundary; `limit.is_boundary()`.
    pub fn is_boundary(&self) -> bool {
        self.boundary
    }
    /// Reports whether this vertex is an interference; `limit.is_interference()`.
    pub fn is_interference(&self) -> bool {
        self.interference
    }

    state_accessors!(
        state_before,
        set_state_before,
        0,
        "Reads the state before the vertex; `limit.state_before()`."
    );
    state_accessors!(
        state_after,
        set_state_after,
        1,
        "Reads the state after the vertex; `limit.state_after()`."
    );
    state_accessors!(
        edge_before,
        set_edge_before,
        2,
        "Reads the edge state before the vertex; `limit.edge_before()`."
    );
    state_accessors!(
        edge_after,
        set_edge_after,
        3,
        "Reads the edge state after the vertex; `limit.edge_after()`."
    );

    /// Returns the previous handle; `let previous = limit.previous()`.
    pub fn previous(&self) -> Handle<Self> {
        self.relations().previous.clone()
    }
    /// Assigns the previous link only; `limit.set_previous(&neighbor)`.
    pub fn set_previous(&self, previous: &Handle<Self>) {
        self.relations().previous = previous.clone();
    }
    /// Returns the next handle; `let next = limit.next()`.
    pub fn next(&self) -> Handle<Self> {
        self.relations().next.clone()
    }
    /// Assigns the next link only; `limit.set_next(&neighbor)`.
    pub fn set_next(&self, next: &Handle<Self>) {
        self.relations().next = next.clone();
    }
    /// Releases both links without changing any other state; `limit.clear()`.
    pub fn clear(&self) {
        let mut relations = self.relations();
        relations.previous = Handle::default();
        relations.next = Handle::default();
    }
}
