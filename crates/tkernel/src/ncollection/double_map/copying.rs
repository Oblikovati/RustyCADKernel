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

use super::{Debug, DoubleMap, HashPolicy};
impl<A: Debug + Clone, B: Debug + Clone, H1: HashPolicy<A> + Clone, H2: HashPolicy<B> + Clone>
    DoubleMap<A, B, H1, H2>
{
    /// Copies bindings retaining destination policies and allocator; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) {
        self.clear(true);
        if source.is_empty() {
            return;
        }
        self.resize(source.len() - 1);
        for (first, second) in source.items() {
            self.insert_pair(first.clone(), second.clone());
        }
    }
}
impl<A: Debug + Clone, B: Debug + Clone, H1: HashPolicy<A> + Clone, H2: HashPolicy<B> + Clone> Clone
    for DoubleMap<A, B, H1, H2>
{
    fn clone(&self) -> Self {
        let mut copy = Self::with_allocator(
            self.first_policy.clone(),
            self.second_policy.clone(),
            self.bucket_count,
            self.resource.clone(),
        );
        copy.assign(self);
        copy
    }
}
