//! `WNT_HIDSpaceMouse`: the decoder of 3Dconnexion Space Mouse raw HID chunks.
//!
//! The class is platform neutral (the Linux library builds it too): it only reads the bytes
//! of one HID report. The native class keeps a pointer and a size; the port borrows the
//! byte slice, whose length is the size.
use crate::aspect::{AspectVKey, AspectVKeyBasic};
use rustycad_tkernel::ncollection::Vec3;
use std::cell::Cell;

/// `VENDOR_ID_LOGITECH`.
pub const WNT_HID_VENDOR_ID_LOGITECH: u32 = 0x46D;
/// `VENDOR_ID_3DCONNEXION`.
pub const WNT_HID_VENDOR_ID_3DCONNEXION: u32 = 0x256F;

/// The data chunk types (`SpaceRawInput_*`).
const RAW_TRANSLATION: u8 = 0x01;
const RAW_ROTATION: u8 = 0x02;
const RAW_KEY_STATE: u8 = 0x03;
/// The raw value range of the tested devices, [-350; 350].
const RAW_RANGE_350: i16 = 350;

/// The keys of the known models (`SpaceVKey`), in native order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpaceVKey {
    Invalid,
    Menu,
    Fit,
    Top,
    Left,
    Right,
    Front,
    Bottom,
    Back,
    RollCw,
    RollCcw,
    Iso1,
    Iso2,
    K1,
    K2,
    K3,
    K4,
    K5,
    K6,
    K7,
    K8,
    K9,
    K10,
    Esc,
    Alt,
    Shift,
    Ctrl,
    Rotate,
    PanZoom,
    Dominant,
    Plus,
    Minus,
}
use SpaceVKey as K;

const PILOT_KEYS: [SpaceVKey; 20] = [
    K::K1,
    K::K2,
    K::K3,
    K::K4,
    K::K5,
    K::K6,
    K::Top,
    K::Left,
    K::Right,
    K::Front,
    K::Esc,
    K::Alt,
    K::Shift,
    K::Ctrl,
    K::Fit,
    K::Menu,
    K::Plus,
    K::Minus,
    K::Dominant,
    K::Rotate,
];
const EXPLORER_KEYS: [SpaceVKey; 15] = [
    K::K1,
    K::K2,
    K::Top,
    K::Left,
    K::Right,
    K::Front,
    K::Esc,
    K::Alt,
    K::Shift,
    K::Ctrl,
    K::Fit,
    K::Menu,
    K::Plus,
    K::Minus,
    K::Rotate,
];
/// Shared by the latest 3Dconnexion hardware.
const SPACEMOUSEPRO_KEYS: [SpaceVKey; 31] = [
    K::Menu,
    K::Fit,
    K::Top,
    K::Left,
    K::Right,
    K::Front,
    K::Bottom,
    K::Back,
    K::RollCw,
    K::RollCcw,
    K::Iso1,
    K::Iso2,
    K::K1,
    K::K2,
    K::K3,
    K::K4,
    K::K5,
    K::K6,
    K::K7,
    K::K8,
    K::K9,
    K::K10,
    K::Esc,
    K::Alt,
    K::Shift,
    K::Ctrl,
    K::Rotate,
    K::PanZoom,
    K::Dominant,
    K::Plus,
    K::Minus,
];
const PID_SPACE_PILOT: u64 = 0xC625;
const PID_SPACE_EXPLORER: u64 = 0xC627;
/// The products with the SpaceMouse Pro key layout.
const PRO_LAYOUT_PIDS: [u64; 10] = [
    0xC626, 0xC628, 0xC629, 0xC62B, 0xC62E, 0xC62F, 0xC631, 0xC632, 0xC633, 0xC635,
];

/// `hidToSpaceKey`: the model key of a key-state bit.
fn hid_to_space_key(product_id: u64, key_bit: u16) -> SpaceVKey {
    let table: &[SpaceVKey] = match product_id {
        PID_SPACE_PILOT => &PILOT_KEYS,
        PID_SPACE_EXPLORER => &EXPLORER_KEYS,
        id if PRO_LAYOUT_PIDS.contains(&id) => &SPACEMOUSEPRO_KEYS,
        _ => return K::Invalid,
    };
    table
        .get(usize::from(key_bit))
        .copied()
        .unwrap_or(K::Invalid)
}

