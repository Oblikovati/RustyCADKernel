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
    let old: usize = map.len();
    for value in other {
        map.add(value);
    }
    old != map.len()
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
    for value in left.into_iter().chain(right) {
        map.add(value);
    }
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
    let old: usize = map.len();
    let mut copy: PackedMap<T> = PackedMap::default();
    for value in map.iter().filter(|value| other.contains(*value)) {
        copy.add(value);
    }
    *map = copy;
    old != map.len()
}
/// Visit the smaller operand first, choosing left on ties; `intersection(&mut result, &left, &right)`.
pub fn intersection<T: PackedInteger>(
    map: &mut PackedMap<T>,
    left: &PackedMap<T>,
    right: &PackedMap<T>,
) {
    map.clear();
    let (smaller, larger) = if left.len() > right.len() {
        (right, left)
    } else {
        (left, right)
    };
    for value in smaller.into_iter().filter(|value| larger.contains(*value)) {
        map.add(value);
    }
}
/// Remove all keys in the other set; `subtract(&mut left, &right)`.
pub fn subtract<T: PackedInteger>(map: &mut PackedMap<T>, other: &PackedMap<T>) -> bool {
    let old: usize = map.len();
    for value in other {
        map.remove(value);
    }
    old != map.len()
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
    let old: usize = map.len();
    for value in other {
        if !map.remove(value) {
            map.add(value);
        }
    }
    old != map.len()
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
    for value in left.into_iter().filter(|value| !right.contains(*value)) {
        map.add(value);
    }
    for value in right.into_iter().filter(|value| !left.contains(*value)) {
        map.add(value);
    }
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
