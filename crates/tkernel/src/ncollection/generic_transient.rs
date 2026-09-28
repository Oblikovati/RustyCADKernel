// Copyright (c) 2002-2023 OPEN CASCADE SAS
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

//! Runtime types for the handle-shared collection templates (`NCollection_HArray1/2`,
//! `NCollection_HSequence`), whose `DEFINE_STANDARD_RTTI_INLINE` gives every
//! instantiation the template's name and `Standard_Transient` as parent.
use crate::standard::{RuntimeType, RuntimeTyped, StandardTransient};
use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Mutex, OnceLock, PoisonError},
};

/// The descriptor of the instantiation `C`, named `name`, created once and cached.
///
/// A Rust generic cannot own a `static` per instantiation (a `static` in a generic
/// function is shared by all of them), so descriptors are leaked once and interned by
/// `TypeId`: `generic_static_type::<HArray1<f64>>("NCollection_HArray1")`.
pub(crate) fn generic_static_type<C: 'static>(name: &'static str) -> &'static RuntimeType {
    static REGISTRY: OnceLock<Mutex<HashMap<TypeId, &'static RuntimeType>>> = OnceLock::new();
    let registry = REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    let mut types = registry.lock().unwrap_or_else(PoisonError::into_inner);
    types.entry(TypeId::of::<C>()).or_insert_with(|| {
        let parent = Some(<StandardTransient as RuntimeTyped>::static_type());
        Box::leak(Box::new(RuntimeType::new::<C>(name, parent)))
    })
}

/// Implements `RuntimeTyped` and `Transient` for a generic handle-shared collection
/// `$class<T>` that stores its `StandardTransient` in a field named `base`.
macro_rules! impl_generic_transient {
    ($class:ident, $name:literal) => {
        impl<T: Send + Sync + 'static> $crate::standard::RuntimeTyped for $class<T> {
            fn static_type() -> &'static $crate::standard::RuntimeType {
                super::generic_transient::generic_static_type::<Self>($name)
            }
        }
        impl<T: Send + Sync + 'static> $crate::standard::Transient for $class<T> {
            fn dynamic_type(&self) -> &'static $crate::standard::RuntimeType {
                <Self as $crate::standard::RuntimeTyped>::static_type()
            }
            fn project(&self, requested: std::any::TypeId) -> Option<&dyn std::any::Any> {
                if requested == std::any::TypeId::of::<Self>() {
                    return Some(self);
                }
                $crate::standard::Transient::project(&self.base, requested)
            }
            fn transient_base(&self) -> &$crate::standard::StandardTransient {
                &self.base
            }
        }
    };
}
pub(crate) use impl_generic_transient;
