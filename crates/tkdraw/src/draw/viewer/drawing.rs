//! Drawing with `Draw_Viewer`: the displayed list, displays, repaint, clear, fit,
//! picking, pointer selection and PostScript output.
use super::{DrawViewer, MAXVIEW};
use crate::draw::color::{DrawColor, DrawColorKind, MAXCOLOR};
use crate::draw::display::{
    DRAWINFINITE, DisplayState, DisplaySurface, DrawBounds, DrawDisplay, DrawingMode,
};
use crate::draw::drawable::DrawableHandle;
use crate::draw::ostream::DrawOStream;
use crate::draw::view::DrawView;
use rustycad_tkmath::gp::Point3;
use std::rc::Rc;

/// The drawing mode `GXcopy` that `MakeDisplay` selects.
const MODE_COPY: i32 = 3;

/// `MakeDisplay` on an existing view: resets the segment buffer, forces the color
/// switch to white (the current color is set to red first) and selects copy mode.
fn display_on<'a>(id: i32, view: &'a mut DrawView, state: &'a mut DisplayState) -> DrawDisplay<'a> {
    state.view_id = id;
    state.segments.clear();
    state.current_color = DrawColorKind::Rouge;
    let mut display = DrawDisplay::new(view, state);
    display.set_color(DrawColor::new(DrawColorKind::Blanc));
    display.set_mode(MODE_COPY);
    display
}

