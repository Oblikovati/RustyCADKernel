// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_CurvePointInterference.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_CurvePointInterference`: an interference of a point (or vertex) on a
//! curve (or edge) at a parameter.
use super::TopOpeBRepDSInterference;
use crate::top_ope_brep_ds::{TopOpeBRepDSKind, TopOpeBRepDSTransition};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::SharedState;
use std::ops::Deref;

/// A curve/point interference (`TopOpeBRepDS_CurvePointInterference`); the
/// interference members are reached through `Deref`;
/// `TopOpeBRepDSCurvePointInterference::new(&t, TopOpeBRepDSKind::Curve, 1, TopOpeBRepDSKind::Point, 2, 0.5)`.
pub struct TopOpeBRepDSCurvePointInterference {
    parent: TopOpeBRepDSInterference,
    my_param: SharedState<f64>,
}
impl_transient!(
    TopOpeBRepDSCurvePointInterference,
    "TopOpeBRepDS_CurvePointInterference",
    TopOpeBRepDSInterference,
    parent
);

impl Deref for TopOpeBRepDSCurvePointInterference {
    type Target = TopOpeBRepDSInterference;

    fn deref(&self) -> &TopOpeBRepDSInterference {
        &self.parent
    }
}

impl TopOpeBRepDSCurvePointInterference {
    /// `TopOpeBRepDS_CurvePointInterference(T, ST, S, GT, G, P)`.
    pub fn new(
        t: &TopOpeBRepDSTransition,
        st: TopOpeBRepDSKind,
        s: i32,
        gt: TopOpeBRepDSKind,
        g: i32,
        p: f64,
    ) -> Self {
        Self {
            parent: TopOpeBRepDSInterference::new(t, st, s, gt, g),
            my_param: SharedState::new(p),
        }
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
