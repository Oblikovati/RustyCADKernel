//! View management of `Draw_Viewer`: creation, projection parameters, pan and zoom.
use super::{DrawViewer, MAXVIEW};
use crate::draw::display::DisplaySurface;
use crate::draw::view::DrawView;
use rustycad_tkmath::gp::{Axis1, Direction2, Direction3, Point3, Transform3};

impl DrawViewer {
    /// `MakeView(id, type, X, Y, W, H)`: a new window view, centred, initialized with
    /// `view_type` (removed again when the type is unknown), then painted.
    pub fn make_view(&mut self, id: i32, view_type: &str, x: i32, y: i32, width: i32, height: i32) {
        let Some(system) = self.system.clone() else {
            return;
        };
        if !(0..MAXVIEW as i32).contains(&id) {
            return;
        }
        self.delete_view(id);
        let window = system.create_window("Win", (x, y), (width, height), 0);
        self.install_view(
            id,
            DrawView::new(window),
            view_type,
            (width / 2, -height / 2),
        );
    }

    /// `MakeView(id, type, window)`: a view on an existing native window given by its
    /// hexadecimal X11 id (`%lx`), or on a new 50x50 window titled `window`.
    pub fn make_view_on(&mut self, id: i32, view_type: &str, window: &str) {
        let Some(system) = self.system.clone() else {
            return;
        };
        if !(0..MAXVIEW as i32).contains(&id) {
            return;
        }
        self.delete_view(id);
        let existing = parse_window_id(window);
        let view = DrawView::new(system.create_window(window, (0, 0), (50, 50), existing));
        let centre = (view.window().width() / 2, -view.window().height() / 2);
        self.install_view(id, view, view_type, centre);
    }

    /// Stores `view` in slot `id` with its pan, initializes it, then repaints it.
    fn install_view(&mut self, id: i32, mut view: DrawView, view_type: &str, (dx, dy): (i32, i32)) {
        view.set_dx(dx);
        view.set_dy(dy);
        let initialized = view.init(view_type);
        self.views[id as usize] = Some(view);
        if !initialized {
            self.delete_view(id);
        }
        self.repaint_view(id);
    }

    /// `SetTitle`.
    pub fn set_title(&mut self, id: i32, title: &str) {
        if let Some(view) = self.active_view_mut(id) {
            view.window_mut().set_title(title);
        }
    }

    /// `ResetView`: re-initializes the view with its type and recentres it.
    pub fn reset_view(&mut self, id: i32) {
        let Some(view) = self.active_view_mut(id) else {
            return;
        };
        let view_type = view.view_type().to_string();
        view.init(&view_type);
        self.config_view(id);
    }

    /// `SetZoom`: zooms about the window centre.
    pub fn set_zoom(&mut self, id: i32, zoom: f64) {
        let Some((_, _, width, height)) = self.pos_size(id) else {
            return;
        };
        let Some(view) = self.active_view_mut(id) else {
            return;
        };
        let ratio = zoom / view.zoom();
        view.set_zoom(zoom);
        let (w, h) = (0.5 * width as f64, 0.5 * height as f64);
        let dx = (w - ratio * (w - view.dx() as f64)) as i32;
        let dy = (-h + ratio * (h + view.dy() as f64)) as i32;
        view.set_dx(dx);
        view.set_dy(dy);
    }

    /// `RotateView(id, gp_Dir2d, angle)`: rotates about the screen direction `direction`
    /// through the screen origin, both taken back to model space.
    pub fn rotate_view_2d(&mut self, id: i32, direction: Direction2, angle: f64) {
        let Some(view) = self.active_view(id) else {
            return;
        };
        let mut inverse = view.matrix();
        // A view matrix is a rigid motion, so it is invertible.
        let _ = inverse.invert();
        let point = Point3::new(0.0, 0.0, 0.0).transformed(inverse);
        let Ok(axis) = Direction3::new(direction.x(), direction.y(), 0.0) else {
            return;
        };
        self.rotate_view(id, point, axis.transformed(inverse), angle);
    }

    /// `RotateView(id, gp_Pnt, gp_Dir, angle)`: right-multiplies the view matrix by the
    /// rotation about the axis `(point, direction)`.
    pub fn rotate_view(&mut self, id: i32, point: Point3, direction: Direction3, angle: f64) {
        let Some(view) = self.active_view_mut(id) else {
            return;
        };
        let mut rotation = Transform3::default();
        // A unit direction always gives a valid rotation axis.
        let _ = rotation.set_rotation(Axis1::new(point, direction), angle);
        view.transform(rotation);
    }

    /// `SetFocal`.
    pub fn set_focal(&mut self, id: i32, distance: f64) {
        if let Some(view) = self.active_view_mut(id) {
            view.set_focal_distance(distance);
        }
    }

    /// `GetType`: the view type, empty when there is no view.
    pub fn view_type(&self, id: i32) -> String {
        self.active_view(id)
            .map(|view| view.view_type().to_string())
            .unwrap_or_default()
    }

    /// `Zoom`: 0 in batch mode (`return false`) and for an absent view.
    pub fn zoom(&self, id: i32) -> f64 {
        self.active_view(id).map_or(0.0, DisplaySurface::zoom)
    }