impl DrawViewer {
    /// `MakeDisplay`: a display on view `id`; the empty display in batch mode or when
    /// there is no such view (where C++ would dereference a null view).
    pub fn make_display(&mut self, id: i32) -> DrawDisplay<'_> {
        if self.is_batch() {
            return DrawDisplay::batch();
        }
        let Some(Some(view)) = usize::try_from(id)
            .ok()
            .and_then(|index| self.views.get_mut(index))
        else {
            return DrawDisplay::batch();
        };
        display_on(id, view, &mut self.state)
    }

    /// The display `d` of a C++ command that keeps drawing after `MakeDisplay`: a
    /// display on view `id` that keeps the current color, mode and segment buffer (a
    /// `Draw_Display` has no state of its own; `MakeDisplay` is what resets them).
    pub fn resume_display(&mut self, id: i32) -> DrawDisplay<'_> {
        if self.is_batch() {
            return DrawDisplay::batch();
        }
        let Some(Some(view)) = usize::try_from(id)
            .ok()
            .and_then(|index| self.views.get_mut(index))
        else {
            return DrawDisplay::batch();
        };
        DrawDisplay::new(view, &mut self.state)
    }

    /// `AddDrawable`: appends `drawable` to the displayed list and marks it visible.
    pub fn add_drawable(&mut self, drawable: &DrawableHandle) {
        if self.is_batch() || drawable.visible() {
            return;
        }
        self.drawables.push(Rc::clone(drawable));
        drawable.set_visible(true);
    }

    /// `RemoveDrawable`: removes `drawable` from the displayed list.
    pub fn remove_drawable(&mut self, drawable: &DrawableHandle) {
        if self.is_batch() || !drawable.visible() {
            return;
        }
        if let Some(index) = self.drawables.iter().position(|d| Rc::ptr_eq(d, drawable)) {
            drawable.set_visible(false);
            self.drawables.remove(index);
        }
    }

    /// `operator<<`: displays `drawable` and draws it in every view.
    pub fn display(&mut self, drawable: &DrawableHandle) {
        if self.is_batch() {
            return;
        }
        self.add_drawable(drawable);
        for id in 0..MAXVIEW as i32 {
            self.draw_on_view(id, drawable);
        }
    }

    /// `DrawOnView`: draws `drawable` in view `id` when their dimensions match, and in
    /// draw mode records the screen bounds it covered.
    pub fn draw_on_view(&mut self, id: i32, drawable: &DrawableHandle) {
        if self.is_batch() {
            return;
        }
        let Some(Some(view)) = usize::try_from(id)
            .ok()
            .and_then(|index| self.views.get_mut(index))
        else {
            return;
        };
        // The display is made (switching the window to white, copy mode) before the
        // dimension test, as upstream; the bounds and frame resets do not depend on it.
        self.state.bounds = DrawBounds::empty();
        view.reset_frame();
        let is_2d = view.is_2d();
        let mut display = display_on(id, view, &mut self.state);
        if drawable.is_3d() == is_2d {
            return;
        }
        drawable.draw_on(&mut display);
        display.flush();
        if self.state.mode == DrawingMode::Draw {
            let bounds = self.state.bounds;
            drawable.set_bounds(bounds.x_min, bounds.x_max, bounds.y_min, bounds.y_max);
        }
    }

    /// `HighlightOnView`: draws `drawable` in `color`.
    pub fn highlight_on_view(&mut self, id: i32, drawable: &DrawableHandle, color: DrawColorKind) {
        if self.is_batch() {
            return;
        }
        self.state.highlight = Some(color);
        self.draw_on_view(id, drawable);
        self.state.highlight = None;
    }

    /// `RepaintView`: clears view `id` and draws every displayed object.
    pub fn repaint_view(&mut self, id: i32) {
        if self.active_view(id).is_none() {
            return;
        }
        self.clear_view(id);
        for drawable in self.drawables.clone() {
            self.draw_on_view(id, &drawable);
        }
    }

    /// `RepaintAll`.
    pub fn repaint_all(&mut self) {
        for id in self.view_ids() {
            self.repaint_view(id);
        }
    }

    /// `Repaint2D`: repaints the 2D views.
    pub fn repaint_2d(&mut self) {
        self.repaint_where(true);
    }

    /// `Repaint3D`: repaints the 3D views.
    pub fn repaint_3d(&mut self) {
        self.repaint_where(false);
    }

    fn repaint_where(&mut self, is_2d: bool) {
        if self.is_batch() {
            return;
        }
        for id in self.view_ids() {
            if self.view(id).is_some_and(|view| view.is_2d() == is_2d) {
                self.repaint_view(id);
            }
        }
    }

    /// `Clear`: hides every displayed object and clears every view.
    pub fn clear(&mut self) {
        if self.is_batch() {
            return;
        }
        for drawable in self.drawables.drain(..) {
            drawable.set_visible(false);
        }
        for id in 0..MAXVIEW as i32 {
            self.clear_view(id);
        }
    }

    /// `Clear2D`: hides the 2D objects and clears the 2D views.
    pub fn clear_2d(&mut self) {
        self.clear_where(false);
    }

    /// `Clear3D`: hides the 3D objects and clears the 3D views.
    pub fn clear_3d(&mut self) {
        self.clear_where(true);
    }

    fn clear_where(&mut self, is_3d: bool) {
        if self.is_batch() {
            return;
        }
        self.drawables.retain(|drawable| {
            let keep = drawable.is_3d() != is_3d;
            if !keep {
                drawable.set_visible(false);
            }
            keep
        });
        for id in self.view_ids() {
            if self.view(id).is_some_and(|view| view.is_2d() == !is_3d) {
                self.clear_view(id);
            }
        }
    }

    /// `LastPick`: the end points and parameter of the last picked segment.
    pub fn last_pick(&self) -> Option<(Point3, Point3, f64)> {
        if self.is_batch() {
            return None;
        }
        let pick = &self.state.pick;
        Some((pick.last_p1, pick.last_p2, pick.last_param))
    }

    /// `PostColor`: the PostScript line width and gray of palette entry `index`.
    pub fn post_color(&mut self, index: i32, width: i32, gray: f64) {
        let Some(index) = usize::try_from(index).ok().filter(|i| *i < MAXCOLOR) else {
            return;
        };
        if self.is_batch() {
            return;
        }
        self.state.postscript.widths[index] = width;
        self.state.postscript.grays[index] = gray;
    }

    /// `PostScriptView`: draws view `id` as PostScript into `output`, mapping the view
    /// rectangle `view` (`[xmin, ymin, xmax, ymax]`) onto the paper rectangle `paper`.
    pub fn postscript_view(
        &mut self,
        id: i32,
        view: [i32; 4],
        paper: [i32; 4],
        output: &mut DrawOStream,
    ) {
        if self.active_view(id).is_none() {
            return;
        }
        let ps = &mut self.state.postscript;
        (ps.vx, ps.vy, ps.px, ps.py) = (view[0], view[1], paper[0], paper[1]);
        ps.kx = (paper[2] - paper[0]) as f64 / (view[2] - view[0]) as f64;
        ps.ky = (paper[3] - paper[1]) as f64 / (view[3] - view[1]) as f64;
        if self.drawables.is_empty() {
            return;
        }
        std::mem::swap(&mut self.state.postscript.stream, output);
        self.state.mode = DrawingMode::PostScript;
        self.draw_postscript(id);
        std::mem::swap(&mut self.state.postscript.stream, output);
        output.put_str("stroke\n");
        self.state.mode = DrawingMode::Draw;
    }

    /// Draws every object of the view's dimension on one display.
    fn draw_postscript(&mut self, id: i32) {
        let drawables = self.drawables.clone();
        let Some(Some(view)) = usize::try_from(id)
            .ok()
            .and_then(|index| self.views.get_mut(index))
        else {
            return;
        };
        let is_2d = view.is_2d();
        let mut display = display_on(id, view, &mut self.state);
        for drawable in drawables
            .iter()
            .filter(|drawable| drawable.is_3d() != is_2d)
        {
            drawable.draw_on(&mut display);
        }
    }

    /// `FitView`: zooms and pans view `id` so that the objects of its dimension fill
    /// the window less `frame` pixels on each side.
    pub fn fit_view(&mut self, id: i32, frame: i32) {
        let Some(is_2d) = self.active_view(id).map(DrawView::is_2d) else {
            return;
        };
        // The count starts at view 1 upstream, so view 0 is never counted.
        let same_kind = (1..MAXVIEW as i32)
            .filter(|i| self.view(*i).is_some_and(|v| v.is_2d() == is_2d))
            .count();
        let Some((_, _, width, height)) = self.pos_size(id) else {
            return;
        };
        if self.drawables.is_empty() {
            return;
        }
        let extent = self.extent_of_kind(id, is_2d, same_kind == 1);
        if let Some(view) = self.view_mut(id) {
            fit_extent(view, extent, (width, height), frame);
        }
    }

    /// The union of the screen bounds of the objects of one dimension (redrawn first
    /// when other views of that dimension exist, since their bounds may be stale).
    fn extent_of_kind(&mut self, id: i32, is_2d: bool, only: bool) -> [f64; 4] {
        let mut extent = [DRAWINFINITE, -DRAWINFINITE, DRAWINFINITE, -DRAWINFINITE];
        for drawable in self.drawables.clone() {
            if drawable.is_3d() == is_2d {
                continue;
            }
            if !only {
                self.draw_on_view(id, &drawable);
            }
            let [u1, u2, v1, v2] = drawable.bounds();
            extent = [
                extent[0].min_c(u1),
                extent[1].max_c(u2),
                extent[2].min_c(v1),
                extent[3].max_c(v2),
            ];
        }
        extent
    }
}

