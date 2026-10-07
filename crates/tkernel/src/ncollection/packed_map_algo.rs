//! Native set algorithms over packed maps, including their change-return conventions.
use super::{PackedInteger, PackedMap};
/// Insert the other set's keys; `unite(&mut left, &right)` returns an extent change.
pub fn unite<T: PackedInteger>(map: &mut PackedMap<T>, other: &PackedMap<T>) -> bool {
    if other.is_empty() {
        return false;
    }
    if map.is_empty() {
        map.assign(other);
        return true;
    }
    let old: i32 = map.extent();
    add_keys_in_order(map, other);
    old != map.extent()
}
/// Replace with union using native left-then-right traversal; `union(&mut result, &left, &right)`.
pub fn union<T: PackedInteger>(map: &mut PackedMap<T>, left: &PackedMap<T>, right: &PackedMap<T>) {
    if left.is_empty() {
        map.assign(right);
        return;
    }
    if right.is_empty() {
        map.assign(left);
        return;
    }
    map.clear();
    add_keys_in_order(map, left);
    add_keys_in_order(map, right);
}
/// Keep common keys, rebuilding storage like OCCT; `intersect(&mut left, &right)`.
pub fn intersect<T: PackedInteger>(map: &mut PackedMap<T>, other: &PackedMap<T>) -> bool {
    if map.is_empty() {
        return false;
    }
    if other.is_empty() {
        map.clear();
        return true;
    }
    let old: i32 = map.extent();
    let mut copy: PackedMap<T> = PackedMap::default();
    copy_selected_keys(&mut copy, map, other, true);
    *map = copy;
    old != map.extent()
}
/// Visit the smaller operand first, choosing left on ties; `intersection(&mut result, &left, &right)`.
pub fn intersection<T: PackedInteger>(
    map: &mut PackedMap<T>,
    left: &PackedMap<T>,
    right: &PackedMap<T>,
) {
    map.clear();
    if left.is_empty() || right.is_empty() {
        return;
    }
    let (smaller, larger) = if left.extent() > right.extent() {
        (right, left)
    } else {
        (left, right)
    };
    copy_selected_keys(map, smaller, larger, true);
}
/// Remove all keys in the other set; `subtract(&mut left, &right)`.
pub fn subtract<T: PackedInteger>(map: &mut PackedMap<T>, other: &PackedMap<T>) -> bool {
    if map.is_empty() || other.is_empty() {
        return false;
    }
    let old: i32 = map.extent();
    visit_keys_in_order(other, |value| {
        map.remove(value);
    });
    old != map.extent()
}
/// Copy left then remove right; `subtraction(&mut result, &left, &right)`.
pub fn subtraction<T: PackedInteger>(
    map: &mut PackedMap<T>,
    left: &PackedMap<T>,
    right: &PackedMap<T>,
) {
    map.assign(left);
    subtract(map, right);
}
/// Toggle other keys and report an extent change, matching the native cardinality quirk.
/// Example: `{1,2}` differ `{2,3}` becomes `{1,3}` but returns false.
pub fn differ<T: PackedInteger>(map: &mut PackedMap<T>, other: &PackedMap<T>) -> bool {
    if other.is_empty() {
        return false;
    }
    if map.is_empty() {
        map.assign(other);
        return true;
    }
    let old: i32 = map.extent();
    visit_keys_in_order(other, |value| toggle_native_key(map, value));
    old != map.extent()
}
/// Retain keys exclusive to each operand; `difference(&mut result, &left, &right)`.
pub fn difference<T: PackedInteger>(
    map: &mut PackedMap<T>,
    left: &PackedMap<T>,
    right: &PackedMap<T>,
) {
    map.clear();
    if std::ptr::eq(left, right) {
        return;
    }
    copy_selected_keys(map, left, right, false);
    copy_selected_keys(map, right, left, false);
}
/// Check a common key, visiting the smaller operand; `has_intersection(&left, &right)`.
pub fn has_intersection<T: PackedInteger>(left: &PackedMap<T>, right: &PackedMap<T>) -> bool {
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if std::ptr::eq(left, right) {
        return true;
    }
    let (smaller, larger) = if left.extent() > right.extent() {
        (right, left)
    } else {
        (left, right)
    };
    has_common_key(smaller, larger)
}
fn has_common_key<T: PackedInteger>(smaller: &PackedMap<T>, larger: &PackedMap<T>) -> bool {
    let mut cursor = smaller.iter();
    while let Some(value) = cursor.key().copied() {
        if larger.contains(value) {
            return true;
        }
        cursor.advance();
    }
    false
}
/// Check every key in the second operand; `contains(&superset, &subset)`.
pub fn contains<T: PackedInteger>(map: &PackedMap<T>, other: &PackedMap<T>) -> bool {
    if std::ptr::eq(map, other) || other.is_empty() {
        return true;
    }
    // Pinned OCCT 3d097a0 compares narrowed signed Extent(), including its wrap behavior.
    if map.extent() < other.extent() {
        return false;
    }
    let mut cursor = other.iter();
    while let Some(value) = cursor.key().copied() {
        if !map.contains(value) {
            return false;
        }
        cursor.advance();
    }
    true
}
/// Check inclusion in the other operand; `is_subset(&subset, &superset)`.
pub fn is_subset<T: PackedInteger>(map: &PackedMap<T>, other: &PackedMap<T>) -> bool {
    contains(other, map)
}
/// Compare contents independent of bucket order; `is_equal(&first, &second)`.
pub fn is_equal<T: PackedInteger>(map: &PackedMap<T>, other: &PackedMap<T>) -> bool {
    std::ptr::eq(map, other) || (map.extent() == other.extent() && contains(map, other))
}

// OCCT 3d097a0 performs each mutation before Next(); Rust Iterator::next pre-advances.
fn visit_keys_in_order<T: PackedInteger>(source: &PackedMap<T>, mut apply: impl FnMut(T)) {
    let mut cursor = source.iter();
    while let Some(value) = cursor.key().copied() {
        apply(value);
        cursor.advance();
    }
}
fn add_keys_in_order<T: PackedInteger>(map: &mut PackedMap<T>, source: &PackedMap<T>) {
    visit_keys_in_order(source, |value| {
        map.add(value);
    });
}
fn copy_selected_keys<T: PackedInteger>(
    map: &mut PackedMap<T>,
    source: &PackedMap<T>,
    other: &PackedMap<T>,
    keep_common: bool,
) {
    visit_keys_in_order(source, |value| {
        if other.contains(value) == keep_common {
            map.add(value);
        }
    });
}
fn toggle_native_key<T: PackedInteger>(map: &mut PackedMap<T>, value: T) {
    if map.contains(value) {
        map.remove(value);
        return;
    }
    map.add(value);
}
