//! Native cursor zoom shares V3d's scale bounds and uses the actual optional picked world point.
use super::{AisViewController, view_controller_camera_primitives::*};
use crate::v3d::{V3dView, limited_zoom_coef};
use rustycad_tkernel::{ncollection::Vec2, standard::Failure};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{aspect::AspectScrollDelta, graphic3d::Graphic3dCamera};
impl AisViewController {
    /// Zoom about cursor or camera center with an optional world hit; `controller.handle_zoom(&view, &scroll, Some(&hit))?`.
    pub fn handle_zoom(
        &mut self,
        view: &V3dView,
        scroll: &AspectScrollDelta,
        hit: Option<&Point3>,
    ) -> Result<(), Failure> {
        if !self.to_allow_zooming() {
            return Ok(());
        }
        self.abort_view_animation()?;
        if let Some(point) = hit {
            self.center_zoom_camera_on_hit(&view.camera(), point)?;
        }
        self.apply_camera_zoom(view, scroll, hit)
    }
    fn apply_camera_zoom(
        &self,
        view: &V3dView,
        scroll: &AspectScrollDelta,
        hit: Option<&Point3>,
    ) -> Result<(), Failure> {
        let coefficient = primitive_scroll_coefficient(scroll.delta);
        if !scroll.has_point() {
            view.set_zoom(coefficient, true)?;
            return invalidate_and_sync_primitive(view);
        }
        if coefficient <= 0.0 {
            return Ok(());
        }
        self.zoom_camera_at_cursor(view, scroll, hit, coefficient)?;
        invalidate_and_sync_primitive(view)
    }
    fn center_zoom_camera_on_hit(
        &self,
        camera: &Graphic3dCamera,
        point: &Point3,
    ) -> Result<(), Failure> {
        let distance = (point.xyz() - camera.eye().xyz()).modulus();
        // C++ std::max keeps its first argument for NaN/equal comparisons.
        let distance = if self.min_zoom_distance() < distance {
            distance
        } else {
            self.min_zoom_distance()
        };
        let center = camera.eye().xyz() + camera.direction().xyz() * distance;
        camera.set_center(&Point3::new(center.x(), center.y(), center.z()))
    }
    fn zoom_camera_at_cursor(
        &self,
        view: &V3dView,
        scroll: &AspectScrollDelta,
        hit: Option<&Point3>,
        coefficient: f64,
    ) -> Result<(), Failure> {
        let camera = view.camera();
        let coefficient = limited_zoom_coef(&camera, coefficient);
        let (x, y) = view.convert_to_view(scroll.point.x, scroll.point.y)?;
        camera.set_scale(camera.scale() / coefficient);
        let pan = if let Some(point) = hit {
            primitive_hit_zoom_pan(view, scroll, point)?
        } else {
            Vec2::new(x - x / coefficient, y - y / coefficient)
        };
        translate_primitive_camera(&camera, pan.x, pan.y)
    }
}
fn primitive_scroll_coefficient(delta: f64) -> f64 {
    let coefficient = delta.abs() / 100.0 + 1.0;
    if delta > 0.0 {
        coefficient
    } else {
        1.0 / coefficient
    }
}
fn primitive_hit_zoom_pan(
    view: &V3dView,
    scroll: &AspectScrollDelta,
    point: &Point3,
) -> Result<Vec2<f64>, Failure> {
    let camera = view.camera();
    let frame = primitive_camera_frame(&camera)?;
    let eye_to_hit = point.xyz() - camera.eye().xyz();
    let dimensions = camera.view_dimensions_at(eye_to_hit.dot(camera.direction().xyz()));
    let (width, height) = primitive_window_size(view)?;
    let x = f64::from(scroll.point.x) - 0.5 * f64::from(width);
    // Native perspective hit correction differs by one pixel from Convert's Y mapping.
    let y = f64::from(height) - f64::from(scroll.point.y) - 1.0 - 0.5 * f64::from(height);
    Ok(Vec2::new(
        eye_to_hit.dot(frame.x_direction().xyz()) - dimensions.x() * x / f64::from(width),
        eye_to_hit.dot(frame.y_direction().xyz()) - dimensions.y() * y / f64::from(height),
    ))
}
