//! `AIS_Line` at OCCT 3d097a0: retained geometry, datum identity and inherited AIS state.
use super::{AisInteractiveObject, AisInteractiveObjectBase, AisKindOfInteractive};
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::{
    standard::{Handle, SharedState, StandardTransient},
    units_api::UnitsApi,
};
use rustycad_tkg3d::geom::{GeomLine, GeomPoint};
use std::sync::Arc;
#[derive(Clone, Default)]
pub(super) struct LineGeometry {
    pub component: Handle<GeomLine>,
    pub start: Handle<dyn GeomPoint>,
    pub end: Handle<dyn GeomPoint>,
    pub segment: bool,
}
/// A finite or infinite datum with live, injected units; `AisLine::new(&geometry, units)`.
pub struct AisLine {
    transient: StandardTransient,
    pub(super) base: AisInteractiveObjectBase,
    pub(super) geometry: SharedState<LineGeometry>,
    pub(super) units: Arc<SharedState<UnitsApi>>,
}
rustycad_tkernel::impl_transient!(AisLine,"AIS_Line",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisLine {
    /// Construct an infinite datum; `AisLine::new(&geometry, units)`.
    pub fn new(line: &Handle<GeomLine>, units: Arc<SharedState<UnitsApi>>) -> Handle<Self> {
        let object = Self::with_geometry(
            LineGeometry {
                component: line.clone(),
                ..Default::default()
            },
            units,
        );
        object.set_infinite_state(true);
        object
    }
    /// Retain two live points as a finite segment; `AisLine::from_points(&start, &end, units)`.
    pub fn from_points(
        start: &Handle<dyn GeomPoint>,
        end: &Handle<dyn GeomPoint>,
        units: Arc<SharedState<UnitsApi>>,
    ) -> Handle<Self> {
        Self::with_geometry(
            LineGeometry {
                start: start.clone(),
                end: end.clone(),
                segment: true,
                ..Default::default()
            },
            units,
        )
    }
    fn with_geometry(geometry: LineGeometry, units: Arc<SharedState<UnitsApi>>) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            geometry: SharedState::new(geometry),
            units,
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
    /// Retain the component, built lazily for segments; `object.line()`.
    pub fn line(&self) -> Handle<GeomLine> {
        self.geometry.read().component.clone()
    }
    /// Retain the first endpoint; `object.start_point()`.
    pub fn start_point(&self) -> Handle<dyn GeomPoint> {
        self.geometry.read().start.clone()
    }
    /// Retain the last endpoint; `object.end_point()`.
    pub fn end_point(&self) -> Handle<dyn GeomPoint> {
        self.geometry.read().end.clone()
    }
    /// Native deprecated Points pair; `let (start, end) = object.points()`.
    pub fn points(&self) -> (Handle<dyn GeomPoint>, Handle<dyn GeomPoint>) {
        (self.start_point(), self.end_point())
    }
    /// Replace the component without changing infinite-state or scheduling updates; `object.set_line(&line)`.
    pub fn set_line(&self, line: &Handle<GeomLine>) {
        let mut geometry = self.geometry.write();
        geometry.component = line.clone();
        geometry.segment = false;
    }
    /// Replace endpoints without discarding the cached component; `object.set_points(&start, &end)`.
    pub fn set_points(&self, start: &Handle<dyn GeomPoint>, end: &Handle<dyn GeomPoint>) {
        let mut geometry = self.geometry.write();
        geometry.start = start.clone();
        geometry.end = end.clone();
        geometry.segment = true;
    }
}
impl AisInteractiveObject for AisLine {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::Datum
    }
    fn signature(&self) -> i32 {
        5
    }
}
