// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_SolidSurfaceInterference.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_SolidSurfaceInterference`: an interference of a surface on a solid.
use super::TopOpeBRepDSInterference;
use crate::top_ope_brep_ds::{TopOpeBRepDSKind, TopOpeBRepDSTransition};
use rustycad_tkernel::impl_transient;
use std::ops::Deref;

/// A solid/surface interference (`TopOpeBRepDS_SolidSurfaceInterference`); the
/// interference members are reached through `Deref`;
/// `TopOpeBRepDSSolidSurfaceInterference::new(&t, TopOpeBRepDSKind::Solid, 1, TopOpeBRepDSKind::Surface, 1)`.
pub struct TopOpeBRepDSSolidSurfaceInterference {
    parent: TopOpeBRepDSInterference,
}
impl_transient!(
    TopOpeBRepDSSolidSurfaceInterference,
    "TopOpeBRepDS_SolidSurfaceInterference",
    TopOpeBRepDSInterference,
    parent
);

impl Deref for TopOpeBRepDSSolidSurfaceInterference {
    type Target = TopOpeBRepDSInterference;

    fn deref(&self) -> &TopOpeBRepDSInterference {
        &self.parent
    }
}

impl TopOpeBRepDSSolidSurfaceInterference {
    /// `TopOpeBRepDS_SolidSurfaceInterference(Transition, SupportType, Support,
    /// GeometryType, Geometry)`.
    pub fn new(
        t: &TopOpeBRepDSTransition,
        st: TopOpeBRepDSKind,
        s: i32,
        gt: TopOpeBRepDSKind,
        g: i32,
    ) -> Self {
        Self {
            parent: TopOpeBRepDSInterference::new(t, st, s, gt, g),
        }
    }
}
