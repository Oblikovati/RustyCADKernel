//! `AIS_Circle` at OCCT 3d097a0: live geometry, full-circle/arc state and datum identity.
use super::{AisInteractiveObject, AisInteractiveObjectBase, AisKindOfInteractive};
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkg3d::geom::GeomCircle;
use rustycad_tkmath::gp::resolution;
use std::f64::consts::TAU;
#[derive(Clone)]
pub(super) struct CircleGeometry {
    pub component: Handle<GeomCircle>,
    pub bounds: [f64; 2],
    pub arc: bool,
    pub filled: bool,
}
/// A circular or arc datum with live appearance; `AisCircle::new(&circle)`.
pub struct AisCircle {
    transient: StandardTransient,
    pub(super) base: AisInteractiveObjectBase,
    pub(super) geometry: SharedState<CircleGeometry>,
}
rustycad_tkernel::impl_transient!(AisCircle,"AIS_Circle",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisCircle {
    /// Retain a full circle with boundary sensitivity; `AisCircle::new(&circle)`.
    pub fn new(circle: &Handle<GeomCircle>) -> Handle<Self> {
        Self::with_parameters(circle, 0., TAU, false)
    }
    /// Retain parameters and filled sensitivity; a span of ±2π is a full circle. `AisCircle::with_parameters(&circle,0.,1.,true)`.
    pub fn with_parameters(
        circle: &Handle<GeomCircle>,
        first: f64,
        last: f64,
        filled: bool,
    ) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            geometry: SharedState::new(CircleGeometry {
                component: circle.clone(),
                bounds: [first, last],
                arc: ((last - first).abs() - TAU).abs() > resolution(),
                filled,
            }),
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
    /// Return the retained circle; `object.circle()`.
    pub fn circle(&self) -> Handle<GeomCircle> {
        self.geometry.read().component.clone()
    }
    /// Return the stored arc parameters, including full-circle constructor values; `object.parameters()`.
    pub fn parameters(&self) -> (f64, f64) {
        let [first, last] = self.geometry.read().bounds;
        (first, last)
    }
    /// Replace geometry without scheduling an update or changing arc state; `object.set_circle(&circle)`.
    pub fn set_circle(&self, circle: &Handle<GeomCircle>) {
        self.geometry.write().component = circle.clone();
    }
    /// Set the start and force arc behavior, without scheduling updates; `object.set_first_param(0.5)`.
    pub fn set_first_param(&self, first: f64) {
        let mut geometry = self.geometry.write();
        geometry.bounds[0] = first;
        geometry.arc = true;
    }
    /// Set the end and force arc behavior, without scheduling updates; `object.set_last_param(2.)`.
    pub fn set_last_param(&self, last: f64) {
        let mut geometry = self.geometry.write();
        geometry.bounds[1] = last;
        geometry.arc = true;
    }
    /// Inspect filled selection sensitivity; `object.is_filled_circle_sens()`.
    pub fn is_filled_circle_sens(&self) -> bool {
        self.geometry.read().filled
    }
    /// Change selection sensitivity without rebuilding selection; `object.set_filled_circle_sens(true)`.
    pub fn set_filled_circle_sens(&self, filled: bool) {
        self.geometry.write().filled = filled;
    }
}
impl AisInteractiveObject for AisCircle {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::Datum
    }
    fn signature(&self) -> i32 {
        6
    }
}
