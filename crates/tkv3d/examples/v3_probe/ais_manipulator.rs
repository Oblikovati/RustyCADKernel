//! AIS_Manipulator attached to a point of a top view, the twin of
//! tools/oracle/tkv3d_probe_ais_manipulator.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::sensitive_display::display_driver::SensitiveDisplayDriver;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkmath::gp::{self, Axis2, Point3, Transform3};
use rustycad_tkservice::aspect::AspectNeutralWindow;
use rustycad_tkservice::graphic3d::Graphic3dTransformPersMembers;
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::prs_mgr::*;
use rustycad_tkv3d::select_mgr::*;
use rustycad_tkv3d::select3d::Select3DSensitiveEntityMembers;
use rustycad_tkv3d::v3d::{V3dTypeOfOrientation, V3dViewSource, V3dViewer};

fn trsf(t: &Transform3) {
    for row in 1..=3 {
        for column in 1..=4 {
            real(t.value(row, column).unwrap());
        }
    }
}

fn state(
    m: &Handle<AisManipulator>,
    ctx: &Handle<AisInteractiveContext>,
    object: &Handle<dyn AisInteractiveObject>,
) {
    flag(m.is_attached());
    flag(m.has_active_mode());
    int(m.active_mode() as i32);
    int(m.active_axis_index());
    flag(m.has_active_transformation());
    flag(m.zoom_persistence());
    int(m.skin_mode() as i32);
    real(f64::from(m.size()));
    flag(m.is_mode_activation_on_detection());
    let p = m.position();
    let (l, d, x) = (p.location(), p.direction(), p.x_direction());
    for v in [
        l.x(),
        l.y(),
        l.z(),
        d.x(),
        d.y(),
        d.z(),
        x.x(),
        x.y(),
        x.z(),
    ] {
        real(v);
    }
    trsf(&m.local_transformation());
    trsf(&object.local_transformation());
    trsf(&m.start_transformation());
    let pers = m.transform_persistence();
    flag(pers.is_null());
    if let Some(pers) = pers.get() {
        int(pers.mode().0);
        if pers.is_zoom_or_rotate() || pers.is_axial() {
            let a = pers.anchor_point().unwrap();
            real(a.x());
            real(a.y());
            real(a.z());
        }
    }
    let this: Handle<dyn AisInteractiveObject> = m.clone().into_cast();
    flag(ctx.is_displayed(&this));
    int(ctx.nb_selected());
    flag(ctx.has_detected());
    for mode in 1..=4 {
        flag(
            ctx.selection_manager()
                .is_activated(&this.cast(), mode)
                .unwrap(),
        );
    }
    int(m.presentations().len() as i32);
    for index in 0..3 {
        for mode in 1..=4 {
            flag(
                m.get_group(index, AisManipulatorMode::from_i32(mode))
                    .is_null(),
            );
        }
    }
}

fn selection(
    m: &Handle<AisManipulator>,
    mode: i32,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let s = Handle::new(SelectMgrSelection::new(mode));
    m.compute_selection(&s, mode)?;
    int(s.entities().len() as i32);
    for e in s.entities() {
        let sensitive = e.base_sensitive();
        int(sensitive.nb_sub_elements());
        int(sensitive.sensitivity_factor());
        let owner = sensitive.owner_id();
        let manipulator_owner = owner.cast::<AisManipulatorOwner>();
        int(manipulator_owner.get().map_or(-1, |o| o.index()));
        int(manipulator_owner.get().map_or(-1, |o| o.mode() as i32));
        int(owner.priority());
        let b = sensitive.bounding_box();
        flag(b.is_valid());
        if b.is_valid() {
            let (lo, hi) = (b.corner_min(), b.corner_max());
            for v in [lo.x, lo.y, lo.z, hi.x, hi.y, hi.z] {
                real(v);
            }
        }
    }
    Ok(())
}

