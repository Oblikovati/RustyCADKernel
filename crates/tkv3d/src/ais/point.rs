//! `AIS_Point` at OCCT 3d097a0: live geometry, datum identity and marker overrides.
use super::{AisInteractiveObject, AisInteractiveObjectBase, AisKindOfInteractive};
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::geom::GeomPoint;
use rustycad_tkservice::aspect::AspectTypeOfMarker;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeVertex;
pub(super) struct PointMarker {
    pub owned: bool,
    pub kind: AspectTypeOfMarker,
}
/// A geometric point datum with native marker and selection behavior; `AisPoint::new(&point)?`.
pub struct AisPoint {
    transient: StandardTransient,
    pub(super) base: AisInteractiveObjectBase,
    component: SharedState<Handle<dyn GeomPoint>>,
    pub(super) marker: SharedState<PointMarker>,
}
rustycad_tkernel::impl_transient!(AisPoint,"AIS_Point",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisPoint {
    /// Retain the component and initialize the two highlight drawers; `AisPoint::new(&point)?`.
    pub fn new(component: &Handle<dyn GeomPoint>) -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            component: SharedState::new(component.clone()),
            marker: SharedState::new(PointMarker {
                owned: false,
                kind: AspectTypeOfMarker::Plus,
            }),
        });
        object.initialize_point_highlights()?;
        prs_mgr_presentable_bind(&object.cast());
        Ok(object)
    }
    /// Retain the live geometric point; `point.component()`.
    pub fn component(&self) -> Handle<dyn GeomPoint> {
        self.component.read().clone()
    }
    /// Replace geometry without scheduling redisplay; `point.set_component(&geometry)`.
    pub fn set_component(&self, component: &Handle<dyn GeomPoint>) {
        *self.component.write() = component.clone();
    }
    /// Override marker type and refresh live aspects; `point.set_marker(AspectTypeOfMarker::Star)?`.
    pub fn set_marker(&self, kind: AspectTypeOfMarker) -> Result<(), Failure> {
        *self.marker.write() = PointMarker { owned: true, kind };
        self.update_point_values()
    }
    /// Restore inherited marker type while retaining other overrides; `point.unset_marker()?`.
    pub fn unset_marker(&self) -> Result<(), Failure> {
        self.marker.write().owned = false;
        self.update_point_values()
    }
    /// Whether the marker type is explicitly owned; `point.has_marker()`.
    pub fn has_marker(&self) -> bool {
        self.marker.read().owned
    }
    /// Build a fresh topological vertex from live geometry; `point.vertex()?`.
    pub fn vertex(&self) -> Result<Vertex, Failure> {
        BRepBuilderAPIMakeVertex::new(self.component().pnt())?.vertex()
    }
}
impl AisInteractiveObject for AisPoint {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::Datum
    }
    fn signature(&self) -> i32 {
        1
    }
}
