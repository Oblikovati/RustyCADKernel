// Created by: NW,JPB,CAL
// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! Linear RGB storage with OCCT color-space conversion and equality tolerance.
use super::NamedColor;
use crate::standard::Failure;
use std::sync::atomic::{AtomicU64, Ordering};
static COLOR_EPSILON: AtomicU64 = AtomicU64::new(0.0001_f64.to_bits());

/// Selects OCCT's component interpretation; `ColorSpace::Rgb` means linear RGB.
#[derive(Clone, Copy, Debug)]
pub enum ColorSpace {
    /// Linear RGB; `Color::new(1.0,0.0,0.0,ColorSpace::Rgb)`.
    Rgb,
    /// Gamma-encoded sRGB; `ColorSpace::Srgb`.
    Srgb,
    /// OCCT hue, maximum channel and saturation; `ColorSpace::Hls`.
    Hls,
    /// CIE L*a*b*; `ColorSpace::Lab`.
    Lab,
    /// CIE lightness, chroma and hue; `ColorSpace::Lch`.
    Lch,
}
/// A color stored as three linear RGB floats; `Color::named(NamedColor::RED)`.
#[derive(Clone, Copy, Debug)]
pub struct Color {
    rgb: [f32; 3],
}
impl Default for Color {
    fn default() -> Self {
        Self::named(NamedColor::YELLOW)
    }
}
impl Color {
    /// Constructs and validates components; `Color::new(1.0,0.0,0.0,ColorSpace::Rgb)?`.
    pub fn new(a: f64, b: f64, c: f64, space: ColorSpace) -> Result<Self, Failure> {
        let mut color = Self::default();
        color.set_values(a, b, c, space)?;
        Ok(color)
    }
    /// Loads exact upstream table values; `Color::named(NamedColor::RED).red() == 1.0`.
    pub fn named(name: NamedColor) -> Self {
        Self { rgb: name.rgb() }
    }
    /// Replaces the named color; `color.set_named(NamedColor::BLUE)`.
    pub fn set_named(&mut self, name: NamedColor) {
        self.rgb = name.rgb();
    }
    /// Borrows stored components by value; `color.rgb()[0]`.
    pub const fn rgb(self) -> [f32; 3] {
        self.rgb
    }
    /// Returns red promoted to double; `Color::default().red() == 1.0`.
    pub const fn red(self) -> f64 {
        self.rgb[0] as f64
    }
    /// Returns green promoted to double; `Color::default().green() == 1.0`.
    pub const fn green(self) -> f64 {
        self.rgb[1] as f64
    }
    /// Returns blue promoted to double; `Color::default().blue() == 0.0`.
    pub const fn blue(self) -> f64 {
        self.rgb[2] as f64
    }
    /// Reads the shared equality threshold; `Color::epsilon()`.
    pub fn epsilon() -> f64 {
        f64::from_bits(COLOR_EPSILON.load(Ordering::Relaxed))
    }
    /// Sets the shared threshold atomically; `Color::set_epsilon(0.0001)`.
    pub fn set_epsilon(value: f64) {
        COLOR_EPSILON.store(value.to_bits(), Ordering::Relaxed);
    }
    /// Computes double-precision squared distance; `color.square_distance(color) == 0.0`.
    pub fn square_distance(self, other: Self) -> f64 {
        let [r, g, b] = std::array::from_fn(|i| f64::from(self.rgb[i]) - f64::from(other.rgb[i]));
        r * r + g * g + b * b
    }
    /// Computes Euclidean distance; `color.distance(color) == 0.0`.
    pub fn distance(self, other: Self) -> f64 {
        self.square_distance(other).sqrt()
    }
    /// Tests the shared tolerance; `color.is_equal(color)`.
    pub fn is_equal(self, other: Self) -> bool {
        self.square_distance(other) <= Self::epsilon() * Self::epsilon()
    }
    /// Tests distance strictly above tolerance; `!color.is_different(color)`.
    pub fn is_different(self, other: Self) -> bool {
        self.square_distance(other) > Self::epsilon() * Self::epsilon()
    }
    /// Returns hue in degrees; `Color::named(NamedColor::RED).hue()`.
    pub fn hue(self) -> f64 {
        self.values(ColorSpace::Hls)[0]
    }
    /// Returns maximum gamma-encoded channel; `Color::named(NamedColor::RED).light()`.
    pub fn light(self) -> f64 {
        self.values(ColorSpace::Hls)[1]
    }
    /// Returns saturation; `Color::named(NamedColor::RED).saturation()`.
    pub fn saturation(self) -> f64 {
        self.values(ColorSpace::Hls)[2]
    }
    /// Converts to the requested components; `color.values(ColorSpace::Lab)`.
    pub fn values(self, space: ColorSpace) -> [f64; 3] {
        match space {
            ColorSpace::Rgb => self.rgb.map(f64::from),
            ColorSpace::Srgb => self.rgb.map(|v| Self::linear_to_srgb(f64::from(v))),
            ColorSpace::Hls => {
                Self::srgb_to_hls(self.rgb.map(Self::linear_to_srgb_f32)).map(f64::from)
            }
            ColorSpace::Lab => Self::linear_rgb_to_lab(self.rgb).map(f64::from),
            ColorSpace::Lch => Self::lab_to_lch(Self::linear_rgb_to_lab(self.rgb)).map(f64::from),
        }
    }
    /// Validates and assigns components; `color.set_values(0.1,0.2,0.3,ColorSpace::Rgb)?`.
    pub fn set_values(&mut self, a: f64, b: f64, c: f64, space: ColorSpace) -> Result<(), Failure> {
        validate_components([a, b, c], space)?;
        let values = [a as f32, b as f32, c as f32];
        self.rgb = match space {
            ColorSpace::Rgb => values,
            ColorSpace::Srgb => [a, b, c].map(|v| Self::srgb_to_linear(v) as f32),
            ColorSpace::Hls => Self::hls_to_srgb(values)?.map(Self::srgb_to_linear_f32),
            ColorSpace::Lab => Self::lab_to_linear_rgb(values),
            ColorSpace::Lch => Self::lab_to_linear_rgb(Self::lch_to_lab(values)),
        };
        Ok(())
    }
}
impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(*other)
    }
    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &Self) -> bool {
        self.is_different(*other)
    }
}
fn validate_components([a, b, c]: [f64; 3], space: ColorSpace) -> Result<(), Failure> {
    let invalid = match space {
        ColorSpace::Rgb | ColorSpace::Srgb => {
            outside(a, 0.0, 1.0) || outside(b, 0.0, 1.0) || outside(c, 0.0, 1.0)
        }
        ColorSpace::Hls => {
            (a < 0.0 && a != -1.0 && c != 0.0)
                || a > 360.0
                || outside(b, 0.0, 1.0)
                || outside(c, 0.0, 1.0)
        }
        ColorSpace::Lab => {
            outside(a, 0.0, 100.0) || outside(b, -100.0, 100.0) || outside(c, -110.0, 100.0)
        }
        ColorSpace::Lch => {
            outside(a, 0.0, 100.0) || outside(b, 0.0, 135.0) || outside(c, 0.0, 360.0)
        }
    };
    if invalid {
        return Err(Failure::out_of_range("Color out"));
    }
    Ok(())
}
// Deliberately preserve OCCT's NaN comparison behavior rather than Range::contains.
fn outside(value: f64, lower: f64, upper: f64) -> bool {
    value < lower || value > upper
}
