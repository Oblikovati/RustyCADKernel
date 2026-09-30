//! The approximation of an iso-curve of the function: `mma2fnc_`.
use super::curve_errors::{PieceErrors, mma1fer_, mma1noc_};
use super::curve_steps::{PieceTables, mma1cdi_, mma1cnt_, mma1fdi_, mma1jak_, mma1nop_};
use crate::adv_app2var::evaluator::AdvApp2VarEvaluatorFunc2Var;
use crate::adv_app2var::math_base::{mmapcmp_, mmfmca8_, mmjacan_, mmveps3_};
use crate::adv_app2var::sys_base::{mcrfill_within, mvriraz_};

/// The inputs of `mma2fnc_` in the native order.
pub struct IsoApproxRequest<'a> {
    /// `ndimen`: total dimension.
    pub ndimen: i32,
    /// `nbsesp`: number of subspaces.
    pub nbsesp: i32,
    /// `ndimse`: dimension of each subspace.
    pub ndimse: &'a [i32],
    /// `uvfonc`: the domain (U0, U1, V0, V1).
    pub uvfonc: [f64; 4],
    /// `tconst`: the value of the constant parameter.
    pub tconst: f64,
    /// `isofav`: 1 for an iso U = tconst, 2 for V = tconst.
    pub isofav: i32,
    /// `nbroot`: number of Legendre roots.
    pub nbroot: i32,
    /// `rootlg`: the Legendre roots.
    pub rootlg: &'a [f64],
    /// `iordre`: continuity order at the ends (-1..=2).
    pub iordre: i32,
    /// `ideriv`: order of the cross derivative approximated.
    pub ideriv: i32,
    /// `ndgjac`: degree of the Jacobi approximation.
    pub ndgjac: i32,
    /// `nbcrmx`: maximum number of pieces.
    pub nbcrmx: i32,
    /// `ncflim`: maximum number of coefficients per piece.
    pub ncflim: i32,
    /// `epsapr`: tolerance of each subspace.
    pub epsapr: &'a [f64],
}

/// The outputs of `mma2fnc_`: per piece the coefficient count, the canonical curve
/// (`(ncflim, ndimen, nbcrmx)`), the folded samples (`(0:nbroot/2, ndimen, nbcrmx)`),
/// the end constraints (`(ndimen, iordre+2, nbcrmx)`), the cut parameters and the
/// errors (`(nbsesp, nbcrmx)`); `nbcrbe` is the number of pieces made.
pub struct IsoApproxOutput<'a> {
    /// `ncoeff`.
    pub ncoeff: &'a mut [i32],
    /// `courbe`.
    pub courbe: &'a mut [f64],
    /// `nbcrbe`.
    pub nbcrbe: i32,
    /// `somtab`.
    pub somtab: &'a mut [f64],
    /// `diftab`.
    pub diftab: &'a mut [f64],
    /// `contr1`.
    pub contr1: &'a mut [f64],
    /// `contr2`.
    pub contr2: &'a mut [f64],
    /// `tabdec`: the `nbcrmx + 1` cut parameters.
    pub tabdec: &'a mut [f64],
    /// `errmax`.
    pub errmax: &'a mut [f64],
    /// `errmoy`.
    pub errmoy: &'a mut [f64],
}

/// The scratch zones of `mma2fnc_` (the five parts of its `wrkar`).
struct CurveScratch {
    ttable: Vec<f64>,
    fpntab: Vec<f64>,
    hermit: Vec<f64>,
    cgauss: Vec<f64>,
    crvjac: Vec<f64>,
}

/// Approximates the derivative `ideriv` across the iso `tconst` by up to `nbcrmx`
/// polynomial pieces, halving a piece whose tolerance is not met (`mma2fnc_`).
/// `iercod`: 0 done, -1 when the last allowed piece misses the tolerance, 1 for a
/// degenerate domain or a failed step, the `mma1fdi_` code (above 100) when the
/// evaluator stops; returns 0.
/// `mma2fnc_(&request, &f, &mut output, &mut ier)`.
pub fn mma2fnc_(
    req: &IsoApproxRequest<'_>,
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    out: &mut IsoApproxOutput<'_>,
    iercod: &mut i32,
) -> i32 {
    *iercod = 0;
    let ilong: i32 = req.ndimen * req.ncflim * req.nbcrmx;
    mvriraz_(ilong, out.courbe);
    let mut eps3: f64 = 0.;
    mmveps3_(&mut eps3);
    let uv: [f64; 4] = req.uvfonc;
    if (uv[1] - uv[0]).abs() < eps3 || (uv[3] - uv[2]).abs() < eps3 {
        *iercod = 1;
        return 0;
    }
    let mut work: CurveScratch = scratch(req);
    let mut uvpav: [f64; 4] = [0.; 4];
    match req.isofav {
        1 => {
            uvpav[0] = uv[0];
            uvpav[1] = uv[1];
            out.tabdec[0] = uv[2];
            out.tabdec[1] = uv[3];
        }
        2 => {
            out.tabdec[0] = uv[0];
            out.tabdec[1] = uv[1];
            uvpav[2] = uv[2];
            uvpav[3] = uv[3];
        }
        _ => {
            *iercod = 1;
            return 0;
        }
    }
    cut_until_done(req, foncnp, out, &mut work, &mut uvpav, iercod);
    0
}

