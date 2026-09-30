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

//! Behaviour of `WeakHandle`, the non-owning raw pointer of the MAT graph: the cases
//! the doc example does not reach (a destroyed object, a null handle, clones).
use rustycad_tkernel::standard::{Handle, StandardTransient, WeakHandle};

fn owned_transient() -> Handle<StandardTransient> {
    Handle::new(StandardTransient::default())
}

#[test]
fn upgrade_of_a_destroyed_object_is_a_null_handle_but_the_pointer_is_not_null() {
    let owner: Handle<StandardTransient> = owned_transient();
    let pointer: WeakHandle<StandardTransient> = owner.downgrade();
    drop(owner);
    assert!(!pointer.is_null(), "C++ keeps a non-null dangling pointer");
    assert!(pointer.upgrade().is_null(), "the upgrade must not revive the object");
}

#[test]
fn downgrade_of_a_null_handle_is_a_null_pointer() {
    let null_handle: Handle<StandardTransient> = Handle::default();
    let pointer: WeakHandle<StandardTransient> = null_handle.downgrade();
    assert!(pointer.is_null());
    assert!(pointer.upgrade().is_null());
}

#[test]
fn pointers_never_change_the_reference_count() {
    let owner: Handle<StandardTransient> = owned_transient();
    let pointer: WeakHandle<StandardTransient> = owner.downgrade();
    let copy: WeakHandle<StandardTransient> = pointer.clone();
    assert_eq!(owner.ref_count(), 1);
    let upgraded: Handle<StandardTransient> = copy.upgrade();
    assert!(upgraded == owner);
    assert_eq!(owner.ref_count(), 2, "only the upgraded handle owns a reference");
}
