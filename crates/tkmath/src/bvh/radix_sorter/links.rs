// Created on: 2016-04-13
// Created by: Denis BOGOLEPOV
// Copyright (c) 2013-2016 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_RadixSorter.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH::RadixSorter`: MSD radix sort of (Morton code, primitive) links.
use rustycad_tkernel::osd::Parallel;
use rustycad_tkernel::stl;

/// `BVH_EncodedLink`: pair of Morton code and primitive ID.
pub type BvhEncodedLink = (u32, i32);

/// `BVH::BitPredicate`: true when the link's code has a 0 at the digit.
/// Example: `BitPredicate::new(0).evaluate(&(2, 0))`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitPredicate {
    bit: u32,
}
impl BitPredicate {
    /// Creates new radix sort predicate for bit `digit`; `BitPredicate::new(29)`.
    pub const fn new(digit: i32) -> Self {
        Self { bit: 1 << digit }
    }
    /// Returns predicate value (0-bit to the left side); `predicate.evaluate(&link)`.
    pub const fn evaluate(&self, link: &BvhEncodedLink) -> bool {
        link.0 & self.bit == 0
    }
}

/// `BVH::BitComparator`: binary-search helper testing only its first argument.
/// Upstream declares it but no algorithm uses it.
/// Example: `BitComparator::new(3).compare(&first, &second)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitComparator {
    bit: u32,
}
impl BitComparator {
    /// Creates new comparator for bit `digit`; `BitComparator::new(3)`.
    pub const fn new(digit: i32) -> Self {
        Self { bit: 1 << digit }
    }
    /// Checks the left value for the given bit; the right value is ignored.
    pub const fn compare(&self, first: &BvhEncodedLink, _second: &BvhEncodedLink) -> bool {
        first.0 & self.bit == 0
    }
}

/// `BVH::RadixSorter::Sort`: partitions on the digits 29..24 and sorts both
/// halves (concurrently when `is_parallel`), then finishes with sequential MSD
/// passes below digit 24. The halves are disjoint, so the result is identical
/// in parallel and serial mode.
/// Example: `sort_links(&mut links, 29, false)`.
pub fn sort_links(links: &mut [BvhEncodedLink], digit: i32, is_parallel: bool) {
    if digit < 24 {
        sort_sequential(links, digit);
        return;
    }
    let predicate = BitPredicate::new(digit);
    let offset = stl::partition(links, |link| predicate.evaluate(link));
    let (left, right) = links.split_at_mut(offset);
    let mut halves = [left, right];
    if !is_parallel {
        // `OSD_Parallel::For(0, 2, functor, true)` runs index 0 then index 1.
        for half in halves.iter_mut() {
            sort_links(half, digit - 1, is_parallel);
        }
        return;
    }
    // `OSD_Parallel::For(0, 2, functor)` locks at most one pool worker beside the caller;
    // nested sorts find the pool busy and run in their caller, as natively.
    Parallel::for_each(
        halves.iter_mut(),
        |half| sort_links(half, digit - 1, true),
        false,
        2,
    )
    .expect("sorting disjoint link halves raises no failure");
}

/// `BVH::RadixSorter::perform`: splits on each digit from high to low and
/// recurses into the 0-part before advancing through the 1-part.
fn sort_sequential(links: &mut [BvhEncodedLink], mut digit: i32) {
    let mut start = 0;
    while start != links.len() && digit >= 0 {
        let predicate = BitPredicate::new(digit);
        digit -= 1;
        let offset = start + stl::partition(&mut links[start..], |link| predicate.evaluate(link));
        sort_sequential(&mut links[start..offset], digit);
        start = offset;
    }
}
