//! `Aspect_WindowInputListener`: the receiver of window, keyboard, mouse, touch and
//! 3D-mouse events (the base of `AIS_ViewController`).
//!
//! It is not a transient: the trait [`AspectWindowInputListener`] holds the pure virtuals
//! and the overridable virtuals with their native bodies, over the embedded
//! [`AspectWindowInputListenerBase`] that owns the event timer, the key set, the mouse
//! state, the touch points and the 3D-mouse settings.
use super::{
    ASPECT_VKEY_FLAGS_NONE, ASPECT_VKEY_MOUSE_NONE, AspectScrollDelta, AspectTouch, AspectVKey,
    AspectVKeyFlags, AspectVKeyMouse, AspectVKeySet,
};
use crate::wnt::WntHidSpaceMouse;
use rustycad_tkernel::ncollection::{IndexedDataMap, Vec2, Vec3};
use rustycad_tkernel::osd::Timer;
use rustycad_tkernel::standard::Failure;

/// `(size_t)-1`: the touch id that clears every touch.
pub const ASPECT_TOUCH_ID_ALL: usize = usize::MAX;

/// The state every input listener embeds (the protected members of the native class).
pub struct AspectWindowInputListenerBase {
    /// `myEventTimer`: timestamps the events.
    pub event_timer: Timer,
    /// `myKeys`: the keyboard state.
    pub keys: AspectVKeySet,
    /// `myMousePositionLast`.
    pub mouse_position_last: Vec2<i32>,
    /// `myMousePressed`: the active mouse buttons.
    pub mouse_pressed: AspectVKeyMouse,
    /// `myMouseModifiers`: the modifiers of the last mouse event.
    pub mouse_modifiers: AspectVKeyFlags,
    /// `myTouchPoints`: the active touches by id.
    pub touch_points: IndexedDataMap<usize, AspectTouch>,
    /// `my3dMouseButtonState`: the cached 3D-mouse button state.
    pub mouse3d_button_state: [bool; 32],
    /// `my3dMouseNoRotate`: the ignored rotation axes.
    pub mouse3d_no_rotate: Vec3<bool>,
    /// `my3dMouseToReverse`: the reversed rotation axes.
    pub mouse3d_to_reverse: Vec3<bool>,
    /// `my3dMouseAccelTrans`.
    pub mouse3d_accel_trans: f32,
    /// `my3dMouseAccelRotate`.
    pub mouse3d_accel_rotate: f32,
    /// `my3dMouseIsQuadric`.
    pub mouse3d_is_quadric: bool,
}

impl Default for AspectWindowInputListenerBase {
    /// The protected constructor: a started native timer and the native 3D-mouse defaults.
    fn default() -> Self {
        Self::with_timer(Timer::new(false))
    }
}

impl AspectWindowInputListenerBase {
    /// The protected constructor over an injected timer, which it starts.
    ///
    /// Example: `AspectWindowInputListenerBase::with_timer(Timer::new(false))`.
    pub fn with_timer(mut event_timer: Timer) -> Self {
        event_timer.start();
        Self {
            event_timer,
            keys: AspectVKeySet::default(),
            mouse_position_last: Vec2::new(0, 0),
            mouse_pressed: ASPECT_VKEY_MOUSE_NONE,
            mouse_modifiers: ASPECT_VKEY_FLAGS_NONE,
            touch_points: IndexedDataMap::default(),
            mouse3d_button_state: [false; 32],
            mouse3d_no_rotate: Vec3::new(false, false, false),
            mouse3d_to_reverse: Vec3::new(true, false, false),
            mouse3d_accel_trans: 2.0,
            mouse3d_accel_rotate: 4.0,
            mouse3d_is_quadric: true,
        }
    }
    /// The native body of the pure virtual `KeyDown`; `base.key_down(key, time, 1.0)?`.
    pub fn key_down(&self, key: AspectVKey, time: f64, pressure: f64) -> Result<(), Failure> {
        self.keys.key_down(key, time, pressure)
    }
    /// The native body of the pure virtual `KeyUp`; `base.key_up(key, time)?`.
    pub fn key_up(&self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        self.keys.key_up(key, time)
    }
    /// The native body of the pure virtual `KeyFromAxis`; `base.key_from_axis(a, b, t, p)?`.
    pub fn key_from_axis(
        &self,
        negative: AspectVKey,
        positive: AspectVKey,
        time: f64,
        pressure: f64,
    ) -> Result<(), Failure> {
        self.keys.key_from_axis(negative, positive, time, pressure)
    }
}

