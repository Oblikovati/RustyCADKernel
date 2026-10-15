// Rust translation of the file statics ChFi3d_Purge, InsertAfter, RemoveSD and
// InsertBefore (ChFi3d_Builder_2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edition of the sequence of surface data of a contour.
use crate::chfi_ds::{CommonPoint, StripeHandle, SurfDataHandle, lock, shared};
use rustycad_tkernel::standard::Failure;
use std::sync::Arc;

/// The sequence of surface data of a contour, created when missing.
fn sequence(stripe: &StripeHandle) -> crate::chfi_ds::SurfDataSequenceHandle {
    let mut s = lock(stripe);
    s.change_set_of_surf_data()
        .get_or_insert_with(|| shared(Vec::new()))
        .clone()
}

/// The same surface data handle (both null, or the same object).
fn same(a: Option<&SurfDataHandle>, b: Option<&SurfDataHandle>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}

/// `ChFi3d_Purge(Stripe, SD, VRef, isfirst, ons, intf, intl)`: the end of the contour
/// marked, and the surface data at it removed (when its other side does not end on an arc,
/// or it twists) or cut back to the common point `vref` on the side `ons`.
pub(super) fn purge(
    stripe: &StripeHandle,
    sd: &SurfDataHandle,
    vref: &CommonPoint,
    isfirst: bool,
    ons: i32,
    (intf, intl): (&mut i32, &mut i32),
) {
    if isfirst {
        *intf = 1;
    } else {
        *intl = 1;
    }
    let opp = 3 - ons;
    let mut g = lock(sd);
    if !g.vertex(isfirst, opp).is_on_arc() || g.twist_on_s1() || g.twist_on_s2() {
        drop(g);
        let seq = sequence(stripe);
        let mut seq = lock(&seq);
        if isfirst {
            if !seq.is_empty() {
                seq.remove(0);
            }
        } else {
            seq.pop();
        }
        return;
    }
    if ons == 1 {
        g.change_index_of_s1(0);
    } else {
        g.change_index_of_s2(0);
    }
    *g.change_vertex(!isfirst, ons) = vref.clone();
    *g.change_vertex(isfirst, ons) = vref.clone();
    let fi = g.change_interference(ons);
    if isfirst {
        let l = fi.last_parameter();
        fi.set_first_parameter(l);
    } else {
        let f = fi.first_parameter();
        fi.set_last_parameter(f);
    }
    fi.set_line_index(0);
}

/// `InsertAfter(Stripe, Ref, Item)`: `item` inserted after `reference` (prepended when the
/// sequence is empty or there is no reference; Standard_Failure for the same data twice).
pub(super) fn insert_after(
    stripe: &StripeHandle,
    reference: Option<&SurfDataHandle>,
    item: &SurfDataHandle,
) -> Result<(), Failure> {
    if same(reference, Some(item)) {
        return Err(Failure::new("InsertAfter : twice the same surfdata."));
    }
    let seq = sequence(stripe);
    let mut seq = lock(&seq);
    if seq.is_empty() || reference.is_none() {
        seq.insert(0, item.clone());
    }
    if let Some(r) = reference
        && let Some(i) = seq.iter().position(|s| Arc::ptr_eq(s, r))
    {
        seq.insert(i + 1, item.clone());
    }
    Ok(())
}

/// `InsertBefore(Stripe, Ref, Item)`: `item` inserted before `reference` (appended when
/// the sequence is empty or there is no reference; Standard_Failure for the same data
/// twice).
pub(super) fn insert_before(
    stripe: &StripeHandle,
    reference: Option<&SurfDataHandle>,
    item: &SurfDataHandle,
) -> Result<(), Failure> {
    if same(reference, Some(item)) {
        return Err(Failure::new("InsertBefore : twice the same surfdata."));
    }
    let seq = sequence(stripe);
    let mut seq = lock(&seq);
    if seq.is_empty() || reference.is_none() {
        seq.push(item.clone());
    }
    if let Some(r) = reference
        && let Some(i) = seq.iter().position(|s| Arc::ptr_eq(s, r))
    {
        seq.insert(i, item.clone());
    }
    Ok(())
}

/// `RemoveSD(Stripe, Prev, Next)`: the surface data strictly between `prev` and `next`
/// removed (from the start without `prev`, to the end without `next`;
/// Standard_OutOfRange for a `prev` not in the sequence before `next`).
pub(super) fn remove_sd(
    stripe: &StripeHandle,
    prev: Option<&SurfDataHandle>,
    next: Option<&SurfDataHandle>,
) -> Result<(), Failure> {
    let seq = sequence(stripe);
    let mut seq = lock(&seq);
    if seq.is_empty() {
        return Ok(());
    }
    let (mut iprev, mut inext) = (0usize, 0usize);
    for (k, s) in seq.iter().enumerate() {
        let i = k + 1;
        if prev.is_some_and(|p| Arc::ptr_eq(s, p)) {
            iprev = i + 1;
        }
        if next.is_some_and(|n| Arc::ptr_eq(s, n)) {
            inext = i - 1;
            break;
        }
    }
    if prev.is_none() {
        iprev = 1;
    }
    if next.is_none() {
        inext = seq.len();
    }
    if iprev <= inext {
        if iprev == 0 {
            return Err(Failure::out_of_range("NCollection_Sequence::Remove"));
        }
        seq.drain(iprev - 1..inext);
    }
    Ok(())
}
