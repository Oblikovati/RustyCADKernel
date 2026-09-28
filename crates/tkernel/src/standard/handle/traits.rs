// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2023 OPEN CASCADE SAS
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

use super::{Handle, RuntimeTyped};
use std::{ops::Deref, sync::Arc};
impl<T: RuntimeTyped> Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
            .expect("Null handle; expected a live object for dereference")
    }
}
impl<T: RuntimeTyped, U: RuntimeTyped> PartialEq<Handle<U>> for Handle<T> {
    fn eq(&self, other: &Handle<U>) -> bool {
        match (&self.owner, &other.owner) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }
}
impl<T: RuntimeTyped> Eq for Handle<T> {}
impl<T: RuntimeTyped> PartialEq<*const T> for Handle<T> {
    fn eq(&self, other: &*const T) -> bool {
        self.as_ptr() == *other
    }
}
impl<T: RuntimeTyped> PartialEq<Handle<T>> for *const T {
    fn eq(&self, other: &Handle<T>) -> bool {
        *self == other.as_ptr()
    }
}