/// `v3_ais_manipulator Count (Op A B C)...`.
pub fn manipulator_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (driver, trace) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let view: Handle<dyn V3dViewSource> = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(400, 300);
    view.set_window(&window.cast(), 0).unwrap();
    let ctx = AisInteractiveContext::new(&viewer).unwrap();
    let point = |x: f64, y: f64| -> Handle<dyn AisInteractiveObject> {
        AisPoint::new(&Handle::new(GeomCartesianPoint::new(x, y, 0.0)).cast())
            .unwrap()
            .into_cast()
    };
    let object = point(10.0, 20.0);
    let other = point(150.0, 120.0);
    ctx.display(&object, false).unwrap();
    ctx.display(&other, false).unwrap();
    view.set_proj_orientation(V3dTypeOfOrientation::ZUP_TOP, false)
        .unwrap();
    view.fit_all(0.1, false).unwrap();
    let (px, py) = view.convert_world_to_pixels(10.0, 20.0, 0.0).unwrap();
    int(px);
    int(py);
    let m = AisManipulator::new().unwrap();
    let mut dump = false;
    trace.write().events.clear();
    state(&m, &ctx, &object);
    for _ in 0..r.integer() {
        let (op, a, b, c) = (r.integer(), r.integer(), r.integer(), r.integer());
        operate(&m, &ctx, &view, &object, (px, py), [op, a, b, c], &mut dump);
        if dump {
            let events = trace.read();
            int(events.events.len() as i32);
            for value in &events.events {
                real(*value);
            }
        }
        trace.write().events.clear();
        state(&m, &ctx, &object);
    }
    end();
}

fn operate(
    m: &Handle<AisManipulator>,
    ctx: &Handle<AisInteractiveContext>,
    view: &Handle<dyn V3dViewSource>,
    object: &Handle<dyn AisInteractiveObject>,
    (px, py): (i32, i32),
    [op, a, b, c]: [i32; 4],
    dump: &mut bool,
) {
    let mode = AisManipulatorMode::from_i32;
    match op {
        0 => {
            let mut options = AisManipulatorOptionsForAttach::default();
            options
                .set_adjust_position(a != 0)
                .set_adjust_size(b != 0)
                .set_enable_modes(c != 0);
            attempt(|| m.attach(object, &options));
        }
        1 => attempt(|| m.set_part(a, mode(b), c != 0)),
        2 => m.set_part_all(mode(a), b != 0),
        3 => m.set_size(a as f32),
        4 => m.set_gap(a as f32 / 10.0),
        5 => m.set_skin_mode(if a == 1 {
            AisManipulatorSkin::Flat
        } else {
            AisManipulatorSkin::Shaded
        }),
        6 => attempt(|| m.set_zoom_persistence(a != 0)),
        7 => m.set_mode_activation_on_detection(a != 0),
        8 => attempt(|| selection(m, a)),
        9 => int(ctx.move_to(px + a, py + b, view, false).unwrap() as i32),
        10 => int(ctx.select_detected(AisSelectionScheme::Replace).unwrap() as i32),
        11 => attempt(|| m.start_transform(px + a, py + b, view)),
        12 => attempt(|| m.transform_at(px + a, py + b, view).map(|t| trsf(&t))),
        13 => m.stop_transform(a != 0).unwrap(),
        14 => attempt(|| m.detach()),
        15 => m.deactivate_current_mode().unwrap(),
        16 => attempt(|| {
            rustycad_tkernel::standard::catch_native_failure(|| {
                m.recompute_transformation(&view.camera());
                Ok(())
            })
        }),
        17 => m
            .set_position(
                &Axis2::new(
                    Point3::new(f64::from(a), f64::from(b), f64::from(c)),
                    gp::dz(),
                )
                .unwrap(),
            )
            .unwrap(),
        18 => attempt(|| {
            rustycad_tkernel::standard::catch_native_failure(|| {
                let at = Vec2::new(px + b, py + c);
                let action = [
                    AisDragAction::Start,
                    AisDragAction::Confirmed,
                    AisDragAction::Update,
                    AisDragAction::Stop,
                    AisDragAction::Abort,
                ][a as usize];
                flag(m.process_dragging(ctx, view, &Handle::default(), at, at, action));
                Ok(())
            })
        }),
        19 => *dump = a != 0,
        20 => ctx.clear_selected(false).unwrap(),
        21 => attempt(|| m.recompute_selection(mode(a))),
        22 => {
            let mut behavior = AisManipulatorBehaviorOnTransform::default();
            behavior
                .set_follow_translation(a != 0)
                .set_follow_rotation(b != 0)
                .set_follow_dragging(c != 0);
            m.set_transform_behavior(behavior);
        }
        _ => ctx.redisplay(&m.clone().into_cast(), false, false).unwrap(),
    }
}
