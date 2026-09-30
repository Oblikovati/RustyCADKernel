// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The class functions of `Law`: blending laws (`MixBnd`, `MixTgt`), scaling laws
//! (`Scale`, `ScaleCub`) and the arc-length reparametrization law (`Reparametrize`).
use super::{LawBSpFunc, LawBSpline, LawFunction, LawInterpolate, LawLinear};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point3;

/// The flat knots of `knots` repeated by `mults`, and the Schoenberg points of
/// `degree` on them, as `MixBnd` and `MixTgt` build them.
fn schoenberg_setup(
    degree: i32,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
) -> Result<(Array1<'static, f64>, Array1<'static, f64>), Failure> {
    let mut flat: Vec<f64> = Vec::new();
    for i in mults.lower()..=mults.upper() {
        for _ in 1..=at(mults, i)? {
            flat.push(at(knots, i)?);
        }
    }
    let mut fk: Array1<'static, f64> = one_based(flat.len() as i32)?;
    fk.as_mut_slice().copy_from_slice(&flat);
    let mut par: Array1<'static, f64> = one_based(flat.len() as i32 - degree - 1)?;
    bsplclib::build_schoenberg_points(degree, &fk, &mut par)?;
    Ok((fk, par))
}

/// `BSplCLib::Interpolate` in dimension 1 without derivative conditions; the native
/// callers ignore the inversion flag.
fn interpolate_values(
    degree: i32,
    fk: &Array1<'_, f64>,
    par: &Array1<'_, f64>,
    pol: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let mut ord: Array1<'static, i32> = one_based(par.len() as i32)?;
    ord.init(0);
    let mut inversion_problem: i32 = 0;
    bsplclib::interpolate_flat(
        degree,
        fk,
        par,
        &ord,
        1,
        pol.as_mut_slice(),
        &mut inversion_problem,
    )
}

/// Law::MixBnd(Lin): a cubic law close to `lin` with null end derivatives, on the
/// knots `f, (3f+l)/4, (f+3l)/4, l`.
///
/// `let blend = mix_bnd_law(&linear)?; blend.value(0.5)?`.
pub fn mix_bnd_law(lin: &Handle<LawLinear>) -> Result<Handle<LawBSpFunc>, Failure> {
    let linear: &LawLinear = lin
        .get()
        .ok_or_else(|| errors::null_object("Law::MixBnd: null Law_Linear"))?;
    let (f, l) = linear.bounds()?;
    let mut knots: Array1<'static, f64> = one_based(4)?;
    for (i, knot) in (1..).zip([f, 0.75 * f + 0.25 * l, 0.25 * f + 0.75 * l, l]) {
        put(&mut knots, i, knot)?;
    }
    let mut mults: Array1<'static, i32> = one_based(4)?;
    for (i, mult) in (1..).zip([4, 1, 1, 4]) {
        put(&mut mults, i, mult)?;
    }
    let pol: Handle<HArray1<f64>> = mix_bnd(3, &knots, &mults, lin)?;
    let poles: Array1<'static, f64> = pol.get().map(|p| p.array1().clone()).unwrap_or_default();
    let curve = Handle::new(LawBSpline::new(&poles, &knots, &mults, 3, false)?);
    let law = LawBSpFunc::new();
    law.set_curve(&curve)?;
    Ok(Handle::new(law))
}

/// Law::MixBnd(Degree, Knots, Mults, Lin): the poles of the law of `degree` on the
/// knots that interpolates `lin` at the Schoenberg points, with the second and
/// next-to-last poles set equal to the end poles (null end derivatives) when there
/// are at least 4 poles. `let poles = mix_bnd(3, &knots, &mults, &linear)?;`
pub fn mix_bnd(
    degree: i32,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    lin: &Handle<LawLinear>,
) -> Result<Handle<HArray1<f64>>, Failure> {
    let linear: &LawLinear = lin
        .get()
        .ok_or_else(|| errors::null_object("Law::MixBnd: null Law_Linear"))?;
    let (fk, par) = schoenberg_setup(degree, knots, mults)?;
    let nbpol: i32 = par.len() as i32;
    let mut pol: Array1<'static, f64> = one_based(nbpol)?;
    for i in 1..=nbpol {
        put(&mut pol, i, linear.value(at(&par, i)?)?)?;
    }
    interpolate_values(degree, &fk, &par, &mut pol)?;
    if nbpol >= 4 {
        let (start, end) = (at(&pol, 1)?, at(&pol, nbpol)?);
        put(&mut pol, 2, start)?;
        put(&mut pol, nbpol - 1, end)?;
    }
    Ok(Handle::new(HArray1::from_owned(pol)))
}

