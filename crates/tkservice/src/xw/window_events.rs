//! `Xw_Window::VirtualKeyFromNative` and `Xw_Window::ProcessMessage`: the X key symbols and
//! events translated to the window input listener.
use super::XwWindow;
use crate::aspect::{
    ASPECT_VKEY_FLAGS_ALT, ASPECT_VKEY_FLAGS_CTRL, ASPECT_VKEY_FLAGS_NONE, ASPECT_VKEY_FLAGS_SHIFT,
    ASPECT_VKEY_MOUSE_LEFT_BUTTON, ASPECT_VKEY_MOUSE_MIDDLE_BUTTON, ASPECT_VKEY_MOUSE_NONE,
    ASPECT_VKEY_MOUSE_RIGHT_BUTTON, AspectScrollDelta, AspectVKey, AspectVKeyBasic as K,
    AspectVKeyFlags, AspectVKeyMouse, AspectWindow as _, AspectWindowInputListener,
    AspectWindowInputListenerMembers as _, AspectXAtom,
};
use crate::xlib::{XwXEvent, keysym as ks, xconst as x, xlib};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::Failure;

/// The key symbols with a fixed virtual key (both shifts, controls and alts included).
const KEY_TABLE: [(u32, K); 45] = [
    (ks::XK_space, K::Space),
    (ks::XK_apostrophe, K::Apostrophe),
    (ks::XK_comma, K::Comma),
    (ks::XK_minus, K::Minus),
    (ks::XK_period, K::Period),
    (ks::XK_semicolon, K::Semicolon),
    (ks::XK_equal, K::Equal),
    (ks::XK_bracketleft, K::BracketLeft),
    (ks::XK_backslash, K::Backslash),
    (ks::XK_bracketright, K::BracketRight),
    (ks::XK_BackSpace, K::Backspace),
    (ks::XK_Tab, K::Tab),
    (ks::XK_Return, K::Enter),
    (ks::XK_KP_Enter, K::Enter),
    (ks::XK_Escape, K::Escape),
    (ks::XK_Home, K::Home),
    (ks::XK_Left, K::Left),
    (ks::XK_Up, K::Up),
    (ks::XK_Right, K::Right),
    (ks::XK_Down, K::Down),
    (ks::XK_Prior, K::PageUp),
    (ks::XK_Next, K::PageDown),
    (ks::XK_End, K::End),
    (ks::XK_Menu, K::Menu),
    (ks::XK_Num_Lock, K::Numlock),
    (ks::XK_KP_Multiply, K::NumpadMultiply),
    (ks::XK_KP_Add, K::NumpadAdd),
    (ks::XK_KP_Subtract, K::NumpadSubtract),
    (ks::XK_KP_Divide, K::NumpadDivide),
    (ks::XK_Shift_L, K::Shift),
    (ks::XK_Shift_R, K::Shift),
    (ks::XK_Control_L, K::Control),
    (ks::XK_Control_R, K::Control),
    (ks::XK_Alt_L, K::Alt),
    (ks::XK_Alt_R, K::Alt),
    (ks::XK_Delete, K::Delete),
    (0x1008_FF11, K::VolumeDown),
    (0x1008_FF12, K::VolumeMute),
    (0x1008_FF13, K::VolumeUp),
    (0x1008_FF14, K::MediaPlayPause),
    (0x1008_FF15, K::MediaStop),
    (0x1008_FF16, K::MediaPreviousTrack),
    (0x1008_FF17, K::MediaNextTrack),
    (0x1008_FF18, K::BrowserHome),
    (0x1008_FF26, K::BrowserBack),
];

/// The browser keys after the table (kept apart to keep the table literal short).
const BROWSER_KEYS: [(u32, K); 3] = [
    (0x1008_FF27, K::BrowserForward),
    (0x1008_FF28, K::BrowserStop),
    (0x1008_FF29, K::BrowserRefresh),
];

