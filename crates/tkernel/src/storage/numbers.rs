// Copyright (c) 1996-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! The friend access of `Storage_Schema` to `Standard_Persistent::_typenum` and `_refnum`.
use crate::standard::StandardPersistent;
use std::sync::atomic::Ordering;

/// Plain reads and writes of the two storage counters of a persistent object.
pub(crate) trait PersistentNumbers {
    /// `_typenum`.
    fn type_number(&self) -> i32;
    /// `_typenum = value`.
    fn set_type_number(&self, value: i32);
    /// `_refnum`.
    fn reference_number(&self) -> i32;
    /// `_refnum = value`.
    fn set_reference_number(&self, value: i32);
}

impl PersistentNumbers for StandardPersistent {
    fn type_number(&self) -> i32 {
        self.type_num().load(Ordering::Relaxed)
    }

    fn set_type_number(&self, value: i32) {
        self.type_num().store(value, Ordering::Relaxed);
    }

    fn reference_number(&self) -> i32 {
        self.ref_num().load(Ordering::Relaxed)
    }

    fn set_reference_number(&self, value: i32) {
        self.ref_num().store(value, Ordering::Relaxed);
    }
}
