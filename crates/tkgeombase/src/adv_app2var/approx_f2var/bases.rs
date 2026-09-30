//! The fixed bases of the patch approximation: `mma1her_` (Hermite polynomials),
//! `mma2roo_` (Legendre roots), `mma2jmx_` (Jacobi maxima) and `mmapptt_` (Gauss
//! projection tables).
use crate::adv_app2var::data::{GaussProjection, getmmapgs0, getmmapgs1, getmmapgs2, getmmapgss};
use crate::adv_app2var::f2c::F2cArrayMut;
use crate::adv_app2var::math_base::{XMAXJ_ORDER2, XMAXJ_ORDER4, XMAXJ_ORDER6, mmrtptt_};

/// The C0 Hermite basis on [-1,1]: each column holds the coefficients of one
/// polynomial (value at -1, value at +1).
const HERMIT_C0: [[f64; 2]; 2] = [[0.5, -0.5], [0.5, 0.5]];
/// The C1 Hermite basis (values, then first derivatives, at -1 and +1).
const HERMIT_C1: [[f64; 4]; 4] = [
    [0.5, -0.75, 0., 0.25],
    [0.5, 0.75, 0., -0.25],
    [0.25, -0.25, -0.25, 0.25],
    [-0.25, -0.25, 0.25, 0.25],
];
/// The C2 Hermite basis (values, first and second derivatives, at -1 and +1).
const HERMIT_C2: [[f64; 6]; 6] = [
    [0.5, -0.9375, 0., 0.625, 0., -0.1875],
    [0.5, 0.9375, 0., -0.625, 0., 0.1875],
    [0.3125, -0.4375, -0.375, 0.625, 0.0625, -0.1875],
    [-0.3125, -0.4375, 0.375, 0.625, -0.0625, -0.1875],
    [0.0625, -0.0625, -0.125, 0.125, 0.0625, -0.0625],
    [0.0625, 0.0625, -0.125, -0.125, 0.0625, 0.0625],
];

/// The canonical coefficients of the Hermite basis of continuity `iordre` on [-1,1]
/// into `hermit` (`(2*iordre+2, 2*iordre+2)`, one polynomial per column)
/// (`mma1her_`). `iercod` is 1 for an order outside 0..=2; returns 0.
/// `mma1her_(1, &mut hermit, &mut ier)`.
pub fn mma1her_(iordre: i32, hermit: &mut [f64], iercod: &mut i32) -> i32 {
    *iercod = 0;
    match iordre {
        0 => write_columns(&HERMIT_C0, hermit),
        1 => write_columns(&HERMIT_C1, hermit),
        2 => write_columns(&HERMIT_C2, hermit),
        _ => *iercod = 1,
    }
    0
}

fn write_columns<const N: usize>(table: &[[f64; N]; N], hermit: &mut [f64]) {
    let dim1: i32 = N as i32;
    let mut her = F2cArrayMut::new(hermit, dim1 + 1);
    for (col, column) in (1..).zip(table) {
        for (row, value) in (1..).zip(column) {
            her[row + col * dim1] = *value;
        }
    }
}

/// The `nbpntu` and `nbpntv` Legendre roots in increasing order into `urootl` and
/// `vrootl`, 0 in the middle of an odd count (`mma2roo_`); returns 0.
/// `mma2roo_(8, 10, &mut uroots, &mut vroots)`.
pub fn mma2roo_(nbpntu: i32, nbpntv: i32, urootl: &mut [f64], vrootl: &mut [f64]) -> i32 {
    legendre_roots(nbpntu, urootl);
    legendre_roots(nbpntv, vrootl);
    0
}

/// One direction of `mma2roo_`: the positive half from `mmrtptt_`, mirrored.
fn legendre_roots(nbpnt: i32, roots: &mut [f64]) {
    let mut roots = F2cArrayMut::new(roots, 1);
    mmrtptt_(nbpnt, roots.tail_mut((nbpnt + 1) / 2 + 1));
    for ii in 1..=nbpnt / 2 {
        roots[ii] = -roots[nbpnt - ii + 1];
    }
    if nbpnt % 2 == 1 {
        roots[nbpnt / 2 + 1] = 0.;
    }
}

/// The maxima on [-1,1] of the normalised Jacobi polynomials of order `iordre`
/// and degrees `2*(iordre+1)..=ndgjac` into `xjacmx` (`mma2jmx_`); nothing for an
/// order outside -1..=2, nor past the table of the order. Returns 0.
/// `mma2jmx_(20, 1, &mut xjacmx)`.
pub fn mma2jmx_(ndgjac: i32, iordre: i32, xjacmx: &mut [f64]) -> i32 {
    let numax: i32 = ndgjac - ((iordre + 1) << 1);
    let table: &[f64] = match iordre {
        -1 => {
            for ii in 0..=numax {
                let bid: f64 = (f64::from(ii) * 2. + 1.) / 2.;
                xjacmx[ii as usize] = bid.sqrt();
            }
            return 0;
        }
        0 => &XMAXJ_ORDER2,
        1 => &XMAXJ_ORDER4,
        2 => &XMAXJ_ORDER6,
        _ => return 0,
    };
    // Natively a degree past the table (57, 55 or 53 maxima) reads the static memory
    // after it; those entries are left unwritten here. Only a Context whose Jacobi
    // degree reaches that far calls it, and mmapptt_ then refuses the degree (31).
    for (slot, value) in xjacmx
        .iter_mut()
        .zip(table)
        .take((numax + 1).max(0) as usize)
    {
        *slot = *value;
    }
    0
}