/// The static `eval1` of `MixTgt`: 0 on the null side of the pivot, else the
/// `t (1 - t)^2` bump scaled to a unit derivative at the free end.
fn eval1(p: f64, first: f64, last: f64, piv: f64, nulr: bool) -> f64 {
    if (nulr && p >= piv) || (!nulr && p <= piv) {
        return 0.0;
    }
    let (span, from_end, to_piv) = if nulr {
        (piv - first, p - first, piv - p)
    } else {
        (last - piv, last - p, p - piv)
    };
    let mut a: f64 = span;
    a *= a;
    a = 1.0 / a;
    a *= from_end;
    a *= to_piv;
    a *= to_piv;
    a
}

/// Law::MixTgt: the poles of the law of `degree` that is null right of
/// `Knots(Index)` (left when `nul_on_the_right` is false) and a C1 `t (1 - t)^2` bump
/// on the other side. `let poles = mix_tgt(3, &knots, &mults, true, 2)?;`
pub fn mix_tgt(
    degree: i32,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    nul_on_the_right: bool,
    index: i32,
) -> Result<Handle<HArray1<f64>>, Failure> {
    let first: f64 = at(knots, knots.lower())?;
    let last: f64 = at(knots, knots.upper())?;
    let piv: f64 = at(knots, index)?;
    let (fk, par) = schoenberg_setup(degree, knots, mults)?;
    let nbpol: i32 = par.len() as i32;
    let mut pol: Array1<'static, f64> = one_based(nbpol)?;
    for i in 1..=nbpol {
        put(
            &mut pol,
            i,
            eval1(at(&par, i)?, first, last, piv, nul_on_the_right),
        )?;
    }
    interpolate_values(degree, &fk, &par, &mut pol)?;
    Ok(Handle::new(HArray1::from_owned(pol)))
}

/// Law::ScaleCub: the cubic law `VFirst, VFirst, 1, VLast, VLast` on the knots
/// `First, (First+Last)/2, Last`, with 1 in place of a missing end value.
/// `let law = scale_cub(0.0, 1.0, true, true, 0.5, 2.0)?;`
pub fn scale_cub(
    first: f64,
    last: f64,
    has_f: bool,
    has_l: bool,
    v_first: f64,
    v_last: f64,
) -> Result<Handle<LawBSpline>, Failure> {
    let milieu: f64 = 0.5 * (first + last);
    let start: f64 = if has_f { v_first } else { 1.0 };
    let end: f64 = if has_l { v_last } else { 1.0 };
    let mut pol: Array1<'static, f64> = one_based(5)?;
    pol.as_mut_slice()
        .copy_from_slice(&[start, start, 1.0, end, end]);
    let mut knot: Array1<'static, f64> = one_based(3)?;
    knot.as_mut_slice().copy_from_slice(&[first, milieu, last]);
    let mut mult: Array1<'static, i32> = one_based(3)?;
    mult.as_mut_slice().copy_from_slice(&[4, 1, 4]);
    Ok(Handle::new(LawBSpline::new(&pol, &knot, &mult, 3, false)?))
}

/// The cubic Bezier law of `Scale` from `from` at `a` to `to` at `b`.
fn scale_piece(a: f64, b: f64, from: f64, to: f64) -> Result<LawBSpline, Failure> {
    let mut pbs: Array1<'static, f64> = one_based(4)?;
    pbs.as_mut_slice().copy_from_slice(&[from, from, to, to]);
    let mut kbs: Array1<'static, f64> = one_based(2)?;
    kbs.as_mut_slice().copy_from_slice(&[a, b]);
    let mut mbs: Array1<'static, i32> = one_based(2)?;
    mbs.init(4);
    LawBSpline::new(&pbs, &kbs, &mbs, 3, false)
}

