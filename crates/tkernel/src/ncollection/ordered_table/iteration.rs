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

use super::OrderedItems;
impl<'a, K, V> OrderedItems<'a, K, V> {
    pub fn current_pair(&self) -> Option<(&'a K, &'a V)> {
        let node = self.slots.get(self.current?)?.as_ref()?;
        Some((&node.key, &node.value))
    }
    pub fn advance(&mut self) {
        self.current = self
            .current
            .and_then(|slot| self.slots[slot].as_ref()?.next);
    }
}
impl<'a, K, V> Iterator for OrderedItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        let pair = self.current_pair()?;
        self.advance();
        Some(pair)
    }
}
impl<K, V> Clone for OrderedItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            slots: self.slots,
            current: self.current,
        }
    }
}
impl<K, V> Default for OrderedItems<'_, K, V> {
    fn default() -> Self {
        Self {
            slots: &[],
            current: None,
        }
    }
}
impl<K, V> PartialEq for OrderedItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.current_pair().map(|(key, _)| key as *const K)
            == other.current_pair().map(|(key, _)| key as *const K)
    }
}
impl<K, V> Eq for OrderedItems<'_, K, V> {}
