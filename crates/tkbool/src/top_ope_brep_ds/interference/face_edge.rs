// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_FaceEdgeInterference.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_FaceEdgeInterference`: an interference of an edge on a face.
use super::TopOpeBRepDSShapeShapeInterference;
use crate::top_ope_brep_ds::{TopOpeBRepDSConfig, TopOpeBRepDSKind, TopOpeBRepDSTransition};
use rustycad_tkernel::impl_transient;
use std::ops::Deref;

/// A face/edge interference (`TopOpeBRepDS_FaceEdgeInterference`): the support is the
/// FACE `s`, the geometry the EDGE `g`; the shape/shape members are reached through
/// `Deref`; `TopOpeBRepDSFaceEdgeInterference::new(&t, 1, 2, false, TopOpeBRepDSConfig::SameOriented)`.
pub struct TopOpeBRepDSFaceEdgeInterference {
    parent: TopOpeBRepDSShapeShapeInterference,
}
impl_transient!(
    TopOpeBRepDSFaceEdgeInterference,
    "TopOpeBRepDS_FaceEdgeInterference",
    TopOpeBRepDSShapeShapeInterference,
    parent
);

impl Deref for TopOpeBRepDSFaceEdgeInterference {
    type Target = TopOpeBRepDSShapeShapeInterference;

    fn deref(&self) -> &TopOpeBRepDSShapeShapeInterference {
        &self.parent
    }
}

impl TopOpeBRepDSFaceEdgeInterference {
    /// `TopOpeBRepDS_FaceEdgeInterference(T, S, G, GIsBound, C)`.
    pub fn new(
        t: &TopOpeBRepDSTransition,
        s: i32,
        g: i32,
        g_is_bound: bool,
        c: TopOpeBRepDSConfig,
    ) -> Self {
        Self {
            parent: TopOpeBRepDSShapeShapeInterference::new(
                t,
                TopOpeBRepDSKind::Face,
                s,
                TopOpeBRepDSKind::Edge,
                g,
                g_is_bound,
                c,
            ),
        }
    }
}
