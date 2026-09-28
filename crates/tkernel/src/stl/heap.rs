// Copyright (c) 2026 OPEN CASCADE SAS
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

//! libstdc++ `bits/stl_heap.h` helpers used by introsort's depth-limit fallback.
//! Distances stay signed because `(hole - 1) / 2` is evaluated at hole 0 upstream.

/// Sorts the whole slice with `__heap_select` + `__sort_heap` (`__partial_sort`
/// with middle == last, as `__introsort_loop` calls it).
pub(super) fn partial_sort<E: Copy>(values: &mut [E], less: &mut impl FnMut(&E, &E) -> bool) {
    make_heap(values, less);
    let mut last = values.len();
    while last > 1 {
        last -= 1;
        pop_heap(values, last, last, less);
    }
}

fn make_heap<E: Copy>(values: &mut [E], less: &mut impl FnMut(&E, &E) -> bool) {
    let length = values.len() as isize;
    if length < 2 {
        return;
    }
    let mut parent = (length - 2) / 2;
    loop {
        let value = values[parent as usize];
        adjust_heap(values, parent, length, value, less);
        if parent == 0 {
            return;
        }
        parent -= 1;
    }
}

fn pop_heap<E: Copy>(
    values: &mut [E],
    last: usize,
    result: usize,
    less: &mut impl FnMut(&E, &E) -> bool,
) {
    let value = values[result];
    values[result] = values[0];
    adjust_heap(values, 0, last as isize, value, less);
}

fn adjust_heap<E: Copy>(
    values: &mut [E],
    mut hole: isize,
    length: isize,
    value: E,
    less: &mut impl FnMut(&E, &E) -> bool,
) {
    let top = hole;
    let mut child = hole;
    while child < (length - 1) / 2 {
        child = 2 * (child + 1);
        if less(&values[child as usize], &values[child as usize - 1]) {
            child -= 1;
        }
        values[hole as usize] = values[child as usize];
        hole = child;
    }
    if length & 1 == 0 && child == (length - 2) / 2 {
        child = 2 * (child + 1);
        values[hole as usize] = values[child as usize - 1];
        hole = child - 1;
    }
    push_heap(values, hole, top, value, less);
}

fn push_heap<E: Copy>(
    values: &mut [E],
    mut hole: isize,
    top: isize,
    value: E,
    less: &mut impl FnMut(&E, &E) -> bool,
) {
    let mut parent = (hole - 1) / 2;
    while hole > top && less(&values[parent as usize], &value) {
        values[hole as usize] = values[parent as usize];
        hole = parent;
        parent = (hole - 1) / 2;
    }
    values[hole as usize] = value;
}
