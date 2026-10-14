//! `Xw_Window`: an Xlib window, created by the window or wrapping an existing one.
use crate::aspect::exceptions::window_definition_error;
use crate::aspect::{
    AspectDisplayConnection, AspectDrawable, AspectFbConfig, AspectTypeOfResize as Resize,
    AspectWindow, AspectWindowBase, AspectWindowMembers as _,
};
use crate::xlib::{XwWindowAttributes, xconst, xlib};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::ffi::CString;

/// The protected fields of an X window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct XwWindowFields {
    /// `myXWindow`: the Xlib window id.
    pub x_window: AspectDrawable,
    /// `myFBConfig`: the GLX framebuffer configuration.
    pub fb_config: AspectFbConfig,
    /// `myXLeft`.
    pub x_left: i32,
    /// `myYTop`.
    pub y_top: i32,
    /// `myXRight`.
    pub x_right: i32,
    /// `myYBottom`.
    pub y_bottom: i32,
    /// `myIsOwnWin`: the window was created here and is destroyed with it.
    pub is_own_win: bool,
}

/// An Xlib window; `XwWindow::create(&display, "Viewer", 0, 0, 640, 480)?`.
pub struct XwWindow {
    transient: StandardTransient,
    base: AspectWindowBase,
    /// The fields, shared by every handle.
    pub fields: SharedState<XwWindowFields>,
    native_closure: SharedState<Option<crate::aspect::AspectNativeWindowClosure>>,
    connection_lease: Option<crate::aspect::AspectDisplayConnectionLease>,
}
rustycad_tkernel::impl_transient!(XwWindow, "Xw_Window", dyn AspectWindow, transient, interfaces: [dyn AspectWindow]);

impl Drop for XwWindow {
    /// The destructor destroys a window created here.
    fn drop(&mut self) {
        let fields = *self.fields.read();
        let display = self.display_address();
        if let (true, Some(api)) = (
            fields.is_own_win && fields.x_window != 0 && display != 0,
            xlib(),
        ) {
            api.destroy_window(display, fields.x_window);
        }
    }
}

/// `Aspect_WindowDefinitionError` of the constructors.
fn definition_error(message: &str) -> Failure {
    window_definition_error(message)
}