/// Mouse and touch callbacks preserve native exception propagation through typed `Failure` results.
/// A window input listener; Martensite HID targets may use `MartensiteSpaceMouseInput` independently.
pub trait AspectWindowInputListener {
    /// The embedded base; `listener.input_base().keys.modifiers()`.
    fn input_base(&self) -> &AspectWindowInputListenerBase;
    /// The embedded base, mutable; `listener.input_base_mut().mouse_pressed = 0`.
    fn input_base_mut(&mut self) -> &mut AspectWindowInputListenerBase;
    /// Handles an expose event; `listener.process_expose()`.
    fn process_expose(&mut self);
    /// Handles a window configuration change; `listener.process_configure(true)`.
    fn process_configure(&mut self, is_resized: bool);
    /// Handles queued user input; `listener.process_input()`.
    fn process_input(&mut self);
    /// Handles focus changes; `listener.process_focus(true)`.
    fn process_focus(&mut self, is_activated: bool);
    /// Handles the window close request; `listener.process_close()`.
    fn process_close(&mut self);
    /// Presses a key (pure virtual with a native body: [`AspectWindowInputListenerBase::key_down`]).
    ///
    /// Example: `listener.key_down(key, 0.0, 1.0)?`.
    fn key_down(&mut self, key: AspectVKey, time: f64, pressure: f64) -> Result<(), Failure>;
    /// Releases a key; `listener.key_up(key, 1.0)?`.
    fn key_up(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure>;
    /// Presses one key of an axis; `listener.key_from_axis(a, b, 1.0, 0.5)?`.
    fn key_from_axis(
        &mut self,
        negative: AspectVKey,
        positive: AspectVKey,
        time: f64,
        pressure: f64,
    ) -> Result<(), Failure>;
    /// Handles a scroll; `listener.update_mouse_scroll(&delta)`.
    fn update_mouse_scroll(&mut self, delta: &AspectScrollDelta) -> bool;
    /// Handles a change of the pressed buttons; `listener.update_mouse_buttons(p, b, m, false)?`.
    fn update_mouse_buttons(
        &mut self,
        point: Vec2<i32>,
        buttons: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        is_emulated: bool,
    ) -> Result<bool, Failure>;
    /// Handles a mouse move; `listener.update_mouse_position(p, b, m, false)?`.
    fn update_mouse_position(
        &mut self,
        point: Vec2<i32>,
        buttons: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        is_emulated: bool,
    ) -> Result<bool, Failure>;
    /// Handles a 3D-mouse report; `listener.update_3d_mouse(&report)?`.
    fn update_3d_mouse(&mut self, event: &WntHidSpaceMouse<'_>) -> Result<bool, Failure>;
    /// Adds a touch, optionally clearing the others first; `listener.add_touch_point(1, p, false)?`.
    fn add_touch_point(
        &mut self,
        id: usize,
        point: Vec2<f64>,
        clear_before: bool,
    ) -> Result<(), Failure> {
        if clear_before {
            self.remove_touch_point(ASPECT_TOUCH_ID_ALL, false)?;
        }
        self.input_base_mut()
            .touch_points
            .add(id, AspectTouch::new(point, false));
        Ok(())
    }
    /// Removes a touch ([`ASPECT_TOUCH_ID_ALL`] removes all); false when the id is unknown.
    /// A single remaining touch restarts from its origin. Example: `listener.remove_touch_point(1, false)?`.
    fn remove_touch_point(
        &mut self,
        id: usize,
        _clear_select_points: bool,
    ) -> Result<bool, Failure> {
        let touches = &mut self.input_base_mut().touch_points;
        if id == ASPECT_TOUCH_ID_ALL {
            touches.clear(false);
        } else if !touches.remove_key(&id) {
            return Ok(false);
        }
        if touches.extent() == 1 {
            let first = touches.change_from_index(1).expect("one touch");
            first.to = first.from;
        }
        Ok(true)
    }
    /// Moves a touch, adding it when unknown; `listener.update_touch_point(1, p)?`.
    fn update_touch_point(&mut self, id: usize, point: Vec2<f64>) -> Result<(), Failure> {
        if let Some(touch) = self.input_base_mut().touch_points.change_seek(&id) {
            touch.to = point;
            return Ok(());
        }
        self.add_touch_point(id, point, false)
    }
    /// Maps a 3D-mouse translation to the slide and walk keys; `listener.update_3d_mouse_translation(&r)?`.
    fn update_3d_mouse_translation(
        &mut self,
        event: &WntHidSpaceMouse<'_>,
    ) -> Result<bool, Failure> {
        super::dispatch_space_mouse_translation(self, event)
    }
    /// Maps a 3D-mouse rotation to the look and roll keys (through the virtual
    /// `key_from_axis`), skipping ignored axes; `listener.update_3d_mouse_rotation(&r)?`.
    fn update_3d_mouse_rotation(&mut self, event: &WntHidSpaceMouse<'_>) -> Result<bool, Failure> {
        super::dispatch_space_mouse_rotation(self, event)
    }
    /// Presses and releases the virtual keys of the 3D-mouse buttons that changed.
    ///
    /// Example: `listener.update_3d_mouse_keys(&report)?`.
    fn update_3d_mouse_keys(&mut self, event: &WntHidSpaceMouse<'_>) -> Result<bool, Failure> {
        super::dispatch_space_mouse_keys(self, event)
    }
}

/// The non-virtual members of `Aspect_WindowInputListener`.
pub trait AspectWindowInputListenerMembers: AspectWindowInputListener {
    /// Seconds since the listener was created; `listener.event_time()`.
    fn event_time(&self) -> f64 {
        self.input_base().event_timer.elapsed_time()
    }
    /// The keyboard state; `listener.keys().modifiers()`.
    fn keys(&self) -> &AspectVKeySet {
        &self.input_base().keys
    }
    /// The keyboard state, mutable (`ChangeKeys`); `listener.change_keys()`.
    fn change_keys(&mut self) -> &mut AspectVKeySet {
        &mut self.input_base_mut().keys
    }
    /// Adds a button to the pressed ones; `listener.press_mouse_button(p, b, m, false)?`.
    fn press_mouse_button(
        &mut self,
        point: Vec2<i32>,
        button: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        is_emulated: bool,
    ) -> Result<bool, Failure> {
        let buttons = self.input_base().mouse_pressed | button;
        self.update_mouse_buttons(point, buttons, modifiers, is_emulated)
    }
    /// Removes a button from the pressed ones; `listener.release_mouse_button(p, b, m, false)?`.
    fn release_mouse_button(
        &mut self,
        point: Vec2<i32>,
        button: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        is_emulated: bool,
    ) -> Result<bool, Failure> {
        let buttons = self.input_base().mouse_pressed & !button;
        self.update_mouse_buttons(point, buttons, modifiers, is_emulated)
    }
    /// The active mouse buttons; `listener.pressed_mouse_buttons()`.
    fn pressed_mouse_buttons(&self) -> AspectVKeyMouse {
        self.input_base().mouse_pressed
    }
    /// The modifiers of the last mouse event; `listener.last_mouse_flags()`.
    fn last_mouse_flags(&self) -> AspectVKeyFlags {
        self.input_base().mouse_modifiers
    }
    /// The last mouse position; `listener.last_mouse_position()`.
    fn last_mouse_position(&self) -> Vec2<i32> {
        self.input_base().mouse_position_last
    }
    /// True when a touch is active; `listener.has_touch_points()`.
    fn has_touch_points(&self) -> bool {
        !self.input_base().touch_points.is_empty()
    }
    /// The active touches by id; `listener.touch_points().extent()`.
    fn touch_points(&self) -> &IndexedDataMap<usize, AspectTouch> {
        &self.input_base().touch_points
    }
    /// The 3D-mouse translation scale; `listener.get_3d_mouse_translation_scale()`.
    fn get_3d_mouse_translation_scale(&self) -> f32 {
        self.input_base().mouse3d_accel_trans
    }
    /// Sets the 3D-mouse translation scale; `listener.set_3d_mouse_translation_scale(1.0)`.
    fn set_3d_mouse_translation_scale(&mut self, scale: f32) {
        self.input_base_mut().mouse3d_accel_trans = scale;
    }
    /// The 3D-mouse rotation scale; `listener.get_3d_mouse_rotation_scale()`.
    fn get_3d_mouse_rotation_scale(&self) -> f32 {
        self.input_base().mouse3d_accel_rotate
    }
    /// Sets the 3D-mouse rotation scale; `listener.set_3d_mouse_rotation_scale(1.0)`.
    fn set_3d_mouse_rotation_scale(&mut self, scale: f32) {
        self.input_base_mut().mouse3d_accel_rotate = scale;
    }
    /// True for quadric (precise) 3D-mouse input; `listener.to_3d_mouse_precise_input()`.
    fn to_3d_mouse_precise_input(&self) -> bool {
        self.input_base().mouse3d_is_quadric
    }
    /// Sets quadric 3D-mouse input; `listener.set_3d_mouse_precise_input(false)`.
    fn set_3d_mouse_precise_input(&mut self, is_quadric: bool) {
        self.input_base_mut().mouse3d_is_quadric = is_quadric;
    }
    /// The ignored rotation axes; `listener.get_3d_mouse_is_no_rotate()`.
    fn get_3d_mouse_is_no_rotate(&self) -> Vec3<bool> {
        self.input_base().mouse3d_no_rotate
    }
    /// The ignored rotation axes, mutable; `listener.change_3d_mouse_is_no_rotate().x = true`.
    fn change_3d_mouse_is_no_rotate(&mut self) -> &mut Vec3<bool> {
        &mut self.input_base_mut().mouse3d_no_rotate
    }
    /// The reversed rotation axes; `listener.get_3d_mouse_to_reverse()`.
    fn get_3d_mouse_to_reverse(&self) -> Vec3<bool> {
        self.input_base().mouse3d_to_reverse
    }
    /// The reversed rotation axes, mutable; `listener.change_3d_mouse_to_reverse().y = true`.
    fn change_3d_mouse_to_reverse(&mut self) -> &mut Vec3<bool> {
        &mut self.input_base_mut().mouse3d_to_reverse
    }
}
impl<T: AspectWindowInputListener + ?Sized> AspectWindowInputListenerMembers for T {}

#[cfg(test)]
mod tests {
    use super::super::AspectVKeyBasic;
    use super::*;

    /// A listener that keeps the native key bodies and records the mouse buttons.
    struct KeyRecordingListener {
        base: AspectWindowInputListenerBase,
        buttons: Vec<AspectVKeyMouse>,
    }
    impl AspectWindowInputListener for KeyRecordingListener {
        fn input_base(&self) -> &AspectWindowInputListenerBase {
            &self.base
        }
        fn input_base_mut(&mut self) -> &mut AspectWindowInputListenerBase {
            &mut self.base
        }
        fn process_expose(&mut self) {}
        fn process_configure(&mut self, _is_resized: bool) {}
        fn process_input(&mut self) {}
        fn process_focus(&mut self, _is_activated: bool) {}
        fn process_close(&mut self) {}
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
        fn update_mouse_scroll(&mut self, _delta: &AspectScrollDelta) -> bool {
            false
        }
        fn update_mouse_buttons(
            &mut self,
            _point: Vec2<i32>,
            buttons: AspectVKeyMouse,
            _modifiers: AspectVKeyFlags,
            _is_emulated: bool,
        ) -> Result<bool, Failure> {
            self.buttons.push(buttons);
            self.base.mouse_pressed = buttons;
            Ok(true)
        }
        fn update_mouse_position(
            &mut self,
            _point: Vec2<i32>,
            _buttons: AspectVKeyMouse,
            _modifiers: AspectVKeyFlags,
            _is_emulated: bool,
        ) -> Result<bool, Failure> {
            Ok(false)
        }
        fn update_3d_mouse(&mut self, event: &WntHidSpaceMouse<'_>) -> Result<bool, Failure> {
            self.update_3d_mouse_keys(event)
        }
    }

    fn listener() -> KeyRecordingListener {
        KeyRecordingListener {
            base: AspectWindowInputListenerBase::default(),
            buttons: Vec::new(),
        }
    }

    #[test]
    fn touches_follow_the_native_rules() {
        let mut listener = listener();
        listener
            .add_touch_point(7, Vec2::new(1.0, 1.0), false)
            .unwrap();
        listener.update_touch_point(7, Vec2::new(2.0, 3.0)).unwrap();
        listener.update_touch_point(8, Vec2::new(5.0, 5.0)).unwrap();
        assert_eq!(listener.touch_points().extent(), 2);
        assert!(listener.remove_touch_point(8, false).unwrap());
        assert!(!listener.remove_touch_point(9, false).unwrap());
        let touch = listener.touch_points().find_from_index(1).unwrap();
        assert_eq!(touch.to, touch.from);
        assert!(
            listener
                .remove_touch_point(ASPECT_TOUCH_ID_ALL, false)
                .unwrap()
        );
        assert!(!listener.has_touch_points());
    }

    #[test]
    fn buttons_and_3d_mouse_keys() {
        let mut listener = listener();
        listener
            .press_mouse_button(Vec2::new(0, 0), 2, 0, false)
            .unwrap();
        listener
            .press_mouse_button(Vec2::new(0, 0), 4, 0, false)
            .unwrap();
        listener
            .release_mouse_button(Vec2::new(0, 0), 2, 0, false)
            .unwrap();
        assert_eq!(listener.buttons, vec![2, 6, 4]);
        let report = [3u8, 2, 0, 0, 0];
        let mouse = WntHidSpaceMouse::new(0xC62B, &report);
        assert!(listener.update_3d_mouse(&mouse).unwrap());
        let fit = AspectVKeyBasic::ViewFitAll as AspectVKey;
        assert!(listener.keys().is_key_down(fit).unwrap());
        assert_eq!(listener.get_3d_mouse_translation_scale(), 2.0);
        assert!(listener.get_3d_mouse_to_reverse().x);
    }
    #[test]
    fn window_listener_space_helpers_use_shared_keyboard_dispatch() {
        let mut listener = listener();
        listener.base.event_timer.stop();
        listener.base.event_timer.reset_to(1.25);
        let listener: &mut dyn AspectWindowInputListener = &mut listener;
        let translation = WntHidSpaceMouse::new(0xC62B, &[1, 94, 1, 0, 0, 0, 0]);
        let rotation = WntHidSpaceMouse::new(0xC62B, &[2, 94, 1, 0, 0, 0, 0]);
        assert!(listener.update_3d_mouse_translation(&translation).unwrap());
        assert!(listener.update_3d_mouse_rotation(&rotation).unwrap());
        assert_space_key(listener, AspectVKeyBasic::NavSlideRight, true);
        assert_space_key(listener, AspectVKeyBasic::NavLookUp, true);
        let press = WntHidSpaceMouse::new(0xC62B, &[3, 2, 0, 0, 0]);
        let release = WntHidSpaceMouse::new(0xC62B, &[3, 0, 0, 0, 0]);
        assert!(listener.update_3d_mouse_keys(&press).unwrap());
        assert!(listener.update_3d_mouse_keys(&release).unwrap());
        assert_space_key(listener, AspectVKeyBasic::ViewFitAll, false);
    }
    fn assert_space_key(
        listener: &dyn AspectWindowInputListener,
        key: AspectVKeyBasic,
        down: bool,
    ) {
        let key = key as AspectVKey;
        assert_eq!(listener.keys().is_key_down(key).unwrap(), down);
    }
}
