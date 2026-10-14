//! `Aspect_Window`: the abstract window a view renders into.
//!
//! The trait [`AspectWindow`] holds the pure virtuals (implemented by `Xw_Window`,
//! `Aspect_NeutralWindow` or an application window) and the overridable virtuals with
//! their native defaults; the non-virtual members are [`AspectWindowMembers`] over the
//! embedded [`AspectWindowBase`].
use super::{
    AspectBackground, AspectDisplayConnection, AspectDrawable, AspectFbConfig, AspectFillMethod,
    AspectGradientBackground, AspectGradientFillMethod, AspectTypeOfResize,
};
use crate::json_dump::{class_begin, field_dumped, field_integer};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, SharedState, Transient};
use rustycad_tkernel::tcollection::AsciiString;
use std::io;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// The protected fields of `Aspect_Window`.
#[derive(Clone, Debug)]
pub struct AspectWindowFields {
    /// `myDisplay`: the display connection, null when none.
    pub display: Handle<AspectDisplayConnection>,
    /// `MyBackground`.
    pub background: AspectBackground,
    /// `MyGradientBackground`.
    pub gradient_background: AspectGradientBackground,
    /// `MyBackgroundFillMethod`.
    pub background_fill_method: AspectFillMethod,
    /// `MyIsVirtual`. Native quirk: the constructor leaves it indeterminate; here false.
    pub is_virtual: bool,
}

/// The state every window embeds; `AspectWindowBase::default()` is the protected constructor.
#[derive(Debug)]
pub struct AspectWindowBase {
    fields: SharedState<AspectWindowFields>,
}
impl Default for AspectWindowBase {
    fn default() -> Self {
        let fields = AspectWindowFields {
            display: Handle::default(),
            background: AspectBackground::default(),
            gradient_background: AspectGradientBackground::default(),
            background_fill_method: AspectFillMethod::None,
            is_virtual: false,
        };
        Self {
            fields: SharedState::new(fields),
        }
    }
}
impl AspectWindowBase {
    /// Reads the protected fields; `base.fields().is_virtual`.
    pub fn fields(&self) -> RwLockReadGuard<'_, AspectWindowFields> {
        self.fields.read()
    }
    /// Writes the protected fields; `base.fields_mut().display = connection`.
    pub fn fields_mut(&self) -> RwLockWriteGuard<'_, AspectWindowFields> {
        self.fields.write()
    }
}

/// The abstract window; an implementation provides the native window behavior.
pub trait AspectWindow: Transient {
    /// The embedded base; `window.window_base().fields().is_virtual`.
    fn window_base(&self) -> &AspectWindowBase;
    /// The native binding, defaulting to display metadata; `window.native_display_connection()`.
    /// Native owners override this when their resource lifetime is pinned independently.
    fn native_display_connection(&self) -> Handle<AspectDisplayConnection> {
        self.window_base().fields().display.clone()
    }
    /// True when the window is mapped on screen; `window.is_mapped()`.
    fn is_mapped(&self) -> bool;
    /// Maps the window; `window.map()`.
    fn map(&self);
    /// Unmaps the window; `window.unmap()`.
    fn unmap(&self);
    /// Applies a resize and reports which borders moved; `window.do_resize()`.
    fn do_resize(&self) -> AspectTypeOfResize;
    /// Applies a (re)mapping; `window.do_mapping()`.
    fn do_mapping(&self) -> bool;
    /// Width over height; `window.ratio()`.
    fn ratio(&self) -> f64;
    /// The corners `(x1, y1, x2, y2)` in screen space; `window.position()`.
    fn position(&self) -> (i32, i32, i32, i32);
    /// `(width, height)` in pixels; `window.size()`.
    fn size(&self) -> (i32, i32);
    /// The native window id; `window.native_handle()`.
    fn native_handle(&self) -> AspectDrawable;
    /// Delivered native closure, if tracked; `window.native_window_closure()` retains its original ID.
    fn native_window_closure(&self) -> Option<super::AspectNativeWindowClosure> {
        None
    }
    /// The native parent window id; `window.native_parent_handle()`.
    fn native_parent_handle(&self) -> AspectDrawable;
    /// The native frame-buffer configuration; `window.native_fb_config()`.
    fn native_fb_config(&self) -> AspectFbConfig;
    /// Sets the title; the native default does nothing; `window.set_title(&title)`.
    fn set_title(&self, _title: &AsciiString) {}
    /// Requests a redraw of the content; the native default does nothing.
    ///
    /// Example: `window.invalidate_content(&Handle::default())`.
    fn invalidate_content(&self, _display: &Handle<AspectDisplayConnection>) {}
    /// The ratio of backing-store pixels to logical units (1.0 natively by default).
    ///
    /// Example: `window.device_pixel_ratio()`.
    fn device_pixel_ratio(&self) -> f64 {
        1.0
    }
    /// A logical point in backing-store pixels; `window.convert_point_to_backing(p)`.
    fn convert_point_to_backing(&self, point: Vec2<f64>) -> Vec2<f64> {
        point * self.device_pixel_ratio()
    }
    /// A backing-store point in logical units; `window.convert_point_from_backing(p)`.
    fn convert_point_from_backing(&self, point: Vec2<f64>) -> Vec2<f64> {
        point / self.device_pixel_ratio()
    }
}

