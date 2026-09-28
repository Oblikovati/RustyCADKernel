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

use super::{HashPolicy, OrderedTable};
type BorrowedNode<'a, K, V> = (&'a K, &'a mut V, Option<usize>);
pub(in crate::ncollection) struct OrderedItemsMut<'a, K, V> {
    entries: Vec<Option<BorrowedNode<'a, K, V>>>,
    current: Option<usize>,
}
impl<K, V, H: HashPolicy<K>> OrderedTable<K, V, H> {
    pub fn items_mut(&mut self) -> OrderedItemsMut<'_, K, V> {
        let entries = self
            .slots
            .iter_mut()
            .map(|slot| {
                slot.as_mut().map(|node| {
                    let node = &mut **node;
                    (&node.key, &mut node.value, node.next)
                })
            })
            .collect();
        OrderedItemsMut {
            entries,
            current: self.first,
        }
    }
}
impl<'a, K, V> Iterator for OrderedItemsMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        let (key, value, next) = self.entries.get_mut(self.current?)?.take()?;
        self.current = next;
        Some((key, value))
    }
}
