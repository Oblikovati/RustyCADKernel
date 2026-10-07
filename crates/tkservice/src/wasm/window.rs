//! `Wasm_Window` without Emscripten: a virtual-size canvas window.
use crate::aspect::{
    ASPECT_VKEY_MOUSE_LEFT_BUTTON, ASPECT_VKEY_MOUSE_MIDDLE_BUTTON, ASPECT_VKEY_MOUSE_NONE,
    ASPECT_VKEY_MOUSE_RIGHT_BUTTON, AspectDisplayConnection, AspectDrawable, AspectFbConfig,
    AspectTypeOfResize, AspectVKey, AspectVKeyBasic, AspectVKeyMouse, AspectWindow,
    AspectWindowBase, AspectWindowInputListener, AspectWindowInputListenerMembers as _,
    AspectWindowMembers as _,
};
use crate::cast::int_from_f64;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// Declares the uninhabited Emscripten event types (none exists without Emscripten).
macro_rules! emscripten_events {
    ($($(#[$doc:meta])* $name:ident;)+) => {
        $(
            $(#[$doc])*
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            pub enum $name {}
        )+
    };
}

emscripten_events! {
    /// `EmscriptenMouseEvent`.
    EmscriptenMouseEvent;
    /// `EmscriptenWheelEvent`.
    EmscriptenWheelEvent;
    /// `EmscriptenTouchEvent`.
    EmscriptenTouchEvent;
    /// `EmscriptenKeyboardEvent`.
    EmscriptenKeyboardEvent;
    /// `EmscriptenUiEvent`.
    EmscriptenUiEvent;
    /// `EmscriptenFocusEvent`.
    EmscriptenFocusEvent;
}

/// The protected fields of a canvas window.
#[derive(Clone, Debug)]
pub struct WasmWindowFields {
    /// `myCanvasId`.
    pub canvas_id: AsciiString,
    /// `mySize`: the backing size.
    pub size: Vec2<i32>,
    /// `myDevicePixelRatio`.
    pub device_pixel_ratio: f64,
    /// `myToScaleBacking`.
    pub to_scale_backing: bool,
    /// `myIsMapped`.
    pub is_mapped: bool,
}

/// A canvas window; `WasmWindow::new(&"#canvas".into(), true)`.
pub struct WasmWindow {
    transient: StandardTransient,
    base: AspectWindowBase,
    /// The fields, shared by every handle.
    pub fields: SharedState<WasmWindowFields>,
}
rustycad_tkernel::impl_transient!(WasmWindow, "Wasm_Window", dyn AspectWindow, transient, interfaces: [dyn AspectWindow]);

impl WasmWindow {
    /// A mapped window of a canvas, of size 0 and pixel ratio 1; `WasmWindow::new(&id, true)`.
    pub fn new(canvas_id: &AsciiString, to_scale_backing: bool) -> Self {
        let fields = WasmWindowFields {
            canvas_id: canvas_id.clone(),
            size: Vec2::new(0, 0),
            device_pixel_ratio: 1.0,
            to_scale_backing,
            is_mapped: true,
        };
        Self {
            transient: StandardTransient::default(),
            base: AspectWindowBase::default(),
            fields: SharedState::new(fields),
        }
    }
    /// `MouseButtonsFromNative`: bits 0, 1 and 2 are the left, right and middle buttons;
    /// `WasmWindow::mouse_buttons_from_native(0x3)`.
    pub fn mouse_buttons_from_native(buttons: u16) -> AspectVKeyMouse {
        let pairs = [
            (0x1, ASPECT_VKEY_MOUSE_LEFT_BUTTON),
            (0x2, ASPECT_VKEY_MOUSE_RIGHT_BUTTON),
            (0x4, ASPECT_VKEY_MOUSE_MIDDLE_BUTTON),
        ];
        pairs
            .iter()
            .filter(|(bit, _)| buttons & bit != 0)
            .fold(ASPECT_VKEY_MOUSE_NONE, |acc, (_, key)| acc | key)
    }
    /// `VirtualKeyFromNative`: unknown without Emscripten; `WasmWindow::virtual_key_from_native(13)`.
    pub fn virtual_key_from_native(key: i32) -> AspectVKey {
        let _ = key;
        AspectVKeyBasic::Unknown as AspectVKey
    }
    /// `SetSizeLogical`: the logical size scaled by the pixel ratio (truncated);
    /// `window.set_size_logical(Vec2::new(320.5, 200.0))`.
    pub fn set_size_logical(&self, size: Vec2<f64>) {
        let mut fields = self.fields.write();
        let ratio = fields.device_pixel_ratio;
        fields.size = Vec2::new(int_from_f64(size.x * ratio), int_from_f64(size.y * ratio));
    }
    /// `SetSizeBacking`; `window.set_size_backing(Vec2::new(640, 400))`.
    pub fn set_size_backing(&self, size: Vec2<i32>) {
        self.fields.write().size = size;
    }
    /// `CanvasId`; `window.canvas_id()`.
    pub fn canvas_id(&self) -> AsciiString {
        self.fields.read().canvas_id.clone()
    }
    /// `SetDevicePixelRatio`; `window.set_device_pixel_ratio(2.0)`.
    pub fn set_device_pixel_ratio(&self, ratio: f64) {
        self.fields.write().device_pixel_ratio = ratio;
    }
    /// `ProcessMessage`: false without Emscripten; `window.process_message(&mut listener, 0)`.
    pub fn process_message(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event_type: i32,
    ) -> bool {
        let _ = (listener, event_type);
        false
    }
    /// `ProcessMouseEvent`: false without Emscripten.
    pub fn process_mouse_event(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event_type: i32,
        event: Option<&EmscriptenMouseEvent>,
    ) -> bool {
        let _ = (listener, event_type, event);
        false
    }
    /// `ProcessWheelEvent`: false without Emscripten.
    pub fn process_wheel_event(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event_type: i32,
        event: Option<&EmscriptenWheelEvent>,
    ) -> bool {
        let _ = (listener, event_type, event);
        false
    }
    /// `ProcessTouchEvent`: whether the listener has touch points; `window.process_touch_event(&mut l, 0, None)`.
    pub fn process_touch_event(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event_type: i32,
        event: Option<&EmscriptenTouchEvent>,
    ) -> bool {
        let _ = (event_type, event);
        listener.has_touch_points()
    }
    /// `ProcessKeyEvent`: false without Emscripten.
    pub fn process_key_event(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event_type: i32,
        event: Option<&EmscriptenKeyboardEvent>,
    ) -> bool {
        let _ = (listener, event_type, event);
        false
    }
    /// `ProcessUiEvent`: a resize configure of the listener; `window.process_ui_event(&mut l, 0, None)`.
    pub fn process_ui_event(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event_type: i32,
        event: Option<&EmscriptenUiEvent>,
    ) -> bool {
        let _ = (event_type, event);
        listener.process_configure(true);
        true
    }
    /// `ProcessFocusEvent`: a focus loss of the listener; `window.process_focus_event(&mut l, 0, None)`.
    pub fn process_focus_event(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event_type: i32,
        event: Option<&EmscriptenFocusEvent>,
    ) -> bool {
        let _ = (event_type, event);
        listener.process_focus(false);
        true
    }
}

impl AspectWindow for WasmWindow {
    fn window_base(&self) -> &AspectWindowBase {
        &self.base
    }
    fn is_mapped(&self) -> bool {
        self.fields.read().is_mapped
    }
    fn map(&self) {
        self.fields.write().is_mapped = true;
    }
    fn unmap(&self) {
        self.fields.write().is_mapped = false;
    }
    /// Unknown without Emscripten.
    fn do_resize(&self) -> AspectTypeOfResize {
        AspectTypeOfResize::Unknown
    }
    fn do_mapping(&self) -> bool {
        true
    }
    /// The width over the height of the size, 1 for an empty size.
    fn ratio(&self) -> f64 {
        let size = self.fields.read().size;
        if size.x != 0 && size.y != 0 {
            f64::from(size.x) / f64::from(size.y)
        } else {
            1.0
        }
    }
    /// `(0, 0, width, height)` for a virtual window; the native leaves the right and bottom
    /// unset otherwise (0 here).
    fn position(&self) -> (i32, i32, i32, i32) {
        let size = self.fields.read().size;
        if self.is_virtual() {
            (0, 0, size.x, size.y)
        } else {
            (0, 0, 0, 0)
        }
    }
    /// The size of a virtual window; the native leaves the outputs unset otherwise (0 here).
    fn size(&self) -> (i32, i32) {
        let size = self.fields.read().size;
        if self.is_virtual() {
            (size.x, size.y)
        } else {
            (0, 0)
        }
    }
    fn native_handle(&self) -> AspectDrawable {
        0
    }
    fn native_parent_handle(&self) -> AspectDrawable {
        0
    }
    fn native_fb_config(&self) -> AspectFbConfig {
        AspectFbConfig::default()
    }
    fn device_pixel_ratio(&self) -> f64 {
        self.fields.read().device_pixel_ratio
    }
    /// Nothing without Emscripten.
    fn invalidate_content(&self, display: &Handle<AspectDisplayConnection>) {
        let _ = display;
    }
}
