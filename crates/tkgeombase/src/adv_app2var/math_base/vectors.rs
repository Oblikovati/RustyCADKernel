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

//! Vector utilities: `mdsptpt_`, `mmunivt_` and `mmvncol_`.
use super::{mzsnorm_, pow__di};
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::sys_base::{maovsr8_, mvriraz_};

/// The distance between two points of dimension `ndimen` (`mdsptpt_`), the
/// `mzsnorm_` of their difference; returns 0.
///
/// Natively the difference vector is allocated only above 100 components and the
/// f2c code dropped the Fortran local array, so a smaller call writes through a null
/// pointer; the port computes the allocated path for every size.
/// `mdsptpt_(3, &p1, &p2, &mut d)`.
pub fn mdsptpt_(ndimen: i32, point1: &[f64], point2: &[f64], distan: &mut f64) -> i32 {
    let (point1, point2) = (F2cArray::new(point1, 1), F2cArray::new(point2, 1));
    let differ: Vec<f64> = (1..=ndimen).map(|i| point2[i] - point1[i]).collect();
    *distan = mzsnorm_(ndimen, &differ);
    0
}

/// The unit vector of `vector` into `vecnrm` (`mmunivt_`), snapped to a signed axis
/// when all components but one are below `10^-9`; `epsiln <= 0` selects the null
/// threshold `10^-17`. `iercod` is 101 (and `vecnrm` zero) for a null vector;
/// returns 0. `mmunivt_(3, &v, &mut unit, 0.0, &mut ier)`.
pub fn mmunivt_(
    ndimen: i32,
    vector: &[f64],
    vecnrm: &mut [f64],
    epsiln: f64,
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    let mut nchif: i32 = 0;
    maovsr8_(&mut nchif);
    let eps0: f64 = if epsiln <= 0. {
        pow__di(10., -nchif)
    } else {
        epsiln
    };
    let vnorm: f64 = mzsnorm_(ndimen, vector);
    if vnorm <= eps0 {
        mvriraz_(ndimen, vecnrm);
        *iercod = 101;
        return 0;
    }
    let vector = F2cArray::new(vector, 1);
    let mut unit = F2cArrayMut::new(vecnrm, 1);
    let eps0: f64 = pow__di(10., (-nchif - 1) / 2);
    let (mut izero, mut iunit) = (0, 1);
    for ii in 1..=ndimen {
        unit[ii] = vector[ii] / vnorm;
        if unit[ii].abs() <= eps0 {
            izero += 1;
        } else {
            iunit = ii;
        }
    }
    if izero == ndimen - 1 {
        let bid: f64 = unit[iunit];
        for ii in 1..=ndimen {
            unit[ii] = 0.;
        }
        unit[iunit] = if bid > 0. { 1. } else { -1. };
    }
    0
}

/// A vector of dimension 2 or 3 that is not collinear with `vecin` (`mmvncol_`),
/// found by adding 1 to one component after the other. `iercod` is 1 for another
/// dimension, a null vector or no success; returns 0.
/// `mmvncol_(3, &[1.0, 0.0, 0.0], &mut out, &mut ier)`.
pub fn mmvncol_(ndimen: i32, vecin: &[f64], vecout: &mut [f64], iercod: &mut i32) -> i32 {
    *iercod = 0;
    if ndimen <= 1 || ndimen > 3 {
        *iercod = 1;
        return 0;
    }
    let vecin = F2cArray::new(vecin, 1);
    if (1..=ndimen).filter(|d| vecin[*d] == 0.).count() as i32 == ndimen {
        *iercod = 1;
        return 0;
    }
    let mut vaux1: [f64; 3] = [0.; 3];
    let mut vaux2: [f64; 3] = [0.; 3];
    for d in 1..=ndimen {
        vaux1[(d - 1) as usize] = vecin[d];
        vaux2[(d - 1) as usize] = vecin[d];
    }
    let mut colin: bool = true;
    let mut d: usize = 0;
    while colin {
        d += 1;
        if d > 3 {
            *iercod = 1;
            return 0;
        }
        vaux2[d - 1] += 1.;
        colin = cross_is_null(&vaux1, &vaux2);
    }
    let mut vecout = F2cArrayMut::new(vecout, 1);
    for d in 1..=ndimen {
        vecout[d] = vaux2[(d - 1) as usize];
    }
    0
}

/// The collinearity test of `mmvncol_`: every component of the cross product is 0,
/// computed in the native order (x, then y, then z).
fn cross_is_null(a: &[f64; 3], b: &[f64; 3]) -> bool {
    a[1] * b[2] - a[2] * b[1] == 0.
        && a[2] * b[0] - a[0] * b[2] == 0.
        && a[0] * b[1] - a[1] * b[0] == 0.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distances_and_unit_vectors() {
        let mut d: f64 = 0.0;
        mdsptpt_(3, &[1.0, 1.0, 1.0], &[4.0, 5.0, 1.0], &mut d);
        assert_eq!(d, 5.0);
        let (mut unit, mut ier) = ([0.0; 3], 9);
        mmunivt_(3, &[0.0, -2.0, 1e-12], &mut unit, 0.0, &mut ier);
        assert_eq!((ier, unit), (0, [0.0, -1.0, 0.0]));
        mmunivt_(3, &[0.0; 3], &mut unit, 0.0, &mut ier);
        assert_eq!(ier, 101);
    }

    #[test]
    fn a_non_collinear_vector_is_found() {
        let (mut out, mut ier) = ([0.0; 3], 9);
        mmvncol_(3, &[1.0, 0.0, 0.0], &mut out, &mut ier);
        // The first component is incremented first, then the second.
        assert_eq!((ier, out), (0, [2.0, 1.0, 0.0]));
        mmvncol_(3, &[0.0; 3], &mut out, &mut ier);
        assert_eq!(ier, 1);
        mmvncol_(1, &[1.0], &mut out, &mut ier);
        assert_eq!(ier, 1);
    }
}