impl XwWindow {
    fn with_fields(display: &Handle<AspectDisplayConnection>, fields: XwWindowFields) -> Self {
        let window = Self {
            transient: StandardTransient::default(),
            base: AspectWindowBase::default(),
            fields: SharedState::new(fields),
            native_closure: SharedState::default(),
            connection_lease: None,
        };
        window.base.fields_mut().display = display.clone();
        window
    }
    fn retain_connection(
        &mut self,
        display: &Handle<AspectDisplayConnection>,
    ) -> Result<(), Failure> {
        self.connection_lease = Some(
            AspectDisplayConnection::acquire_lease(
                display,
                crate::aspect::AspectDisplayLeasePurpose::Window,
            )
            .map_err(crate::aspect::AspectDisplayLifecycleFailure::into_legacy_failure)?,
        );
        Ok(())
    }
    /// The Xlib display address of the connection (0 without one).
    pub(crate) fn display_address(&self) -> usize {
        self.connection_lease
            .as_ref()
            .map_or(0, |lease| lease.display_address())
    }
    fn expose_connection_lease(
        &self,
        override_display: &Handle<AspectDisplayConnection>,
    ) -> Result<
        crate::aspect::AspectDisplayConnectionLease,
        crate::aspect::AspectDisplayLifecycleFailure,
    > {
        let display = if override_display.is_null() {
            self.native_display_connection()
        } else {
            override_display.clone()
        };
        AspectDisplayConnection::acquire_lease(
            &display,
            crate::aspect::AspectDisplayLeasePurpose::Window,
        )
    }
    fn with_expose_connection(
        &self,
        display: &Handle<AspectDisplayConnection>,
        send: impl FnOnce(usize),
    ) -> Result<(), crate::aspect::AspectDisplayLifecycleFailure> {
        let lease = self.expose_connection_lease(display)?;
        send(lease.display_address());
        Ok(())
    }
    /// `Xw_Window(display, title, left, top, width, height)`: a new window on the default
    /// visual; a non-positive size, a null display or a failed creation raise
    /// `Aspect_WindowDefinitionError`. Example: `XwWindow::create(&display, "Viewer", 0, 0, 640, 480)?`.
    pub fn create(
        display: &Handle<AspectDisplayConnection>,
        title: &str,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    ) -> Result<Self, Failure> {
        let fields = XwWindowFields {
            x_window: 0,
            fb_config: 0,
            x_left: left,
            y_top: top,
            x_right: left.wrapping_add(width),
            y_bottom: top.wrapping_add(height),
            is_own_win: true,
        };
        let mut window = Self::with_fields(display, fields);
        if width <= 0 || height <= 0 {
            return Err(definition_error(&format!(
                "Xw_Window, Coordinate(s) out of range: {width}x{height}; expected a positive size"
            )));
        }
        let Some(connection) = display.get() else {
            return Err(definition_error(
                "Xw_Window, X Display connection is undefined",
            ));
        };
        window.retain_connection(display)?;
        let disp = connection.get_display_aspect();
        let created = xlib().map_or(0, |api| {
            api.create_window(
                disp,
                connection.get_default_visual_info(),
                [left, top, width, height],
            )
        });
        window.fields.write().fb_config = connection.get_default_fb_config();
        if created == 0 {
            return Err(definition_error("Xw_Window, Unable to create window"));
        }
        window.fields.write().x_window = created as AspectDrawable;
        let title = CString::new(title.replace('\0', "")).unwrap_or_default();
        if let Some(api) = xlib() {
            api.set_standard_properties(disp, created, &title, [left, top, width, height]);
            api.flush(disp);
        }
        Ok(window)
    }
    /// `Xw_Window(display, xWindow, fbConfig)`: wraps an existing window (512 x 512 until
    /// resized); a null window or display, or an unavailable visual raise
    /// `Aspect_WindowDefinitionError`. Example: `XwWindow::wrap(&display, xid, 0)?`.
    pub fn wrap(
        display: &Handle<AspectDisplayConnection>,
        x_window: AspectDrawable,
        fb_config: AspectFbConfig,
    ) -> Result<Self, Failure> {
        let fields = XwWindowFields {
            x_window,
            fb_config,
            x_left: 0,
            y_top: 0,
            x_right: 512,
            y_bottom: 512,
            is_own_win: false,
        };
        let mut window = Self::with_fields(display, fields);
        if x_window == 0 {
            return Err(definition_error("Xw_Window, given invalid X window"));
        }
        let Some(connection) = display.get() else {
            return Err(definition_error(
                "Xw_Window, X Display connection is undefined",
            ));
        };
        window.retain_connection(display)?;
        let disp = connection.get_display_aspect();
        let Some(api) = xlib() else {
            return Err(definition_error("Xw_Window, Visual is unavailable"));
        };
        let attributes = api.window_attributes(disp, x_window);
        if !api.has_visual(disp, attributes.visual_id, api.default_screen(disp)) {
            return Err(definition_error("Xw_Window, Visual is unavailable"));
        }
        window.do_resize();
        Ok(window)
    }
    /// `XWindow`; `window.x_window()`.
    pub fn x_window(&self) -> AspectDrawable {
        self.fields.read().x_window
    }
    /// The attributes of the live window (flushing first), or `None` for a virtual window,
    /// no window or no Xlib.
    pub(crate) fn live_attributes(&self) -> Option<(usize, XwWindowAttributes)> {
        let window = self.fields.read().x_window;
        if self.is_virtual() || window == 0 {
            return None;
        }
        let api = xlib()?;
        let display = self.display_address();
        api.flush(display);
        Some((display, api.window_attributes(display, window)))
    }

    pub(super) fn retire_destroyed_window(&self, destroyed: u64) -> Option<bool> {
        let mut fields = self.fields.write();
        let mut closure = self.native_closure.write();
        if let Some(previous) = *closure {
            return (previous.original_handle == destroyed).then_some(false);
        }
        if destroyed == 0 || fields.x_window != destroyed {
            return None;
        }
        *closure = Some(crate::aspect::AspectNativeWindowClosure {
            original_handle: destroyed,
            cause: crate::aspect::AspectNativeWindowClosureCause::DestroyNotify,
            behavior_version: 1,
        });
        *fields = XwWindowFields::default();
        Some(true)
    }
}

/// The resize mode of the border moves: bit 1 left, 2 right, 4 top, 8 bottom.
fn resize_mode(mask: i32) -> Resize {
    match mask {
        0 => Resize::NoBorder,
        1 => Resize::LeftBorder,
        2 => Resize::RightBorder,
        4 => Resize::TopBorder,
        5 => Resize::LeftAndTopBorder,
        6 => Resize::TopAndRightBorder,
        8 => Resize::BottomBorder,
        9 => Resize::BottomAndLeftBorder,
        10 => Resize::RightAndBottomBorder,
        _ => Resize::Unknown,
    }
}

