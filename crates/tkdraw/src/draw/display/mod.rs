//! `Draw_Display`: what a drawable draws with. 3D methods project through the view
//! matrix (and perspective); 2D methods draw in the projection plane; screen sizes are
//! lengths divided by the zoom.
//!
//! A display either targets a view (a [`DisplaySurface`] plus the shared
//! [`DisplayState`]) or is the empty display of batch mode, where every operation does
//! nothing and the queries return the C++ batch values (zoom 1, view 0, nothing picked).
mod clipping;
mod pick;
mod shapes;
mod state;

pub use clipping::trim_segment;
pub use state::{
    DRAWINFINITE, DisplayState, DisplaySurface, DrawBounds, DrawXSegment, DrawingMode, MAXSEGMENT,
    PickState, PostScriptState,
};

use crate::draw::color::DrawColor;
use rustycad_tkmath::gp::{Point2, Point3};

/// `precpers`: the fraction of the focal distance beyond which points are behind the eye.
const PERSPECTIVE_LIMIT: f64 = 0.95;

/// Coordinates beyond this magnitude are not drawn (the `1.e09` guards).
const COORDINATE_LIMIT: f64 = 1.0e09;

/// The view parameters a drawable may query while drawing.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewQuery {
    /// The view type (`+X+Y`, `-2D-`, ...).
    pub view_type: String,
    /// `(xmin, ymin, xmax, ymax)` of the visible area in view pixels.
    pub frame: (i32, i32, i32, i32),
    /// The view matrix.
    pub matrix: rustycad_tkmath::gp::Transform3,
    /// The zoom.
    pub zoom: f64,
}

/// A view and the shared state a display draws with.
struct DisplayTarget<'a> {
    surface: &'a mut dyn DisplaySurface,
    state: &'a mut DisplayState,
}

/// A drawing context on one view (`Draw_Display`).
pub struct DrawDisplay<'a> {
    target: Option<DisplayTarget<'a>>,
}

impl<'a> DrawDisplay<'a> {
    /// The display of batch mode: draws nothing.
    pub fn batch() -> Self {
        Self { target: None }
    }

    /// `Draw_Display()` on the current view: the current points start at the projected
    /// origin.
    pub fn new(surface: &'a mut dyn DisplaySurface, state: &'a mut DisplayState) -> Self {
        let zoom = surface.zoom();
        let origin = Point3::new(0.0, 0.0, 0.0).transformed(surface.matrix());
        state.perspective_point = origin;
        state.current_point = Point2::new(origin.x() * zoom, origin.y() * zoom);
        Self {
            target: Some(DisplayTarget { surface, state }),
        }
    }

    /// `SetColor`: following drawings use `color` (flushing pending segments first).
    pub fn set_color(&mut self, color: DrawColor) {
        let Some(target) = self.target.as_mut() else {
            return;
        };
        if color.id() == target.state.current_color {
            return;
        }
        target.state.current_color = color.id();
        match target.state.mode {
            DrawingMode::Draw => {
                target.state.flush_segments(target.surface);
                target.surface.set_color(color.id());
            }
            DrawingMode::PostScript => write_postscript_color(target.state, color),
            DrawingMode::Pick => {}
        }
    }

    /// `SetMode`: the drawing mode of the window (3 = copy, 6 = xor).
    pub fn set_mode(&mut self, mode: i32) {
        let Some(target) = self.target.as_mut() else {
            return;
        };
        if target.state.mode == DrawingMode::Draw {
            target.state.flush_segments(target.surface);
            target.surface.set_mode(mode);
        }
    }

    /// `Flush`: sends the pending segments to the window.
    pub fn flush(&mut self) {
        if let Some(target) = self.target.as_mut() {
            target.state.flush_segments(target.surface);
        }
    }

    /// `Zoom`: the view zoom (1 in batch mode).
    pub fn zoom(&self) -> f64 {
        self.target
            .as_ref()
            .map_or(1.0, |target| target.surface.zoom())
    }

    /// `ViewId`: the view drawn on (0 in batch mode).
    pub fn view_id(&self) -> i32 {
        self.target
            .as_ref()
            .map_or(0, |target| target.state.view_id)
    }

    /// What `Draw_Grid` asks the global viewer about the view being drawn (`HasView`,
    /// `GetType`, `GetFrame`, `GetTrsf`, `Zoom`); `None` in batch mode, where the
    /// viewer has no views.
    pub fn view_query(&self) -> Option<ViewQuery> {
        let target = self.target.as_ref()?;
        let surface = &target.surface;
        Some(ViewQuery {
            view_type: surface.view_type(),
            frame: surface.viewer_frame(),
            matrix: surface.matrix(),
            zoom: surface.zoom(),
        })
    }

