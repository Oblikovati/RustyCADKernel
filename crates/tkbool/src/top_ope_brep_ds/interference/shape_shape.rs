// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_ShapeShapeInterference.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_ShapeShapeInterference`: an interference of a shape on a shape, with
//! the bound flag of its geometry and the orientation configuration of the shapes.
use super::TopOpeBRepDSInterference;
use crate::top_ope_brep_ds::{TopOpeBRepDSConfig, TopOpeBRepDSKind, TopOpeBRepDSTransition};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::SharedState;
use std::ops::Deref;

/// A shape/shape interference (`TopOpeBRepDS_ShapeShapeInterference`); the
/// interference members are reached through `Deref`;
/// `TopOpeBRepDSShapeShapeInterference::new(&t, TopOpeBRepDSKind::Face, 1, TopOpeBRepDSKind::Edge, 2, false, TopOpeBRepDSConfig::SameOriented)`.
pub struct TopOpeBRepDSShapeShapeInterference {
    parent: TopOpeBRepDSInterference,
    my_gbound: SharedState<bool>,
    my_c: TopOpeBRepDSConfig,
}
impl_transient!(
    TopOpeBRepDSShapeShapeInterference,
    "TopOpeBRepDS_ShapeShapeInterference",
    TopOpeBRepDSInterference,
    parent
);

impl Deref for TopOpeBRepDSShapeShapeInterference {
    type Target = TopOpeBRepDSInterference;

    fn deref(&self) -> &TopOpeBRepDSInterference {
        &self.parent
    }
}

impl TopOpeBRepDSShapeShapeInterference {
    /// `TopOpeBRepDS_ShapeShapeInterference(T, ST, S, GT, G, GBound, C)`.
    pub fn new(
        t: &TopOpeBRepDSTransition,
        st: TopOpeBRepDSKind,
        s: i32,
        gt: TopOpeBRepDSKind,
        g: i32,
        gbound: bool,
        c: TopOpeBRepDSConfig,
    ) -> Self {
        Self {
            parent: TopOpeBRepDSInterference::new(t, st, s, gt, g),
            my_gbound: SharedState::new(gbound),
            my_c: c,
        }
    }

    /// `Config()`; `i.config()`.
    pub fn config(&self) -> TopOpeBRepDSConfig {
        self.my_c
    }

    /// `GBound()`: whether the geometry is a bound of the support; `i.gbound()`.
    pub fn gbound(&self) -> bool {
        *self.my_gbound.read()
    }

    /// `SetGBound(b)`; `i.set_gbound(true)`.
    pub fn set_gbound(&self, b: bool) {
        *self.my_gbound.write() = b;
    }
}
