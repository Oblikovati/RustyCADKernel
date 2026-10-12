//! Named real camera subclass records dispatch and native intrusive counts without owning aliases.
use super::super::{view_cube_interaction_fixture::WidgetScene, *};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use std::sync::{Arc, atomic::AtomicU64};
pub(super) struct CubeTransportTrace {
    pub(super) scenario: usize,
    pub(super) cube: WeakHandle<AisViewCube>,
    pub(super) probes: SharedState<Vec<WeakHandle<CubeTransportAnimation>>>,
    pub(super) rows: SharedState<Vec<String>>,
}
impl CubeTransportTrace {
    pub(super) fn slot(&self) -> i32 {
        let address = self.cube.upgrade().view_animation_ref().object_address();
        if address.is_null() {
            return 0;
        }
        self.probes
            .read()
            .iter()
            .position(|probe| probe.object_address() == address)
            .map_or(-1, |id| id as i32 + 1)
    }
    pub(super) fn record(&self, event: &str, actor: Option<&CubeTransportAnimation>) {
        let channels = actor.map_or([0; 7], CubeTransportAnimation::channels);
        self.rows.write().push(format!(
            "event {} {event} {} {} {}",
            self.scenario,
            channels[0],
            self.slot(),
            join_transport_channels(&channels[1..])
        ));
    }
    pub(super) fn replace(&self, id: usize) {
        let probe = self.probes.read()[id].upgrade();
        self.cube.upgrade().set_view_animation(&probe.cast());
    }
    pub(super) fn reject(&self, stage: &str) -> Failure {
        Failure::new(&format!(
            "{stage} failure value={}; expected success",
            self.scenario
        ))
    }
}
pub(super) struct CubeTransportAnimation {
    transient: StandardTransient,
    pub(super) base: AisAnimationCameraBase,
    pub(super) trace: Arc<CubeTransportTrace>,
    pub(super) identity: i32,
    pub(super) next: AtomicU64,
}
rustycad_tkernel::impl_transient!(CubeTransportAnimation,"TransportAnimation",dyn AisAnimationCameraSource,transient,
    interfaces:[dyn AisAnimationCameraSource,dyn AisAnimationSource]);
impl CubeTransportAnimation {
    fn new(trace: &Arc<CubeTransportTrace>, identity: i32) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisAnimationCameraBase::new("transport", &Handle::default()),
            trace: trace.clone(),
            identity,
            next: AtomicU64::new(0.125_f64.to_bits()),
        })
    }
    pub(super) fn channels(&self) -> [i32; 7] {
        let fields = self.base.fields.read();
        [
            self.identity,
            self.transient.ref_count(),
            fields.view.is_null() as i32,
            fields.camera_start.ref_count(),
            fields.camera_end.ref_count(),
            fields.view.ref_count(),
            self.is_stopped() as i32,
        ]
    }
}
impl AisAnimationCameraSource for CubeTransportAnimation {
    fn camera_animation_base(&self) -> &AisAnimationCameraBase {
        &self.base
    }
}
pub(super) struct CubeTransportScene {
    pub(super) scene: WidgetScene,
    pub(super) trace: Arc<CubeTransportTrace>,
    pub(super) owners: [Handle<CubeTransportAnimation>; 3],
}
impl CubeTransportScene {
    pub(super) fn new(scenario: usize) -> Self {
        let scene = WidgetScene::new(false);
        let trace = cube_transport_trace(&scene, scenario);
        let owners = std::array::from_fn(|id| CubeTransportAnimation::new(&trace, id as i32 + 1));
        *trace.probes.write() = owners.iter().map(Handle::downgrade).collect();
        for owner in &owners {
            owner.set_own_duration(0.5);
        }
        owners[1].set_own_duration(if scenario == 1 { 1.0 } else { 0.2 });
        scene.cube.set_view_animation(&owners[0].cast());
        scene.cube.set_animation_events(Some(trace.clone()));
        Self {
            scene,
            trace,
            owners,
        }
    }
    pub(super) fn snapshot(&self, stage: &str) {
        let channels = self
            .owners
            .iter()
            .flat_map(|probe| {
                let c = probe.channels();
                [c[1], c[6], c[2], c[3], c[4]]
            })
            .collect::<Vec<_>>();
        self.trace.rows.write().push(format!(
            "snapshot {} {stage} {} {}",
            self.trace.scenario,
            self.trace.slot(),
            join_transport_channels(&channels)
        ));
    }
}
pub(super) fn join_transport_channels(channels: &[i32]) -> String {
    channels
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

fn cube_transport_trace(scene: &WidgetScene, scenario: usize) -> Arc<CubeTransportTrace> {
    Arc::new(CubeTransportTrace {
        scenario,
        cube: scene.cube.downgrade(),
        probes: SharedState::new(Vec::new()),
        rows: SharedState::new(Vec::new()),
    })
}
