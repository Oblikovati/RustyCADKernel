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

//! OCCT's CIEDE2000 formula, including its epsilon-based hue handling.
use super::{Color, ColorSpace};
const RAD: f64 = std::f64::consts::PI / 180.0;
const DEG: f64 = 180.0 / std::f64::consts::PI;
const POW_25_7: f64 = 6103515625.0;
impl Color {
    /// Evaluates perceptual difference through OCCT's Lab conversion; `color.delta_e2000(color) == 0.0`.
    pub fn delta_e2000(self, other: Self) -> f64 {
        let [l1, a1, b1] = self.values(ColorSpace::Lab);
        let [l2, a2, b2] = other.values(ColorSpace::Lab);
        let (c1, c2, h1, h2) = adjusted_chromas(a1, b1, a2, b2);
        let (mean_hue, delta_hue) = mean_delta_hue(h1, h2);
        let mean_light = 0.5 * (l1 + l2);
        let mean_chroma = 0.5 * (c1 + c2);
        let delta_h = 2.0 * (c1 * c2).sqrt() * (0.5 * delta_hue * RAD).sin();
        let [sl, sc, sh, rotation] = difference_factors(mean_light, mean_chroma, mean_hue);
        let dl = (l2 - l1) / sl;
        let dc = (c2 - c1) / sc;
        let dh = delta_h / sh;
        (dl * dl + dc * dc + dh * dh + rotation * dc * dh).sqrt()
    }
}
fn adjusted_chromas(a1: f64, b1: f64, a2: f64, b2: f64) -> (f64, f64, f64, f64) {
    let c1 = (a1 * a1 + b1 * b1).sqrt();
    let c2 = (a2 * a2 + b2 * b2).sqrt();
    let power = (0.5 * (c1 + c2)).powf(7.0);
    let correction = 0.5 * (1.0 - (power / (power + POW_25_7)).sqrt());
    let a1 = a1 * (1.0 + correction);
    let a2 = a2 * (1.0 + correction);
    let c1 = (a1 * a1 + b1 * b1).sqrt();
    let c2 = (a2 * a2 + b2 * b2).sqrt();
    (c1, c2, adjusted_hue(a1, b1, c1), adjusted_hue(a2, b2, c2))
}
fn adjusted_hue(a: f64, b: f64, chroma: f64) -> f64 {
    let mut hue = if chroma > Color::epsilon() {
        b.atan2(a) * DEG
    } else {
        270.0
    };
    if hue < 0.0 {
        hue += 360.0;
    }
    hue
}
fn mean_delta_hue(first: f64, second: f64) -> (f64, f64) {
    let mut mean = 0.5 * (first + second);
    let mut delta = second - first;
    if delta.abs() > 180.0 {
        mean += if mean < 180.0 { 180.0 } else { -180.0 };
        delta += if first >= second { 360.0 } else { -360.0 };
    }
    (mean, delta)
}
fn difference_factors(light: f64, chroma: f64, hue: f64) -> [f64; 4] {
    let t = 1.0 - 0.17 * ((hue - 30.0) * RAD).cos()
        + 0.24 * ((2.0 * hue) * RAD).cos()
        + 0.32 * ((3.0 * hue + 6.0) * RAD).cos()
        - 0.20 * ((4.0 * hue - 63.0) * RAD).cos();
    let light50 = (light - 50.0) * (light - 50.0);
    let sl = 1.0 + 0.015 * light50 / (20.0 + light50).sqrt();
    let sc = 1.0 + 0.045 * chroma;
    let sh = 1.0 + 0.015 * chroma * t;
    let theta = 30.0 * (-(hue - 275.0) * (hue - 275.0) / 625.0).exp();
    let power = chroma.powf(7.0);
    let rc = 2.0 * (power / (power + POW_25_7)).sqrt();
    [sl, sc, sh, -rc * (2.0 * theta * RAD).sin()]
}
