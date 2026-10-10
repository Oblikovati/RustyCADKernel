//! Public picking builds the active volume before entering the shared traversal.
use super::*;
use crate::{select_basics::SelectBasicsSelectingVolumeManager, v3d::V3dView};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle, exceptions::null_object},
};
use rustycad_tkmath::gp::{Axis1, Point2};
use rustycad_tkservice::graphic3d::Graphic3dCViewMembers;

/// Build point, area or axis queries against a view; `selector.pick_point(40,30,&view)?`.
pub trait SelectMgrViewerSelectorPickMembers: SelectMgrViewerSelectorSource {
    /// Apply the current sensitivity tolerance to a pixel query.
    /// Example: `selector.pick_point(40,30,&view)?`.
    fn pick_point(&self, x: i32, y: i32, view: &V3dView) -> Result<(), Failure> {
        self.update_z_layers(view)?;
        let base = self.viewer_selector_base();
        base.selecting_volume
            .write()
            .init_point_selecting_volume(Point2::new(f64::from(x), f64::from(y)));
        let tolerance = base.tolerances.read().tolerance();
        let volume = base.selecting_volume.read().clone();
        volume.set_pixel_tolerance(tolerance)?;
        configure_pick_view(base, view)?;
        finish_selector_pick(self, view)
    }
    /// Select a pixel rectangle, retaining the rectangular volume's tolerance.
    /// Example: `selector.pick_box(20,15,60,45,&view)?`.
    fn pick_box(
        &self,
        xmin: i32,
        ymin: i32,
        xmax: i32,
        ymax: i32,
        view: &V3dView,
    ) -> Result<(), Failure> {
        self.update_z_layers(view)?;
        let base = self.viewer_selector_base();
        base.selecting_volume.write().init_box_selecting_volume(
            Point2::new(f64::from(xmin), f64::from(ymin)),
            Point2::new(f64::from(xmax), f64::from(ymax)),
        );
        configure_pick_view(base, view)?;
        finish_selector_pick(self, view)
    }
    /// Build a polygon query in pixel coordinates.
    /// Example: `selector.pick_polyline(&pixels,&view)?`.
    fn pick_polyline(&self, points: &Array1<'_, Point2>, view: &V3dView) -> Result<(), Failure> {
        self.update_z_layers(view)?;
        let base = self.viewer_selector_base();
        base.selecting_volume
            .write()
            .init_polyline_selecting_volume(points)?;
        configure_pick_view(base, view)?;
        finish_selector_pick(self, view)
    }
    /// Pick along a world-space axis without assigning view camera or window dimensions.
    /// Example: `selector.pick_axis(Axis1::default(),&view)?`.
    fn pick_axis(&self, axis: Axis1, view: &V3dView) -> Result<(), Failure> {
        self.update_z_layers(view)?;
        self.viewer_selector_base()
            .selecting_volume
            .write()
            .init_axis_selecting_volume(axis);
        finish_selector_pick(self, view)
    }
}
impl<T: SelectMgrViewerSelectorSource + ?Sized> SelectMgrViewerSelectorPickMembers for T {}
fn configure_pick_view(base: &SelectMgrViewerSelectorBase, view: &V3dView) -> Result<(), Failure> {
    let volume = base.selecting_volume.read().clone();
    volume.set_camera(&view.camera())?;
    let window = view.window();
    let window = window.get().ok_or_else(|| {
        null_object("ViewerSelector::Pick: window=null; expected an attached view window")
    })?;
    // Window::Size is client code and may inspect or replace the active volume.
    let (width, height) = window.size();
    let current = base.selecting_volume.read().clone();
    current.set_window_size(width, height)
}
fn finish_selector_pick<T: SelectMgrViewerSelectorSource + ?Sized>(
    selector: &T,
    view: &V3dView,
) -> Result<(), Failure> {
    let base = selector.viewer_selector_base();
    let volume = base.selecting_volume.read().clone();
    volume.build_selecting_volume()?;
    set_pick_clipping(base, view)?;
    selector.traverse_sensitives(view.view().identification())
}
fn set_pick_clipping(base: &SelectMgrViewerSelectorBase, view: &V3dView) -> Result<(), Failure> {
    let planes = view.clip_planes();
    {
        let mut volume = base.selecting_volume.write();
        volume.view_planes = planes.clone();
        volume.object_planes = Handle::default();
    }
    let current = base.selecting_volume.read().clone();
    if current.get_active_selection_type() != SelectMgrSelectionType::Point {
        return Ok(());
    }
    base.selecting_volume.write().clip_range.set_void();
    let Some(planes) = planes.get().filter(|planes| !planes.is_empty()) else {
        return Ok(());
    };
    let near = current.required_volume("Pick clipping")?.get_near_pnt()?;
    let direction = current
        .required_volume("Pick clipping")?
        .get_view_ray_direction()?;
    base.selecting_volume
        .write()
        .clip_range
        .add_clipping_planes(planes, &Axis1::new(near, direction))
}
