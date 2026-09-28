// Created on: 2019-03-22
// Created by: Timur Izmaylov
// Copyright (c) 2019 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Quantity_ColorRGBA's vector constructors, names, hex output, hashing and JSON dumps.
use super::{
    Color, ColorRgba,
    color_codec::{hex_text, srgb_bytes},
};
use crate::{
    ncollection::{HashKey, Vec3, Vec4},
    standard::{
        Failure,
        dump::{self, DumpOutput},
        hash_bytes,
    },
    tcollection::AsciiString,
};
use std::io;
impl ColorRgba {
    /// `Quantity_ColorRGBA(const Quantity_Color&)`: opaque; `ColorRgba::from_color(color)`.
    pub const fn from_color(rgb: Color) -> Self {
        Self::from_rgb(rgb, 1.0)
    }
    /// `Quantity_ColorRGBA(const NCollection_Vec4<float>&)`: RGB validated to [0; 1], alpha
    /// kept verbatim; `ColorRgba::from_vec4(Vec4::new(1.0, 0.0, 0.0, 0.5))?`.
    pub fn from_vec4(rgba: Vec4<f32>) -> Result<Self, Failure> {
        let rgb = Color::from_vec3(Vec3::new(rgba.x, rgba.y, rgba.z))?;
        Ok(Self::from_rgb(rgb, rgba.w))
    }
    /// The `NCollection_Vec4<float>` conversion operator; `rgba.to_vec4().w`.
    pub const fn to_vec4(self) -> Vec4<f32> {
        let [r, g, b] = self.rgb().rgb();
        Vec4::new(r, g, b, self.alpha())
    }
    /// `SetRGB`: replaces the color, keeping alpha; `rgba.set_rgb(Color::default())`.
    pub const fn set_rgb(&mut self, rgb: Color) {
        *self.change_rgb() = rgb;
    }
    /// `ColorFromName`: a named color with alpha 1.0; `ColorRgba::from_name("RED")`.
    pub fn from_name(text: &str) -> Option<Self> {
        Color::from_name(text).map(Self::from_color)
    }
    /// `ColorToHex`: `#RRGGBBAA` from sRGB channels and the unconverted alpha, each rounded
    /// as `int(v * 255.0f + 0.5f)`; `ColorRgba::to_hex(ColorRgba::default(), true)`.
    pub fn to_hex(color: Self, prefix_hash: bool) -> AsciiString {
        let [r, g, b, a] = Self::linear_to_srgb_vec4(color.to_vec4().data());
        AsciiString::new(&hex_text(&srgb_bytes([r, g, b, a]), prefix_hash))
    }
    /// `Convert_LinearRGB_To_sRGB(const NCollection_Vec4<float>&)`: alpha passes through;
    /// `ColorRgba::linear_to_srgb_vec4([0.0, 0.0, 0.0, 0.5])`.
    pub fn linear_to_srgb_vec4([r, g, b, a]: [f32; 4]) -> [f32; 4] {
        let [r, g, b] = Color::linear_to_srgb_vec3_f32([r, g, b]);
        [r, g, b, a]
    }
    /// `Convert_sRGB_To_LinearRGB(const NCollection_Vec4<float>&)`: alpha passes through;
    /// `ColorRgba::srgb_to_linear_vec4([1.0, 1.0, 1.0, 0.5])`.
    pub fn srgb_to_linear_vec4([r, g, b, a]: [f32; 4]) -> [f32; 4] {
        let [r, g, b] = Color::srgb_to_linear_vec3_f32([r, g, b]);
        [r, g, b, a]
    }
    /// `DumpJson`: `"RGBA": [r, g, b, a]`; `rgba.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "RGBA", &self.to_vec4().data().map(f64::from))
    }
    /// `InitFromJson`. Native quirk kept: the caller's stream position is never advanced;
    /// out-of-range RGB raises `Standard_OutOfRange`;
    /// `rgba.init_from_json("\"RGBA\": [1, 0, 0, 1]", &mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        let mut values = [0.0; 4];
        if !dump::init_vector(text, "RGBA", &mut position.clone(), &mut values)? {
            return Ok(false);
        }
        let [r, g, b, a] = values.map(|value| value as f32);
        self.set_values(r, g, b, a)?;
        Ok(true)
    }
}
/// `std::hash<Quantity_ColorRGBA>`: `hashBytes` over `{100*alpha, 255*r, 255*g, 255*b}` as
/// unsigned chars. Alpha outside [0; 2.55] is undefined natively; Rust saturates.
impl HashKey for ColorRgba {
    fn occt_hash(&self) -> usize {
        let rgb = self.rgb();
        let channels = [rgb.red(), rgb.green(), rgb.blue()].map(|v| (255.0 * v) as u8);
        hash_bytes(&[
            (100.0 * self.alpha()) as u8,
            channels[0],
            channels[1],
            channels[2],
        ])
    }
}