/// Law::Scale: a C1 cubic law equal to 1 at the middle of `[First, Last]`, `VFirst`
/// at `First` and `VLast` at `Last` (each with a null derivative) when given; it
/// interpolates the two end pieces at the Schoenberg points of the double middle knot.
/// `let law = scale(0.0, 1.0, true, false, 0.5, 1.0)?;`
pub fn scale(
    first: f64,
    last: f64,
    has_f: bool,
    has_l: bool,
    v_first: f64,
    v_last: f64,
) -> Result<Handle<LawBSpline>, Failure> {
    let milieu: f64 = 0.5 * (first + last);
    let mut fknot: Array1<'static, f64> = one_based(10)?;
    fknot.as_mut_slice().copy_from_slice(&[
        first, first, first, first, milieu, milieu, last, last, last, last,
    ]);
    let bs1: Option<LawBSpline> = if has_f {
        Some(scale_piece(first, milieu, v_first, 1.0)?)
    } else {
        None
    };
    let bs2: Option<LawBSpline> = if has_l {
        Some(scale_piece(milieu, last, 1.0, v_last)?)
    } else {
        None
    };
    let mut par: Array1<'static, f64> = one_based(6)?;
    bsplclib::build_schoenberg_points(3, &fknot, &mut par)?;
    let mut pol: Array1<'static, f64> = one_based(6)?;
    for i in 1..=6 {
        let p: f64 = at(&par, i)?;
        // The static eval2: the first piece before the middle, the last one after it.
        let value: f64 = match (&bs1, &bs2) {
            (Some(piece), _) if p < milieu => piece.value(p)?,
            (_, Some(piece)) if p > milieu => piece.value(p)?,
            _ => 1.0,
        };
        put(&mut pol, i, value)?;
    }
    interpolate_values(3, &fknot, &par, &mut pol)?;
    let mut knot: Array1<'static, f64> = one_based(3)?;
    knot.as_mut_slice().copy_from_slice(&[first, milieu, last]);
    let mut mult: Array1<'static, i32> = one_based(3)?;
    mult.as_mut_slice().copy_from_slice(&[4, 2, 4]);
    Ok(Handle::new(LawBSpline::new(&pol, &knot, &mult, 3, false)?))
}

/// The curve parameters and cumulated chord lengths at `2 * nb_points` equally
/// spaced parameters, from the end given by `rev`.
fn sample_lengths(
    curve: &dyn Adaptor3dCurve,
    rev: bool,
    nb_points: i32,
) -> Result<(Vec<f64>, Vec<f64>), Failure> {
    let (u1, u2) = (curve.first_parameter()?, curve.last_parameter()?);
    let count: i32 = 2 * nb_points;
    let (mut u, du): (f64, f64) = if rev {
        (u2, (u1 - u2) / f64::from(count - 1))
    } else {
        (u1, (u2 - u1) / f64::from(count - 1))
    };
    let mut p1: Point3 = curve.value(u)?;
    let (mut cumdist, mut ucourbe) = (Vec::new(), Vec::new());
    let mut length: f64 = 0.0;
    for _ in 1..=count {
        let p2: Point3 = curve.value(u)?;
        length += p2.distance(&p1);
        cumdist.push(length);
        ucourbe.push(u);
        u += du;
        p1 = p2;
    }
    if let Some(end) = ucourbe.last_mut() {
        *end = if rev { u1 } else { u2 };
    }
    Ok((cumdist, ucourbe))
}

/// One-based view of a sample list, so that the native indices read the same values
/// (index 0 raises Standard_OutOfRange, as the native checked access).
fn sample(values: &[f64], index: i32) -> Result<f64, Failure> {
    let slot: Option<&f64> = usize::try_from(index - 1).ok().and_then(|i| values.get(i));
    slot.copied().ok_or_else(|| {
        errors::out_of_range(&format!(
            "Law::Reparametrize: sample {index} outside 1..={}",
            values.len()
        ))
    })
}

