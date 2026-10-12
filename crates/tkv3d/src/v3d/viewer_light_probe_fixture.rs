//! Native-style callback subjects embed real view storage; controls contain nonowning native targets.
use super::*;
use rustycad_tkernel::standard::{Handle, HandleReference, SharedState, WeakHandle};
use rustycad_tkservice::{
    aspect::AspectNeutralWindow,
    graphic3d::{Graphic3dCLight, Graphic3dLightSet},
};
use std::sync::Arc;
pub(super) struct ViewerLightProbeControl {
    pub viewer: WeakHandle<V3dViewer>,
    pub lights: SharedState<[WeakHandle<Graphic3dCLight>; 3]>,
    pub views: SharedState<[WeakHandle<dyn V3dViewSource>; 3]>,
    pub caller: SharedState<Handle<dyn V3dViewSource>>,
    pub caller_set: SharedState<Handle<Graphic3dLightSet>>,
    pub replacement_set: SharedState<WeakHandle<Graphic3dLightSet>>,
    pub policy: usize,
    pub ordinal: SharedState<usize>,
    pub armed: SharedState<bool>,
    pub rows: SharedState<Vec<String>>,
}
pub(super) struct LiveLightViewSource {
    pub base: V3dView,
    pub control: Arc<ViewerLightProbeControl>,
    pub identity: usize,
}
rustycad_tkernel::impl_transient!(LiveLightViewSource,"Martensite_LiveLightViewSource",V3dView,base,
    interfaces:[dyn V3dViewSource]);
impl V3dViewSource for LiveLightViewSource {
    fn view_base(&self) -> &V3dView {
        &self.base
    }
    fn redraw(&self) {
        if *self.control.armed.read() {
            let owners = self.base.this.read().observe().ref_count();
            let implementation = self.base.view_ref().observe().ref_count();
            self.control.rows.write().push(format!(
                "redraw {} {owners} {implementation}{}",
                self.identity,
                probe_light_counts(&self.control)
            ));
        }
        reject_light_probe_virtual(&self.control, 14, "Redraw");
        v3d_view_redraw(&self.base);
    }
}
pub(super) fn fresh_light_probe_control(
    viewer: &Handle<V3dViewer>,
    policy: usize,
) -> Arc<ViewerLightProbeControl> {
    Arc::new(ViewerLightProbeControl {
        viewer: viewer.downgrade(),
        lights: Default::default(),
        views: Default::default(),
        caller: Default::default(),
        caller_set: Default::default(),
        replacement_set: Default::default(),
        policy,
        ordinal: SharedState::new(0),
        armed: SharedState::new(false),
        rows: Default::default(),
    })
}
pub(super) fn mapped_light_probe_view(
    viewer: &Handle<V3dViewer>,
    control: &Arc<ViewerLightProbeControl>,
    identity: usize,
) -> Handle<dyn V3dViewSource> {
    let source: Handle<dyn V3dViewSource> = Handle::new(LiveLightViewSource {
        base: V3dView::new_embedded(viewer, V3dTypeOfView::Orthographic).unwrap(),
        control: control.clone(),
        identity,
    })
    .cast();
    V3dView::bind_source(&source).unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(640, 480);
    source.set_window(&window.cast(), 0).unwrap();
    source.set_immediate_update(false);
    source.set_auto_z_fit_mode(false, 1.).unwrap();
    viewer.set_view_off_one(&source);
    source
}
pub(super) fn probe_light_counts(control: &ViewerLightProbeControl) -> String {
    control
        .lights
        .read()
        .iter()
        .map(|light| format!(" {}", light.observe().ref_count()))
        .collect()
}
pub(super) fn probe_list_light_keys(chain: V3dLightListRef<'_>) -> String {
    let mut cursor = chain.cursor();
    let mut keys = String::new();
    while cursor.more() {
        let light = cursor.value_ref().observe();
        keys.push_str(&light.get().map_or("0".into(), |light| {
            String::from_utf8_lossy(light.name().as_bytes()).into_owned()
        }));
        keys.push(',');
        cursor.next();
    }
    if keys.is_empty() {
        return "-".into();
    }
    keys
}

pub(super) fn reject_light_probe_virtual(
    control: &ViewerLightProbeControl,
    policy: usize,
    operation: &str,
) {
    if !*control.armed.read() || control.policy != policy {
        return;
    }
    rustycad_tkernel::standard::raise_native_failure::<()>(Err(
        rustycad_tkernel::standard::Failure::new(&format!(
            "{operation} callback=failed; expected success"
        )),
    ));
}