    /// `Draw_Bounds = enabled`: whether drawing grows the bounds (drawables such as the
    /// axes switch it off while they draw).
    pub fn set_draw_bounds(&mut self, enabled: bool) {
        if let Some(target) = self.target.as_mut() {
            target.state.draw_bounds = enabled;
        }
    }

    /// `HasPicked`: true when the last operations hit the pick point.
    pub fn has_picked(&self) -> bool {
        self.target
            .as_ref()
            .is_some_and(|target| target.state.pick.found)
    }

    /// `Project`: the 2D projection of a 3D point (the origin in batch mode, where the
    /// C++ leaves its output point untouched).
    pub fn project(&self, point: Point3) -> Point2 {
        let Some(target) = self.target.as_ref() else {
            return Point2::new(0.0, 0.0);
        };
        let projected = point.transformed(target.surface.matrix());
        let (mut x, mut y) = (projected.x(), projected.y());
        if target.surface.is_perspective() {
            let distance = target.surface.focal_distance();
            x = x * distance / (distance - projected.z());
            y = y * distance / (distance - projected.z());
        }
        Point2::new(x, y)
    }

    /// `MoveTo(gp_Pnt2d)`: moves the pen in the projection plane.
    pub fn move_to(&mut self, point: Point2) {
        let Some(target) = self.target.as_mut() else {
            return;
        };
        let zoom = target.surface.zoom();
        let zoomed = Point2::new(point.x() * zoom, point.y() * zoom);
        match target.state.mode {
            DrawingMode::Draw => {
                target.state.current_point = zoomed;
                if target.state.draw_bounds {
                    target.state.bounds.include(zoomed.x(), zoomed.y());
                }
            }
            DrawingMode::Pick => target.state.current_point = zoomed,
            DrawingMode::PostScript => write_postscript_move(target.state, zoomed),
        }
    }

    /// `DrawTo(gp_Pnt2d)`: draws a line from the pen to `point` in the projection plane.
    pub fn draw_to(&mut self, point: Point2) {
        let Some(target) = self.target.as_mut() else {
            return;
        };
        if out_of_range(point) {
            return;
        }
        let zoom = target.surface.zoom();
        let end = Point2::new(point.x() * zoom, point.y() * zoom);
        let start = target.state.current_point;
        if out_of_range(end) || out_of_range(start) {
            return;
        }
        target.state.current_point = end;
        match target.state.mode {
            DrawingMode::Draw => draw_window_segment(target, start, end),
            DrawingMode::Pick => pick::pick_segment(&mut target.state.pick, start, end),
            DrawingMode::PostScript => write_postscript_line(target.state, end),
        }
    }

    /// `Draw(gp_Pnt2d, gp_Pnt2d)`.
    pub fn draw(&mut self, start: Point2, end: Point2) {
        if self.target.is_none() {
            return;
        }
        self.move_to(start);
        self.draw_to(end);
    }

    /// `Draw(gp_Pnt, gp_Pnt)`.
    pub fn draw_3d(&mut self, start: Point3, end: Point3) {
        if self.target.is_none() {
            return;
        }
        self.move_to_3d(start);
        self.draw_to_3d(end);
    }

    /// `MoveTo(gp_Pnt)`: moves the pen to a 3D point (perspective applied in front of
    /// the eye only).
    pub fn move_to_3d(&mut self, point: Point3) {
        let Some(target) = self.target.as_mut() else {
            return;
        };
        if target.state.mode == DrawingMode::Pick {
            if target.state.pick.found {
                return;
            }
            target.state.pick.last_p1 = point;
        }
        let projected = point.transformed(target.surface.matrix());
        target.state.perspective_point = projected;
        let (mut x, mut y) = (projected.x(), projected.y());
        if target.surface.is_perspective() {
            let distance = target.surface.focal_distance();
            if projected.z() < distance * PERSPECTIVE_LIMIT {
                x = x * distance / (distance - projected.z());
                y = y * distance / (distance - projected.z());
            }
        }
        self.move_to(Point2::new(x, y));
    }

    /// `DrawTo(gp_Pnt)`: draws to a 3D point, clipping the segment at the eye plane in
    /// perspective.
    pub fn draw_to_3d(&mut self, point: Point3) {
        let Some(target) = self.target.as_mut() else {
            return;
        };
        if target.state.mode == DrawingMode::Pick && target.state.pick.found {
            return;
        }
        let projected = point.transformed(target.surface.matrix());
        let Some(end) = self.perspective_end(projected) else {
            return;
        };
        self.draw_to(end);
        let Some(target) = self.target.as_mut() else {
            return;
        };
        if target.state.mode == DrawingMode::Pick {
            if target.state.pick.found {
                target.state.pick.last_p2 = point;
            } else {
                target.state.pick.last_p1 = point;
            }
        }
    }