/// The scratch sizes of `mma2fnc_`.
fn scratch(req: &IsoApproxRequest<'_>) -> CurveScratch {
    let (ndimen, nbroot, iordre) = (req.ndimen, req.nbroot, req.iordre);
    let isz1: i32 = nbroot + 2;
    let isz2: i32 = (ndimen * (nbroot + 2))
        .max(((iordre + 1) << 1) * nbroot)
        .max((((req.ncflim - 1) / 2 + 1) << 1) * ndimen);
    let isz3: i32 = ((iordre + 1) << 2) * (iordre + 1);
    let isz4: i32 = (nbroot / 2 + 1) * (req.ndgjac + 1 - ((iordre + 1) << 1));
    let isz5: i32 = (req.ndgjac + 1) * ndimen;
    let zone = |n: i32| vec![0.; n.max(0) as usize];
    CurveScratch {
        ttable: zone(isz1),
        fpntab: zone(isz2),
        hermit: zone(isz3),
        cgauss: zone(isz4),
        crvjac: zone(isz5),
    }
}

/// The piece loop of `mma2fnc_` (label L1000): approximate the next piece, keep
/// it when it meets the tolerance (or when no more cut is allowed), otherwise
/// insert the middle of the piece in `tabdec` and retry.
fn cut_until_done(
    req: &IsoApproxRequest<'_>,
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    out: &mut IsoApproxOutput<'_>,
    work: &mut CurveScratch,
    uvpav: &mut [f64; 4],
    iercod: &mut i32,
) {
    let mut nupil: i32 = 1;
    out.nbcrbe = 0;
    let mut ier: i32 = 0;
    while nupil - out.nbcrbe != 0 {
        let nbcrbe: usize = out.nbcrbe as usize;
        let piece: [f64; 2] = [out.tabdec[nbcrbe], out.tabdec[nbcrbe + 1]];
        if req.isofav == 1 {
            uvpav[2..4].copy_from_slice(&piece);
        } else {
            uvpav[0..2].copy_from_slice(&piece);
        }
        match approximate_piece(req, foncnp, out, work, uvpav, &mut ier, iercod) {
            PieceOutcome::Failed => return,
            PieceOutcome::Approximated => {}
        }
        if ier == 0 || (ier == -1 && nupil == req.nbcrmx) {
            if ier == -1 {
                *iercod = -1;
            }
            store_piece(req, out, work, uvpav);
            out.nbcrbe += 1;
        } else {
            let tmil: f64 = (out.tabdec[nbcrbe + 1] + out.tabdec[nbcrbe]) / 2.;
            let ideb: usize = nbcrbe + 1;
            mcrfill_within((nupil - out.nbcrbe) << 3, out.tabdec, ideb, ideb + 1);
            out.tabdec[ideb] = tmil;
            nupil += 1;
        }
    }
}

enum PieceOutcome {
    Approximated,
    Failed,
}

/// The views of piece `ncb1` (1-based) in the output tables.
fn piece_tables<'a>(
    req: &IsoApproxRequest<'_>,
    out: &'a mut IsoApproxOutput<'_>,
    ncb1: i32,
) -> PieceTables<'a> {
    let sblock: usize = ((req.nbroot / 2 + 1) * req.ndimen * (ncb1 - 1)) as usize;
    let cblock: usize = (req.ndimen * (req.iordre + 2) * (ncb1 - 1)) as usize;
    PieceTables {
        somtab: &mut out.somtab[sblock..],
        diftab: &mut out.diftab[sblock..],
        contr1: &mut out.contr1[cblock..],
        contr2: &mut out.contr2[cblock..],
    }
}

