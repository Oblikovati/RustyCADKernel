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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_Data.pxx and
// AdvApp2Var_Data_*.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_Data`: the Fortran common blocks of the f2c kernels.
//!
//! Natively each `Get<block>()` returns a reference to a function-local static that
//! the kernels only read. The blocks are therefore constant tables here, not shared
//! mutable state: each accessor flattens the generated fragments of
//! [`tables`](self::tables) once and hands out read-only slices with the native member
//! names and lengths.
mod tables;

use std::sync::OnceLock;

/// Common `MDNOMBR`: the angular and small numeric constants (`mdnombr_1_`).
///
/// `getmdnombr().pi`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mdnombr {
    /// Pi.
    pub pi: f64,
    /// Two pi.
    pub deuxpi: f64,
    /// Pi over two, as the native literal rounds it.
    pub pisur2: f64,
    /// Radians per degree.
    pub pis180: f64,
    /// Degrees per radian.
    pub c180pi: f64,
    /// 0.
    pub zero: f64,
    /// 1.
    pub one: f64,
    /// 180.
    pub a180: f64,
    /// 360.
    pub a360: f64,
    /// 90.
    pub a90: f64,
}

/// Common `MAOVPAR`: the machine limits of the ported Fortran runtime (`maovpar_1_`).
///
/// `getmaovpar().r8ncs == 17`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Maovpar {
    /// Smallest double (underflow bound).
    pub r8und: f64,
    /// Largest double (overflow bound).
    pub r8ovr: f64,
    /// Smallest single, as a double.
    pub x4und: f64,
    /// Largest single, as a double.
    pub x4ovr: f64,
    /// Smallest single.
    pub r4und: f32,
    /// Largest single.
    pub r4ovr: f32,
    /// Exponent bits of a single.
    pub r4nbe: i32,
    /// Mantissa bits of a double.
    pub r8nbm: i32,
    /// Exponent bits of a double.
    pub r8nbe: i32,
    /// Largest int.
    pub i4ovr: i32,
    /// Smallest int as the Fortran runtime declares it.
    pub i4ovn: i32,
    /// Largest decimal exponent of a single.
    pub r4exp: i32,
    /// Largest decimal exponent of a double.
    pub r8exp: i32,
    /// Smallest decimal exponent of a single.
    pub r4exn: i32,
    /// Smallest decimal exponent of a double.
    pub r8exn: i32,
    /// Significant digits of a single.
    pub r4ncs: i32,
    /// Significant digits of a double.
    pub r8ncs: i32,
    /// Mantissa bits of a single.
    pub r4nbm: i32,
    /// Largest short.
    pub i2ovr: i16,
    /// Smallest short.
    pub i2ovn: i16,
}

/// Common `MAOVPCH`: machine name and Fortran formats (`maovpch_1_`); `getmaovpch().cnmmac`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Maovpch {
    /// Machine name.
    pub cnmmac: [u8; 16],
    /// Fortran format of a single.
    pub frmr4: [u8; 8],
    /// Fortran format of a double.
    pub frmr8: [u8; 8],
    /// Floating-point encoding name.
    pub cdcode: [u8; 8],
}

/// Common `MLGDRTL`: Legendre roots and Gauss weights (`mlgdrtl_1_`).
///
/// `rootab` and `hiltab` are Fortran `(465,2)` arrays (column 1 even degrees, column 2
/// odd); `hi0tab` holds the weight of the zero root; `getmlgdrtl().rootab[0]`.
pub struct Mlgdrtl {
    /// Positive Legendre roots, `(465, 2)`.
    pub rootab: &'static [f64],
    /// Gauss weights of those roots, `(465, 2)`.
    pub hiltab: &'static [f64],
    /// Gauss weight of the zero root per odd degree.
    pub hi0tab: &'static [f64],
}

/// Common `MMJCOBI`: Jacobi <-> canonical bases, Fortran `(496,2,4)` arrays (`mmjcobi_1_`).
///
/// `getmmjcobi().plgcan[0]`.
pub struct Mmjcobi {
    /// Jacobi to canonical coefficients, `(496, 2, 4)`.
    pub plgcan: &'static [f64],
    /// Canonical to Jacobi coefficients, `(496, 2, 4)`.
    pub canjac: &'static [f64],
}

/// Common `MMCMCNP`: the binomial coefficients `Cnp`, a Fortran `(61,61)` array
/// (`mmcmcnp_1_`); `getmmcmcnp().cnp[0] == 1.0`.
pub struct Mmcmcnp {
    /// Binomial coefficients, `(61, 61)`.
    pub cnp: &'static [f64],
}

