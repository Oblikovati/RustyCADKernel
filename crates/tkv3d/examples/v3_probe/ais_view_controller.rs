//! AIS_ViewController event dispatch over three displayed points, the twin of
//! tools/oracle/tkv3d_probe_ais_view_controller.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkservice::aspect::{AspectNeutralWindow, AspectVKey};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::v3d::{V3dTypeOfOrientation, V3dViewSource, V3dViewer};
use std::sync::{Arc, Mutex};

/// The recorded virtual callbacks (1 selection, 10 + action dragging, 3 subview).
struct Recorder(Arc<Mutex<Vec<i32>>>);
impl AisViewControllerHooks for Recorder {
    fn on_selection_changed(
        &mut self,
        _: &mut AisViewController,
        _: &Handle<AisInteractiveContext>,
        _: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        self.0.lock().unwrap().push(1);
        Ok(())
    }
    fn on_object_dragged(
        &mut self,
        controller: &mut AisViewController,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
        action: AisDragAction,
    ) -> Result<(), Failure> {
        self.0.lock().unwrap().push(10 + action as i32);
        controller.object_dragged_base(context, view, action)
    }
    fn on_subview_changed(
        &mut self,
        _: &mut AisViewController,
        _: &Handle<AisInteractiveContext>,
        _: &Handle<dyn V3dViewSource>,
        _: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        self.0.lock().unwrap().push(3);
        Ok(())
    }
}

fn camera_rows(camera: &Graphic3dCamera) -> [f64; 6] {
    let (eye, up) = (camera.eye(), camera.up());
    [eye.x(), eye.y(), eye.z(), up.x(), up.y(), up.z()]
}

fn state(
    c: &AisViewController,
    ctx: &Handle<AisInteractiveContext>,
    view: &Handle<dyn V3dViewSource>,
    points: &[Handle<dyn AisInteractiveObject>],
    events: &Arc<Mutex<Vec<i32>>>,
) {
    int(ctx.nb_selected());
    for p in points {
        flag(ctx.is_selected_object(p));
    }
    flag(ctx.has_detected());
    let detected = if ctx.has_detected() {
        let object = ctx.detected_interactive();
        points
            .iter()
            .position(|p| *p == object)
            .map_or(-1, |i| i as i32)
    } else {
        -1
    };
    int(detected);
    let band: Handle<dyn AisInteractiveObject> = c.rubber_band().clone().into_cast();
    flag(ctx.is_displayed(&band));
    let band_points: Vec<Vec2<i32>> = c.rubber_band().points().read().iter().copied().collect();
    int(band_points.len() as i32);
    for p in band_points {
        int(p.x);
        int(p.y);
    }
    flag(c.rubber_band().is_polygon_closed());
    flag(!c.drag_object().is_null());
    flag(c.to_ask_next_frame());
    flag(c.has_previous_move_to());
    int(c.previous_move_to().x);
    int(c.previous_move_to().y);
    let animation = c.view_animation();
    flag(animation.is_stopped());
    flag(animation.target_view() == *view);
    let camera = view.camera();
    let mut values = camera_rows(&camera).to_vec();
    values.push(camera.scale());
    for v in values {
        real((v * 100.0).round() / 100.0 + 0.0);
    }
    for end_camera in [animation.camera_start(), animation.camera_end()] {
        flag(end_camera.is_null());
        if let Some(end_camera) = end_camera.get() {
            for v in camera_rows(end_camera) {
                real(v);
            }
        }
    }
    flag(view.is_invalidated_immediate());
    let mut events = events.lock().unwrap();
    int(events.len() as i32);
    for e in events.iter() {
        int(*e);
    }
    events.clear();
}

fn scheme(value: i32) -> AisSelectionScheme {
    match value {
        0 => AisSelectionScheme::Replace,
        1 => AisSelectionScheme::Add,
        2 => AisSelectionScheme::Remove,
        3 => AisSelectionScheme::Xor,
        4 => AisSelectionScheme::Clear,
        5 => AisSelectionScheme::ReplaceExtra,
        _ => AisSelectionScheme::Unknown,
    }
}

fn tool(value: i32) -> AisViewSelectionTool {
    match value {
        1 => AisViewSelectionTool::RubberBand,
        2 => AisViewSelectionTool::Polygon,
        3 => AisViewSelectionTool::ZoomWindow,
        _ => AisViewSelectionTool::Picking,
    }
}

/// `v3_ais_view_controller Count (Op A B C D)...`.
pub fn view_controller_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(200, 100);
    view.set_window(&window.cast(), 0).unwrap();
    let ctx = AisInteractiveContext::new(&viewer).unwrap();
    let mut points: Vec<Handle<dyn AisInteractiveObject>> = Vec::new();
    for x in [-10.0, 0.0, 10.0] {
        let geometry = Handle::new(GeomCartesianPoint::new(x, 0.0, 0.0));
        let point: Handle<dyn AisInteractiveObject> =
            AisPoint::new(&geometry.cast()).unwrap().into_cast();
        ctx.display(&point, false).unwrap();
        points.push(point);
    }
    view.set_proj_orientation(V3dTypeOfOrientation::ZUP_TOP, false)
        .unwrap();
    view.fit_all(0.1, false).unwrap();
    for x in [-10.0, 0.0, 10.0] {
        let (px, py) = view.convert_world_to_pixels(x, 0.0, 0.0).unwrap();
        int(px);
        int(py);
    }
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut c = AisViewController::new().unwrap();
    c.set_hooks(Box::new(Recorder(events.clone())));
    state(&c, &ctx, &view, &points, &events);
    for _ in 0..r.integer() {
        let (op, a, b, d, e) = (
            r.integer(),
            r.integer(),
            r.integer(),
            r.integer(),
            r.integer(),
        );
        let ui = c.change_input_buffer(AisViewInputBufferType::Ui);
        match op {
            0 => c.select_in_viewer(Vec2::new(a, b), scheme(d)),
            1 => {
                ui.move_to.to_hilight = true;
                ui.move_to.point = Vec2::new(a, b);
            }
            2 => c.update_rubber_band(Vec2::new(a, b), Vec2::new(d, e)),
            3 => c.update_poly_selection(Vec2::new(a, b), d != 0),
            4 => {
                ui.selection.to_apply_tool = true;
                ui.selection.scheme = scheme(a);
            }
            5 => ui.selection.tool = tool(a),
            6 => c.key_down(a as AspectVKey, 0.0, 1.0).unwrap(),
            7 => c.update_view_orientation(orientation(a), b != 0),
            8 => attempt(|| c.flush_view_events(&ctx, &view, a != 0)),
            9 => {
                ui.dragging.to_start = true;
                ui.dragging.point_start = Vec2::new(a, b);
            }
            10 => ui.dragging.to_stop = true,
            11 => c.set_continuous_redraw(a != 0),
            12 => c.set_allow_highlight(a != 0),
            13 => c.reset_previous_move_to(),
            _ => c.abort_view_animation().unwrap(),
        }
        state(&c, &ctx, &view, &points, &events);
    }
    end();
}

fn orientation(value: i32) -> V3dTypeOfOrientation {
    V3dTypeOfOrientation::ALL
        .into_iter()
        .find(|o| *o as i32 == value)
        .unwrap()
}
