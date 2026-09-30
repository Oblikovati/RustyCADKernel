//! Quantity_Color's packed encodings: hex strings, ARGB integers, hashing and JSON dumps.
use super::{Color, ColorRgba, ColorSpace};
use crate::{
    ncollection::HashKey,
    standard::{
        Failure,
        dump::{self, DumpOutput},
        hash_bytes,
    },
    tcollection::AsciiString,
};
use std::io;
impl Color {
    /// `ColorFromHex(const char*, Quantity_Color&)`: RGB-only hex through
    /// `Quantity_ColorRGBA::ColorFromHex(..., true)`; `Color::from_hex("#FF0000")`.
    pub fn from_hex(text: &str) -> Option<Self> {
        ColorRgba::from_hex(text, true).map(|rgba| *rgba.rgb())
    }
    /// `ColorToHex`: sRGB bytes rounded as `int(v * 255.0f + 0.5f)`, printed with `%02X`;
    /// `Color::to_hex(Color::default(), true) == "#FFFF00"`.
    pub fn to_hex(color: Self, prefix_hash: bool) -> AsciiString {
        let bytes = srgb_bytes(Self::linear_to_srgb_vec3_f32(color.rgb()));
        AsciiString::new(&hex_text(&bytes, prefix_hash))
    }
    /// `Color2argb`: linear (not gamma-encoded) channels packed as `0x00RRGGBB`;
    /// `Color::to_argb(Color::named(NamedColor::RED)) == 0xFF0000`.
    pub fn to_argb(color: Self) -> i32 {
        let [r, g, b] = color
            .rgb()
            .map(|channel| (255.0_f32 * channel + 0.5) as i32);
        ((r & 0xff) << 16) | ((g & 0xff) << 8) | (b & 0xff)
    }
    /// `Argb2color`: alpha bits ignored, bytes read as sRGB; `Color::from_argb(0x00FF0000)`.
    pub fn from_argb(argb: i32) -> Self {
        let channels = [
            f64::from((argb & 0xff0000) >> 16),
            f64::from((argb & 0x00ff00) >> 8),
            f64::from(argb & 0x0000ff),
        ];
        let [r, g, b] = channels.map(|channel| channel / 255.0);
        Self::new(r, g, b, ColorSpace::Srgb).expect("byte channels are within [0; 1]")
    }
    /// `DumpJson`: `"RGB": [r, g, b]`; `color.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "RGB", &self.rgb().map(f64::from))
    }
    /// `InitFromJson`. Native quirk kept: the caller's stream position is never advanced
    /// (the macro updates a local copy); out-of-range values raise `Standard_OutOfRange`;
    /// `color.init_from_json("\"RGB\": [1, 0, 0]", &mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        let mut values = [0.0; 3];
        if !dump::init_vector(text, "RGB", &mut position.clone(), &mut values)? {
            return Ok(false);
        }
        let [r, g, b] = values.map(|value| f64::from(value as f32));
        self.set_values(r, g, b, ColorSpace::Rgb)?;
        Ok(true)
    }
}
/// `std::hash<Quantity_Color>`: `hashBytes` over `unsigned char(255 * channel)`.
impl HashKey for Color {
    fn occt_hash(&self) -> usize {
        hash_bytes(&[self.red(), self.green(), self.blue()].map(|v| (255.0 * v) as u8))
    }
}
/// sRGB channels rounded to integers as `NCollection_Vec3<int>(v * 255.0f + Vec(0.5f))`.
pub(super) fn srgb_bytes<const N: usize>(channels: [f32; N]) -> [i32; N] {
    channels.map(|channel| (channel * 255.0 + 0.5) as i32)
}
/// `Sprintf("%02X...")` of each integer, with an optional leading `#`; negative values
/// print as their 32-bit two's complement, as `%X` does with the int argument.
pub(super) fn hex_text(values: &[i32], prefix_hash: bool) -> String {
    let digits: String = values.iter().map(|value| format!("{value:02X}")).collect();
    if prefix_hash {
        return format!("#{digits}");
    }
    digits
}
