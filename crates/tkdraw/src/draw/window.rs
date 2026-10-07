//! The window layer DRAW draws through (`Draw_Window`), as a thin interface owned by
//! this project. The C++ class talks to X11, Win32 or Cocoa directly; the Rust viewer
//! only needs the operations below, implemented by a native backend (see
//! `docs/draw-design.md`) or by a recording fake in tests.
use super::display::DrawXSegment;

/// One window of a view (`Draw_Window` instance methods).
pub trait DrawWindow {
    /// `GetPosition`: the top-left corner on the screen.
    fn position(&self) -> (i32, i32);
    /// `SetPosition`.
    fn set_position(&mut self, x: i32, y: i32);
    /// `WidthWin`.
    fn width(&self) -> i32;
    /// `HeightWin`.
    fn height(&self) -> i32;
    /// `SetDimension`.
    fn set_dimension(&mut self, width: i32, height: i32);
    /// `GetTitle`.
    fn title(&self) -> String;
    /// `SetTitle`.
    fn set_title(&mut self, title: &str);
    /// `IsMapped`: the window is shown.
    fn is_mapped(&self) -> bool;
    /// `DisplayWindow`: shows the window.
    fn display(&mut self);
    /// `Hide`.
    fn hide(&mut self);
    /// `Clear`: fills the window with the background.
    fn clear(&mut self);
    /// `SetColor`: the palette index for the following drawing.
    fn set_color(&mut self, color: i32);
    /// `SetMode`: the raster operation (3 = copy, 6 = xor).
    fn set_mode(&mut self, mode: i32);
    /// `DrawString`.
    fn draw_string(&mut self, x: i32, y: i32, text: &str);
    /// `DrawSegments`.
    fn draw_segments(&mut self, segments: &[DrawXSegment]);
    /// `Redraw`: shows the off-screen buffer.
    fn redraw(&mut self);
    /// `Save`: writes a snapshot image; false on failure.
    fn save(&self, file_name: &str) -> bool;
    /// `Wait(bool)`: selects (true) or deselects pointer motion events.
    fn wait(&mut self, to_wait: bool);
    /// `IsEqualWindows`: true when `window` is this window's native identifier.
    fn is_window(&self, window: u64) -> bool;
}

/// A pointer event of the window system (`Draw_Window::Draw_XEvent`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DrawWindowEvent {
    /// 4 for a button press, 6 for a motion, 0 for anything else (X11 event types).
    pub kind: i32,
    /// The native identifier of the window.
    pub window: u64,
    /// The button, 0 for motion.
    pub button: i32,
    /// Pointer x in window pixels.
    pub x: i32,
    /// Pointer y in window pixels.
    pub y: i32,
}

/// The X11 `ButtonPress` event type.
pub const BUTTON_PRESS: i32 = 4;
/// The X11 `MotionNotify` event type.
pub const MOTION_NOTIFY: i32 = 6;

/// The window system connection (`Draw_Window` static members and the display).
pub trait DrawWindowSystem {
    /// Creates a window (`Draw_Window` constructor); `existing` attaches to a native
    /// window instead of creating one (0 for a new window).
    fn create_window(
        &self,
        title: &str,
        position: (i32, i32),
        size: (i32, i32),
        existing: u64,
    ) -> Box<dyn DrawWindow>;
    /// `DefineColor`: allocates palette entry `index` with the named color.
    fn define_color(&self, index: usize, name: &str) -> bool;
    /// `Flush`: sends pending drawing to the screen.
    fn flush(&self);
    /// `GetNextEvent`: waits for the next pointer event.
    fn next_event(&self) -> DrawWindowEvent;
}
