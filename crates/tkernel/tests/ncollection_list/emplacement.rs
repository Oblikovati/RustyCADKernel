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

struct Pair(i32, f64);
struct MoveOnly(i32);
fn pair(value: &Pair, integer: i32, real: f64) {
    assert_eq!(value.0, integer);
    assert!((value.1 - real).abs() < 1e-10);
}
#[test]
fn NCollection_ListTest__EmplaceAppend() {
    let mut list = List::default();
    pair(list.emplace_append(|| Pair(42, 3.14)), 42, 3.14);
    assert_eq!(list.len(), 1);
    pair(list.emplace_append(|| Pair(100, 2.71)), 100, 2.71);
    assert_eq!(list.len(), 2);
    assert_eq!(list.first().unwrap().0, 42);
    assert_eq!(list.last().unwrap().0, 100);
}
#[test]
fn NCollection_ListTest__EmplacePrepend() {
    let mut list = List::default();
    pair(list.emplace_prepend(|| Pair(42, 3.14)), 42, 3.14);
    assert_eq!(list.len(), 1);
    pair(list.emplace_prepend(|| Pair(100, 2.71)), 100, 2.71);
    assert_eq!(list.len(), 2);
    assert_eq!(list.first().unwrap().0, 100);
    assert_eq!(list.last().unwrap().0, 42);
}
fn check_order(list: &List<Pair>) {
    assert_eq!(list.len(), 3);
    assert_eq!(
        list.iter().map(|value| value.0).collect::<Vec<_>>(),
        [10, 20, 30]
    );
}
#[test]
fn NCollection_ListTest__EmplaceBefore() {
    let mut list = List::default();
    list.emplace_append(|| Pair(10, 1.0));
    list.emplace_append(|| Pair(30, 3.0));
    let mut cursor = list.cursor();
    cursor.next(&list);
    pair(list.emplace_before(&cursor, || Pair(20, 2.0)), 20, 2.0);
    check_order(&list);
}
#[test]
fn NCollection_ListTest__EmplaceAfter() {
    let mut list = List::default();
    list.emplace_append(|| Pair(10, 1.0));
    list.emplace_append(|| Pair(30, 3.0));
    let cursor = list.cursor();
    pair(list.emplace_after(&cursor, || Pair(20, 2.0)), 20, 2.0);
    check_order(&list);
}
#[test]
fn NCollection_ListTest__EmplaceWithMoveOnlyType() {
    let mut list = List::default();
    assert_eq!(list.emplace_append(|| MoveOnly(42)).0, 42);
    assert_eq!(list.emplace_prepend(|| MoveOnly(100)).0, 100);
    assert_eq!(list.len(), 2);
    assert_eq!(list.first().unwrap().0, 100);
    assert_eq!(list.last().unwrap().0, 42);
}