/// The virtual key of a model key, `Aspect_VKey_UNKNOWN` for keys without one.
fn space_key_to_vkey(key: SpaceVKey) -> AspectVKeyBasic {
    use AspectVKeyBasic as V;
    match key {
        K::K1 => V::Digit1,
        K::K2 => V::Digit2,
        K::K3 => V::Digit3,
        K::K4 => V::Digit4,
        K::K5 => V::Digit5,
        K::K6 => V::Digit6,
        K::K7 => V::Digit7,
        K::K8 => V::Digit8,
        K::K9 => V::Digit9,
        // Native quirk kept: (SpaceVKey_10 - SpaceVKey_1) + Aspect_VKey_1 is Aspect_VKey_F1.
        K::K10 => V::F1,
        K::Esc => V::Escape,
        K::Shift => V::Shift,
        K::Alt => V::Alt,
        K::Ctrl => V::Control,
        K::Top => V::ViewTop,
        K::Bottom => V::ViewBottom,
        K::Left => V::ViewLeft,
        K::Right => V::ViewRight,
        K::Front => V::ViewFront,
        K::Back => V::ViewBack,
        K::Iso1 => V::ViewAxoLeftProj,
        K::Iso2 => V::ViewAxoRightProj,
        K::Fit => V::ViewFitAll,
        K::RollCw => V::ViewRoll90CW,
        K::RollCcw => V::ViewRoll90CCW,
        K::Rotate => V::ViewSwitchRotate,
        _ => V::Unknown,
    }
}

/// One Space Mouse HID report; `WntHidSpaceMouse::new(0xC62B, &report)`.
#[derive(Debug)]
pub struct WntHidSpaceMouse<'a> {
    data: &'a [u8],
    product_id: u64,
    /// `myValueRange`: mutable natively, widened by the first out-of-range value read.
    value_range: Cell<i16>,
}

impl<'a> WntHidSpaceMouse<'a> {
    /// True when the product id is a known model; `WntHidSpaceMouse::is_known_product(0xC626)`.
    pub fn is_known_product(product_id: u64) -> bool {
        matches!(product_id, PID_SPACE_PILOT | PID_SPACE_EXPLORER)
            || PRO_LAYOUT_PIDS.contains(&product_id)
    }
    /// Wraps one report (`theData`, `theSize` is the slice length); `WntHidSpaceMouse::new(id, &bytes)`.
    pub fn new(product_id: u64, data: &'a [u8]) -> Self {
        Self {
            data,
            product_id,
            value_range: Cell::new(RAW_RANGE_350),
        }
    }
    /// The raw value range; `mouse.raw_value_range()`.
    pub fn raw_value_range(&self) -> i16 {
        self.value_range.get()
    }
    /// Widens (never narrows) the raw value range; `mouse.set_raw_value_range(1000)`.
    pub fn set_raw_value_range(&self, range: i16) {
        self.value_range.set(range.max(self.value_range.get()));
    }
    fn chunk_type(&self) -> Option<u8> {
        self.data.first().copied()
    }
    /// True for a translation chunk (7 or 13 bytes); `mouse.is_translation()`.
    pub fn is_translation(&self) -> bool {
        self.chunk_type() == Some(RAW_TRANSLATION) && matches!(self.data.len(), 7 | 13)
    }
    /// True for a rotation chunk (or the rotation half of a 13-byte translation chunk).
    ///
    /// Example: `mouse.is_rotation()`.
    pub fn is_rotation(&self) -> bool {
        (self.chunk_type() == Some(RAW_ROTATION) && self.data.len() == 7)
            || (self.chunk_type() == Some(RAW_TRANSLATION) && self.data.len() == 13)
    }
    /// True for a key-state chunk; `mouse.is_key_state()`.
    pub fn is_key_state(&self) -> bool {
        self.chunk_type() == Some(RAW_KEY_STATE)
    }
    /// The pressed-key bits (native-endian bytes 1..5); 0 when the chunk is shorter, where
    /// the native code reads past the chunk. Example: `mouse.key_state()`.
    pub fn key_state(&self) -> u32 {
        self.data.get(1..5).map_or(0, |bytes| {
            u32::from_ne_bytes(bytes.try_into().expect("4 bytes"))
        })
    }
    /// The translation in [-1, 1] per axis and whether it is idle; zero for another chunk.
    ///
    /// Example: `let (vector, is_idle) = mouse.translation(true);`.
    pub fn translation(&self, is_quadric: bool) -> (Vec3<f64>, bool) {
        if !self.is_translation() {
            return (Vec3::new(0.0, 0.0, 0.0), true);
        }
        self.decode_raw_vec3(&self.data[1..7], true, is_quadric)
    }
    /// The rotation (tilt, roll, spin) in [-1, 1] and whether it is idle.
    ///
    /// Example: `let (vector, is_idle) = mouse.rotation(true);`.
    pub fn rotation(&self, is_quadric: bool) -> (Vec3<f64>, bool) {
        match (self.chunk_type(), self.data.len()) {
            (Some(RAW_ROTATION), 7) => self.decode_raw_vec3(&self.data[1..7], false, is_quadric),
            (Some(RAW_TRANSLATION), 13) => {
                self.decode_raw_vec3(&self.data[7..13], false, is_quadric)
            }
            _ => (Vec3::new(0.0, 0.0, 0.0), true),
        }
    }
    /// `fromRawVec3`: three native-endian int16 values, dead zones applied to translations,
    /// normalized by the raw range (squared with its sign when quadric).
    fn decode_raw_vec3(
        &self,
        bytes: &[u8],
        is_translation: bool,
        is_quadric: bool,
    ) -> (Vec3<f64>, bool) {
        let raw: [i16; 3] =
            std::array::from_fn(|i| i16::from_ne_bytes([bytes[2 * i], bytes[2 * i + 1]]));
        let (mut values, is_idle) = if is_translation {
            translation_dead_zone(raw)
        } else {
            (raw.map(f64::from), raw.iter().all(|value| *value == 0))
        };
        self.widen_range(raw);
        let range = f64::from(self.value_range.get());
        if !is_quadric {
            return (Vec3::new(values[0], values[1], values[2]) / range, is_idle);
        }
        for (value, sign) in values.iter_mut().zip(raw) {
            *value = if sign > 0 {
                *value * *value
            } else {
                -*value * *value
            };
        }
        (
            Vec3::new(values[0], values[1], values[2]) / (range * range),
            is_idle,
        )
    }
    /// Switches to the int16 range once a value leaves the current one.
    fn widen_range(&self, raw: [i16; 3]) {
        let range = i32::from(self.value_range.get());
        if raw
            .iter()
            .any(|value| i32::from(*value) > range || -i32::from(*value) > range)
        {
            self.value_range.set(i16::MAX);
        }
    }
    /// The virtual key of a key-state bit, `Aspect_VKey_UNKNOWN` when it has none.
    ///
    /// Example: `mouse.hid_to_space_key(4)`.
    pub fn hid_to_space_key(&self, key_bit: u16) -> AspectVKey {
        space_key_to_vkey(hid_to_space_key(self.product_id, key_bit)) as AspectVKey
    }
}

