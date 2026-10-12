//! Real widget subclass retains argument aliases through endpoint replacement and inherited fitting.
use crate::v3d::V3dViewSource;
use super::super::*;
use super::view_cube_animation_source_fixture::*;
use crate::{prs_mgr::*, select_mgr::*, v3d::V3dView};
use rustycad_tkernel::standard::{
    Failure, Handle, HandleFieldRef, HandleReference, SharedState, StandardTransient,
};
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
pub(super) struct CubeFitReferenceProbe {
    transient: StandardTransient,
    pub(super) base: AisViewCubeInteractionBase,
    pub(super) policy: usize,
    pub(super) trace: Arc<CubeTransportTrace>,
    pub(super) original_end: Handle<Graphic3dCamera>,
    pub(super) replacement: Handle<Graphic3dCamera>,
}
rustycad_tkernel::impl_transient!(CubeFitReferenceProbe,"FitReferenceCube",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn AisViewCubeSource,dyn PrsMgrPresentableObject,dyn SelectMgrSelectableObject]);
impl CubeFitReferenceProbe {
    pub(super) fn new(
        policy: usize,
        trace: &Arc<CubeTransportTrace>,
        view: &V3dView,
    ) -> Handle<Self> {
        let base = AisViewCubeInteractionBase::new().unwrap();
        let original_end = base.fields.read().end.clone();
        let replacement = fit_reference_replacement_camera(view);
        let cube = Handle::new(Self {
            transient: StandardTransient::default(),
            base,
            policy,
            trace: trace.clone(),
            original_end,
            replacement,
        });
        prs_mgr_presentable_bind(&cube.cast());
        cube
    }
    fn capture(
        &self,
        stage: &str,
        view: &dyn HandleReference<dyn V3dViewSource>,
        camera: &dyn HandleReference<Graphic3dCamera>,
    ) {
        let channels = self.argument_channels(view, camera);
        self.trace.rows.write().push(format!(
            "fit {} {stage} {} {}",
            self.policy,
            join_transport_channels(&channels),
            join_transport_channels(&self.trace.widget_reference_counts())
        ));
    }
    fn argument_channels(
        &self,
        view: &dyn HandleReference<dyn V3dViewSource>,
        camera: &dyn HandleReference<Graphic3dCamera>,
    ) -> [i32; 9] {
        let fields = &self.base.fields;
        let own = HandleFieldRef::new(fields, fit_reference_end_slot);
        let observer = camera.observe();
        [
            i32::from(camera.reference_address() == own.reference_address()),
            camera.is_null() as i32,
            i32::from(camera.object_address() == self.original_end.object_address()),
            i32::from(camera.object_address() == self.replacement.object_address()),
            observer.ref_count(),
            fields.read().start.ref_count(),
            view.observe().ref_count(),
            self.original_end.ref_count(),
            self.replacement.ref_count(),
        ]
    }
    fn change_fit_arguments(
        &self,
        view: &dyn HandleReference<dyn V3dViewSource>,
        camera: &dyn HandleReference<Graphic3dCamera>,
    ) {
        if matches!(self.policy, 1..=3) {
            self.base.fields.write().end = self.replacement.clone();
        }
        if self.policy == 2 {
            self.base.fields.write().end = Handle::default();
            self.capture("null", view, camera);
            self.base.fields.write().end = self.replacement.clone();
        }
        if self.policy == 5 {
            self.trace.replace(1);
        }
    }
}
impl AisInteractiveObject for CubeFitReferenceProbe {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.presentation.interactive
    }
}
impl AisViewCubePresentationSource for CubeFitReferenceProbe {
    fn view_cube_presentation_base(&self) -> &AisViewCubePresentationBase {
        &self.base.presentation
    }
}
impl AisViewCubeSource for CubeFitReferenceProbe {
    fn interaction_base(&self) -> &AisViewCubeInteractionBase {
        &self.base
    }
    fn view_fit_all(
        &self,
        view: &dyn HandleReference<dyn V3dViewSource>,
        camera: &dyn HandleReference<Graphic3dCamera>,
    ) -> Result<(), Failure> {
        self.capture("before", view, camera);
        self.change_fit_arguments(view, camera);
        self.capture("changed", view, camera);
        if self.policy == 3 {
            return Err(Failure::new("fit-before failure value=3; expected success"));
        }
        ais_view_cube_view_fit_all(self, view, camera)?;
        self.capture("after", view, camera);
        if self.policy == 4 {
            return Err(Failure::new("fit-after failure value=4; expected success"));
        }
        Ok(())
    }
}
impl SelectMgrSelectableObject for CubeFitReferenceProbe {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.presentation.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_view_cube_compute_selection(self, selection, mode)
    }
}
impl PrsMgrPresentableObject for CubeFitReferenceProbe {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentation.interactive.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_view_cube_compute(self, prs, mode)
    }
}
fn fit_reference_end_slot(fields: &AisViewCubeInteractionFields) -> &Handle<Graphic3dCamera> {
    // Address identity is the actual protected slot, independently of its current target.
    &fields.end
}
pub(super) fn fit_reference_trace(policy: usize) -> Arc<CubeTransportTrace> {
    Arc::new(CubeTransportTrace {
        scenario: 14 + policy,
        cube: SharedState::new(Default::default()),
        clicked_owner: SharedState::new(Default::default()),
        probes: SharedState::new(Vec::new()),
        rows: SharedState::new(Vec::new()),
    })
}

fn fit_reference_replacement_camera(view: &V3dView) -> Handle<Graphic3dCamera> {
    let replacement = Handle::new(Graphic3dCamera::from_camera(&view.camera()));
    replacement
        .set_center(&rustycad_tkmath::gp::Point3::new(-4., 8., 12.))
        .unwrap();
    replacement.set_scale(777.);
    replacement
}
