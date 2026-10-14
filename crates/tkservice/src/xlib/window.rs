//! The Xlib window calls of `Xw_Window` and the project-owned X event (visualization-design.md
//! section 2): windows and displays cross as addresses and ids.
#![allow(unsafe_code)]
use super::XlibApi;
use std::ffi::CStr;
use x11_dl::xlib::{
    CWBorderPixel, CWColormap, CWEventMask, CWOverrideRedirect, Display, ExposureMask, InputOutput,
    PPosition, PSize, StructureNotifyMask, Visual, XEvent, XSetWindowAttributes, XSizeHints,
    XVisualInfo, XWindowAttributes,
};

/// The geometry and state of a window (`XWindowAttributes`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct XwWindowAttributes {
    /// `x`.
    pub x: i32,
    /// `y`.
    pub y: i32,
    /// `width`.
    pub width: i32,
    /// `height`.
    pub height: i32,
    /// `map_state` (`IsUnmapped`, `IsUnviewable`, `IsViewable`).
    pub map_state: i32,
    /// `root`: the root window of the screen.
    pub root: u64,
    /// The visual id of the window.
    pub visual_id: u64,
}

/// An X event (`XEvent`), as the C union bytes; `XwXEvent::default()` is a zeroed event.
#[derive(Clone, Copy)]
pub struct XwXEvent {
    pub(crate) raw: XEvent,
}

impl Default for XwXEvent {
    fn default() -> Self {
        // SAFETY: XEvent is a C union of plain integer structs; all zero bits is a valid
        // value of each member.
        Self {
            raw: unsafe { std::mem::zeroed() },
        }
    }
}

impl XwXEvent {
    /// The size of the C `XEvent` union in bytes.
    pub const SIZE: usize = std::mem::size_of::<XEvent>();
    /// An event from the C union bytes (as `XNextEvent` fills them); `XwXEvent::from_bytes(&bytes)`.
    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Self {
        // SAFETY: the union is plain data of exactly SIZE bytes; any bit pattern is valid.
        Self {
            raw: unsafe { std::mem::transmute_copy::<[u8; Self::SIZE], XEvent>(bytes) },
        }
    }
    /// The C union bytes; `event.to_bytes()`.
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        // SAFETY: the union is plain data of exactly SIZE bytes.
        unsafe { std::mem::transmute_copy::<XEvent, [u8; Self::SIZE]>(&self.raw) }
    }
    /// The event `type`; `event.event_type()`.
    pub fn event_type(&self) -> i32 {
        self.raw.get_type()
    }
    /// The window of a client, focus, expose or motion event (`xany.window`).
    pub(crate) fn any_window(&self) -> u64 {
        // SAFETY: every member starts with the XAnyEvent fields.
        unsafe { self.raw.any.window }
    }
    /// `xconfigure.window` (after the `event` field, unlike the other events).
    pub(crate) fn configure_window(&self) -> u64 {
        // SAFETY: plain integer member of the union.
        unsafe { self.raw.configure.window }
    }
    /// The destroyed ID, distinct from the event's receiving parent.
    pub(crate) fn destroyed_window(&self) -> u64 {
        // SAFETY: the caller has identified DestroyNotify before reading this member.
        unsafe { self.raw.destroy_window.window }
    }
    /// `xclient.data.l[0]`.
    pub(crate) fn client_long0(&self) -> i64 {
        // SAFETY: plain integer member of the union.
        unsafe { self.raw.client_message.data.get_long(0) }
    }
    /// `(x, y, button, state)` of a button event.
    pub(crate) fn button_fields(&self) -> (i32, i32, u32, u32) {
        // SAFETY: plain integer members of the union.
        unsafe {
            (
                self.raw.button.x,
                self.raw.button.y,
                self.raw.button.button,
                self.raw.button.state,
            )
        }
    }
    /// `(x, y, state)` of a motion event.
    pub(crate) fn motion_fields(&self) -> (i32, i32, u32) {
        // SAFETY: plain integer members of the union.
        unsafe { (self.raw.motion.x, self.raw.motion.y, self.raw.motion.state) }
    }
}

