//! Real null-driver view and public pick operations shared by tests and oracle.
use super::traversal_scene::volume::{TraversalConfig, traversal_planes, traversal_volume};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Axis1, Direction3, Point2, Point3};
use rustycad_tkservice::{aspect::AspectNeutralWindow, null_driver::NullGraphicDriver};
use rustycad_tkv3d::v3d::V3dViewSource;
use rustycad_tkv3d::{
    select_mgr::*,
    v3d::{V3dView, V3dViewer},
};
pub fn pick_view(config: TraversalConfig) -> (Handle<V3dViewer>, Handle<dyn V3dViewSource>) {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(80, 60);
    view.set_window(&window.cast(), 0).unwrap();
    let volume = traversal_volume(TraversalConfig { kind: 1, ..config });
    view.set_camera(&volume.camera()).unwrap();
    if config.clipping == 1 || config.clipping == 3 {
        view.set_clip_planes(&traversal_planes(1., 0.));
    }
    (viewer, view)
}
pub fn run_public_pick(
    selector: &dyn SelectMgrViewerSelectorSource,
    view: &V3dView,
    kind: i32,
) -> Result<(), Failure> {
    match kind {
        1 => selector.pick_point(40, 30, view),
        2 => selector.pick_box(20, 15, 60, 45, view),
        3 => selector.pick_axis(
            Axis1::new(Point3::new(0., 0., 20.), Direction3::new(0., 0., -1.)?),
            view,
        ),
        4 => {
            let vertices = [
                Point2::new(20., 15.),
                Point2::new(60., 15.),
                Point2::new(60., 45.),
                Point2::new(20., 45.),
            ];
            selector.pick_polyline(&Array1::from_shared(&vertices, 1), view)
        }
        _ => unreachable!(),
    }
}
