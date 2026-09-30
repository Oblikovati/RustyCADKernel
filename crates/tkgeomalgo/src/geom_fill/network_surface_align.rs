// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of GeomFill_NetworkSurface.cxx (sameKnotRange, KnotEntry,
// normalizedKnot, collectKnots, compactKnots, uniteKnots, alignSurfaces) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The alignment of the profile skin, the guide skin and the reference surface on one
//! knot basis.
use super::network_surface_skin::array;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::stl;
use rustycad_tkg3d::geom::GeomBSplineSurface;

/// `KnotEntry`: a candidate knot with its multiplicity.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct KnotEntry {
    parameter: f64,
    multiplicity: i32,
}

/// A knot vector with its multiplicities.
pub(crate) type KnotVector = (Array1<'static, f64>, Array1<'static, i32>);

/// `sameKnotRange`: the first knot vector's range when both start and end within
/// Precision::PConfusion() and the range is not empty.
fn same_knot_range(
    the_knots1: &Array1<'_, f64>,
    the_knots2: &Array1<'_, f64>,
) -> Option<(f64, f64)> {
    let (the_start, the_end) = (
        the_knots1[the_knots1.lower()],
        the_knots1[the_knots1.upper()],
    );
    let same: bool = (the_start - the_knots2[the_knots2.lower()]).abs() <= precision::p_confusion()
        && (the_end - the_knots2[the_knots2.upper()]).abs() <= precision::p_confusion()
        && the_start < the_end - precision::p_confusion();
    same.then_some((the_start, the_end))
}

/// `normalizedKnot`: a knot within Precision::PConfusion() of an end moved to it.
fn normalized_knot(the_start: f64, the_end: f64, the_knot: f64) -> f64 {
    if (the_knot - the_start).abs() <= precision::p_confusion() {
        return the_start;
    }
    if (the_knot - the_end).abs() <= precision::p_confusion() {
        return the_end;
    }
    the_knot
}

/// `collectKnots`: the knots of one basis, boundaries clamped, multiplicities at most
/// the degree (degree + 1 at a boundary).
fn collect_knots(
    the_degree: i32,
    (the_start, the_end): (f64, f64),
    (the_knots, the_mults): (&Array1<'_, f64>, &Array1<'_, i32>),
    the_entries: &mut Vec<KnotEntry>,
) {
    for i in the_knots.lower()..=the_knots.upper() {
        let a_knot: f64 = normalized_knot(the_start, the_end, the_knots[i]);
        let is_boundary: bool = a_knot == the_start || a_knot == the_end;
        let a_max_mult: i32 = if is_boundary {
            the_degree + 1
        } else {
            the_degree
        };
        the_entries.push(KnotEntry {
            parameter: a_knot,
            multiplicity: the_mults[the_mults.lower() + (i - the_knots.lower())].min(a_max_mult),
        });
    }
}

/// `compactKnots`: the entries sorted (libstdc++ `std::sort`), each group within
/// Precision::PConfusion() of its first knot kept once with its largest multiplicity.
fn compact_knots(the_entries: &mut [KnotEntry]) -> (Vec<f64>, Vec<i32>) {
    let (mut the_knots, mut the_mults): (Vec<f64>, Vec<i32>) = (Vec::new(), Vec::new());
    if the_entries.is_empty() {
        return (the_knots, the_mults);
    }
    stl::sort_by(the_entries, |left, right| left.parameter < right.parameter);
    let mut a_current: KnotEntry = the_entries[0];
    for an_entry in &the_entries[1..] {
        if (an_entry.parameter - a_current.parameter).abs() <= precision::p_confusion() {
            a_current.multiplicity = a_current.multiplicity.max(an_entry.multiplicity);
            continue;
        }
        the_knots.push(a_current.parameter);
        the_mults.push(a_current.multiplicity);
        a_current = *an_entry;
    }
    the_knots.push(a_current.parameter);
    the_mults.push(a_current.multiplicity);
    (the_knots, the_mults)
}

/// `uniteKnots`: the union of two knot vectors over their common range; None when the
/// ranges differ.
fn unite_knots(
    the_degree: i32,
    first: (&Array1<'_, f64>, &Array1<'_, i32>),
    second: (&Array1<'_, f64>, &Array1<'_, i32>),
) -> Result<Option<KnotVector>, Failure> {
    let Some(range) = same_knot_range(first.0, second.0) else {
        return Ok(None);
    };
    let mut a_entries: Vec<KnotEntry> = Vec::new();
    collect_knots(the_degree, range, first, &mut a_entries);
    collect_knots(the_degree, range, second, &mut a_entries);
    let (knots, mults) = compact_knots(&mut a_entries);
    if knots.is_empty() || mults.is_empty() {
        return Ok(None);
    }
    let (mut a_knots, mut a_mults): KnotVector = (array(knots.len())?, array(mults.len())?);
    for (i, (k, m)) in knots.iter().zip(&mults).enumerate() {
        a_knots[i as i32 + 1] = *k;
        a_mults[i as i32 + 1] = *m;
    }
    Ok(Some((a_knots, a_mults)))
}

// The U (or V) knots and multiplicities of a surface.
fn knots_of(surface: &GeomBSplineSurface, u: bool) -> KnotVector {
    if u {
        (surface.u_knots(), surface.u_multiplicities())
    } else {
        (surface.v_knots(), surface.v_multiplicities())
    }
}

// The union of the U (or V) knots of the three surfaces (profile and guide first,
// then the reference); None when a union fails.
fn united_direction(
    surfaces: [&GeomBSplineSurface; 3],
    degree: i32,
    u: bool,
) -> Result<Option<KnotVector>, Failure> {
    let (k0, m0) = knots_of(surfaces[0], u);
    let (k1, m1) = knots_of(surfaces[1], u);
    let Some((k12, m12)) = unite_knots(degree, (&k0, &m0), (&k1, &m1))? else {
        return Ok(None);
    };
    let (k2, m2) = knots_of(surfaces[2], u);
    unite_knots(degree, (&k12, &m12), (&k2, &m2))
}

/// `alignSurfaces`: the three surfaces raised to the largest degrees, then refined with
/// the union of their U knots and of their V knots (inserted with
/// Precision::PConfusion(), not added); false when a union fails.
pub(crate) fn align_surfaces(surfaces: [&Handle<GeomBSplineSurface>; 3]) -> Result<bool, Failure> {
    let null = || {
        Failure::with_type(
            "Standard_NullObject",
            "GeomFill_NetworkSurface - null surface",
            "",
        )
    };
    let s: [&GeomBSplineSurface; 3] = [
        surfaces[0].get().ok_or_else(null)?,
        surfaces[1].get().ok_or_else(null)?,
        surfaces[2].get().ok_or_else(null)?,
    ];
    let a_u_degree: i32 = s[0].u_degree().max(s[1].u_degree()).max(s[2].u_degree());
    let a_v_degree: i32 = s[0].v_degree().max(s[1].v_degree()).max(s[2].v_degree());
    for surface in s {
        surface.increase_degree(a_u_degree, a_v_degree)?;
    }
    let Some((u_knots, u_mults)) = united_direction(s, a_u_degree, true)? else {
        return Ok(false);
    };
    let Some((v_knots, v_mults)) = united_direction(s, a_v_degree, false)? else {
        return Ok(false);
    };
    for surface in s {
        surface.insert_u_knots(&u_knots, &u_mults, precision::p_confusion(), false)?;
    }
    for surface in s {
        surface.insert_v_knots(&v_knots, &v_mults, precision::p_confusion(), false)?;
    }
    Ok(true)
}
