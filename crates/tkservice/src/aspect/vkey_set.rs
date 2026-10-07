//! `Aspect_VKeySet`: the thread-safe pressed state of every virtual key.
use super::{
    ASPECT_VKEY_FLAGS_NONE, ASPECT_VKEY_MAX, AspectVKey, AspectVKeyFlags, aspect_vkey_to_modifier,
};
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use std::sync::RwLockReadGuard;

/// `KeyStatus`: the life cycle of one key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyStatus {
    Free,
    Pressed,
    /// Just released: the transient state before `Free`, until `HoldDuration` reads it.
    Released,
}

/// `KeyState`: press and release times, pressure and status of one key.
#[derive(Clone, Copy, Debug, PartialEq)]
struct KeyState {
    time_down: f64,
    time_up: f64,
    pressure: f64,
    status: KeyStatus,
}
impl Default for KeyState {
    fn default() -> Self {
        Self {
            time_down: 0.0,
            time_up: 0.0,
            pressure: 1.0,
            status: KeyStatus::Free,
        }
    }
}

/// The locked fields: one state per key `0..=Aspect_VKey_MAX` and the active modifiers.
#[derive(Debug)]
pub struct AspectVKeySetState {
    keys: Vec<KeyState>,
    modifiers: AspectVKeyFlags,
}
impl AspectVKeySetState {
    /// The active modifier flags; `set.lock().modifiers()`.
    pub fn modifiers(&self) -> AspectVKeyFlags {
        self.modifiers
    }
    /// `myKeys[theKey]`: `Standard_OutOfRange` beyond `Aspect_VKey_MAX`, as the array checks.
    fn key(&self, key: AspectVKey) -> Result<&KeyState, Failure> {
        self.keys
            .get(key as usize)
            .ok_or_else(|| key_out_of_range(key))
    }
    fn key_mut(&mut self, key: AspectVKey) -> Result<&mut KeyState, Failure> {
        self.keys
            .get_mut(key as usize)
            .ok_or_else(|| key_out_of_range(key))
    }
    /// `KeyDown_Unlocked`.
    fn key_down(&mut self, key: AspectVKey, time: f64, pressure: f64) -> Result<(), Failure> {
        let state = self.key_mut(key)?;
        if state.status != KeyStatus::Pressed {
            state.status = KeyStatus::Pressed;
            state.time_down = time;
        }
        state.pressure = pressure;
        self.modifiers |= aspect_vkey_to_modifier(key);
        Ok(())
    }
    /// `KeyUp_Unlocked`.
    fn key_up(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        let state = self.key_mut(key)?;
        if state.status == KeyStatus::Pressed {
            state.status = KeyStatus::Released;
            state.time_up = time;
        }
        let modifier = aspect_vkey_to_modifier(key);
        if modifier != 0 {
            self.modifiers &= !modifier;
        }
        Ok(())
    }
    /// Releases `key` when it is pressed, as `KeyFromAxis` does for the opposite key.
    fn release_if_pressed(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        if self.key(key)?.status == KeyStatus::Pressed {
            self.key_up(key, time)?;
        }
        Ok(())
    }
}

fn key_out_of_range(key: AspectVKey) -> Failure {
    out_of_range(&format!(
        "NCollection_Array1::Value: key {key} outside [0, {ASPECT_VKEY_MAX}]"
    ))
}

