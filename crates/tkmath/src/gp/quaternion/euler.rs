// Created on: 2010-05-11
// Created by: Kirill GAVRILOV
// Copyright (c) 2010-2014 OPEN CASCADE SAS
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

// Note: implementation is based on free samples from
// http://www.gamedev.ru/code/articles/?id=4215&page=3
// and maths found in Wikipedia and elsewhere

use super::{EulerSequence, Quaternion, sequence::Parameters};
impl Quaternion {
    /// Applies the native Shoemake parameterization for all 24 sequences; `Quaternion::from_euler_angles(order,a,b,c)`.
    pub fn from_euler_angles(order: EulerSequence, alpha: f64, beta: f64, gamma: f64) -> Self {
        let parameters = order.parameters();
        let [a, b, c] = input_angles(parameters, [alpha, beta, gamma]);
        let mut values = euler_components(parameters, half_products(a, b, c));
        if parameters.odd {
            values[parameters.j + 1] = -values[parameters.j + 1];
        }
        Self::new(values[1], values[2], values[3], values[0])
    }
    /// Replaces the quaternion from a sequence of rotations; `quaternion.set_euler_angles(order,a,b,c)`.
    pub fn set_euler_angles(&mut self, order: EulerSequence, alpha: f64, beta: f64, gamma: f64) {
        *self = Self::from_euler_angles(order, alpha, beta, gamma);
    }
    /// Returns native singularity branches and angle conventions; `quaternion.euler_angles(order)`.
    pub fn euler_angles(&self, order: EulerSequence) -> [f64; 3] {
        let parameters = order.parameters();
        let matrix = *self.matrix().coefficients();
        let mut angles = if parameters.two {
            proper_angles(matrix, parameters)
        } else {
            tait_bryan_angles(matrix, parameters)
        };
        if parameters.odd {
            angles = angles.map(|a| -a);
        }
        if !parameters.extrinsic {
            angles.swap(0, 2);
        }
        angles
    }
}
fn input_angles(parameters: Parameters, mut angles: [f64; 3]) -> [f64; 3] {
    if !parameters.extrinsic {
        angles.swap(0, 2);
    }
    if parameters.odd {
        angles[1] = -angles[1];
    }
    angles
}
fn half_products(a: f64, b: f64, c: f64) -> [f64; 6] {
    let (ti, tj, th) = (0.5 * a, 0.5 * b, 0.5 * c);
    let (ci, cj, ch) = (ti.cos(), tj.cos(), th.cos());
    let (si, sj, sh) = (ti.sin(), tj.sin(), th.sin());
    [cj, sj, ci * ch, ci * sh, si * ch, si * sh]
}
fn euler_components(p: Parameters, products: [f64; 6]) -> [f64; 4] {
    let [cj, sj, cc, cs, sc, ss] = products;
    let mut values = [0.0; 4];
    if p.two {
        values[p.i + 1] = cj * (cs + sc);
        values[p.j + 1] = sj * (cc + ss);
        values[p.k + 1] = sj * (cs - sc);
        values[0] = cj * (cc - ss);
    } else {
        values[p.i + 1] = cj * sc - sj * cs;
        values[p.j + 1] = cj * ss + sj * cc;
        values[p.k + 1] = cj * cs - sj * sc;
        values[0] = cj * cc + sj * ss;
    }
    values
}
fn proper_angles(m: [[f64; 3]; 3], p: Parameters) -> [f64; 3] {
    let (i, j, k) = (p.i, p.j, p.k);
    let sy = (m[i][j] * m[i][j] + m[i][k] * m[i][k]).sqrt();
    let beta = sy.atan2(m[i][i]);
    if sy > 16.0 * f64::EPSILON {
        return [m[i][j].atan2(m[i][k]), beta, m[j][i].atan2(-m[k][i])];
    }
    [(-m[j][k]).atan2(m[j][j]), beta, 0.0]
}
fn tait_bryan_angles(m: [[f64; 3]; 3], p: Parameters) -> [f64; 3] {
    let (i, j, k) = (p.i, p.j, p.k);
    let cy = (m[i][i] * m[i][i] + m[j][i] * m[j][i]).sqrt();
    let beta = (-m[k][i]).atan2(cy);
    if cy > 16.0 * f64::EPSILON {
        return [m[k][j].atan2(m[k][k]), beta, m[j][i].atan2(m[i][i])];
    }
    [(-m[j][k]).atan2(m[j][j]), beta, 0.0]
}