/// The Gauss projection table of the Jacobi polynomials of order `jordre` and
/// degrees up to `ndgjac` on `nbpnts` points into `cgauss` (`(0:nbpnts/2, 0:ndgjac)`)
/// (`mmapptt_`). `iercod` is 11 for a point count other than 8, 10, 15, 20, 25, 30,
/// 40, 50 or 61, 21 for an order outside -1..=2 and 31 for a degree outside
/// `[2*(jordre+1), nbpnts)`; returns 0. `mmapptt_(20, 30, 1, &mut cgauss, &mut ier)`.
pub fn mmapptt_(
    ndgjac: i32,
    nbpnts: i32,
    jordre: i32,
    cgauss: &mut [f64],
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    let infdg: i32 = (jordre + 1) << 1;
    if ![8, 10, 15, 20, 25, 30, 40, 50, 61].contains(&nbpnts) {
        *iercod = 11;
        return 0;
    }
    if !(-1..=2).contains(&jordre) {
        *iercod = 21;
        return 0;
    }
    if ndgjac >= nbpnts || ndgjac < infdg {
        *iercod = 31;
        return 0;
    }
    let (table, mxjac): (&GaussProjection, i32) = match jordre {
        -1 => (getmmapgss(), ndgjac),
        0 => (getmmapgs0(), ndgjac - infdg),
        1 => (getmmapgs1(), ndgjac - infdg),
        _ => (getmmapgs2(), ndgjac - infdg),
    };
    let (iptdb, ipdb0) = table_starts(nbpnts, infdg);
    copy_projection(table, nbpnts, mxjac, [iptdb, ipdb0], cgauss);
    0
}

/// The first positions of the `nbpnts` block in the `gslxj*` and `gsl0j*` tables:
/// the sizes of the blocks of the smaller point counts, summed.
fn table_starts(nbpnts: i32, infdg: i32) -> (i32, i32) {
    let mut iptdb: i32 = 0;
    for (count, half) in [
        (8, 4),
        (10, 5),
        (15, 7),
        (20, 10),
        (25, 12),
        (30, 15),
        (40, 20),
        (50, 25),
    ] {
        if nbpnts > count {
            iptdb += (count - infdg) * half;
        }
    }
    let mut ipdb0: i32 = 1;
    if nbpnts > 15 {
        ipdb0 = ipdb0 + (14 - infdg) / 2 + 1;
    }
    if nbpnts > 25 {
        ipdb0 = ipdb0 + (24 - infdg) / 2 + 1;
    }
    (iptdb, ipdb0)
}

/// The copy loops of `mmapptt_` (labels L1000..L4000): the positive-root rows,
/// then for an odd count the zero-root row, zero at the odd degrees.
fn copy_projection(
    table: &GaussProjection,
    nbpnts: i32,
    mxjac: i32,
    starts: [i32; 2],
    cgauss: &mut [f64],
) {
    let dim1: i32 = nbpnts / 2 + 1;
    let half: usize = (nbpnts / 2) as usize;
    for kjac in 0..=mxjac {
        let iptt: i32 = starts[0] + kjac * (nbpnts / 2) + 1;
        let (source, target) = ((iptt - 1) as usize, (kjac * dim1 + 1) as usize);
        cgauss[target..target + half].copy_from_slice(&table.gslxj[source..source + half]);
    }
    if nbpnts % 2 != 1 {
        return;
    }
    for (iptt, kjac) in (starts[1]..).zip((0..=mxjac).step_by(2)) {
        cgauss[(kjac * dim1) as usize] = table.gsl0j[(iptt - 1) as usize];
    }
    for kjac in (1..=mxjac).step_by(2) {
        cgauss[(kjac * dim1) as usize] = 0.;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hermite_bases_fill_their_columns() {
        let (mut hermit, mut ier) = ([0.0; 16], 9);
        mma1her_(1, &mut hermit, &mut ier);
        assert_eq!((ier, &hermit[4..8]), (0, &[0.5, 0.75, 0.0, -0.25][..]));
        mma1her_(3, &mut hermit, &mut ier);
        assert_eq!(ier, 1);
    }

    #[test]
    fn roots_are_symmetric() {
        let (mut u, mut v) = ([0.0; 3], [0.0; 4]);
        mma2roo_(3, 4, &mut u, &mut v);
        assert_eq!((u[0], u[1]), (-u[2], 0.0));
        assert_eq!((v[0], v[1]), (-v[3], -v[2]));
    }

    #[test]
    fn projection_tables_check_their_arguments() {
        let (mut cgauss, mut ier) = ([0.0; 200], 0);
        mmapptt_(10, 9, 0, &mut cgauss, &mut ier);
        assert_eq!(ier, 11);
        mmapptt_(10, 15, 3, &mut cgauss, &mut ier);
        assert_eq!(ier, 21);
        mmapptt_(15, 15, 0, &mut cgauss, &mut ier);
        assert_eq!(ier, 31);
        mmapptt_(10, 15, 0, &mut cgauss, &mut ier);
        assert_eq!(ier, 0);
        let mut xjacmx: [f64; 3] = [0.0; 3];
        mma2jmx_(2, -1, &mut xjacmx);
        assert_eq!(xjacmx[0], 0.5f64.sqrt());
    }
}
