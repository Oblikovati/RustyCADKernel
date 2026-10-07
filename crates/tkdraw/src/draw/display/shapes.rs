//! Markers, circles and strings of `Draw_Display` (`Draw_Display.cxx` and the
//! `DrawString` methods of `Draw_Viewer.cxx`).
use super::{DrawDisplay, DrawingMode, out_of_range, postscript_pixel};
use crate::draw::color::DrawMarkerShape;
use rustycad_tkmath::elclib::{circle_value, circle_value_2d};
use rustycad_tkmath::gp::{Circle2, Circle3, Point2, Point3, Vector2, Vector3};
use std::f64::consts::PI;

/// `MAXPNT`: the most points of a drawn circle.
const MAXPNT: i32 = 200;
/// `DEFLECTION`: the target chord length of a circle in pixels.
const DEFLECTION: f64 = 5.0;

impl DrawDisplay<'_> {
    /// `DrawMarker(gp_Pnt, shape, int)`.
    pub fn draw_marker_3d(&mut self, point: Point3, shape: DrawMarkerShape, size: i32) {
        let projected = self.project(point);
        self.draw_marker(projected, shape, size);
    }

    /// `DrawMarker(gp_Pnt2d, shape, int size)`: a marker of `size` pixels; the marker
    /// does not grow the drawing bounds.
    pub fn draw_marker(&mut self, point: Point2, shape: DrawMarkerShape, pixel_size: i32) {
        self.set_draw_bounds(false);
        let size = pixel_size as f64 / self.zoom();
        let (mut p1, mut p2) = (point, point);
        let step = |p: &mut Point2, x: f64, y: f64| p.translate(Vector2::new(x, y));
        match shape {
            DrawMarkerShape::Square => {
                step(&mut p1, -size, -size);
                step(&mut p2, size, -size);
                self.draw(p1, p2);
                step(&mut p1, 2.0 * size, 2.0 * size);
                self.draw(p1, p2);
                step(&mut p2, -2.0 * size, 2.0 * size);
                self.draw(p1, p2);
                step(&mut p1, -2.0 * size, -2.0 * size);
                self.draw(p1, p2);
            }
            DrawMarkerShape::Losange => {
                step(&mut p1, -size, 0.0);
                step(&mut p2, 0.0, size);
                self.draw(p1, p2);
                step(&mut p1, 2.0 * size, 0.0);
                self.draw(p1, p2);
                step(&mut p2, 0.0, -2.0 * size);
                self.draw(p1, p2);
                step(&mut p1, -2.0 * size, 0.0);
                self.draw(p1, p2);
            }
            DrawMarkerShape::X => {
                step(&mut p1, -size, -size);
                step(&mut p2, size, size);
                self.draw(p1, p2);
                step(&mut p1, 2.0 * size, 0.0);
                step(&mut p2, -2.0 * size, 0.0);
                self.draw(p1, p2);
            }
            DrawMarkerShape::Plus => {
                step(&mut p1, -size, 0.0);
                step(&mut p2, size, 0.0);
                self.draw(p1, p2);
                step(&mut p1, size, size);
                step(&mut p2, -size, -size);
                self.draw(p1, p2);
            }
            DrawMarkerShape::Circle => self.draw_marker_circle(point, pixel_size),
            DrawMarkerShape::CircleZoom => {}
        }
        self.set_draw_bounds(true);
        self.move_to(point);
    }

    /// The `Draw_Circle` marker: a circle of `pixel_size` pixels around `point`.
    fn draw_marker_circle(&mut self, point: Point2, pixel_size: i32) {
        let mut circle = Circle2::default();
        // A cast `int` is never negative here (marker sizes are positive), as in C++.
        let _ = circle.set_radius(pixel_size as f64);
        circle.set_location(point);
        self.draw_circle(&circle, 0.0, 2.0 * PI, false);
    }

    /// `DrawMarker(gp_Pnt, shape, double)`.
    pub fn draw_marker_3d_scaled(&mut self, point: Point3, shape: DrawMarkerShape, size: f64) {
        let projected = self.project(point);
        self.draw_marker_scaled(projected, shape, size);
    }

    /// `DrawMarker(gp_Pnt2d, shape, double R)`: fixed shapes use the integer part of
    /// `R` as pixels (nothing for 0); `CircleZoom` is a circle of radius `R` in model
    /// units, or a plus when it would be smaller than 2 pixels.
    pub fn draw_marker_scaled(&mut self, point: Point2, shape: DrawMarkerShape, radius: f64) {
        if shape != DrawMarkerShape::CircleZoom {
            let pixels = radius as i32;
            if pixels == 0 {
                return;
            }
            self.draw_marker(point, shape, pixels);
        } else {
            if radius == 0.0 {
                return;
            }
            self.draw_zoomed_circle_marker(point, radius);
        }
        self.set_draw_bounds(true);
        self.move_to(point);
    }

    /// The `Draw_CircleZoom` marker body.
    fn draw_zoomed_circle_marker(&mut self, point: Point2, radius: f64) {
        let mut circle = Circle2::default();
        // R != 0 here; a negative R keeps the default radius, as gp_Circ2d raises.
        let _ = circle.set_radius(radius);
        circle.set_location(point);
        if radius * self.zoom() > 2.0 {
            self.draw_circle(&circle, 0.0, 2.0 * PI, true);
        } else {
            self.draw_marker(point, DrawMarkerShape::Plus, 5);
        }
    }

    /// `Draw(gp_Circ, A1, A2, ModifyWithZoom)`: the arc from `a1` to `a2` (radians) as a
    /// polyline with about 5-pixel chords, between 6 and 200 points, by the rotation
    /// recurrence `V(i+1) = 2 cos(angle) V(i) - V(i-1)`. Without zoom modification the
    /// radius is taken in pixels (cast to an integer).
    pub fn draw_circle_3d(&mut self, circle: &Circle3, a1: f64, a3: f64, modify_with_zoom: bool) {
        let (angle, count) = arc_steps(a1, a3, circle.radius() * self.zoom(), true);
        let a2 = end_angle(a1, a3);
        let mut local = *circle;
        if !modify_with_zoom {
            let pixels = (local.radius() / self.zoom()) as i32;
            let _ = local.set_radius(pixels as f64);
        }
        let center = local.location();
        let value = |u: f64| circle_value(u, local.position(), local.radius());
        let first = value(a1);
        self.move_to_3d(first);
        let second = value(a1 + angle);
        let (mut v1, mut v2) = (
            Vector3::between(center, first),
            Vector3::between(center, second),
        );
        self.draw_to_3d(second);
        let factor = 2.0 * angle.cos();
        for _ in 2..count {
            let next = factor * v2 - v1;
            v1 = v2;
            v2 = next;
            self.draw_to_3d(center.translated(next));
        }
        self.draw_to_3d(value(a2));
    }

    /// `Draw(gp_Circ2d, A1, A2, ModifyWithZoom)`: as [`Self::draw_circle_3d`] in the
    /// projection plane (here the radius is not cast to an integer).
    pub fn draw_circle(&mut self, circle: &Circle2, a1: f64, a3: f64, modify_with_zoom: bool) {
        let (angle, count) = arc_steps(a1, a3, circle.radius() * self.zoom(), false);
        let a2 = end_angle(a1, a3);
        let mut local = *circle;
        if !modify_with_zoom {
            let _ = local.set_radius(local.radius() / self.zoom());
        }
        let center = local.location();
        let value = |u: f64| circle_value_2d(u, local.position(), local.radius());
        let first = value(a1);
        self.move_to(first);
        let second = value(a1 + angle);
        let (mut v1, mut v2) = (
            Vector2::between(center, first),
            Vector2::between(center, second),
        );
        self.draw_to(second);
        let factor = 2.0 * angle.cos();
        for _ in 2..count {
            let next = factor * v2 - v1;
            v1 = v2;
            v2 = next;
            self.draw_to(center.translated(next));
        }
        self.draw_to(value(a2));
    }

    /// `DrawString(gp_Pnt, S)`.
    pub fn draw_string_3d(&mut self, point: Point3, text: &str) {
        let projected = self.project(point);
        self.draw_string_moved(projected, text, 0.0, 0.0);
    }

    /// `DrawString(gp_Pnt, S, moveX, moveY)`.
    pub fn draw_string_moved_3d(&mut self, point: Point3, text: &str, move_x: f64, move_y: f64) {
        let projected = self.project(point);
        self.draw_string_moved(projected, text, move_x, move_y);
    }

    /// `DrawString(gp_Pnt2d, S)`.
    pub fn draw_string(&mut self, point: Point2, text: &str) {
        self.draw_string_moved(point, text, 0.0, 0.0);
    }

    /// `DrawString(gp_Pnt2d, S, moveX, moveY)`: text at a point, shifted by pixels.
    pub fn draw_string_moved(&mut self, point: Point2, text: &str, move_x: f64, move_y: f64) {
        let Some(target) = self.target.as_mut() else {
            return;
        };
        if out_of_range(point) {
            return;
        }
        let zoom = target.surface.zoom();
        let zoomed = Point2::new(point.x() * zoom, point.y() * zoom);
        if out_of_range(zoomed) {
            return;
        }
        match target.state.mode {
            DrawingMode::Draw => {
                let x = (zoomed.x() + move_x + target.surface.dx() as f64) as i32;
                let y = (-zoomed.y() + move_y - target.surface.dy() as f64) as i32;
                target.surface.draw_string(x, y, text);
                if target.state.draw_bounds {
                    grow_string_bounds(&mut target.state.bounds, zoomed, move_x, move_y);
                }
            }
            DrawingMode::PostScript => {
                let (x, y) = postscript_pixel(
                    target.state,
                    Point2::new(zoomed.x() + move_x, zoomed.y() + move_y),
                );
                let stream = &mut target.state.postscript.stream;
                stream
                    .put_str("stroke\n")
                    .put_int(x.into())
                    .put_str(" ")
                    .put_int(y.into())
                    .put_str(" m\n");
                stream
                    .put_str("(")
                    .put_str(text)
                    .put_str(") show\nnewpath\n");
            }
            DrawingMode::Pick => {}
        }
    }
}

