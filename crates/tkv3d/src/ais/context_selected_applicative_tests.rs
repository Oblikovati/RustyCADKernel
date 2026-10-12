use super::*;
use crate::{select_mgr::SelectMgrEntityOwner, v3d::V3dViewer};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkservice::null_driver::NullGraphicDriver;

struct SelectedApplicationScene {
    context: Handle<AisInteractiveContext>,
    object: Handle<AisPoint>,
    first: Handle<StandardTransient>,
    second: Handle<StandardTransient>,
}

impl SelectedApplicationScene {
    fn new() -> Self {
        let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let object = application_point(0.0);
        append_application_selection(&context, &object);
        context.init_selected();
        Self {
            context,
            object,
            first: Handle::new(StandardTransient::default()),
            second: Handle::new(StandardTransient::default()),
        }
    }

    fn snapshot(&self, step: i32) -> String {
        let application = self.context.applicative().unwrap();
        let identity = if application.is_null() { 0 } else if application == self.first { 1 } else { 2 };
        format!(
            "{step} {} {identity} {} {}",
            i32::from(self.context.has_applicative().unwrap()),
            self.first.ref_count(),
            self.second.ref_count()
        )
    }

    fn replacement_trace(&self) -> Vec<String> {
        let mut trace = vec![self.snapshot(0)];
        self.object.set_owner(&self.first);
        trace.push(self.snapshot(1));
        let retained = self.context.applicative().unwrap();
        self.object.set_owner(&self.second);
        trace.push(self.snapshot(2));
        assert_eq!(retained, self.first);
        self.object.clear_owner();
        trace.push(self.snapshot(3));
        drop(retained);
        trace.push(self.snapshot(4));
        trace
    }

    fn next_object_trace(&self) -> Vec<String> {
        let other = application_point(1.0);
        other.set_owner(&self.first);
        append_application_selection(&self.context, &other);
        self.context.next_selected();
        let next = self.snapshot(5);
        self.object.set_owner(&self.second);
        vec![next, self.snapshot(6)]
    }
}

fn application_point(x: f64) -> Handle<AisPoint> {
    let component = Handle::new(GeomCartesianPoint::new(x, 0.0, 0.0));
    AisPoint::new(&component.cast()).unwrap()
}

fn append_application_selection(context: &AisInteractiveContext, object: &Handle<AisPoint>) {
    let owner = SelectMgrEntityOwner::with_selectable(&object.cast(), 0);
    context.fields.read().selection.add_select(&owner.cast()).unwrap();
}

#[test]
fn selected_application_native_owner_and_cursor_rows_replay() {
    let scene = SelectedApplicationScene::new();
    let mut trace = scene.replacement_trace();
    trace.extend(scene.next_object_trace());
    assert_eq!(trace.join("\n") + "\n", include_str!("../../tests/fixtures/selected-applicative.txt"));
}

#[test]
fn selected_application_missing_interactive_precondition_is_diagnosed() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    assert!(context.has_applicative().unwrap_err().message().contains("address=0"));
    assert!(context.applicative().unwrap_err().message().contains("expected a live AIS object"));
}
