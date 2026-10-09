// Rust translation of
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_SurfaceCurveInterference.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_SurfaceCurveInterference`: an interference of a curve on a surface
//! (or face), with the p-curve of the curve on it.
use super::TopOpeBRepDSInterference;
use crate::top_ope_brep_ds::{TopOpeBRepDSKind, TopOpeBRepDSTransition};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use std::ops::Deref;

/// A surface/curve interference (`TopOpeBRepDS_SurfaceCurveInterference`); the
/// interference members are reached through `Deref`;
/// `TopOpeBRepDSSurfaceCurveInterference::new(&t, TopOpeBRepDSKind::Face, 1, TopOpeBRepDSKind::Curve, 1, &pc)`.
pub struct TopOpeBRepDSSurfaceCurveInterference {
    parent: TopOpeBRepDSInterference,
    my_pcurve: SharedState<Handle<dyn Geom2dCurve>>,
}
impl_transient!(
    TopOpeBRepDSSurfaceCurveInterference,
    "TopOpeBRepDS_SurfaceCurveInterference",
    TopOpeBRepDSInterference,
    parent
);

impl Deref for TopOpeBRepDSSurfaceCurveInterference {
    type Target = TopOpeBRepDSInterference;

    fn deref(&self) -> &TopOpeBRepDSInterference {
        &self.parent
    }
}

impl Default for TopOpeBRepDSSurfaceCurveInterference {
    /// `TopOpeBRepDS_SurfaceCurveInterference()`: a default interference, no p-curve.
    fn default() -> Self {
        Self {
            parent: TopOpeBRepDSInterference::default(),
            my_pcurve: SharedState::new(Handle::default()),
        }
    }
}

impl TopOpeBRepDSSurfaceCurveInterference {
    /// `TopOpeBRepDS_SurfaceCurveInterference(Transition, SupportType, Support,
    /// GeometryType, Geometry, PC)`.
    pub fn new(
        t: &TopOpeBRepDSTransition,
        st: TopOpeBRepDSKind,
        s: i32,
        gt: TopOpeBRepDSKind,
        g: i32,
        pc: &Handle<dyn Geom2dCurve>,
    ) -> Self {
        Self {
            parent: TopOpeBRepDSInterference::new(t, st, s, gt, g),
            my_pcurve: SharedState::new(pc.clone()),
        }
    }

    /// `TopOpeBRepDS_SurfaceCurveInterference(I)`: the transition, support and geometry
    /// of `i`, no p-curve; Standard_NullObject for a null handle;
    /// `TopOpeBRepDSSurfaceCurveInterference::from_interference(&i)?`.
    pub fn from_interference(i: &Handle<TopOpeBRepDSInterference>) -> Result<Self, Failure> {
        Ok(Self {
            parent: TopOpeBRepDSInterference::from_interference(i)?,
            my_pcurve: SharedState::new(Handle::default()),
        })
    }

    /// `PCurve()`; `i.pcurve()`.
    pub fn pcurve(&self) -> Handle<dyn Geom2dCurve> {
        self.my_pcurve.read().clone()
    }

    /// `PCurve(PC)`; `i.set_pcurve(&pc)`.
    pub fn set_pcurve(&self, pc: &Handle<dyn Geom2dCurve>) {
        *self.my_pcurve.write() = pc.clone();
    }
}