/// The string bounds update of `DrawString`: the comparison includes the shift but the
/// stored value does not (an upstream quirk, kept).
fn grow_string_bounds(bounds: &mut super::DrawBounds, point: Point2, move_x: f64, move_y: f64) {
    if point.x() + move_x > bounds.x_max {
        bounds.x_max = point.x();
    }
    if point.x() + move_x < bounds.x_min {
        bounds.x_min = point.x();
    }
    if -point.y() - move_y > bounds.y_max {
        bounds.y_max = -point.y();
    }
    if -point.y() - move_y < bounds.y_min {
        bounds.y_min = -point.y();
    }
}

/// `A2`: the end angle moved past the start by whole turns.
fn end_angle(a1: f64, a3: f64) -> f64 {
    let mut a2 = a3;
    while a2 < a1 {
        a2 += 2.0 * PI;
    }
    a2
}

/// The step angle and point count of an arc whose radius is `pixel_radius` pixels. The
/// 3D variant applies the lower bound after the upper one in all cases; the 2D variant
/// only when the upper bound did not apply (`else if`), as upstream.
fn arc_steps(a1: f64, a3: f64, pixel_radius: f64, independent_bounds: bool) -> (f64, i32) {
    let a2 = end_angle(a1, a3);
    let mut angle = DEFLECTION / pixel_radius;
    let mut count = ((a2 - a1) / angle) as i32;
    let capped = count > MAXPNT;
    if capped {
        angle = (a2 - a1) / MAXPNT as f64;
        count = MAXPNT;
    }
    if (independent_bounds || !capped) && count <= 6 {
        angle = (a2 - a1) / 6.0;
        count = 6;
    }
    (angle, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_steps_follow_the_bounds() {
        assert_eq!(arc_steps(0.0, 2.0 * PI, 1.0, true), (2.0 * PI / 6.0, 6));
        let (angle, count) = arc_steps(0.0, 2.0 * PI, 1.0e6, false);
        assert_eq!(count, MAXPNT);
        assert_eq!(angle, 2.0 * PI / 200.0);
        assert_eq!(end_angle(1.0, 0.5), 0.5 + 2.0 * PI);
    }
}