/// The state of every virtual key, shared between the input and the rendering threads.
///
/// Example: `let keys = AspectVKeySet::default(); keys.key_down(1, 0.0, 1.0)?;`.
pub struct AspectVKeySet {
    transient: StandardTransient,
    state: SharedState<AspectVKeySetState>,
}
impl Default for AspectVKeySet {
    /// Every key free and no modifier, as `Aspect_VKeySet()`.
    fn default() -> Self {
        let state = AspectVKeySetState {
            keys: vec![KeyState::default(); ASPECT_VKEY_MAX as usize + 1],
            modifiers: ASPECT_VKEY_FLAGS_NONE,
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
}
rustycad_tkernel::impl_transient!(
    AspectVKeySet,
    "Aspect_VKeySet",
    StandardTransient,
    transient
);

impl AspectVKeySet {
    /// The active modifiers; `keys.modifiers()`.
    pub fn modifiers(&self) -> AspectVKeyFlags {
        self.state.read().modifiers
    }
    /// The time of the last press; `keys.down_time(key)?`.
    pub fn down_time(&self, key: AspectVKey) -> Result<f64, Failure> {
        Ok(self.state.read().key(key)?.time_down)
    }
    /// The time of the last release; `keys.time_up(key)?`.
    pub fn time_up(&self, key: AspectVKey) -> Result<f64, Failure> {
        Ok(self.state.read().key(key)?.time_up)
    }
    /// True when the key is neither pressed nor just released; `keys.is_free_key(key)?`.
    pub fn is_free_key(&self, key: AspectVKey) -> Result<bool, Failure> {
        Ok(self.state.read().key(key)?.status == KeyStatus::Free)
    }
    /// True when the key is pressed; `keys.is_key_down(key)?`.
    pub fn is_key_down(&self, key: AspectVKey) -> Result<bool, Failure> {
        Ok(self.state.read().key(key)?.status == KeyStatus::Pressed)
    }
    /// `Mutex()`: holds the shared lock, so several reads see one consistent state.
    ///
    /// Example: `let locked = keys.lock(); locked.modifiers()`.
    pub fn lock(&self) -> RwLockReadGuard<'_, AspectVKeySetState> {
        self.state.read()
    }
    /// Frees every key and clears the modifiers; `keys.reset()`.
    pub fn reset(&self) {
        let mut state = self.state.write();
        state.modifiers = 0;
        state
            .keys
            .iter_mut()
            .for_each(|key| *key = KeyState::default());
    }
    /// Presses a key (default pressure 1.0 natively); `keys.key_down(key, 0.5, 1.0)?`.
    pub fn key_down(&self, key: AspectVKey, time: f64, pressure: f64) -> Result<(), Failure> {
        self.state.write().key_down(key, time, pressure)
    }
    /// Releases a key; `keys.key_up(key, 1.5)?`.
    pub fn key_up(&self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        self.state.write().key_up(key, time)
    }
    /// Presses the key of the pressure sign and releases the other one; a zero pressure
    /// releases both. Example: `keys.key_from_axis(left, right, 0.0, -0.5)?`.
    pub fn key_from_axis(
        &self,
        negative: AspectVKey,
        positive: AspectVKey,
        time: f64,
        pressure: f64,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        if pressure == 0.0 {
            state.release_if_pressed(negative, time)?;
            return state.release_if_pressed(positive, time);
        }
        let down = if pressure > 0.0 { positive } else { negative };
        let up = if pressure < 0.0 { positive } else { negative };
        state.key_down(down, time, pressure.abs())?;
        state.release_if_pressed(up, time)
    }
    /// `HoldDuration(key, time, duration)`: whether the key is held or was just released,
    /// and for how long (0 for a free key). Reading a released key frees it.
    ///
    /// Example: `let (held, duration) = keys.hold_duration(key, 2.0)?;`.
    pub fn hold_duration(&self, key: AspectVKey, time: f64) -> Result<(bool, f64), Failure> {
        let (held, duration, _) = self.hold_duration_pressure(key, time, -1.0)?;
        Ok((held, duration))
    }
    /// `HoldDuration(key, time, duration, pressure)`: as [`Self::hold_duration`], plus the
    /// key pressure; a free key leaves `pressure` (the in-out argument) unchanged.
    ///
    /// Example: `let (held, duration, pressure) = keys.hold_duration_pressure(key, 2.0, -1.0)?;`.
    pub fn hold_duration_pressure(
        &self,
        key: AspectVKey,
        time: f64,
        pressure: f64,
    ) -> Result<(bool, f64, f64), Failure> {
        let mut locked = self.state.write();
        let state = locked.key_mut(key)?;
        Ok(match state.status {
            KeyStatus::Free => (false, 0.0, pressure),
            KeyStatus::Released => {
                state.status = KeyStatus::Free;
                (true, state.time_up - state.time_down, state.pressure)
            }
            KeyStatus::Pressed => (true, time - state.time_down, state.pressure),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspect::{ASPECT_VKEY_FLAGS_SHIFT, AspectVKeyBasic};

    const A: AspectVKey = AspectVKeyBasic::A as AspectVKey;
    const B: AspectVKey = AspectVKeyBasic::B as AspectVKey;

    #[test]
    fn a_key_is_held_until_its_release_is_read() {
        let keys = AspectVKeySet::default();
        keys.key_down(A, 1.0, 0.5).unwrap();
        assert!(keys.is_key_down(A).unwrap());
        assert_eq!(
            keys.hold_duration_pressure(A, 3.0, -1.0).unwrap(),
            (true, 2.0, 0.5)
        );
        keys.key_up(A, 4.0).unwrap();
        assert!(!keys.is_free_key(A).unwrap());
        assert_eq!(keys.hold_duration(A, 9.0).unwrap(), (true, 3.0));
        assert_eq!(
            keys.hold_duration_pressure(A, 9.0, 7.0).unwrap(),
            (false, 0.0, 7.0)
        );
        assert!(keys.is_free_key(A).unwrap());
    }

    #[test]
    fn modifiers_follow_their_keys() {
        let keys = AspectVKeySet::default();
        let shift = AspectVKeyBasic::Shift as AspectVKey;
        keys.key_down(shift, 0.0, 1.0).unwrap();
        assert_eq!(keys.modifiers(), ASPECT_VKEY_FLAGS_SHIFT);
        assert_eq!(keys.lock().modifiers(), ASPECT_VKEY_FLAGS_SHIFT);
        keys.key_up(shift, 1.0).unwrap();
        assert_eq!(keys.modifiers(), 0);
        keys.key_down(shift, 0.0, 1.0).unwrap();
        keys.reset();
        assert_eq!(keys.modifiers(), 0);
        assert!(keys.is_free_key(shift).unwrap());
    }

    #[test]
    fn axis_presses_one_side_and_releases_the_other() {
        let keys = AspectVKeySet::default();
        keys.key_from_axis(A, B, 0.0, 0.7).unwrap();
        assert!(keys.is_key_down(B).unwrap() && !keys.is_key_down(A).unwrap());
        keys.key_from_axis(A, B, 1.0, -0.2).unwrap();
        assert!(keys.is_key_down(A).unwrap() && !keys.is_key_down(B).unwrap());
        assert_eq!(keys.time_up(B).unwrap(), 1.0);
        keys.key_from_axis(A, B, 2.0, 0.0).unwrap();
        assert!(!keys.is_key_down(A).unwrap());
        assert_eq!(keys.down_time(A).unwrap(), 1.0);
    }

    #[test]
    fn keys_beyond_the_maximum_are_out_of_range() {
        let keys = AspectVKeySet::default();
        let failure = keys.key_down(256, 0.0, 1.0).unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_OutOfRange");
        assert!(keys.is_key_down(255).is_ok());
    }
}