    /// `Focal`: 1 in batch mode, 0 for an absent view.
    pub fn focal(&self, id: i32) -> f64 {
        if self.is_batch() {
            return 1.0;
        }
        self.active_view(id)
            .map_or(0.0, DisplaySurface::focal_distance)
    }

    /// `GetTrsf`: the view matrix, `None` when there is no view (C++ leaves `T` as is).
    pub fn trsf(&self, id: i32) -> Option<Transform3> {
        self.active_view(id).map(DisplaySurface::matrix)
    }

    /// `SetTrsf`.
    pub fn set_trsf(&mut self, id: i32, transform: Transform3) {
        if let Some(view) = self.active_view_mut(id) {
            view.set_matrix(transform);
        }
    }

    /// `Is3D`.
    pub fn is_3d(&self, id: i32) -> bool {
        self.active_view(id).is_some_and(|view| !view.is_2d())
    }

    /// `GetPosSize`: `(x, y, width, height)` of the view window.
    pub fn pos_size(&self, id: i32) -> Option<(i32, i32, i32, i32)> {
        let window = self.active_view(id)?.window();
        let (x, y) = window.position();
        Some((x, y, window.width(), window.height()))
    }

    /// `GetFrame`: `(xmin, ymin, xmax, ymax)` of the visible area in view pixels.
    pub fn frame(&self, id: i32) -> Option<(i32, i32, i32, i32)> {
        Some(self.active_view(id)?.compute_frame())
    }

    /// `PanView`: shifts the pan.
    pub fn pan_view(&mut self, id: i32, dx: i32, dy: i32) {
        if let Some(view) = self.active_view_mut(id) {
            view.set_dx(view.dx() + dx);
            view.set_dy(view.dy() + dy);
        }
    }

    /// `SetPan`.
    pub fn set_pan(&mut self, id: i32, dx: i32, dy: i32) {
        if let Some(view) = self.active_view_mut(id) {
            view.set_dx(dx);
            view.set_dy(dy);
        }
    }

    /// `GetPan`.
    pub fn pan(&self, id: i32) -> Option<(i32, i32)> {
        self.active_view(id).map(|view| (view.dx(), view.dy()))
    }

    /// `HasView`.
    pub fn has_view(&self, id: i32) -> bool {
        self.active_view(id).is_some()
    }

    /// `DisplayView`: shows the window.
    pub fn display_view(&mut self, id: i32) {
        if let Some(view) = self.active_view_mut(id) {
            view.window_mut().display();
        }
    }

    /// `HideView`: does nothing upstream (the body is empty).
    pub fn hide_view(&self, _id: i32) {}

    /// `ClearView`: clears the window.
    pub fn clear_view(&mut self, id: i32) {
        if let Some(view) = self.active_view_mut(id) {
            view.window_mut().clear();
        }
    }

    /// `RemoveView`: deletes the view.
    pub fn remove_view(&mut self, id: i32) {
        self.delete_view(id);
    }

    /// `DeleteView`: deletes the view (its window is destroyed with it).
    pub fn delete_view(&mut self, id: i32) {
        if self.is_batch() {
            return;
        }
        if let Some(slot) = usize::try_from(id)
            .ok()
            .and_then(|index| self.views.get_mut(index))
        {
            *slot = None;
        }
    }

    /// `ConfigView`: centres the origin in the window.
    pub fn config_view(&mut self, id: i32) {
        if let Some(view) = self.active_view_mut(id) {
            let (width, height) = (view.window().width(), view.window().height());
            view.set_dx(width / 2);
            view.set_dy(-height / 2);
        }
    }

    /// `SaveView`: flushes, then saves the window image; reports a missing view on the
    /// error stream text returned in `Err`.
    pub fn save_view(&mut self, id: i32, file_name: &str) -> Result<bool, String> {
        if self.is_batch() {
            return Ok(false);
        }
        self.flush();
        match self.active_view(id) {
            Some(view) => Ok(view.window().save(file_name)),
            None => Err(format!("View {id} doesn't exists!\n")),
        }
    }

    /// The view `id` unless in batch mode.
    pub(super) fn active_view(&self, id: i32) -> Option<&DrawView> {
        if self.is_batch() { None } else { self.view(id) }
    }

    /// The view `id` unless in batch mode, for modification.
    pub(super) fn active_view_mut(&mut self, id: i32) -> Option<&mut DrawView> {
        if self.is_batch() {
            None
        } else {
            self.view_mut(id)
        }
    }
}

/// `sscanf(theWindow, "%lx", &aWindow)`: the leading hexadecimal number, 0 when none.
fn parse_window_id(text: &str) -> u64 {
    let text = text.trim_start();
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    let end = digits
        .find(|c: char| !c.is_ascii_hexdigit())
        .unwrap_or(digits.len());
    u64::from_str_radix(&digits[..end], 16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::parse_window_id;

    #[test]
    fn window_ids_parse_like_sscanf_lx() {
        assert_eq!(parse_window_id("1a2b"), 0x1a2b);
        assert_eq!(parse_window_id("0x10 title"), 16);
        // sscanf reads the hexadecimal digit 'D' of a title, as upstream.
        assert_eq!(parse_window_id("Draw View"), 0xD);
        assert_eq!(parse_window_id("view"), 0);
    }
}