/// The comparisons of `FitView`, which keep the old value on ties and NaN.
trait CppMinMax {
    fn min_c(self, other: f64) -> f64;
    fn max_c(self, other: f64) -> f64;
}

impl CppMinMax for f64 {
    fn min_c(self, other: f64) -> f64 {
        if other < self { other } else { self }
    }
    fn max_c(self, other: f64) -> f64 {
        if other > self { other } else { self }
    }
}

/// Applies the zoom and pan of `FitView` for the extent `[umin, umax, vmin, vmax]`.
fn fit_extent(
    view: &mut DrawView,
    [umin, umax, vmin, vmax]: [f64; 4],
    (width, height): (i32, i32),
    frame: i32,
) {
    let zoom = view.zoom();
    let (umin, vmin, umax, vmax) = (umin / zoom, vmin / zoom, umax / zoom, vmax / zoom);
    let z = if (umax - umin) < 1.0e-6 {
        if (vmax - vmin) < 1.0e-6 {
            return;
        }
        (height - 2 * frame) as f64 / (vmax - vmin)
    } else {
        let z = (width - 2 * frame) as f64 / (umax - umin);
        let z2 = (height - 2 * frame) as f64 / (vmax - vmin);
        if (vmax - vmin) > 1.0e-6 && z2 < z {
            z2
        } else {
            z
        }
    };
    view.set_zoom(z);
    view.set_dx(((width / 2) as f64 - 0.5 * (umin + umax) * z) as i32);
    view.set_dy(((-height / 2) as f64 - 0.5 * (vmin + vmax) * z) as i32);
}
