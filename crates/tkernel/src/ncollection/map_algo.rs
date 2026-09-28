// Copyright (c) 2024 OPEN CASCADE SAS
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

//! OCCT set operations preserving operand traversal order and destination policy.
use super::{HashPolicy, Map};
use std::fmt::Debug;
/// Checks subset membership; `contains(&superset,&subset)`.
pub fn contains<K: Debug, H: HashPolicy<K>>(map: &Map<K, H>, other: &Map<K, H>) -> bool {
    map.len() >= other.len() && other.iter().all(|key| map.contains(key))
}
/// Replaces destination with both operands' keys; `union(&mut result,&left,&right)`.
pub fn union<K: Debug + Clone, H: HashPolicy<K> + Clone>(
    result: &mut Map<K, H>,
    left: &Map<K, H>,
    right: &Map<K, H>,
) {
    if std::ptr::eq(left, right) {
        result.assign(left);
        return;
    }
    result.clear(true);
    for key in left.iter().chain(right.iter()) {
        result.add(key.clone());
    }
}
/// Intersects in OCCT's smaller-operand traversal order; `intersection(&mut result,&left,&right)`.
pub fn intersection<K: Debug + Clone, H: HashPolicy<K> + Clone>(
    result: &mut Map<K, H>,
    left: &Map<K, H>,
    right: &Map<K, H>,
) {
    if std::ptr::eq(left, right) {
        result.assign(left);
        return;
    }
    result.clear(true);
    let (smaller, larger) = if left.len() < right.len() {
        (left, right)
    } else {
        (right, left)
    };
    for key in smaller.iter().filter(|key| larger.contains(key)) {
        result.add(key.clone());
    }
}
/// Copies the left map and removes the right keys; `subtraction(&mut result,&left,&right)`.
pub fn subtraction<K: Debug + Clone, H: HashPolicy<K> + Clone>(
    result: &mut Map<K, H>,
    left: &Map<K, H>,
    right: &Map<K, H>,
) {
    result.assign(left);
    for key in right {
        result.remove(key);
    }
}
/// Keeps keys exclusive to either operand; `difference(&mut result,&left,&right)`.
pub fn difference<K: Debug + Clone, H: HashPolicy<K>>(
    result: &mut Map<K, H>,
    left: &Map<K, H>,
    right: &Map<K, H>,
) {
    result.clear(true);
    let only_left = left.iter().filter(|key| !right.contains(key));
    let only_right = right.iter().filter(|key| !left.contains(key));
    for key in only_left.chain(only_right) {
        result.add(key.clone());
    }
}
