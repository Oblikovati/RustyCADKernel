// Created on: 1993-07-07
// Created by: Jean Claude VAUTHIER
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (Inertia, AxeOfInertia)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib::Inertia` and `GeomLib::AxeOfInertia`: the principal axes of a point
//! cloud.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Xyz};
use rustycad_tkmath::math::{Jacobi, Matrix, Vector};

/// The principal axes of inertia of a point cloud: the barycentre, the directions
/// of greatest (X) and middle (Y) dispersion, and the dispersions along X, Y and Z.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InertiaAxes {
    /// `Bary`: the barycentre.
    pub bary: Point3,
    /// `XDir`: the eigenvector of the greatest eigenvalue.
    pub x_dir: Direction3,
    /// `YDir`: the eigenvector of the middle eigenvalue.
    pub y_dir: Direction3,
    /// `Xgap`: square root of the greatest eigenvalue.
    pub x_gap: f64,
    /// `YGap`: square root of the middle eigenvalue.
    pub y_gap: f64,
    /// `ZGap`: square root of the smallest eigenvalue.
    pub z_gap: f64,
}

/// `GeomLib::Inertia(Points, Bary, XDir, YDir, Xgap, YGap, ZGap)`: the eigen
/// decomposition (math_Jacobi) of the covariance matrix of `points`.
///
/// The eigenvalues are ordered with the native equality tests on `min`, so ties keep
/// the lower index. The native code reads the eigenvalues without testing IsDone, so a
/// Jacobi failure is StdFail_NotDone from math_Jacobi::Value; a null eigenvector is
/// Standard_ConstructionError (gp_Dir::SetCoord);
/// `let axes = geom_lib::inertia(&points)?;`.
pub fn inertia(points: &Array1<'_, Point3>) -> Result<InertiaAxes, Failure> {
    let nb: i32 = points.length();
    let mut gb = Xyz::new(0.0, 0.0, 0.0);
    for point in points.iter() {
        gb += point.xyz();
    }
    gb /= f64::from(nb);
    let m: Matrix<'static> = covariance(points, gb, nb)?;
    let j = Jacobi::new(&m)?;
    let (n1, n2, n3): (f64, f64, f64) = (j.value(1)?, j.value(2)?, j.value(3)?);
    let [m1, m2, m3] = eigen_order(n1, n2, n3);
    let mut v2: Vector<'static> = Vector::with_bounds(1, 3)?;
    let mut v3: Vector<'static> = Vector::with_bounds(1, 3)?;
    j.vector(m2, &mut v2)?;
    j.vector(m3, &mut v3)?;
    Ok(InertiaAxes {
        bary: Point3::new(gb.x(), gb.y(), gb.z()),
        x_dir: Direction3::new(*v3.value(1)?, *v3.value(2)?, *v3.value(3)?)?,
        y_dir: Direction3::new(*v2.value(1)?, *v2.value(2)?, *v2.value(3)?)?,
        z_gap: j.value(m1)?.abs().sqrt(),
        y_gap: j.value(m2)?.abs().sqrt(),
        x_gap: j.value(m3)?.abs().sqrt(),
    })
}

// M(i, j) = sum of the centred products / nb, symmetric.
fn covariance(points: &Array1<'_, Point3>, gb: Xyz, nb: i32) -> Result<Matrix<'static>, Failure> {
    let mut m: Matrix<'static> = crate::array_access::matrix(1, 3, 1, 3, 0.0)?;
    let mut sums = [[0.0f64; 3]; 3];
    for point in points.iter() {
        let mut diff = Xyz::default();
        diff.set_scaled_sum(-1.0, point.xyz(), gb);
        let d: [f64; 3] = [diff.x(), diff.y(), diff.z()];
        sums[0][0] += d[0] * d[0];
        sums[1][1] += d[1] * d[1];
        sums[2][2] += d[2] * d[2];
        sums[0][1] += d[0] * d[1];
        sums[0][2] += d[0] * d[2];
        sums[1][2] += d[1] * d[2];
    }
    (sums[1][0], sums[2][0], sums[2][1]) = (sums[0][1], sums[0][2], sums[1][2]);
    for (r, row) in (1..).zip(sums) {
        for (c, value) in (1..).zip(row) {
            *m.change_value(r, c)? = value / f64::from(nb);
        }
    }
    Ok(m)
}

// The indices of the smallest, middle and greatest eigenvalues, with the native
// chain of `std::min` equality tests.
fn eigen_order(n1: f64, n2: f64, n3: f64) -> [i32; 3] {
    let r1: f64 = std_min(std_min(n1, n2), n3);
    if r1 == n1 {
        let r2: f64 = std_min(n2, n3);
        return if r2 == n2 { [1, 2, 3] } else { [1, 3, 2] };
    }
    if r1 == n2 {
        let r2: f64 = std_min(n1, n3);
        return if r2 == n1 { [2, 1, 3] } else { [2, 3, 1] };
    }
    let r2: f64 = std_min(n1, n2);
    if r2 == n1 { [3, 1, 2] } else { [3, 2, 1] }
}

/// The result of AxeOfInertia: the frame and whether the points are on a line.
#[derive(Clone, Copy, Debug)]
pub struct AxeOfInertia {
    /// `Axe`: location at the barycentre, X along the greatest inertia axis, main
    /// direction normal to the average plane.
    pub axe: Axis2,
    /// `IsSingular`: the middle dispersion times the point count is within `tol`.
    pub is_singular: bool,
}

/// `GeomLib::AxeOfInertia(Points, Axe, IsSingular, Tol = 1e-7)`: the frame of the
/// principal axes; when the points are on a line (`YGap * NbPoints <= Tol`) the Y
/// direction is any direction normal to X (the X direction of `gp_Ax2(Bary, OX)`);
/// `let result = geom_lib::axe_of_inertia(&points, 1e-7)?;`.
pub fn axe_of_inertia(points: &Array1<'_, Point3>, tol: f64) -> Result<AxeOfInertia, Failure> {
    let axes: InertiaAxes = inertia(points)?;
    let (ox, mut oy): (Direction3, Direction3) = (axes.x_dir, axes.y_dir);
    let is_singular: bool = axes.y_gap * f64::from(points.length()) <= tol;
    if is_singular {
        oy = Axis2::new(axes.bary, ox)?.x_direction();
    }
    let oz: Direction3 = ox.crossed(oy)?;
    Ok(AxeOfInertia {
        axe: Axis2::with_x_direction(axes.bary, oz, ox)?,
        is_singular,
    })
}
