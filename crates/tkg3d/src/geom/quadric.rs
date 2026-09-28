// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_CylindricalSurface.cxx, Geom_ConicalSurface.cxx, Geom_SphericalSurface.cxx (Coefficients) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Coefficients` members of the cylinder, cone and sphere: the implicit
//! equation `A1.X^2 + A2.Y^2 + A3.Z^2 + 2.(B1.X.Y + B2.X.Z + B3.Y.Z) +
//! 2.(C1.X + C2.Y + C3.Z) + D = 0` in the absolute frame, as
//! `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]`. Geom computes them itself (not
//! through gp), so the operation order here is the Geom one.
use rustycad_tkmath::gp::{Axis3, Transform3};

/// The rows of `T.SetTransformation(pos)`, read with `T.Value(row, col)`.
pub(crate) fn frame_rows(pos: Axis3) -> [[f64; 4]; 3] {
    let mut transform = Transform3::default();
    transform.set_transformation_to(pos);
    let mut rows = [[0.0; 4]; 3];
    for (row, values) in rows.iter_mut().enumerate() {
        for (col, value) in values.iter_mut().enumerate() {
            // The indices are in range, so Value cannot raise.
            *value = transform
                .value(row as i32 + 1, col as i32 + 1)
                .unwrap_or_default();
        }
    }
    rows
}

/// `Geom_CylindricalSurface::Coefficients`; `cylinder_coefficients(pos, 2.0)`.
pub fn cylinder_coefficients(pos: Axis3, radius: f64) -> [f64; 10] {
    let [[t11, t12, t13, t14], [t21, t22, t23, t24], _] = frame_rows(pos);
    [
        t11 * t11 + t21 * t21,
        t12 * t12 + t22 * t22,
        t13 * t13 + t23 * t23,
        t11 * t12 + t21 * t22,
        t11 * t13 + t21 * t23,
        t12 * t13 + t22 * t23,
        t11 * t14 + t21 * t24,
        t12 * t14 + t22 * t24,
        t13 * t14 + t23 * t24,
        t14 * t14 + t24 * t24 - radius * radius,
    ]
}

/// `Geom_ConicalSurface::Coefficients`: the third row is scaled by `tan(SemiAngle)`;
/// `cone_coefficients(pos, 2.0, 0.3)`.
pub fn cone_coefficients(pos: Axis3, radius: f64, semi_angle: f64) -> [f64; 10] {
    let k_ang: f64 = semi_angle.tan();
    let [[t11, t12, t13, t14], [t21, t22, t23, t24], third] = frame_rows(pos);
    let (t31, t32, t33, t34) = (
        third[0] * k_ang,
        third[1] * k_ang,
        third[2] * k_ang,
        third[3] * k_ang,
    );
    [
        t11 * t11 + t21 * t21 - t31 * t31,
        t12 * t12 + t22 * t22 - t32 * t32,
        t13 * t13 + t23 * t23 - t33 * t33,
        t11 * t12 + t21 * t22 - t31 * t32,
        t11 * t13 + t21 * t23 - t31 * t33,
        t12 * t13 + t22 * t23 - t32 * t33,
        t11 * t14 + t21 * t24 + radius * t31,
        t12 * t14 + t22 * t24 + radius * t32,
        t13 * t14 + t23 * t24 + radius * t33,
        t14 * t14 + t24 * t24 - radius * radius - t34 * t34 + 2.0 * radius * t34,
    ]
}

/// `Geom_SphericalSurface::Coefficients`; `sphere_coefficients(pos, 2.0)`.
pub fn sphere_coefficients(pos: Axis3, radius: f64) -> [f64; 10] {
    let [
        [t11, t12, t13, t14],
        [t21, t22, t23, t24],
        [t31, t32, t33, t34],
    ] = frame_rows(pos);
    [
        t11 * t11 + t21 * t21 + t31 * t31,
        t12 * t12 + t22 * t22 + t32 * t32,
        t13 * t13 + t23 * t23 + t33 * t33,
        t11 * t12 + t21 * t22 + t31 * t32,
        t11 * t13 + t21 * t23 + t31 * t33,
        t12 * t13 + t22 * t23 + t32 * t33,
        t11 * t14 + t21 * t24 + t31 * t34,
        t12 * t14 + t22 * t24 + t32 * t34,
        t13 * t14 + t23 * t24 + t33 * t34,
        t14 * t14 + t24 * t24 + t34 * t34 - radius * radius,
    ]
}
