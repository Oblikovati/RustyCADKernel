//! Camera, clipping and persistence inputs shared by traversal tests and the native twin.
use rustycad_tkernel::{
    ncollection::{Array1, Vec4},
    standard::Handle,
};
use rustycad_tkmath::gp::{Axis1, Direction3, Point2, Point3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::select_mgr::*;
#[derive(Clone, Copy)]
pub struct TraversalConfig {
    pub kind: i32,
    pub camera: i32,
    pub persistence: i32,
    pub entity_persistence: i32,
    pub transform: i32,
    pub clipping: i32,
    pub overlap: bool,
    pub flags: i32,
    pub failing: i32,
    pub count: i32,
}
impl Default for TraversalConfig {
    fn default() -> Self {
        Self {
            kind: 1,
            camera: 1,
            persistence: 0,
            entity_persistence: 0,
            transform: 0,
            clipping: 0,
            overlap: true,
            flags: 0,
            failing: -1,
            count: 3,
        }
    }
}
pub fn traversal_planes(coefficient: f64, offset: f64) -> Handle<Graphic3dSequenceOfHClipPlane> {
    let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    planes.append(Handle::new(
        Graphic3dClipPlane::from_equation(Vec4::new(coefficient, 0., 0., offset)).unwrap(),
    ));
    planes
}
pub fn traversal_persistence(code: i32) -> Handle<dyn Graphic3dTransformPersSource> {
    if code == 0 {
        return Handle::default();
    }
    let mode = [0, 2, 64, 258, 320, 128][code as usize];
    Handle::new(Graphic3dTransformPers::new(Graphic3dTransModeFlags(mode)).unwrap()).cast()
}
pub fn traversal_volume(config: TraversalConfig) -> SelectMgrSelectingVolumeManager {
    let mut volume = SelectMgrSelectingVolumeManager::new();
    volume.allow_overlap_detection(config.overlap);
    match config.kind {
        0 => return volume,
        1 => volume.init_point_selecting_volume(Point2::new(40., 30.)),
        2 => volume.init_box_selecting_volume(Point2::new(20., 15.), Point2::new(60., 45.)),
        3 => volume.init_axis_selecting_volume(Axis1::new(
            Point3::new(0., 0., 20.),
            Direction3::new(0., 0., -1.).unwrap(),
        )),
        4 => {
            let vertices = [
                Point2::new(20., 15.),
                Point2::new(60., 15.),
                Point2::new(60., 45.),
                Point2::new(20., 45.),
            ];
            let points = Array1::from_shared(&vertices, 1);
            volume.init_polyline_selecting_volume(&points).unwrap();
        }
        _ => unreachable!(),
    }
    configure_traversal_camera(&volume, config);
    volume.build_selecting_volume().unwrap();
    if config.clipping == 1 || config.clipping == 3 {
        volume
            .set_view_clipping(&traversal_planes(1., 0.), &Handle::default(), None)
            .unwrap();
    }
    if config.camera == 0 {
        volume.set_camera(&Handle::default()).unwrap();
    }
    volume
}
fn configure_traversal_camera(volume: &SelectMgrSelectingVolumeManager, config: TraversalConfig) {
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_scale(20.);
    camera.set_aspect(4. / 3.);
    camera
        .set_eye_and_center(&Point3::new(0., 0., 20.), &Point3::default())
        .unwrap();
    if config.camera == 2 {
        camera.set_projection_type(Graphic3dCameraProjection::Perspective);
    }
    volume.set_camera(&camera).unwrap();
    volume.set_window_size(80, 60).unwrap();
    volume.set_pixel_tolerance(3).unwrap();
}
