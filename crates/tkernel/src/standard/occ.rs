// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2025 OPEN CASCADE SAS
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

//! The `occ::` helpers of Standard_Handle.hxx and Standard_Type.hxx.
//!
//! C++ overloads each helper on a raw `const Standard_Transient*` and on a handle. A
//! raw pointer is an `Option<&dyn Transient>` here (`None` is `nullptr`); the handle
//! forms carry a `_handle` suffix because Rust has no overloading.
use super::{Handle, HandleView, RuntimeType, RuntimeTyped, Transient};

/// `occ::is_kind<T>(ptr)`: the object is not null and is a `T` or inherits from it;
/// `is_kind::<StandardTransient>(Some(&object))`.
pub fn is_kind<T: RuntimeTyped>(object: Option<&dyn Transient>) -> bool {
    object.is_some_and(|object| object.is_kind(T::static_type()))
}
/// `occ::is_kind<T>(handle)`; `is_kind_handle::<StandardTransient, _>(&handle)`.
pub fn is_kind_handle<T: RuntimeTyped, B: RuntimeTyped>(object: &Handle<B>) -> bool {
    !object.is_null() && object.is_kind(T::static_type())
}
/// `occ::is_instance<T>(ptr)`: the object is not null and exactly a `T`;
/// `is_instance::<StandardTransient>(Some(&object))`.
pub fn is_instance<T: RuntimeTyped>(object: Option<&dyn Transient>) -> bool {
    object.is_some_and(|object| object.is_instance(T::static_type()))
}
/// `occ::is_instance<T>(handle)`; `is_instance_handle::<StandardTransient, _>(&handle)`.
pub fn is_instance_handle<T: RuntimeTyped, B: RuntimeTyped>(object: &Handle<B>) -> bool {
    !object.is_null() && object.is_instance(T::static_type())
}
/// `occ::type_of<T>()`, the same descriptor as `STANDARD_TYPE(T)`;
/// `type_of::<StandardTransient>().name() == "Standard_Transient"`.
pub fn type_of<T: RuntimeTyped>() -> &'static RuntimeType {
    T::static_type()
}
/// `occ::down_cast<TDerived>(handle)`; `down_cast::<Derived, _>(&base)`.
pub fn down_cast<D: RuntimeTyped, B: RuntimeTyped>(object: &Handle<B>) -> Handle<D> {
    Handle::<D>::down_cast(object)
}
/// `occ::down_cast<TDerived>(ptr)` for an object a handle owns (a view is the pointer);
/// `down_cast_view::<Derived, _>(&base.view())`.
pub fn down_cast_view<D: RuntimeTyped, B: RuntimeTyped>(object: &HandleView<'_, B>) -> Handle<D> {
    Handle::<D>::from_view(object)
}
