//! Real viewer and recorded selectable objects for both DisplaySensitive overloads.
use super::{
    boxed_selectable::BoxedSelectable,
    display_driver::{DisplayTrace, SensitiveDisplayDriver},
    display_scene::*,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{aspect::AspectNeutralWindow, graphic3d::*};
use rustycad_tkv3d::{
    prs_mgr::*,
    select_mgr::*,
    v3d::{V3dView, V3dViewer},
};
pub struct SensitiveDisplayView {
    pub _viewer: Handle<V3dViewer>,
    pub view: Handle<V3dView>,
    pub selector: Handle<SelectMgrViewerSelector>,
    pub selection: Handle<SelectMgrSelection>,
    pub trace: DisplayTrace,
}
impl SensitiveDisplayView {
    pub fn new(kind: i32, transform: i32, flags: i32, fail: i32) -> Result<Self, Failure> {
        let (driver, trace) = SensitiveDisplayDriver::new_handle(fail);
        let viewer = V3dViewer::new(&driver)?;
        let view = display_test_view(&viewer)?;
        let selection = display_selection(kind, flags)?;
        selection.set_selection_state(if flags & 4 != 0 {
            SelectMgrStateOfSelection::Deactivated
        } else {
            SelectMgrStateOfSelection::Activated
        });
        let selector = Handle::new(SelectMgrViewerSelector::new());
        populate_display_objects(&selector, &selection, transform, flags, &trace)?;
        if flags & 8 != 0 {
            trace.write().append_selection = selection.clone();
        }
        Ok(Self {
            _viewer: viewer,
            view,
            selector,
            selection,
            trace,
        })
    }
    pub fn run(&self, mode: i32, transform: i32) -> Result<(), Failure> {
        if mode == 0 {
            return self.selector.display_sensitive(&self.view);
        }
        self.selector.display_sensitive_selection(
            &self.selection,
            &display_location(transform),
            &self.view,
            mode == 2,
        )
    }
}
fn populate_display_objects(
    selector: &Handle<SelectMgrViewerSelector>,
    selection: &Handle<SelectMgrSelection>,
    transform: i32,
    flags: i32,
    trace: &DisplayTrace,
) -> Result<(), Failure> {
    let object = BoxedSelectable::new(0., false);
    object.set_local_transformation(&display_location(transform))?;
    if flags & 2 != 0 {
        object.set_transform_persistence(
            &Handle::new(Graphic3dTransformPers::with_point(
                Graphic3dTransModeFlags::ZOOM_PERS,
                &Point3::new(1., 2., 3.),
            )?)
            .cast(),
        );
    }
    object.add_selection(selection, 0)?;
    if flags & 16 != 0 {
        trace.write().append_object = object.cast();
    }
    selector.add_selectable_object(&object.cast())?;
    selector.add_selectable_object(&BoxedSelectable::new(0., false).cast())?;
    Ok(())
}

fn display_test_view(viewer: &Handle<V3dViewer>) -> Result<Handle<V3dView>, Failure> {
    let view = viewer.create_view()?;
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(80, 60);
    view.set_window(&window.cast(), 0)?;
    Ok(view)
}
