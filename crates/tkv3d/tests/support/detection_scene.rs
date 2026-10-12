//! A real context/selector/view used by Rust tests and the native comparison twin.
use rustycad_tkv3d::v3d::V3dViewSource;
#[path = "detection_object.rs"]
pub mod object;
use object::*;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkservice::{
    aspect::AspectNeutralWindow, graphic3d::*, null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, select3d::Select3DSensitivePoint, v3d::*,
};
use std::sync::Arc;
pub struct DetectionScene {
    pub context: Handle<AisInteractiveContext>,
    pub view: Handle<dyn V3dViewSource>,
    pub object: Handle<DetectionObject>,
    pub owners: Vec<Handle<dyn SelectMgrEntityOwnerSource>>,
    pub trace: DetectionEvents,
}
impl DetectionScene {
    pub fn new(flags: i32, mask: i32, style: i32, mutation: i32, failing: i32) -> Self {
        let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let view = detection_view(&viewer);
        let trace = Arc::new(SharedState::new(DetectionTrace {
            mask,
            mutation,
            failing,
            accept: flags & 32 == 0,
            forced: flags & 8 != 0,
            ..Default::default()
        }));
        let object = DetectionObject::new(trace.clone());
        object.set_context(&context);
        let owners = (0..3)
            .map(|id| DetectionOwner::new_handle(&object.cast(), id, trace.clone()))
            .collect();
        let scene = Self {
            context,
            view,
            object,
            owners,
            trace,
        };
        scene.configure(flags, style);
        if flags & 256 != 0 {
            scene.view.viewer().set_computed_mode(true);
            scene.view.set_computed_mode(true);
        }
        if flags & 512 != 0 {
            scene
                .view
                .camera()
                .set_projection_type(Graphic3dCameraProjection::Perspective);
        }
        scene
    }
    fn configure(&self, flags: i32, style: i32) {
        let mut fields = self.context.fields.write();
        fields.automatic_hilight = flags & 2 == 0;
        fields.to_hilight_selected = flags & 4 == 0;
        fields.picking_strategy = if flags & 16 != 0 {
            SelectMgrPickingStrategy::OnlyTopmost
        } else {
            SelectMgrPickingStrategy::FirstAcceptable
        };
        fields
            .filters
            .add(&DetectionFilter::new_handle(self.trace.clone()));
        drop(fields);
        self.owners[0].set_selected(flags & 1 != 0);
        self.owners[1]
            .owner_base()
            .fields
            .write()
            .from_decomposition = true;
        if flags & 64 != 0 {
            self.owners[2].set_selectable(&Handle::default());
        }
        let drawer = Handle::new(Prs3dDrawer::new());
        drawer.set_display_mode(style);
        drawer.set_z_layer(if flags & 128 != 0 {
            GRAPHIC3D_ZLAYER_ID_UNKNOWN
        } else {
            GRAPHIC3D_ZLAYER_ID_TOP
        });
        if self.trace.read().mutation == 1 {
            self.object.set_dynamic_hilight_attributes(&drawer);
        } else {
            self.context.set_dynamic_highlight_style(&drawer);
        }
        self.context.default_drawer().set_display_mode(3);
    }
    pub fn selector(&self) -> Handle<dyn SelectMgrViewerSelectorSource> {
        self.context.main_selector().clone()
    }
    pub fn seed(&self, mask: i32) {
        let selector = self.selector();
        selector.clear_picked();
        let mut fields = selector.viewer_selector_base().fields.write();
        for (id, owner) in self.owners.iter().enumerate() {
            if mask & (1 << id) == 0 {
                continue;
            }
            let criterion = SelectMgrSortCriterion {
                depth: id as f64,
                ..Default::default()
            };
            fields.results.stored.add(owner.clone(), criterion);
        }
        fields.results.is_sorted = false;
    }
    pub fn operation(&self, op: i32, value: i32) -> Result<Option<i32>, Failure> {
        match op {
            0 => self.seed(value),
            1 => {
                return self
                    .context
                    .move_to_detected(&self.view, value != 0)
                    .map(|status| Some(status as i32));
            }
            2 => return Ok(Some(i32::from(self.context.clear_detected(value != 0)))),
            3 => self.context.init_detected(),
            4 => self.context.next_detected(),
            5 => {
                return self
                    .context
                    .hilight_next_detected(&self.view, value != 0)
                    .map(Some);
            }
            6 => {
                return self
                    .context
                    .hilight_previous_detected(&self.view, value != 0)
                    .map(Some);
            }
            7 => self.trace.write().mask = value,
            8 => self.trace.write().failing = value,
            9 => {
                return self
                    .context
                    .move_to(40 + value, 30, &self.view, true)
                    .map(|s| Some(s as i32));
            }
            10 => {
                return self
                    .context
                    .move_to_axis(
                        Axis1::new(
                            Point3::new(value as f64, 0., 20.),
                            Direction3::new(0., 0., -1.)?,
                        ),
                        &self.view,
                        true,
                    )
                    .map(|s| Some(s as i32));
            }
            11 => self.install_geometry()?,
            12 => {
                let viewer = V3dViewer::new(&NullGraphicDriver::new_handle())?;
                let view = detection_view(&viewer);
                return self
                    .context
                    .move_to(40, 30, &view, true)
                    .map(|s| Some(s as i32));
            }
            13 => {
                let style = if value < -1 {
                    Handle::default()
                } else {
                    self.context.dynamic_highlight_style().clone()
                };
                return Ok(Some(self.context.get_hilight_mode(
                    &self.object.cast(),
                    &style,
                    if value < -1 { -1 } else { value },
                )));
            }
            14 => self.object.attributes().set_display_mode(value),
            15 => {
                return Ok(Some(i32::from(self.context.is_slow_hi_style(
                    &self.owners[value as usize],
                    &self.view.viewer(),
                )?)));
            }
            _ => unreachable!(),
        }
        Ok(None)
    }
    pub fn install_geometry(&self) -> Result<(), Failure> {
        let selection = Handle::new(SelectMgrSelection::new(0));
        for (id, owner) in self.owners.iter().enumerate() {
            selection.add(
                &Handle::new(Select3DSensitivePoint::new(
                    owner,
                    Point3::new(0., 0., -(id as f64)),
                ))
                .cast(),
            )?;
        }
        selection.set_update_status(SelectMgrTypeOfUpdate::Partial);
        selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::Add);
        self.object.add_selection(&selection, 0)?;
        self.context
            .selection_manager()
            .clone()
            .load(&self.object.cast(), 0)?;
        self.context
            .selection_manager()
            .clone()
            .activate(&self.object.cast(), 0)?;
        Ok(())
    }
    pub fn owner_id(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> i32 {
        self.owners
            .iter()
            .position(|known| known == owner)
            .map_or(-1, |index| index as i32)
    }
}
fn detection_view(viewer: &Handle<V3dViewer>) -> Handle<dyn V3dViewSource> {
    let view = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(80, 60);
    view.set_window(&window.cast(), 0).unwrap();
    let camera = view.camera();
    camera.set_eye(&Point3::new(0., 0., 20.)).unwrap();
    camera.set_center(&Point3::default()).unwrap();
    camera.set_up(&Direction3::new(0., 1., 0.).unwrap());
    camera.set_scale(20.);
    camera.set_aspect(80. / 60.);
    view
}