/// The key of a symbol in a range starting at `first` mapped onto keys from `base`.
fn in_range(key: u64, first: u32, last: u32, base: K) -> Option<AspectVKey> {
    (key >= u64::from(first) && key <= u64::from(last))
        .then(|| (key - u64::from(first)) as AspectVKey + base as AspectVKey)
}

impl XwWindow {
    /// `VirtualKeyFromNative`: digits, letters (both cases), F1..F12 and the named keys;
    /// unknown otherwise; `XwWindow::virtual_key_from_native(0x61)` is `A`.
    pub fn virtual_key_from_native(key: u64) -> AspectVKey {
        let ranged = in_range(key, ks::XK_0, ks::XK_9, K::Digit0)
            .or_else(|| in_range(key, ks::XK_A, ks::XK_Z, K::A))
            .or_else(|| in_range(key, ks::XK_a, ks::XK_z, K::A));
        if let Some(found) = ranged {
            return found;
        }
        if key >= u64::from(ks::XK_F1) && key <= u64::from(ks::XK_F24) {
            return in_range(key, ks::XK_F1, ks::XK_F12, K::F1).unwrap_or(K::Unknown as AspectVKey);
        }
        let named = KEY_TABLE
            .iter()
            .chain(BROWSER_KEYS.iter())
            .find(|(symbol, _)| u64::from(*symbol) == key);
        named.map_or(K::Unknown as AspectVKey, |(_, vkey)| *vkey as AspectVKey)
    }
    /// `ProcessMessage`: dispatches an X event of this window to the listener (close, focus,
    /// expose, configure, keys, buttons, wheel and motion); false for an event it ignores.
    /// The pending expose, configure and motion events of the window are folded into the
    /// last one, as natively. Example: `window.process_message(&mut listener, &mut event)?`.
    pub fn process_message(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event: &mut XwXEvent,
    ) -> Result<bool, Failure> {
        let window = self.fields.read().x_window;
        match event.event_type() {
            x::DestroyNotify => Ok(self.process_destroyed_window(listener, event)),
            _ if self.native_window_closure().is_some() => Ok(false),
            x::ClientMessage => Ok(self.process_client(listener, event, window)),
            x::FocusIn | x::FocusOut => {
                if event.any_window() == window {
                    listener.process_focus(event.event_type() == x::FocusIn);
                }
                Ok(true)
            }
            x::Expose => {
                if event.any_window() == window {
                    listener.process_expose();
                }
                self.fold_pending(event, x::ExposureMask);
                Ok(true)
            }
            x::ConfigureNotify => {
                self.fold_pending(event, x::StructureNotifyMask);
                if event.configure_window() == window {
                    listener.process_configure(true);
                }
                Ok(true)
            }
            x::KeyPress | x::KeyRelease => process_key(listener, event),
            x::ButtonPress | x::ButtonRelease => process_button(listener, event),
            x::MotionNotify => self.process_motion(listener, event, window),
            _ => Ok(false),
        }
    }
    fn process_destroyed_window(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event: &XwXEvent,
    ) -> bool {
        let Some(notify) = self.retire_destroyed_window(event.destroyed_window()) else {
            return false;
        };
        if notify {
            listener.process_close();
        }
        true
    }
    /// The close request of the window manager.
    fn process_client(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event: &XwXEvent,
        window: u64,
    ) -> bool {
        let display = self.base_display();
        let delete = display
            .get()
            .and_then(|d| d.get_atom(AspectXAtom::DeleteWindow).ok());
        if delete == Some(event.client_long0() as u64) && event.any_window() == window {
            listener.process_close();
            return true;
        }
        false
    }
    /// Replaces `event` by the last pending event of the mask for this window.
    fn fold_pending(&self, event: &mut XwXEvent, mask: i64) {
        let Some(api) = xlib() else { return };
        let (display, window) = (self.display_address(), self.fields.read().x_window);
        for _ in 0..api.pending(display).max(0) {
            if !api.check_window_event(display, window, mask, event) {
                break;
            }
        }
    }
    /// A pointer motion of this window with its buttons and modifiers.
    fn process_motion(
        &self,
        listener: &mut dyn AspectWindowInputListener,
        event: &mut XwXEvent,
        window: u64,
    ) -> Result<bool, Failure> {
        if event.any_window() != window {
            return Ok(false);
        }
        self.fold_pending(event, x::ButtonMotionMask | x::PointerMotionMask);
        let (px, py, state) = event.motion_fields();
        let masks = [
            (x::Button1Mask, ASPECT_VKEY_MOUSE_LEFT_BUTTON),
            (x::Button2Mask, ASPECT_VKEY_MOUSE_MIDDLE_BUTTON),
            (x::Button3Mask, ASPECT_VKEY_MOUSE_RIGHT_BUTTON),
        ];
        let buttons = masks
            .iter()
            .filter(|(mask, _)| state & mask != 0)
            .fold(ASPECT_VKEY_MOUSE_NONE, |acc, (_, b)| acc | b);
        let flags = modifier_flags(listener, state)?;
        listener.update_mouse_position(Vec2::new(px, py), buttons, flags, false)?;
        listener.process_input();
        Ok(true)
    }
    /// The display connection of the window.
    fn base_display(
        &self,
    ) -> rustycad_tkernel::standard::Handle<crate::aspect::AspectDisplayConnection> {
        use crate::aspect::AspectWindow as _;
        self.window_base().fields().display.clone()
    }
}

