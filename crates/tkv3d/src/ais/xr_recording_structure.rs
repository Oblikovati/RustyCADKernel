//! Named CPU driver structure injects checked callback faults into the actual XR presentation path.
use super::xr_recording_group::{XrRecordingEvents, XrRecordingGroup};
use rustycad_tkernel::standard::Failure as XrProbeFailure;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use std::sync::Arc;
pub(super) fn xr_recording_presentation(
    events: &Arc<XrRecordingEvents>,
) -> Handle<dyn Graphic3dStructureSource> {
    let manager =
        Handle::new(Graphic3dStructureManager::new(&NullGraphicDriver::new_handle()).unwrap());
    let prs = Graphic3dStructure::new(&manager, &Handle::default())
        .unwrap()
        .cast::<dyn Graphic3dStructureSource>();
    let wrapper = Handle::new(XrRecordingStructure {
        transient: StandardTransient::default(),
        inner: prs.cstructure(),
        events: events.clone(),
    });
    prs.structure_base().fields.write().cstructure = wrapper.cast();
    prs
}
struct XrRecordingStructure {
    transient: StandardTransient,
    inner: Handle<dyn Graphic3dCStructure>,
    events: Arc<XrRecordingEvents>,
}
rustycad_tkernel::impl_transient!(XrRecordingStructure,"Martensite_XrRecordingStructure",StandardTransient,transient,interfaces:[dyn Graphic3dCStructure]);
impl Graphic3dCStructure for XrRecordingStructure {
    fn cstructure_base(&self) -> &Graphic3dCStructureBase {
        self.inner.cstructure_base()
    }
    fn on_visibility_changed(&self) {
        self.inner.on_visibility_changed();
    }
    fn clear(&self) {
        self.inner.clear();
    }
    fn connect(&self, other: &dyn Graphic3dCStructure) {
        self.inner.connect(other);
    }
    fn disconnect(&self, other: &dyn Graphic3dCStructure) {
        self.inner.disconnect(other);
    }
    fn graphic_highlight(&self, style: &Handle<dyn Graphic3dPresentationAttributesSource>) {
        self.inner.graphic_highlight(style);
    }
    fn graphic_unhighlight(&self) {
        self.inner.graphic_unhighlight();
    }
    fn shadow_link(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        self.inner.shadow_link(manager)
    }
    fn remove_group(&self, group: &Handle<dyn Graphic3dGroup>) {
        self.inner.remove_group(group);
    }

    fn new_group(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        let index = self.cstructure_base().fields.read().groups.len() as i32;
        self.events.callbacks.write().push(400 + index);
        if *self.events.fail_new_group.read() == index {
            return Err(XrProbeFailure::new(&format!(
                "XR new-group index={index}; expected live group"
            )));
        }
        let group = Handle::new(XrRecordingGroup::new(structure, self.events.clone(), index))
            .cast::<dyn Graphic3dGroup>();
        self.cstructure_base()
            .fields
            .write()
            .groups
            .push(group.clone());
        Ok(group)
    }
}
