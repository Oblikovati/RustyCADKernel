//! `Draw_Viewer`: up to `MAXVIEW` views and the list of displayed drawables (`dout`).
//!
//! In batch mode (no window system) every operation does nothing and the queries return
//! the C++ batch values, exactly as each C++ method starts with `if (Draw_Batch) return`.
//! A view index outside `0..MAXVIEW` is treated as an absent view (the C++ code indexes
//! its array without a check).
mod drawing;
mod picking;
mod views;

pub use picking::DrawSelection;

use super::display::DisplayState;
use super::drawable::DrawableHandle;
use super::view::DrawView;
use super::window::DrawWindowSystem;
use std::rc::Rc;

/// `MAXVIEW`: the number of view slots.
pub const MAXVIEW: usize = 30;

/// The DRAW viewer.
pub struct DrawViewer {
    system: Option<Rc<dyn DrawWindowSystem>>,
    views: Vec<Option<DrawView>>,
    drawables: Vec<DrawableHandle>,
    state: DisplayState,
}

impl DrawViewer {
    /// The viewer of batch mode (`Draw_Batch` true): no views, no drawing.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::DrawViewer;
    /// let viewer = DrawViewer::batch();
    /// assert!(viewer.is_batch());
    /// assert!(!viewer.has_view(0));
    /// ```
    pub fn batch() -> Self {
        Self {
            system: None,
            views: Vec::new(),
            drawables: Vec::new(),
            state: DisplayState::default(),
        }
    }

    /// `Draw_Viewer()` with windows: empty view slots, PostScript widths 1 and grays 0.
    pub fn with_window_system(system: Rc<dyn DrawWindowSystem>) -> Self {
        let views = (0..MAXVIEW).map(|_| None).collect();
        Self {
            system: Some(system),
            views,
            drawables: Vec::new(),
            state: DisplayState::default(),
        }
    }

    /// `Draw_Batch`: true without a window system.
    pub fn is_batch(&self) -> bool {
        self.system.is_none()
    }

    /// `DefineColor`: allocates palette entry `index` (true in batch mode).
    pub fn define_color(&self, index: usize, name: &str) -> bool {
        self.system
            .as_ref()
            .is_none_or(|system| system.define_color(index, name))
    }

    /// `Flush`: sends pending drawing to the screen.
    pub fn flush(&self) {
        if let Some(system) = &self.system {
            system.flush();
        }
    }

    /// `GetDrawables`: the displayed drawables, in display order.
    pub fn drawables(&self) -> &[DrawableHandle] {
        &self.drawables
    }

    /// The shared drawing state (read access for tests and PostScript output).
    pub fn display_state(&self) -> &DisplayState {
        &self.state
    }

    /// The view in slot `id`, if any.
    pub fn view(&self, id: i32) -> Option<&DrawView> {
        let index = usize::try_from(id).ok()?;
        self.views.get(index)?.as_ref()
    }

    /// The view in slot `id`, for modification.
    pub fn view_mut(&mut self, id: i32) -> Option<&mut DrawView> {
        let index = usize::try_from(id).ok()?;
        self.views.get_mut(index)?.as_mut()
    }

    /// The ids of the existing views.
    fn view_ids(&self) -> Vec<i32> {
        (0..self.views.len() as i32)
            .filter(|id| self.view(*id).is_some())
            .collect()
    }
}

#[cfg(test)]
mod tests;
