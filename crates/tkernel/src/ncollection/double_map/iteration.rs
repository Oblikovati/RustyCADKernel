// Copyright (c) 2002-2014 OPEN CASCADE SAS
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

use super::{Debug, DoubleMap, DoubleMapItems, DoubleMapValues, HashPolicy};
impl<A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> DoubleMap<A, B, H1, H2> {
    /// Borrows both keys in first-key bucket order; `for (first,second) in map.items() {}`.
    pub fn items(&self) -> DoubleMapItems<'_, A, B> {
        let mut iter = DoubleMapItems {
            slots: &self.slots,
            buckets: &self.first_buckets,
            bucket: 0,
            index: 0,
        };
        iter.seek();
        iter
    }
    /// Borrows second keys in first-key bucket order; `map.iter().count()`.
    pub fn iter(&self) -> DoubleMapValues<'_, A, B> {
        DoubleMapValues {
            pairs: self.items(),
        }
    }
    /// Returns an exhausted value iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> DoubleMapValues<'_, A, B> {
        DoubleMapValues {
            pairs: DoubleMapItems {
                slots: &self.slots,
                buckets: &self.first_buckets,
                bucket: self.first_buckets.len(),
                index: 0,
            },
        }
    }
}
impl<A, B> DoubleMapItems<'_, A, B> {
    pub(super) fn seek(&mut self) {
        while self.bucket < self.buckets.len() && self.index >= self.buckets[self.bucket].len() {
            self.bucket += 1;
            self.index = 0;
        }
    }
    pub(super) fn current(&self) -> Option<*const super::Pair<A, B>> {
        let slot = *self.buckets.get(self.bucket)?.get(self.index)?;
        Some(&**self.slots[slot].as_ref()? as *const super::Pair<A, B>)
    }
}
impl<'a, A, B> Iterator for DoubleMapItems<'a, A, B> {
    type Item = (&'a A, &'a B);
    fn next(&mut self) -> Option<Self::Item> {
        let slot = *self.buckets.get(self.bucket)?.get(self.index)?;
        let pair = self.slots[slot].as_ref()?;
        self.index += 1;
        self.seek();
        Some((&pair.first, &pair.second))
    }
}
impl<A, B> Clone for DoubleMapItems<'_, A, B> {
    fn clone(&self) -> Self {
        Self {
            slots: self.slots,
            buckets: self.buckets,
            bucket: self.bucket,
            index: self.index,
        }
    }
}
impl<A, B> PartialEq for DoubleMapItems<'_, A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.current() == other.current()
    }
}
impl<A, B> Eq for DoubleMapItems<'_, A, B> {}
impl<'a, A, B> Iterator for DoubleMapValues<'a, A, B> {
    type Item = &'a B;
    fn next(&mut self) -> Option<Self::Item> {
        self.pairs.next().map(|(_, second)| second)
    }
}
impl<A, B> Clone for DoubleMapValues<'_, A, B> {
    fn clone(&self) -> Self {
        Self {
            pairs: self.pairs.clone(),
        }
    }
}
impl<A, B> PartialEq for DoubleMapValues<'_, A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.pairs == other.pairs
    }
}
impl<A, B> Eq for DoubleMapValues<'_, A, B> {}
impl<'a, A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> IntoIterator
    for &'a DoubleMap<A, B, H1, H2>
{
    type Item = &'a B;
    type IntoIter = DoubleMapValues<'a, A, B>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
