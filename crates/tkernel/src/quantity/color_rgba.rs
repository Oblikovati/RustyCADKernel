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

//! Linear RGB plus opacity and OCCT's sRGB hexadecimal input forms.
use super::{Color, ColorSpace};
use crate::standard::Failure;
/// RGB plus an independent opacity channel; `ColorRgba::default()` is opaque yellow.
#[derive(Clone, Copy, Debug)]
pub struct ColorRgba {
    rgb: Color,
    alpha: f32,
}
impl Default for ColorRgba {
    fn default() -> Self {
        Self {
            rgb: Color::default(),
            alpha: 1.0,
        }
    }
}
impl ColorRgba {
    /// Validates RGB and retains alpha verbatim; `ColorRgba::new(1.0,0.0,0.0,0.5)?`.
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Result<Self, Failure> {
        Ok(Self {
            rgb: Color::new(red.into(), green.into(), blue.into(), ColorSpace::Rgb)?,
            alpha,
        })
    }
    /// Combines an existing color and opacity; `ColorRgba::from_rgb(Color::default(),0.5)`.
    pub const fn from_rgb(rgb: Color, alpha: f32) -> Self {
        Self { rgb, alpha }
    }
    /// Borrows the color channels; `rgba.rgb().red()`.
    pub const fn rgb(&self) -> &Color {
        &self.rgb
    }
    /// Mutates RGB without changing opacity; `rgba.change_rgb().set_named(name)`.
    pub const fn change_rgb(&mut self) -> &mut Color {
        &mut self.rgb
    }
    /// Reads opacity; `ColorRgba::default().alpha() == 1.0`.
    pub const fn alpha(self) -> f32 {
        self.alpha
    }
    /// Assigns opacity independently; `rgba.set_alpha(0.5)`.
    pub const fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }
    /// Replaces channels and opacity; `rgba.set_values(0.1,0.2,0.3,0.4)?`.
    pub fn set_values(&mut self, r: f32, g: f32, b: f32, a: f32) -> Result<(), Failure> {
        self.rgb
            .set_values(r.into(), g.into(), b.into(), ColorSpace::Rgb)?;
        self.alpha = a;
        Ok(())
    }
    /// Compares RGB distance and opacity with OCCT epsilon; `rgba.is_equal(rgba)`.
    pub fn is_equal(self, other: Self) -> bool {
        self.rgb.is_equal(other.rgb) && (self.alpha - other.alpha).abs() <= Color::epsilon() as f32
    }
    /// Compares RGB distance and opacity strictly above epsilon; `!rgba.is_different(rgba)`.
    pub fn is_different(self, other: Self) -> bool {
        self.rgb.is_different(other.rgb)
            || (self.alpha - other.alpha).abs() > Color::epsilon() as f32
    }
    /// Parses hex and updates only on success; `rgba.set_hex("#FF000080",false)`.
    pub fn set_hex(&mut self, text: &str, alpha_off: bool) -> bool {
        let Some((mut integer, base, has_alpha)) = parse_hex(text, alpha_off) else {
            return false;
        };
        let alpha = if has_alpha {
            take_component(&mut integer, base)
        } else {
            1.0
        };
        let mut rgb = [0.0; 3];
        for channel in rgb.iter_mut().rev() {
            *channel = Color::srgb_to_linear_f32(take_component(&mut integer, base));
        }
        self.rgb = Color::new(rgb[0].into(), rgb[1].into(), rgb[2].into(), ColorSpace::Rgb)
            .expect("hex channels in range");
        self.alpha = alpha;
        true
    }
}
impl PartialEq for ColorRgba {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(*other)
    }
    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &Self) -> bool {
        self.is_different(*other)
    }
}
fn parse_hex(text: &str, alpha_off: bool) -> Option<(u32, u32, bool)> {
    let text = text.split('\0').next()?;
    let prefixed = text.starts_with('#');
    let digits = text.strip_prefix('#').unwrap_or(text);
    if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let (base, alpha) = match digits.len() {
        3 if prefixed => (16, false),
        4 => (16, true),
        6 => (256, false),
        8 => (256, true),
        _ => return None,
    };
    if alpha && alpha_off {
        return None;
    }
    Some((u32::from_str_radix(digits, 16).ok()?, base, alpha))
}
fn take_component(integer: &mut u32, base: u32) -> f32 {
    let component = (*integer % base) as f32 * 1.0 / (base - 1) as f32;
    *integer /= base;
    component
}