/// Zeroes the translation components inside the dead zones (|v| < 4, and |z| < 8).
fn translation_dead_zone(raw: [i16; 3]) -> ([f64; 3], bool) {
    let mut values = raw.map(f64::from);
    let mut is_idle = true;
    for (value, component) in values.iter_mut().zip(raw) {
        if component > -4 && component < 4 {
            *value = 0.0;
        } else {
            is_idle = false;
        }
    }
    if raw[2] > -8 && raw[2] < 8 {
        values[2] = 0.0;
    }
    (values, is_idle)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(kind: u8, values: &[i16]) -> Vec<u8> {
        let mut bytes = vec![kind];
        values
            .iter()
            .for_each(|value| bytes.extend_from_slice(&value.to_ne_bytes()));
        bytes
    }

    #[test]
    fn translation_applies_the_dead_zone_and_scale() {
        let bytes = chunk(RAW_TRANSLATION, &[350, 3, -6]);
        let mouse = WntHidSpaceMouse::new(0xC62B, &bytes);
        assert!(mouse.is_translation() && !mouse.is_rotation());
        let (vector, idle) = mouse.translation(false);
        assert!(!idle);
        assert_eq!((vector.x, vector.y, vector.z), (1.0, 0.0, 0.0));
        let (quadric, _) = mouse.translation(true);
        // NCollection_Vec3<double>::operator/= multiplies by the reciprocal natively.
        assert_eq!(quadric.x, 122500.0 * (1.0 / 122500.0));
    }

    #[test]
    fn large_values_widen_the_range_and_rotation_reads_the_second_half() {
        let bytes = chunk(RAW_TRANSLATION, &[0, 0, 0, 0, 1000, -2]);
        let mouse = WntHidSpaceMouse::new(0xC62B, &bytes);
        let (vector, idle) = mouse.rotation(false);
        assert!(!idle);
        assert_eq!(mouse.raw_value_range(), i16::MAX);
        assert_eq!(vector.y, 1000.0 * (1.0 / 32767.0));
        mouse.set_raw_value_range(10);
        assert_eq!(mouse.raw_value_range(), i16::MAX);
    }

    #[test]
    fn keys_follow_the_model_tables() {
        let bytes = [RAW_KEY_STATE, 5, 0, 0, 0];
        let pro = WntHidSpaceMouse::new(0xC62B, &bytes);
        assert!(pro.is_key_state());
        assert_eq!(pro.key_state(), u32::from_ne_bytes([5, 0, 0, 0]));
        assert_eq!(
            pro.hid_to_space_key(1),
            AspectVKeyBasic::ViewFitAll as AspectVKey
        );
        assert_eq!(pro.hid_to_space_key(21), AspectVKeyBasic::F1 as AspectVKey);
        assert_eq!(
            pro.hid_to_space_key(40),
            AspectVKeyBasic::Unknown as AspectVKey
        );
        let pilot = WntHidSpaceMouse::new(0xC625, &bytes);
        assert_eq!(
            pilot.hid_to_space_key(0),
            AspectVKeyBasic::Digit1 as AspectVKey
        );
        assert!(WntHidSpaceMouse::is_known_product(0xC635));
        assert!(!WntHidSpaceMouse::is_known_product(0xC603));
    }
}