impl XlibApi {
    /// `DefaultScreen(display)`.
    pub(crate) fn default_screen(&self, display: usize) -> i32 {
        // SAFETY: `display` is a live display of the connection.
        unsafe { (self.lib.XDefaultScreen)(display as *mut Display) }
    }
    /// `XFlush(display)`.
    pub(crate) fn flush(&self, display: usize) {
        // SAFETY: `display` is a live display of the connection.
        unsafe {
            (self.lib.XFlush)(display as *mut Display);
        }
    }
    /// `XCreateWindow` of an exposure and structure listening child of the root window,
    /// on the visual of `visual_info` (an `XVisualInfo` address, 0 for the parent's visual);
    /// the window id, 0 on failure.
    pub(crate) fn create_window(&self, display: usize, visual_info: usize, rect: [i32; 4]) -> u64 {
        let disp = display as *mut Display;
        let info = visual_info as *const XVisualInfo;
        // SAFETY: `display` is live; `visual_info` is null or the connection's visual info;
        // the attributes are a zeroed plain struct filled before the call.
        unsafe {
            let parent = (self.lib.XRootWindow)(disp, (self.lib.XDefaultScreen)(disp));
            let mut attributes: XSetWindowAttributes = std::mem::zeroed();
            attributes.event_mask = ExposureMask | StructureNotifyMask;
            let (depth, visual) = match info.as_ref() {
                Some(info) => {
                    attributes.colormap = (self.lib.XCreateColormap)(disp, parent, info.visual, 0);
                    (info.depth, info.visual)
                }
                None => (0, std::ptr::null_mut::<Visual>()),
            };
            let mask = CWBorderPixel | CWColormap | CWEventMask | CWOverrideRedirect;
            let [x, y, width, height] = rect;
            let window = (self.lib.XCreateWindow)(
                disp,
                parent,
                x,
                y,
                width as u32,
                height as u32,
                0,
                depth,
                InputOutput as u32,
                visual,
                mask,
                &mut attributes,
            );
            #[allow(clippy::unnecessary_cast)]
            let window = window as u64;
            window
        }
    }
    /// `XSetStandardProperties` with the title and the position and size hints.
    pub(crate) fn set_standard_properties(
        &self,
        display: usize,
        window: u64,
        title: &CStr,
        rect: [i32; 4],
    ) {
        // SAFETY: `display` and `window` are live; the hints are a zeroed plain struct.
        unsafe {
            let mut hints: XSizeHints = std::mem::zeroed();
            (hints.x, hints.y, hints.width, hints.height) = (rect[0], rect[1], rect[2], rect[3]);
            hints.flags = PPosition | PSize;
            (self.lib.XSetStandardProperties)(
                display as *mut Display,
                window as _,
                title.as_ptr(),
                title.as_ptr(),
                0,
                std::ptr::null_mut(),
                0,
                &mut hints,
            );
        }
    }
    /// `XGetWindowAttributes`, zero for a failed call.
    pub(crate) fn window_attributes(&self, display: usize, window: u64) -> XwWindowAttributes {
        // SAFETY: `display` is live; the attributes are a zeroed plain struct the call fills.
        unsafe {
            let mut raw: XWindowAttributes = std::mem::zeroed();
            (self.lib.XGetWindowAttributes)(display as *mut Display, window as _, &mut raw);
            let visual_id = raw.visual.as_ref().map_or(0, |v| v.visualid);
            XwWindowAttributes {
                x: raw.x,
                y: raw.y,
                width: raw.width,
                height: raw.height,
                map_state: raw.map_state,
                root: raw.root as u64,
                visual_id,
            }
        }
    }
    /// Whether the screen has the visual `visual_id` (`XGetVisualInfo`, freed again).
    pub(crate) fn has_visual(&self, display: usize, visual_id: u64, screen: i32) -> bool {
        // SAFETY: `display` is live; the template is a zeroed plain struct; the result is
        // freed with XFree.
        unsafe {
            let mut template: XVisualInfo = std::mem::zeroed();
            template.visualid = visual_id as _;
            template.screen = screen;
            let mut count = 0;
            let found = (self.lib.XGetVisualInfo)(
                display as *mut Display,
                0x1 | 0x2,
                &mut template,
                &mut count,
            );
            if found.is_null() {
                return false;
            }
            (self.lib.XFree)(found.cast());
            true
        }
    }
    /// `XDestroyWindow`.
    pub(crate) fn destroy_window(&self, display: usize, window: u64) {
        // SAFETY: `display` is live and `window` is a window the caller created.
        unsafe {
            (self.lib.XDestroyWindow)(display as *mut Display, window as _);
        }
    }
    /// `XMapWindow` then `XFlush`.
    pub(crate) fn map_window(&self, display: usize, window: u64) {
        // SAFETY: `display` and `window` are live.
        unsafe {
            (self.lib.XMapWindow)(display as *mut Display, window as _);
        }
        self.flush(display);
    }
    /// `XIconifyWindow` on the default screen.
    pub(crate) fn iconify_window(&self, display: usize, window: u64) {
        let screen = self.default_screen(display);
        // SAFETY: `display` and `window` are live.
        unsafe {
            (self.lib.XIconifyWindow)(display as *mut Display, window as _, screen);
        }
    }
    /// `XTranslateCoordinates(root, window, 0, 0)`: the root origin in window coordinates.
    pub(crate) fn translate_origin(&self, display: usize, root: u64, window: u64) -> (i32, i32) {
        let (mut x, mut y, mut child) = (0, 0, 0);
        // SAFETY: `display`, `root` and `window` are live; the outputs are local.
        unsafe {
            (self.lib.XTranslateCoordinates)(
                display as *mut Display,
                root as _,
                window as _,
                0,
                0,
                &mut x,
                &mut y,
                &mut child,
            );
        }
        (x, y)
    }
    /// `XStoreName`.
    pub(crate) fn store_name(&self, display: usize, window: u64, title: &CStr) {
        // SAFETY: `display` and `window` are live; `title` is NUL-terminated.
        unsafe {
            (self.lib.XStoreName)(display as *mut Display, window as _, title.as_ptr());
        }
    }
    /// `XSendEvent` of an `Expose` event to the window, then `XFlush`.
    pub(crate) fn send_expose(&self, display: usize, window: u64) {
        let mut event = XwXEvent::default();
        // SAFETY: writing the expose member of a zeroed union; `display` and `window` live.
        unsafe {
            event.raw.expose.type_ = x11_dl::xlib::Expose;
            event.raw.expose.window = window as _;
            (self.lib.XSendEvent)(
                display as *mut Display,
                window as _,
                0,
                ExposureMask,
                &mut event.raw,
            );
        }
        self.flush(display);
    }
    /// `XPending`.
    pub(crate) fn pending(&self, display: usize) -> i32 {
        // SAFETY: `display` is live.
        unsafe { (self.lib.XPending)(display as *mut Display) }
    }
    /// `XCheckWindowEvent`: replaces `event` by the next matching event; false for none.
    pub(crate) fn check_window_event(
        &self,
        display: usize,
        window: u64,
        mask: i64,
        event: &mut XwXEvent,
    ) -> bool {
        // SAFETY: `display` and `window` are live; `event` is a whole XEvent.
        unsafe {
            (self.lib.XCheckWindowEvent)(
                display as *mut Display,
                window as _,
                mask as _,
                &mut event.raw,
            ) != 0
        }
    }
    /// `XLookupKeysym(&event.xkey, 0)`.
    pub(crate) fn lookup_keysym(&self, event: &mut XwXEvent) -> u64 {
        // SAFETY: the key member of a key event; XLookupKeysym only reads it.
        let keysym = unsafe { (self.lib.XLookupKeysym)(&mut event.raw.key, 0) };
        #[allow(clippy::unnecessary_cast)]
        let keysym = keysym as u64;
        keysym
    }
}
