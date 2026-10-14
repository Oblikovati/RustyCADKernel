//! Exact native object interpolation and invalidation state on recording views.
use super::common::{Reader, attempt, end, flag, int, real};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Vector3};
use rustycad_tkservice::{
    aspect::AspectNeutralWindow, graphic3d::*, null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::v3d::V3dViewSource;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, v3d::*};
struct ObjectAnimationScene {
    context: Handle<AisInteractiveContext>,
    object: Handle<AisPoint>,
    _parent: Handle<AisPoint>,
    views: [Handle<dyn V3dViewSource>; 2],
    _viewer: Handle<V3dViewer>,
}
/// Compare one native timeline request; `object_case(&[0.,0.,2.,0.,1.])`.
pub fn object_case(args: &[f64]) {
    let mut reader = Reader::new(args);
    let flags = reader.integer();
    let kind = reader.integer();
    let (duration, start, pts) = (reader.real(), reader.real(), reader.real());
    let scene = ObjectAnimationScene::new(flags);
    let object = if flags & 2 != 0 {
        Handle::default()
    } else {
        scene.object.cast()
    };
    let animation = AisAnimationObject::new(
        "move",
        &scene.target_context(flags),
        &object,
        endpoint(kind, false),
        endpoint(kind, true),
    );
    animation.set_own_duration(duration);
    animation.set_start_pts(start);
    animation.start(false).unwrap();
    if flags & 4 != 0 {
        animation.pause().unwrap();
    }
    attempt(|| {
        flag(animation.update(pts)?);
        Ok(())
    });
    scene.state(&animation);
}
/// Compare the protected nonvirtual base update body; `base_case(&[16.,0.])`.
pub fn base_case(args: &[f64]) {
    let mut reader = Reader::new(args);
    let flags = reader.integer();
    let scene = ObjectAnimationScene::new(flags);
    let animation =
        AisBaseAnimationObject::new("base", &scene.target_context(flags), &scene.object.cast());
    animation.start(true).unwrap();
    attempt(|| {
        ais_base_animation_object_update_transform(&animation, &endpoint(reader.integer(), true))
    });
    scene.state(&animation);
}
/// `v3_animation_axis_rotation Flags Start End Duration Pts`: a rotation about (1,2,3)+Z.
pub fn axis_rotation_case(args: &[f64]) {
    let mut reader = Reader::new(args);
    let flags = reader.integer();
    let (start, end_angle, duration, pts) =
        (reader.real(), reader.real(), reader.real(), reader.real());
    let scene = ObjectAnimationScene::new(flags);
    let object = if flags & 2 != 0 {
        Handle::default()
    } else {
        scene.object.cast()
    };
    let axis = Axis1::new(
        Point3::new(1., 2., 3.),
        Direction3::new(0., 0., 1.).unwrap(),
    );
    let animation = AisAnimationAxisRotation::new(
        "spin",
        &scene.target_context(flags),
        &object,
        axis,
        start,
        end_angle,
    );
    animation.set_own_duration(duration);
    animation.start(false).unwrap();
    attempt(|| {
        flag(animation.update(pts)?);
        Ok(())
    });
    scene.state(&animation);
}
fn endpoint(kind: i32, end: bool) -> Transform3 {
    let mut result = Transform3::default();
    if kind == 1 {
        result
            .set_rotation(
                Axis1::new(Point3::default(), Direction3::new(0., 0., 1.).unwrap()),
                if end { 1.2 } else { 0.3 },
            )
            .unwrap();
    }
    let scale = match (kind, end) {
        (2, true) => -2.,
        (2, false) | (3, true) => -1.,
        (1, true) => 2.,
        _ => 1.,
    };
    result.set_scale_factor(scale).unwrap();
    result.set_translation_part(Vector3::new(if end { 6. } else { 2. }, 0., 0.));
    result
}
impl ObjectAnimationScene {
    fn new(flags: i32) -> Self {
        let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
        let views = [
            configured_view(&viewer, flags),
            configured_view(&viewer, flags),
        ];
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let point = Handle::new(GeomCartesianPoint::new(0., 0., 0.));
        let object = AisPoint::new(&point.cast()).unwrap();
        let parent = AisPoint::new(&point.cast()).unwrap();
        seed_object(&context, &object, &parent, flags);
        if flags & 32 != 0 {
            viewer.set_view_off_one(&views[1]);
        }
        for view in &views {
            view.get().unwrap().view().redraw();
        }
        Self {
            context,
            object,
            _parent: parent,
            views,
            _viewer: viewer,
        }
    }
    fn target_context(&self, flags: i32) -> Handle<AisInteractiveContext> {
        if flags & 1 != 0 {
            Handle::default()
        } else {
            self.context.clone()
        }
    }
    fn state(&self, animation: &dyn AisAnimationSource) {
        flag(animation.is_stopped());
        flag(self.object.local_transformation_geom().is_null());
        for row in 1..=3 {
            for col in 1..=4 {
                real(self.object.local_transformation().value(row, col).unwrap());
            }
        }
        for view in &self.views {
            flag(view.is_invalidated());
            real(view.camera().z_near());
            real(view.camera().z_far());
        }
        int(self.object.selections().len() as i32);
        for selection in self.object.selections() {
            int(selection.update_status() as i32);
            int(selection.bvh_update_status() as i32);
        }
        end();
    }
}
fn configured_view(viewer: &V3dViewer, flags: i32) -> Handle<dyn V3dViewSource> {
    let view = viewer.create_view().unwrap();
    view.set_immediate_update(false);
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(64, 64);
    view.set_window(&window.cast(), 0).unwrap();
    view.set_auto_z_fit_mode(false, 1.).unwrap();
    let camera = view.camera();
    camera.set_eye(&Point3::new(0., 0., 20.)).unwrap();
    camera.set_center(&Point3::default()).unwrap();
    camera.set_up(&Direction3::new(0., 1., 0.).unwrap());
    camera.set_scale(20.);
    camera
        .set_z_range(1., if flags & 64 != 0 { 5. } else { 100. })
        .unwrap();
    view
}
fn seed_object(
    context: &AisInteractiveContext,
    object: &Handle<AisPoint>,
    parent: &Handle<AisPoint>,
    flags: i32,
) {
    object
        .set_local_transformation(&Transform3::translation_vector(Vector3::new(9., 0., 0.)))
        .unwrap();
    if flags & 128 != 0 {
        parent
            .set_local_transformation(&Transform3::translation_vector(Vector3::new(10., 0., 0.)))
            .unwrap();
        parent.add_child(&object.cast()).unwrap();
    }
    if flags & 8 != 0 {
        object.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOP);
    }
    if flags & 16 != 0 {
        context.display(&object.cast(), false).unwrap();
    }
}