/// The control, shift and alt flags of an event state (alt from the listener's keys).
fn modifier_flags(
    listener: &dyn AspectWindowInputListener,
    state: u32,
) -> Result<AspectVKeyFlags, Failure> {
    let mut flags = ASPECT_VKEY_FLAGS_NONE;
    if state & x::ControlMask != 0 {
        flags |= ASPECT_VKEY_FLAGS_CTRL;
    }
    if state & x::ShiftMask != 0 {
        flags |= ASPECT_VKEY_FLAGS_SHIFT;
    }
    if listener.keys().is_key_down(K::Alt as AspectVKey)? {
        flags |= ASPECT_VKEY_FLAGS_ALT;
    }
    Ok(flags)
}

/// A key press or release of a known key.
fn process_key(
    listener: &mut dyn AspectWindowInputListener,
    event: &mut XwXEvent,
) -> Result<bool, Failure> {
    let symbol = xlib().map_or(0, |api| api.lookup_keysym(event));
    let key = XwWindow::virtual_key_from_native(symbol);
    if key == K::Unknown as AspectVKey {
        return Ok(true);
    }
    let time = listener.event_time();
    if event.event_type() == x::KeyPress {
        listener.key_down(key, time, 1.0)?;
    } else {
        listener.key_up(key, time)?;
    }
    listener.process_input();
    Ok(true)
}

/// The mouse button of an X button number (buttons 4 and 5 are the wheel).
fn button_of(button: u32) -> AspectVKeyMouse {
    match button {
        x::Button1 => ASPECT_VKEY_MOUSE_LEFT_BUTTON,
        x::Button2 => ASPECT_VKEY_MOUSE_MIDDLE_BUTTON,
        x::Button3 => ASPECT_VKEY_MOUSE_RIGHT_BUTTON,
        _ => ASPECT_VKEY_MOUSE_NONE,
    }
}

/// A button press or release, or a wheel step on press.
fn process_button(
    listener: &mut dyn AspectWindowInputListener,
    event: &XwXEvent,
) -> Result<bool, Failure> {
    let (px, py, number, state) = event.button_fields();
    let point = Vec2::new(px, py);
    let flags = modifier_flags(listener, state)?;
    let is_press = event.event_type() == x::ButtonPress;
    if number == x::Button4 || number == x::Button5 {
        if !is_press {
            return Ok(true);
        }
        let delta = if number == x::Button4 { 1.0 } else { -1.0 };
        listener.update_mouse_scroll(&AspectScrollDelta::new(point, delta, flags));
    } else if is_press {
        listener.press_mouse_button(point, button_of(number), flags, false)?;
    } else {
        listener.release_mouse_button(point, button_of(number), flags, false)?;
    }
    listener.process_input();
    Ok(true)
}
