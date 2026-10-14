// SPDX-License-Identifier: GPL-2.0-only
//! Ordered sRGB byte pairs from OCCT MeshVS_TwoColors.hxx/.cxx at 3d097a0.
use rustycad_tkernel::{
    ncollection::HashKey,
    quantity::{Color, ColorSpace},
};

/// Two independently quantized colors; construct through [`bind_two_colors`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeshVSTwoColors {
    /// First red sRGB byte; `pair.r1`.
    pub r1: u8,
    /// First green sRGB byte; `pair.g1`.
    pub g1: u8,
    /// First blue sRGB byte; `pair.b1`.
    pub b1: u8,
    /// Second red sRGB byte; `pair.r2`.
    pub r2: u8,
    /// Second green sRGB byte; `pair.g2`.
    pub g2: u8,
    /// Second blue sRGB byte; `pair.b2`.
    pub b2: u8,
}

impl HashKey for MeshVSTwoColors {
    fn occt_hash(&self) -> usize {
        let mut hash = 0_u32;
        for byte in [self.r1, self.g1, self.b1, self.r2, self.g2, self.b2] {
            hash = hash.wrapping_add(u32::from(byte));
            hash = hash.wrapping_add(hash << 10);
            hash ^= hash >> 6;
        }
        hash as usize
    }
}

/// Truncate the sRGB channels after scaling by 255; `bind_two_colors(a, b)`.
pub fn bind_two_colors(first: Color, second: Color) -> MeshVSTwoColors {
    let [r1, g1, b1] = first
        .values(ColorSpace::Srgb)
        .map(|value| (value * 255.0) as u8);
    let [r2, g2, b2] = second
        .values(ColorSpace::Srgb)
        .map(|value| (value * 255.0) as u8);
    MeshVSTwoColors {
        r1,
        g1,
        b1,
        r2,
        g2,
        b2,
    }
}

/// Extract index 1 or 2; other indices return native default yellow; `extract_color(&mut pair, 1)`.
pub fn extract_color(pair: &mut MeshVSTwoColors, index: i32) -> Color {
    match index {
        1 => decode_srgb_bytes([pair.r1, pair.g1, pair.b1]),
        2 => decode_srgb_bytes([pair.r2, pair.g2, pair.b2]),
        _ => Color::default(),
    }
}

/// Overwrite both outputs in their native order; `extract_colors(&mut pair, &mut a, &mut b)`.
pub fn extract_colors(pair: &mut MeshVSTwoColors, first: &mut Color, second: &mut Color) {
    extract_colors_into(pair, MeshVSColorOutputs::Distinct { first, second });
}

/// Preserve native output aliasing without overlapping mutable references.
pub enum MeshVSColorOutputs<'a> {
    /// Two separate native output variables; `MeshVSColorOutputs::Distinct { first, second }`.
    Distinct {
        /// First output reference.
        first: &'a mut Color,
        /// Second output reference.
        second: &'a mut Color,
    },
    /// One variable passed twice; `MeshVSColorOutputs::Aliased(&mut color)`.
    Aliased(&'a mut Color),
}

/// Model both legal native output arrangements; `extract_colors_into(pair, MeshVSColorOutputs::Aliased(color))`.
pub fn extract_colors_into(pair: &mut MeshVSTwoColors, outputs: MeshVSColorOutputs<'_>) {
    let first_color = extract_color(pair, 1);
    let second_color = extract_color(pair, 2);
    match outputs {
        MeshVSColorOutputs::Distinct { first, second } => {
            *first = first_color;
            *second = second_color;
        }
        MeshVSColorOutputs::Aliased(output) => {
            *output = first_color;
            *output = second_color;
        }
    }
}

fn decode_srgb_bytes(bytes: [u8; 3]) -> Color {
    let [red, green, blue] = bytes.map(|value| f64::from(value) / 255.0);
    // Every bitfield value is in range; this cannot introduce a native failure path.
    Color::new(red, green, blue, ColorSpace::Srgb).unwrap_or_else(|cause| {
        panic!("sRGB bytes={bytes:?}; expected channels/255 within [0,1]: {cause}")
    })
}
