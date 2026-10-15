// Rust translation of the static LocAfter/LocBefore of LocOpe_CSIntersector.cxx and the
// Localize members of LocOpe_CurveShapeIntersector.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The search of the first franc intersection point (not EXTERNAL) after or before a
//! parameter or an index in a sequence of points sorted by parameter. Indices are
//! 1-based as natively; the output arguments are written as the native ones are, also
//! when no point is found.
use super::PntFace;
use rustycad_tkg3d::top_abs::Orientation;

/// The output arguments `Or`, `IndFrom` and `IndTo` of the Localize members.
pub(crate) struct Localized<'a> {
    pub(crate) or: &'a mut Orientation,
    pub(crate) ind_from: &'a mut i32,
    pub(crate) ind_to: &'a mut i32,
}

fn at(spt: &[PntFace], i: i32) -> &PntFace {
    &spt[(i - 1) as usize]
}

/// The forward walk from `ifirst`: groups of points closer than `tol` merge their
/// orientations; an EXTERNAL group is skipped.
fn walk_after(spt: &[PntFace], ifirst: i32, tol: f64, out: Localized<'_>) -> bool {
    let nbpoints = spt.len() as i32;
    if ifirst > nbpoints {
        return false;
    }
    let mut i = ifirst;
    *out.ind_from = ifirst;
    loop {
        *out.or = at(spt, i).orientation();
        let param = at(spt, i).parameter();
        i += 1;
        while i <= nbpoints && at(spt, i).parameter() - param <= tol {
            if *out.or != Orientation::External && *out.or != at(spt, i).orientation() {
                *out.or = Orientation::External;
            }
            i += 1;
        }
        if *out.or != Orientation::External {
            // on a une intersection franche
            *out.ind_to = i - 1;
            return true;
        }
        *out.ind_from = i;
        if i > nbpoints {
            return false;
        }
    }
}

/// The backward walk from `ifirst`, the mirror of [`walk_after`].
fn walk_before(spt: &[PntFace], ifirst: i32, tol: f64, out: Localized<'_>) -> bool {
    if ifirst < 1 {
        return false;
    }
    let mut i = ifirst;
    *out.ind_to = ifirst;
    loop {
        *out.or = at(spt, i).orientation();
        let param = at(spt, i).parameter();
        i -= 1;
        while i >= 1 && param - at(spt, i).parameter() <= tol {
            if *out.or != Orientation::External && *out.or != at(spt, i).orientation() {
                *out.or = Orientation::External;
            }
            i -= 1;
        }
        if *out.or != Orientation::External {
            // on a une intersection franche
            *out.ind_from = i + 1;
            return true;
        }
        *out.ind_to = i;
        if i < 1 {
            return false;
        }
    }
}

/// `LocAfter(Spt, From, Tol, ...)`: the first franc point at a parameter of at least
/// `from - tol`.
pub(crate) fn loc_after(spt: &[PntFace], from: f64, tol: f64, out: Localized<'_>) -> bool {
    let fmeps = from - tol;
    let ifirst = spt
        .iter()
        .position(|p| p.parameter() >= fmeps)
        .map_or(spt.len() as i32 + 1, |k| k as i32 + 1);
    walk_after(spt, ifirst, tol, out)
}

/// `LocBefore(Spt, From, Tol, ...)`: the last franc point at a parameter of at most
/// `from + tol`.
pub(crate) fn loc_before(spt: &[PntFace], from: f64, tol: f64, out: Localized<'_>) -> bool {
    let fpeps = from + tol;
    let ifirst = spt
        .iter()
        .rposition(|p| p.parameter() <= fpeps)
        .map_or(0, |k| k as i32 + 1);
    walk_before(spt, ifirst, tol, out)
}

/// `LocAfter(Spt, FromInd, Tol, ...)`: the first franc point after the index
/// `from_ind` (from the first point when `from_ind < 1`).
pub(crate) fn loc_after_index(
    spt: &[PntFace],
    from_ind: i32,
    tol: f64,
    out: Localized<'_>,
) -> bool {
    let nbpoints = spt.len() as i32;
    if from_ind >= nbpoints {
        return false;
    }
    let mut ifirst = 1;
    if from_ind >= 1 {
        let fmeps = at(spt, from_ind).parameter() - tol;
        ifirst = from_ind + 1;
        while ifirst <= nbpoints && !(at(spt, ifirst).parameter() >= fmeps) {
            ifirst += 1;
        }
    }
    walk_after(spt, ifirst, tol, out)
}

/// `LocalizeBefore(FromInd, ...)` of LocOpe_CurveShapeIntersector: the last franc point
/// before the index `from_ind` (from the last point when `from_ind` is past the end).
pub(crate) fn loc_before_index(
    spt: &[PntFace],
    from_ind: i32,
    tol: f64,
    out: Localized<'_>,
) -> bool {
    let nbpoints = spt.len() as i32;
    if from_ind <= 1 {
        return false;
    }
    let mut ifirst = nbpoints;
    if from_ind <= nbpoints {
        let fpeps = at(spt, from_ind).parameter() + tol;
        ifirst = from_ind - 1;
        while ifirst >= 1 && !(at(spt, ifirst).parameter() <= fpeps) {
            ifirst -= 1;
        }
    }
    walk_before(spt, ifirst, tol, out)
}
