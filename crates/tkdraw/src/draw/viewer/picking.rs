//! Interactive picking of `Draw_Viewer`: `Select` (wait for the pointer) and `Pick`
//! (find the displayed object under a point).
use super::{DrawViewer, MAXVIEW};
use crate::draw::display::{DisplaySurface, DrawingMode, PickState};
use crate::draw::drawable::DrawableHandle;
use crate::draw::window::{BUTTON_PRESS, MOTION_NOTIFY};

/// The result of `Select`: view, pointer position in view coordinates, button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DrawSelection {
    /// The view, -1 when the click was outside every view.
    pub view: i32,
    /// Pointer x relative to the view origin.
    pub x: i32,
    /// Pointer y relative to the view origin (upwards).
    pub y: i32,
    /// The button pressed, 0 for a motion.
    pub button: i32,
}

impl DrawViewer {
    /// `Select`: waits for a click in any view (`wait`), or for a click or motion in
    /// view 0 (`!wait`). Upstream resets the view id to 0 before using it in the
    /// non-waiting case, so the caller's view is ignored; kept. `None` in batch mode
    /// (the C++ leaves the outputs untouched); with no mapped view, prints the reason to
    /// `errors` and returns the zero selection.
    pub fn select(&mut self, wait: bool, errors: &mut String) -> Option<DrawSelection> {
        let system = self.system.clone()?;
        let mut selection = DrawSelection::default();
        let has_mapped_view = self
            .views
            .iter()
            .flatten()
            .any(|view| view.window().is_mapped());
        if !has_mapped_view {
            errors.push_str("No selection is possible with no open views\n");
            return Some(selection);
        }
        self.flush();
        self.set_waiting(wait, selection.view, wait);
        loop {
            let event = system.next_event();
            if event.kind == BUTTON_PRESS {
                let index = self.view_of_window(event.window);
                if wait || selection.view == index {
                    selection = DrawSelection {
                        view: if index < MAXVIEW as i32 { index } else { -1 },
                        ..selection
                    };
                    if selection.view != -1 {
                        (selection.x, selection.y, selection.button) =
                            (event.x, event.y, event.button);
                    }
                    break;
                }
            } else if event.kind == MOTION_NOTIFY && !wait {
                (selection.x, selection.y, selection.button) = (event.x, event.y, 0);
                break;
            }
        }
        self.to_view_coordinates(&mut selection);
        if !wait {
            self.set_waiting(false, selection.view, true);
        }
        Some(selection)
    }

    /// `Wait` on view `id` (not waiting) or on every view (waiting).
    fn set_waiting(&mut self, wait: bool, id: i32, to_wait: bool) {
        if wait {
            self.views
                .iter_mut()
                .flatten()
                .for_each(|view| view.window_mut().wait(to_wait));
        } else if let Some(view) = self.view_mut(id) {
            view.window_mut().wait(to_wait);
        }
    }

    /// The slot of the view whose window is `window`, `MAXVIEW` when none.
    fn view_of_window(&self, window: u64) -> i32 {
        let found = (0..MAXVIEW as i32).find(|id| {
            self.view(*id)
                .is_some_and(|view| view.window().is_window(window))
        });
        found.unwrap_or(MAXVIEW as i32)
    }

    /// Converts window pixels to view coordinates (`X - dx`, `-Y - dy`).
    fn to_view_coordinates(&self, selection: &mut DrawSelection) {
        if selection.view == -1 {
            return;
        }
        if let Some(view) = self.view(selection.view) {
            selection.x -= view.dx();
            selection.y = -selection.y - view.dy();
        }
    }

    /// `Pick`: the 1-based index and the object of the first displayed object after
    /// `first` drawn within `precision` pixels of `(x, y)` in view `id`; `(0, None)`
    /// when none. When the view is the only one of its dimension, objects whose stored
    /// bounds miss the point are skipped without drawing.
    pub fn pick(
        &mut self,
        id: i32,
        x: i32,
        y: i32,
        precision: i32,
        first: usize,
    ) -> (usize, Option<DrawableHandle>) {
        let Some(is_2d) = self.active_view(id).map(|view| view.is_2d()) else {
            return (0, None);
        };
        let same_kind = (0..MAXVIEW as i32)
            .filter(|i| self.view(*i).is_some_and(|v| v.is_2d() == is_2d))
            .count();
        self.state.mode = DrawingMode::Pick;
        self.state.pick = PickState {
            x,
            y,
            precision,
            found: false,
            ..self.state.pick
        };
        let hit = self.first_hit(id, first, same_kind == 1);
        self.state.mode = DrawingMode::Draw;
        self.state.pick.found = false;
        match hit {
            Some(index) => (index + 1, Some(self.drawables[index].clone())),
            None => (0, None),
        }
    }

    /// Draws the candidates in pick mode until one is hit; returns its 0-based index.
    fn first_hit(&mut self, id: i32, first: usize, only: bool) -> Option<usize> {
        let pick = self.state.pick;
        for index in first..self.drawables.len() {
            let drawable = self.drawables[index].clone();
            let [x1, x2, y1, y2] = drawable.bounds();
            let (px, py, prec) = (pick.x as f64, pick.y as f64, pick.precision as f64);
            if only
                && ((px + prec < x1) || (px - prec > x2) || (py + prec < y1) || (py - prec > y2))
            {
                continue;
            }
            self.draw_on_view(id, &drawable);
            if self.state.pick.found {
                return Some(index);
            }
        }
        None
    }
}
