// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_EdgeVertexInterference.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_EdgeVertexInterference`: an interference of a vertex on an edge (or
//! another support) at a parameter of the edge.
use super::TopOpeBRepDSShapeShapeInterference;
use crate::top_ope_brep_ds::{TopOpeBRepDSConfig, TopOpeBRepDSKind, TopOpeBRepDSTransition};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::SharedState;
use std::ops::Deref;

/// An edge/vertex interference (`TopOpeBRepDS_EdgeVertexInterference`); the
/// shape/shape members are reached through `Deref`;
/// `TopOpeBRepDSEdgeVertexInterference::on_edge(&t, 1, 2, true, TopOpeBRepDSConfig::UnshGeometry, 0.5)`.
pub struct TopOpeBRepDSEdgeVertexInterference {
    parent: TopOpeBRepDSShapeShapeInterference,
    my_param: SharedState<f64>,
}
impl_transient!(
    TopOpeBRepDSEdgeVertexInterference,
    "TopOpeBRepDS_EdgeVertexInterference",
    TopOpeBRepDSShapeShapeInterference,
    parent
);

impl Deref for TopOpeBRepDSEdgeVertexInterference {
    type Target = TopOpeBRepDSShapeShapeInterference;

    fn deref(&self) -> &TopOpeBRepDSShapeShapeInterference {
        &self.parent
    }
}

impl TopOpeBRepDSEdgeVertexInterference {
    /// `TopOpeBRepDS_EdgeVertexInterference(T, ST, S, G, GIsBound, C, P)`: the geometry
    /// is the VERTEX `g`.
    pub fn new(
        t: &TopOpeBRepDSTransition,
        st: TopOpeBRepDSKind,
        s: i32,
        g: i32,
        g_is_bound: bool,
        c: TopOpeBRepDSConfig,
        p: f64,
    ) -> Self {
        let parent = TopOpeBRepDSShapeShapeInterference::new(
            t,
            st,
            s,
            TopOpeBRepDSKind::Vertex,
            g,
            g_is_bound,
            c,
        );
        Self {
            parent,
            my_param: SharedState::new(p),
        }
    }

    /// `TopOpeBRepDS_EdgeVertexInterference(T, S, G, GIsBound, C, P)`: the support is
    /// the EDGE `s`.
    pub fn on_edge(
        t: &TopOpeBRepDSTransition,
        s: i32,
        g: i32,
        g_is_bound: bool,
        c: TopOpeBRepDSConfig,
        p: f64,
    ) -> Self {
        Self::new(t, TopOpeBRepDSKind::Edge, s, g, g_is_bound, c, p)
    }

    /// `Parameter()`; `i.parameter()`.
    pub fn parameter(&self) -> f64 {
        *self.my_param.read()
    }

    /// `Parameter(P)`; `i.set_parameter(0.25)`.
    pub fn set_parameter(&self, p: f64) {
        *self.my_param.write() = p;
    }
}