    /// The 2D end point of a 3D segment ending at `projected` (view space); in
    /// perspective, moves the pen when the start lies behind the eye, and returns `None`
    /// when the whole segment does.
    fn perspective_end(&mut self, projected: Point3) -> Option<Point2> {
        let target = self.target.as_mut()?;
        if !target.surface.is_perspective() {
            return Some(Point2::new(projected.x(), projected.y()));
        }
        let zoom = target.surface.zoom();
        let distance = target.surface.focal_distance();
        let start = target.state.perspective_point;
        target.state.perspective_point = projected;
        let limit = distance * PERSPECTIVE_LIMIT;
        let (x1, y1, z1) = (start.x(), start.y(), start.z());
        let (mut x2, mut y2, mut z2) = (projected.x(), projected.y(), projected.z());
        if z1 >= limit && z2 >= limit {
            return None;
        } else if z1 >= limit {
            let x = x1 + (x2 - x1) * (limit - z1) / (z2 - z1);
            let y = y1 + (y2 - y1) * (limit - z1) / (z2 - z1);
            let z = limit;
            let (xp, yp) = (x * distance / (distance - z), y * distance / (distance - z));
            self.move_to(Point2::new(xp * zoom, yp * zoom));
        } else if z2 >= limit {
            x2 = x2 + (x1 - x2) * (limit - z2) / (z1 - z2);
            y2 = y2 + (y1 - y2) * (limit - z2) / (z1 - z2);
            z2 = limit;
        }
        Some(Point2::new(
            x2 * distance / (distance - z2),
            y2 * distance / (distance - z2),
        ))
    }
}

/// True when a coordinate exceeds the drawable range (`> 1.e09` or `< -1.e09`).
fn out_of_range(point: Point2) -> bool {
    point.x() > COORDINATE_LIMIT
        || point.x() < -COORDINATE_LIMIT
        || point.y() > COORDINATE_LIMIT
        || point.y() < -COORDINATE_LIMIT
}

/// The `DRAW` branch of `DrawTo`: clips the zoomed segment to the view frame, buffers it
/// in window pixels, flushes a full buffer, and grows the bounds.
fn draw_window_segment(target: &mut DisplayTarget<'_>, start: Point2, end: Point2) {
    let (x0, y0, x1, y1) = target.surface.frame();
    let (mut a, mut b) = (start, end);
    if trim_segment(&mut a, &mut b, [x0 as f64, y0 as f64, x1 as f64, y1 as f64]) {
        let (dx, dy) = (target.surface.dx() as f64, target.surface.dy() as f64);
        let segment = DrawXSegment::new(
            (a.x() + dx) as i32,
            (-a.y() - dy) as i32,
            (b.x() + dx) as i32,
            (-b.y() - dy) as i32,
        );
        target.state.segments.push(segment);
    }
    if target.state.segments.len() == MAXSEGMENT {
        target.state.flush_segments(target.surface);
    }
    if target.state.draw_bounds {
        target.state.bounds.include(end.x(), end.y());
    }
}

/// The PostScript pixel of a zoomed point.
fn postscript_pixel(state: &DisplayState, point: Point2) -> (i32, i32) {
    let ps = &state.postscript;
    let x = ((point.x() - ps.vx as f64) * ps.kx + ps.px as f64) as i32;
    let y = ((point.y() - ps.vy as f64) * ps.ky + ps.py as f64) as i32;
    (x, y)
}

/// `MoveTo` in PostScript mode.
fn write_postscript_move(state: &mut DisplayState, point: Point2) {
    let (x, y) = postscript_pixel(state, point);
    state
        .postscript
        .stream
        .put_str("stroke\nnewpath\n")
        .put_int(x.into())
        .put_str(" ")
        .put_int(y.into())
        .put_str(" m\n");
}

/// `DrawTo` in PostScript mode.
fn write_postscript_line(state: &mut DisplayState, point: Point2) {
    let (x, y) = postscript_pixel(state, point);
    state
        .postscript
        .stream
        .put_int(x.into())
        .put_str(" ")
        .put_int(y.into())
        .put_str(" l\n");
}

/// `SetColor` in PostScript mode: line width and gray of the palette entry.
fn write_postscript_color(state: &mut DisplayState, color: DrawColor) {
    let index = color.id().index();
    let (width, gray) = (
        state.postscript.widths[index],
        state.postscript.grays[index],
    );
    let stream = &mut state.postscript.stream;
    stream
        .put_str("stroke\nnewpath\n")
        .put_int(width.into())
        .put_str(" setlinewidth\n");
    stream.put_real(gray).put_str(" setgray\n");
}

#[cfg(test)]
mod tests;