impl AspectWindow for XwWindow {
    fn window_base(&self) -> &AspectWindowBase {
        &self.base
    }
    fn native_display_connection(&self) -> Handle<AspectDisplayConnection> {
        self.connection_lease
            .as_ref()
            .map_or_else(Handle::default, |lease| lease.retained_connection())
    }
    /// A virtual window is mapped; a live one when viewable or unviewable.
    fn is_mapped(&self) -> bool {
        if self.fields.read().x_window == 0 {
            return false;
        }
        if self.is_virtual() {
            return true;
        }
        self.live_attributes().is_some_and(|(_, a)| {
            a.map_state == xconst::IsUnviewable || a.map_state == xconst::IsViewable
        })
    }
    fn map(&self) {
        let window = self.fields.read().x_window;
        if let (false, Some(api)) = (self.is_virtual() || window == 0, xlib()) {
            api.map_window(self.display_address(), window);
        }
    }
    fn unmap(&self) {
        let window = self.fields.read().x_window;
        if let (false, Some(api)) = (self.is_virtual() || window == 0, xlib()) {
            api.iconify_window(self.display_address(), window);
        }
    }
    /// The borders that moved by more than 2 pixels, then the new position.
    fn do_resize(&self) -> Resize {
        let Some((_, a)) = self
            .live_attributes()
            .filter(|(_, a)| a.map_state != xconst::IsUnmapped)
        else {
            return Resize::Unknown;
        };
        let mut fields = self.fields.write();
        let moved = [
            (a.x - fields.x_left, 1),
            (a.x + a.width - fields.x_right, 2),
            (a.y - fields.y_top, 4),
            (a.y + a.height - fields.y_bottom, 8),
        ];
        let mask = moved
            .iter()
            .filter(|(delta, _)| delta.abs() > 2)
            .fold(0, |m, (_, bit)| m | bit);
        (fields.x_left, fields.x_right, fields.y_top, fields.y_bottom) =
            (a.x, a.x + a.width, a.y, a.y + a.height);
        resize_mode(mask)
    }
    fn do_mapping(&self) -> bool {
        self.native_window_closure().is_none()
    }
    /// The width over the height (of the stored rectangle for a virtual window).
    fn ratio(&self) -> f64 {
        if self.native_window_closure().is_some() {
            return 0.;
        }
        match self.live_attributes() {
            Some((_, a)) => f64::from(a.width) / f64::from(a.height),
            None => {
                let f = self.fields.read();
                f64::from(f.x_right - f.x_left) / f64::from(f.y_bottom - f.y_top)
            }
        }
    }
    /// The rectangle (from the root origin for a live window).
    fn position(&self) -> (i32, i32, i32, i32) {
        let Some((display, a)) = self.live_attributes() else {
            let f = self.fields.read();
            return (f.x_left, f.y_top, f.x_right, f.y_bottom);
        };
        let window = self.fields.read().x_window;
        let (x, y) = xlib().map_or((0, 0), |api| api.translate_origin(display, a.root, window));
        (-x, -y, -x + a.width, -y + a.height)
    }
    fn size(&self) -> (i32, i32) {
        match self.live_attributes() {
            Some((_, a)) => (a.width, a.height),
            None => {
                let f = self.fields.read();
                (f.x_right - f.x_left, f.y_bottom - f.y_top)
            }
        }
    }
    fn native_handle(&self) -> AspectDrawable {
        self.fields.read().x_window
    }
    fn native_window_closure(&self) -> Option<crate::aspect::AspectNativeWindowClosure> {
        *self.native_closure.read()
    }
    fn native_parent_handle(&self) -> AspectDrawable {
        0
    }
    fn native_fb_config(&self) -> AspectFbConfig {
        self.fields.read().fb_config
    }
    fn set_title(&self, title: &AsciiString) {
        let window = self.fields.read().x_window;
        let text = CString::new(
            title
                .as_bytes()
                .split(|b| *b == 0)
                .next()
                .unwrap_or_default(),
        )
        .unwrap_or_default();
        if let (true, Some(api)) = (window != 0, xlib()) {
            api.store_name(self.display_address(), window, &text);
        }
    }
    /// Sends an expose event through `display`, or the pinned binding when null.
    /// The explicit override requires caller-owned same-server XID validity; unavailable
    /// connections send nothing. Example: `window.invalidate_content(&Handle::default())`.
    fn invalidate_content(&self, display: &Handle<AspectDisplayConnection>) {
        let window = self.fields.read().x_window;
        if let (true, Some(api)) = (window != 0, xlib()) {
            // This legacy unit-return API cannot expose the typed admission failure.
            let _ =
                self.with_expose_connection(display, |address| api.send_expose(address, window));
        }
    }
}

#[cfg(test)]
#[path = "window_closure_tests.rs"]
mod window_closure_tests;

#[cfg(all(test, target_os = "linux"))]
#[path = "window_display_binding_tests.rs"]
mod window_display_binding_tests;
