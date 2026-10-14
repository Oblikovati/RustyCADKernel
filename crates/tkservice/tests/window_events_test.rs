//! Wasm_Window and Xw_Window: the event translations into a window input listener.
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::Failure;
#[cfg(target_os = "linux")]
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::{
    ASPECT_VKEY_MOUSE_LEFT_BUTTON, AspectScrollDelta, AspectVKey, AspectVKeyFlags, AspectVKeyMouse,
    AspectWindow, AspectWindowInputListener, AspectWindowInputListenerBase, AspectWindowMembers,
};
#[cfg(target_os = "linux")]
use rustycad_tkservice::aspect::{AspectDisplayConnection, AspectVKeyBasic};
use rustycad_tkservice::wasm::WasmWindow;
use rustycad_tkservice::wnt::WntHidSpaceMouse;
#[cfg(target_os = "linux")]
use rustycad_tkservice::xw::{XwWindow, XwXEvent};

/// A listener that records the window notifications it receives.
#[derive(Default)]
struct RecordingInputListener {
    base: AspectWindowInputListenerBase,
    events: Vec<String>,
    fail_pointer: bool,
}
impl AspectWindowInputListener for RecordingInputListener {
    fn input_base(&self) -> &AspectWindowInputListenerBase {
        &self.base
    }
    fn input_base_mut(&mut self) -> &mut AspectWindowInputListenerBase {
        &mut self.base
    }
    fn process_expose(&mut self) {
        self.events.push("expose".into());
    }
    fn process_configure(&mut self, is_resized: bool) {
        self.events.push(format!("configure {is_resized}"));
    }
    fn process_input(&mut self) {
        self.events.push("input".into());
    }
    fn process_focus(&mut self, is_activated: bool) {
        self.events.push(format!("focus {is_activated}"));
    }
    fn process_close(&mut self) {
        self.events.push("close".into());
    }
    fn key_down(&mut self, key: AspectVKey, time: f64, pressure: f64) -> Result<(), Failure> {
        self.base.key_down(key, time, pressure)
    }
    fn key_up(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        self.base.key_up(key, time)
    }
    fn key_from_axis(
        &mut self,
        negative: AspectVKey,
        positive: AspectVKey,
        time: f64,
        pressure: f64,
    ) -> Result<(), Failure> {
        self.base.key_from_axis(negative, positive, time, pressure)
    }
    fn update_mouse_scroll(&mut self, delta: &AspectScrollDelta) -> bool {
        self.events.push(format!("scroll {}", delta.delta));
        true
    }
    fn update_mouse_buttons(
        &mut self,
        _point: Vec2<i32>,
        buttons: AspectVKeyMouse,
        _modifiers: AspectVKeyFlags,
        _is_emulated: bool,
    ) -> Result<bool, Failure> {
        if self.fail_pointer {
            return Err(Failure::new(
                "pointer buttons failed; expected accepted event",
            ));
        }
        self.events.push(format!("buttons {buttons}"));
        Ok(true)
    }
    fn update_mouse_position(
        &mut self,
        point: Vec2<i32>,
        _buttons: AspectVKeyMouse,
        _modifiers: AspectVKeyFlags,
        _is_emulated: bool,
    ) -> Result<bool, Failure> {
        if self.fail_pointer {
            return Err(Failure::new(
                "pointer motion failed; expected accepted event",
            ));
        }
        self.events.push(format!("move {} {}", point.x, point.y));
        Ok(true)
    }
    fn update_3d_mouse(&mut self, event: &WntHidSpaceMouse<'_>) -> Result<bool, Failure> {
        self.update_3d_mouse_keys(event)
    }
}

#[test]
fn canvas_window_events_without_emscripten() {
    let window = WasmWindow::new(&AsciiString::from("#canvas"), true);
    let mut listener = RecordingInputListener::default();
    assert!(window.process_ui_event(&mut listener, 0, None));
    assert!(window.process_focus_event(&mut listener, 0, None));
    assert!(!window.process_touch_event(&mut listener, 0, None));
    listener
        .add_touch_point(1, Vec2::new(1.0, 2.0), false)
        .unwrap();
    assert!(window.process_touch_event(&mut listener, 0, None));
    assert!(
        !window.process_message(&mut listener, 0)
            && !window.process_key_event(&mut listener, 0, None)
    );
    assert_eq!(listener.events, ["configure true", "focus false"]);
    window.set_virtual(true);
    window.set_size_backing(Vec2::new(640, 480));
    assert_eq!(
        (window.size(), window.position()),
        ((640, 480), (0, 0, 640, 480))
    );
    assert_eq!(
        WasmWindow::mouse_buttons_from_native(1),
        ASPECT_VKEY_MOUSE_LEFT_BUTTON
    );
}