/// One of the Gauss projection commons `MMAPGSS`, `MMAPGS0`, `MMAPGS1`, `MMAPGS2`: the
/// Jacobi values times the Gauss weights at the positive roots (`gslxj*`) and at the
/// zero root (`gsl0j*`); `getmmapgss().gslxj[0]`.
pub struct GaussProjection {
    /// Jacobi values times Gauss weights at the positive roots.
    pub gslxj: &'static [f64],
    /// The same at the zero root.
    pub gsl0j: &'static [f64],
}

/// Returns `MDNOMBR` (`AdvApp2Var_Data::Getmdnombr`); `getmdnombr().deuxpi`.
pub fn getmdnombr() -> &'static Mdnombr {
    // The literals are the native ones, not std::f64::consts: `pisur2` rounds to a
    // different double than FRAC_PI_2, and the port keeps that value.
    #[allow(clippy::approx_constant)]
    static MDNOMBR: Mdnombr = Mdnombr {
        pi: 3.141592653589793,
        deuxpi: 6.283185307179586,
        pisur2: 1.570796326794897,
        pis180: 0.017453292519943296,
        c180pi: 57.29577951308232,
        zero: 0.,
        one: 1.,
        a180: 180.,
        a360: 360.,
        a90: 90.,
    };
    &MDNOMBR
}

/// Returns `MINOMBR`, the integers `0..=1000` the kernels pass by address
/// (`AdvApp2Var_Data::Getminombr().nbr`); `getminombr()[1] == 1`.
pub fn getminombr() -> &'static [i32; 1001] {
    static MINOMBR: [i32; 1001] = identity_integers();
    &MINOMBR
}

const fn identity_integers() -> [i32; 1001] {
    let mut values: [i32; 1001] = [0; 1001];
    let mut index: usize = 0;
    while index < 1001 {
        values[index] = index as i32;
        index += 1;
    }
    values
}

/// Returns `MAOVPAR` (`AdvApp2Var_Data::Getmaovpar`); `getmaovpar().r8ovr`.
pub fn getmaovpar() -> &'static Maovpar {
    // The single-precision limits keep the native decimal spelling.
    #[allow(clippy::excessive_precision)]
    static MAOVPAR: Maovpar = Maovpar {
        r8und: 2.225073858507201e-307,
        r8ovr: 1.797693134862315e307,
        x4und: 1.17549435e-38,
        x4ovr: 3.40282347e38,
        r4und: 1.17549435e-38,
        r4ovr: 3.40282347e38,
        r4nbe: 8,
        r8nbm: 52,
        r8nbe: 8,
        i4ovr: 2147483647,
        i4ovn: -2147483647,
        r4exp: 38,
        r8exp: 308,
        r4exn: -37,
        r8exn: -307,
        r4ncs: 8,
        r8ncs: 17,
        r4nbm: 23,
        i2ovr: 32767,
        i2ovn: -32768,
    };
    &MAOVPAR
}

/// Returns `MAOVPCH` (`AdvApp2Var_Data::Getmaovpch`); `getmaovpch().cdcode`.
pub fn getmaovpch() -> &'static Maovpch {
    static MAOVPCH: Maovpch = Maovpch {
        cnmmac: *b"SGI_IRIS        ",
        frmr4: *b"(D15.6) ",
        frmr8: *b"(D23.16)",
        cdcode: *b"D_FLOAT ",
    };
    &MAOVPCH
}

/// Returns `MLGDRTL` (`AdvApp2Var_Data::Getmlgdrtl`); `getmlgdrtl().hiltab[0]`.
pub fn getmlgdrtl() -> &'static Mlgdrtl {
    static MLGDRTL: OnceLock<Mlgdrtl> = OnceLock::new();
    MLGDRTL.get_or_init(|| {
        let [rootab, hiltab, hi0tab] = split_block(tables::MLGDRTL, [930, 930, 31]);
        Mlgdrtl {
            rootab,
            hiltab,
            hi0tab,
        }
    })
}

/// Returns `MMJCOBI` (`AdvApp2Var_Data::Getmmjcobi`); `getmmjcobi().canjac[0]`.
pub fn getmmjcobi() -> &'static Mmjcobi {
    static MMJCOBI: OnceLock<Mmjcobi> = OnceLock::new();
    MMJCOBI.get_or_init(|| {
        let [plgcan, canjac] = split_block(tables::MMJCOBI, [3968, 3968]);
        Mmjcobi { plgcan, canjac }
    })
}