/// The steps of one piece: parameters, samples, constraint removal, Jacobi
/// projection, constraint addition, truncation. `ier` receives the `mma1fer_`
/// code; a failed step sets `iercod` and ends `mma2fnc_`.
fn approximate_piece(
    req: &IsoApproxRequest<'_>,
    foncnp: &dyn AdvApp2VarEvaluatorFunc2Var,
    out: &mut IsoApproxOutput<'_>,
    work: &mut CurveScratch,
    uvpav: &[f64; 4],
    ier: &mut i32,
    iercod: &mut i32,
) -> PieceOutcome {
    let ncb1: i32 = out.nbcrbe + 1;
    mma1nop_(
        req.nbroot,
        req.rootlg,
        uvpav,
        req.isofav,
        &mut work.ttable,
        ier,
    );
    if *ier > 0 {
        *iercod = 1;
        return PieceOutcome::Failed;
    }
    let mut piece: PieceTables<'_> = piece_tables(req, out, ncb1);
    mma1fdi_(
        req,
        uvpav,
        foncnp,
        &work.ttable,
        &mut work.fpntab,
        &mut piece,
        iercod,
    );
    if *iercod > 0 {
        return PieceOutcome::Failed;
    }
    if req.iordre >= 0 {
        mma1cdi_(req, &mut piece, &mut work.fpntab, &mut work.hermit, ier);
        if *ier > 0 {
            *iercod = 1;
            return PieceOutcome::Failed;
        }
    }
    mma1jak_(req, &piece, &mut work.cgauss, &mut work.crvjac, ier);
    if *ier > 0 {
        *iercod = 1;
        return PieceOutcome::Failed;
    }
    if req.iordre >= 0 {
        mma1cnt_(req, &piece, &work.hermit, &mut work.crvjac);
    }
    let column: usize = (req.nbsesp * (ncb1 - 1)) as usize;
    let mut errors = PieceErrors {
        errmax: &mut out.errmax[column..],
        errmoy: &mut out.errmoy[column..],
        ncoeff: 0,
    };
    mma1fer_(req, &mut work.crvjac, &mut work.fpntab, &mut errors, ier);
    out.ncoeff[(ncb1 - 1) as usize] = errors.ncoeff;
    if *ier > 0 {
        *iercod = 1;
        return PieceOutcome::Failed;
    }
    PieceOutcome::Approximated
}

/// Keeps the current piece: its Jacobi coefficients go to the canonical base on
/// [-1,1] in `courbe`, and every table of the piece is rescaled to its domain.
fn store_piece(
    req: &IsoApproxRequest<'_>,
    out: &mut IsoApproxOutput<'_>,
    work: &mut CurveScratch,
    uvpav: &[f64; 4],
) {
    let ncb1: i32 = out.nbcrbe + 1;
    let ndimen: i32 = req.ndimen;
    let ncoeff: i32 = out.ncoeff[(ncb1 - 1) as usize];
    let ncfja: i32 = req.ndgjac + 1;
    mmapcmp_(ndimen, ncfja, ncoeff, &work.crvjac, &mut work.fpntab);
    mvriraz_(ndimen * req.ncflim, &mut work.crvjac);
    let ndgre: i32 = ncoeff - 1;
    for nd in 1..=ndimen {
        let iptt: usize = (((nd - 1) << 1) * (ndgre / 2 + 1)) as usize;
        let jptt: usize = ((nd - 1) * ncoeff) as usize;
        mmjacan_(
            req.iordre,
            ndgre,
            &work.fpntab[iptt..],
            &mut work.crvjac[jptt..],
        );
    }
    let cblock: usize = (req.ncflim * ndimen * (ncb1 - 1)) as usize;
    mmfmca8_(
        ncoeff,
        ndimen,
        1,
        req.ncflim,
        ndimen,
        1,
        &work.crvjac,
        &mut out.courbe[cblock..],
    );
    rescale_piece(req, out, ncb1, ncoeff, uvpav);
}

/// The `mma1noc_` calls of the kept piece: constraints of each order, folded
/// samples and curve coefficients, from [-1,1]x[-1,1] to the piece's domain.
fn rescale_piece(
    req: &IsoApproxRequest<'_>,
    out: &mut IsoApproxOutput<'_>,
    ncb1: i32,
    ncoeff: i32,
    uvpav: &[f64; 4],
) {
    const UV11: [f64; 4] = [-1., 1., -1., 1.];
    let (ndimen, isofav, ideriv) = (req.ndimen, req.isofav, req.ideriv);
    let piece: PieceTables<'_> = piece_tables(req, out, ncb1);
    for ii in 0..=req.iordre {
        let start: usize = (ii * ndimen) as usize;
        mma1noc_(
            &UV11,
            ndimen,
            ii,
            &mut piece.contr1[start..],
            uvpav,
            isofav,
            ideriv,
        );
        mma1noc_(
            &UV11,
            ndimen,
            ii,
            &mut piece.contr2[start..],
            uvpav,
            isofav,
            ideriv,
        );
    }
    let ibid1: i32 = (req.nbroot / 2 + 1) * ndimen;
    mma1noc_(&UV11, ibid1, 0, piece.somtab, uvpav, isofav, ideriv);
    mma1noc_(&UV11, ibid1, 0, piece.diftab, uvpav, isofav, ideriv);
    let cblock: i32 = req.ncflim * ndimen * (ncb1 - 1);
    for nd in 1..=ndimen {
        let start: usize = (cblock + (nd - 1) * req.ncflim) as usize;
        mma1noc_(
            &UV11,
            ncoeff,
            0,
            &mut out.courbe[start..],
            uvpav,
            isofav,
            ideriv,
        );
    }
}