/// A button event of the C `XEvent` layout (64-bit Linux): type, window, x, y, state, button.
#[cfg(target_os = "linux")]
fn button_event(event_type: i32, window: u64, xy: [i32; 2], button: u32) -> XwXEvent {
    let mut bytes = [0u8; XwXEvent::SIZE];
    bytes[0..4].copy_from_slice(&event_type.to_ne_bytes());
    bytes[32..40].copy_from_slice(&window.to_ne_bytes());
    bytes[64..68].copy_from_slice(&xy[0].to_ne_bytes());
    bytes[68..72].copy_from_slice(&xy[1].to_ne_bytes());
    bytes[84..88].copy_from_slice(&button.to_ne_bytes());
    XwXEvent::from_bytes(&bytes)
}

#[cfg(target_os = "linux")]
#[test]
fn x_window_keys_constructors_and_events() {
    assert_eq!(
        XwWindow::virtual_key_from_native(0x61),
        AspectVKeyBasic::A as AspectVKey
    );
    assert_eq!(
        XwWindow::virtual_key_from_native(0xFFCA),
        AspectVKeyBasic::Unknown as AspectVKey
    );
    assert!(XwWindow::create(&Handle::default(), "t", 0, 0, 0, 10).is_err());
    assert!(XwWindow::wrap(&Handle::default(), 0, 0).is_err());
    let event = button_event(4, 9, [3, 4], 1);
    assert_eq!(XwXEvent::from_bytes(&event.to_bytes()).event_type(), 4);
    // The rest needs an X server; without one the connection fails and there is nothing to do.
    let Ok(display) = AspectDisplayConnection::new() else {
        return;
    };
    let display = Handle::new(display);
    let window = XwWindow::create(&display, "test", 10, 20, 64, 48).expect("window");
    assert!(window.x_window() != 0 && window.size() == (64, 48));
    let mut listener = RecordingInputListener::default();
    let mut press = button_event(4, window.x_window(), [3, 4], 1);
    assert!(
        window
            .process_message(&mut listener, &mut press)
            .expect("press")
    );
    let mut wheel = button_event(4, window.x_window(), [3, 4], 5);
    assert!(
        window
            .process_message(&mut listener, &mut wheel)
            .expect("wheel")
    );
    assert_eq!(
        listener.events,
        [
            format!("buttons {ASPECT_VKEY_MOUSE_LEFT_BUTTON}"),
            "input".into(),
            "scroll -1".into(),
            "input".into()
        ]
    );
}

#[cfg(target_os = "linux")]
#[test]
fn x_window_pointer_failures_stop_before_processing_input() {
    let Ok(display) = AspectDisplayConnection::new() else {
        return;
    };
    let window = XwWindow::create(&Handle::new(display), "pointer failure", 0, 0, 16, 16).unwrap();
    let mut listener = RecordingInputListener {
        fail_pointer: true,
        ..Default::default()
    };
    let mut press = button_event(4, window.x_window(), [3, 4], 1);
    assert!(window.process_message(&mut listener, &mut press).is_err());
    let mut release = button_event(5, window.x_window(), [3, 4], 1);
    assert!(window.process_message(&mut listener, &mut release).is_err());
    let mut motion = button_event(6, window.x_window(), [3, 4], 0);
    assert!(window.process_message(&mut listener, &mut motion).is_err());
    assert!(listener.events.is_empty());
}

/// A destruction event keeps its parent and destroyed XID distinct on 64-bit Linux.
#[cfg(target_os = "linux")]
fn destroyed_window_event(parent: u64, destroyed: u64) -> XwXEvent {
    let mut bytes = [0u8; XwXEvent::SIZE];
    bytes[0..4].copy_from_slice(&17_i32.to_ne_bytes());
    bytes[32..40].copy_from_slice(&parent.to_ne_bytes());
    bytes[40..48].copy_from_slice(&destroyed.to_ne_bytes());
    XwXEvent::from_bytes(&bytes)
}

#[cfg(target_os = "linux")]
#[test]
fn delivered_destroy_matches_target_and_notifies_close_once() {
    let Ok(display) = AspectDisplayConnection::new() else { return; };
    let display = Handle::new(display);
    let owner = XwWindow::create(&display, "destroy event", 0, 0, 16, 16).unwrap();
    let window = XwWindow::wrap(&display, owner.native_handle(), 0).unwrap();
    let original = window.native_handle();
    let mut listener = RecordingInputListener::default();
    let mut foreign = destroyed_window_event(original, original + 1);
    assert!(!window.process_message(&mut listener, &mut foreign).unwrap());
    let mut destroyed = destroyed_window_event(original + 1, original);
    assert!(window.process_message(&mut listener, &mut destroyed).unwrap());
    assert!(window.process_message(&mut listener, &mut destroyed).unwrap());
    assert_eq!(listener.events, ["close"]);
    assert_eq!(window.native_window_closure().unwrap().original_handle, original);
    assert_eq!(window.native_handle(), 0);
}