/// The curve parameters at quasi-constant abscissa and the law parameters they
/// map to, the ends included.
fn abscissa_points(
    (cumdist, ucourbe): (&[f64], &[f64]),
    (first, last): (f64, f64),
    nb_points: i32,
) -> Result<(Array1<'static, f64>, Array1<'static, f64>), Failure> {
    let (mut point, mut param) = (one_based::<f64>(nb_points)?, one_based::<f64>(nb_points)?);
    let length: f64 = sample(cumdist, 2 * nb_points)?;
    let dcorde: f64 = length / f64::from(nb_points - 1);
    let fac: f64 = 1.0 / f64::from(nb_points - 1);
    let (mut corde, mut index): (f64, i32) = (dcorde, 1);
    put(&mut point, 1, sample(ucourbe, 1)?)?;
    put(&mut param, 1, first)?;
    put(&mut point, nb_points, sample(ucourbe, 2 * nb_points)?)?;
    put(&mut param, nb_points, last)?;
    for i in 2..nb_points {
        while sample(cumdist, index)? < corde {
            index += 1;
        }
        let (d0, d1) = (sample(cumdist, index - 1)?, sample(cumdist, index)?);
        let alpha: f64 = (corde - d0) / (d1 - d0);
        let (u0, u1) = (sample(ucourbe, index - 1)?, sample(ucourbe, index)?);
        put(&mut point, i, u0 + alpha * (u1 - u0))?;
        let weighted: f64 = f64::from(nb_points - i) * first + f64::from(i - 1) * last;
        put(&mut param, i, weighted * fac)?;
        corde = f64::from(i) * dcorde;
    }
    Ok((point, param))
}

/// Law::Reparametrize: the law from `[First, Last]` to the parameters of `curve`
/// at quasi-constant arc length, interpolating `nb_points` samples, with optional end
/// derivatives (negated when `rev`). Standard_Failure when the interpolation fails.
/// `let law = reparametrize(&curve, 0.0, 1.0, false, false, 0.0, 0.0, false, 20)?;`
#[allow(clippy::too_many_arguments)] // Mirrors the native nine-argument signature.
pub fn reparametrize(
    curve: &dyn Adaptor3dCurve,
    first: f64,
    last: f64,
    has_df: bool,
    has_dl: bool,
    d_first: f64,
    d_last: f64,
    rev: bool,
    nb_points: i32,
) -> Result<Handle<LawBSpline>, Failure> {
    let dd_first: f64 = if has_df && rev { -d_first } else { d_first };
    let dd_last: f64 = if has_dl && rev { -d_last } else { d_last };
    let (cumdist, ucourbe) = sample_lengths(curve, rev, nb_points)?;
    let (point, param) = abscissa_points((&cumdist, &ucourbe), (first, last), nb_points)?;
    let points = Handle::new(HArray1::from_owned(point));
    let params = Handle::new(HArray1::from_owned(param));
    let mut inter = LawInterpolate::with_parameters(&points, &params, false, 1.0e-9)?;
    if has_df || has_dl {
        let mut tgt: Array1<'static, f64> = one_based(nb_points)?;
        let flags =
            Handle::new(HArray1::with_value(1, nb_points, false).map_err(errors::array_bounds)?);
        let flag_array = flags
            .get()
            .ok_or_else(|| errors::null_object("Law::Reparametrize: flags"))?;
        if has_df {
            put(&mut flag_array.change_array1(), 1, true)?;
            put(&mut tgt, 1, dd_first)?;
        }
        if has_dl {
            put(&mut flag_array.change_array1(), nb_points, true)?;
            put(&mut tgt, nb_points, dd_last)?;
        }
        inter.load(&tgt, &flags)?;
    }
    inter.perform()?;
    if !inter.is_done() {
        return Err(Failure::with_message(
            "Standard_Failure",
            "Law::Reparametrize echec interpolation",
        ));
    }
    inter.curve()
}