/// Returns `MMCMCNP` (`AdvApp2Var_Data::Getmmcmcnp`); `getmmcmcnp().cnp[61]`.
pub fn getmmcmcnp() -> &'static Mmcmcnp {
    static MMCMCNP: OnceLock<Mmcmcnp> = OnceLock::new();
    MMCMCNP.get_or_init(|| {
        let [cnp] = split_block(tables::MMCMCNP, [3721]);
        Mmcmcnp { cnp }
    })
}

/// Returns `MMAPGSS` (`AdvApp2Var_Data::Getmmapgss`); `getmmapgss().gsl0j[0]`.
pub fn getmmapgss() -> &'static GaussProjection {
    static MMAPGSS: OnceLock<GaussProjection> = OnceLock::new();
    MMAPGSS.get_or_init(|| gauss_projection(tables::MMAPGSS, 5017, 52))
}

/// Returns `MMAPGS0` (`AdvApp2Var_Data::Getmmapgs0`); `getmmapgs0().gslxj[0]`.
pub fn getmmapgs0() -> &'static GaussProjection {
    static MMAPGS0: OnceLock<GaussProjection> = OnceLock::new();
    MMAPGS0.get_or_init(|| gauss_projection(tables::MMAPGS0, 4761, 49))
}

/// Returns `MMAPGS1` (`AdvApp2Var_Data::Getmmapgs1`); `getmmapgs1().gslxj[0]`.
pub fn getmmapgs1() -> &'static GaussProjection {
    static MMAPGS1: OnceLock<GaussProjection> = OnceLock::new();
    MMAPGS1.get_or_init(|| gauss_projection(tables::MMAPGS1, 4505, 46))
}

/// Returns `MMAPGS2` (`AdvApp2Var_Data::Getmmapgs2`); `getmmapgs2().gslxj[0]`.
pub fn getmmapgs2() -> &'static GaussProjection {
    static MMAPGS2: OnceLock<GaussProjection> = OnceLock::new();
    MMAPGS2.get_or_init(|| gauss_projection(tables::MMAPGS2, 4249, 43))
}

fn gauss_projection(fragments: &[&[f64]], nbxj: usize, nb0j: usize) -> GaussProjection {
    let [gslxj, gsl0j] = split_block(fragments, [nbxj, nb0j]);
    GaussProjection { gslxj, gsl0j }
}

/// Flattens a block's fragments once and cuts it into the struct members, in the
/// order of the C struct that the f2c code overlays on the flat array.
fn split_block<const N: usize>(fragments: &[&[f64]], sizes: [usize; N]) -> [&'static [f64]; N] {
    let flat: &'static [f64] = Vec::leak(fragments.concat());
    assert_eq!(flat.len(), sizes.iter().sum::<usize>(), "common block size");
    let mut rest: &'static [f64] = flat;
    sizes.map(|size| {
        let (member, tail) = rest.split_at(size);
        rest = tail;
        member
    })
}

#[cfg(test)]
#[allow(clippy::excessive_precision, clippy::approx_constant)] // The pxx literals, verbatim.
mod tests {
    use super::*;

    #[test]
    fn blocks_have_the_native_member_lengths() {
        assert_eq!(getmlgdrtl().rootab.len(), 930);
        assert_eq!(getmlgdrtl().hi0tab.len(), 31);
        assert_eq!(getmmjcobi().canjac.len(), 3968);
        assert_eq!(getmmcmcnp().cnp.len(), 3721);
        assert_eq!(getmmapgs2().gsl0j.len(), 43);
    }

    #[test]
    fn first_and_last_literals_match_the_pxx_files() {
        assert_eq!(getmlgdrtl().rootab[0], 0.577350269189625764509148780501958);
        assert_eq!(getmmjcobi().plgcan[0], 0.70710678118654752440084436210485);
        assert_eq!(
            *getmmjcobi().canjac.last().unwrap(),
            -1.0284250949746758839876395691897e-18
        );
        assert_eq!(
            *getmmapgss().gsl0j.last().unwrap(),
            0.0407561407383435779483091365130057
        );
        assert_eq!(getminombr()[1000], 1000);
        assert_eq!(getmaovpar().r8ncs, 17);
        assert_eq!(getmdnombr().pi, std::f64::consts::PI);
    }
}