/// The non-virtual members of `Aspect_Window`.
pub trait AspectWindowMembers: AspectWindow {
    /// True for a virtual (off-screen) window; `window.is_virtual()`.
    fn is_virtual(&self) -> bool {
        self.window_base().fields().is_virtual
    }
    /// Marks the window virtual; `window.set_virtual(true)`.
    fn set_virtual(&self, is_virtual: bool) {
        self.window_base().fields_mut().is_virtual = is_virtual;
    }
    /// The top-left corner; `window.top_left()`.
    fn top_left(&self) -> Vec2<i32> {
        let (x1, y1, _, _) = self.position();
        Vec2::new(x1, y1)
    }
    /// The size as a vector; `window.dimensions()`.
    fn dimensions(&self) -> Vec2<i32> {
        let (width, height) = self.size();
        Vec2::new(width, height)
    }
    /// The display connection; `window.display_connection()`.
    fn display_connection(&self) -> Handle<AspectDisplayConnection> {
        self.window_base().fields().display.clone()
    }
    /// The plain background; `window.background()`.
    fn background(&self) -> AspectBackground {
        self.window_base().fields().background
    }
    /// The image fill method; `window.background_fill_method()`.
    fn background_fill_method(&self) -> AspectFillMethod {
        self.window_base().fields().background_fill_method
    }
    /// The gradient background; `window.gradient_background()`.
    fn gradient_background(&self) -> AspectGradientBackground {
        self.window_base().fields().gradient_background
    }
    /// `SetBackground(const Aspect_Background&)`: takes the color only.
    ///
    /// Example: `window.set_background(&AspectBackground::default())`.
    fn set_background(&self, background: &AspectBackground) {
        self.set_background_color(background.color());
    }
    /// `SetBackground(const Quantity_Color&)`; `window.set_background_color(black)`.
    fn set_background_color(&self, color: Color) {
        self.window_base().fields_mut().background.set_color(color);
    }
    /// `SetBackground(const Aspect_GradientBackground&)`.
    ///
    /// Example: `window.set_gradient_background(&AspectGradientBackground::default())`.
    fn set_gradient_background(&self, background: &AspectGradientBackground) {
        let (first, second) = background.colors();
        self.set_gradient_background_colors(first, second, background.bg_gradient_fill_method());
    }
    /// `SetBackground(color1, color2, method)`; `window.set_gradient_background_colors(a, b, m)`.
    fn set_gradient_background_colors(
        &self,
        first: Color,
        second: Color,
        method: AspectGradientFillMethod,
    ) {
        self.window_base()
            .fields_mut()
            .gradient_background
            .set_colors(first, second, method);
    }
    /// `DumpJson`: class name, both backgrounds, fill method and virtual flag.
    ///
    /// Example: `window.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let fields = self.window_base().fields().clone();
        // OCCT_DUMP_TRANSIENT_CLASS_BEGIN names the static class of the member.
        class_begin(stream, "Aspect_Window")?;
        field_dumped(stream, depth, "&MyBackground", &|s, d| {
            fields.background.dump_json(s, d)
        })?;
        let gradient = fields.gradient_background;
        field_dumped(stream, depth, "&MyGradientBackground", &|s, d| {
            gradient.dump_json(s, d)
        })?;
        field_integer(
            stream,
            "MyBackgroundFillMethod",
            fields.background_fill_method as i32,
        )?;
        field_integer(stream, "MyIsVirtual", i32::from(fields.is_virtual))
    }
}
impl<T: AspectWindow + ?Sized> AspectWindowMembers for T {}

rustycad_tkernel::impl_transient_interface!(
    dyn AspectWindow,
    "Aspect_Window",
    rustycad_tkernel::standard::StandardTransient
);
