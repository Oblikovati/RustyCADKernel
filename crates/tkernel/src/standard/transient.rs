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

//! Transient base with native atomic counter ordering and safe base projections.
use super::{Failure, RuntimeType, RuntimeTyped};
use std::{
    any::{Any, TypeId},
    sync::{
        OnceLock,
        atomic::{AtomicI32, Ordering, fence},
    },
};
/// Runtime object interface retained by every owning handle.
pub trait Transient: Any + Send + Sync {
    /// Dispatch the most-derived OCCT descriptor; `object.dynamic_type()`.
    fn dynamic_type(&self) -> &'static RuntimeType;
    /// Project an actual base subobject, without pointer reinterpreting; `object.project(TypeId::of::<Base>())`.
    fn project(&self, requested: TypeId) -> Option<&dyn Any>;
    /// Borrow the embedded reference-count base; `object.transient_base().ref_count()`.
    fn transient_base(&self) -> &StandardTransient;
}
/// Root object with native atomic reference-count behavior; `StandardTransient::default()`.
#[repr(C)]
#[derive(Default)]
pub struct StandardTransient {
    references: AtomicI32,
}
impl StandardTransient {
    /// Read the intrusive count; a standalone object has zero, `object.ref_count()`.
    pub fn ref_count(&self) -> i32 {
        self.references.load(Ordering::Relaxed)
    }
    /// Atomically acquire one native-style reference; `object.increment_ref_counter()`.
    pub fn increment_ref_counter(&self) {
        self.references.fetch_add(1, Ordering::Relaxed);
    }
    /// Release one reference using native release/acquire ordering; `object.decrement_ref_counter()`.
    pub fn decrement_ref_counter(&self) -> i32 {
        let remaining: i32 = self
            .references
            .fetch_sub(1, Ordering::Release)
            .wrapping_sub(1);
        if remaining == 0 {
            fence(Ordering::Acquire);
        }
        remaining
    }
    /// Reject unowned stack objects as OCCT This does; `object.this()?`.
    pub fn this(&self) -> Result<&Self, Failure> {
        if self.ref_count() == 0 {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "Transient reference count 0; expected a constructed handle-owned object",
                "",
            ));
        }
        Ok(self)
    }
}
impl Clone for StandardTransient {
    fn clone(&self) -> Self {
        Self::default()
    }
    fn clone_from(&mut self, _: &Self) { /* Native assignment preserves the counter. */
    }
}
impl RuntimeTyped for StandardTransient {
    fn static_type() -> &'static RuntimeType {
        static DESCRIPTOR: OnceLock<RuntimeType> = OnceLock::new();
        DESCRIPTOR.get_or_init(|| RuntimeType::new::<Self>("Standard_Transient", None))
    }
}
impl Transient for StandardTransient {
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        (requested == TypeId::of::<Self>()).then_some(self)
    }
    fn transient_base(&self) -> &StandardTransient {
        self
    }
}
/// Implement safe single inheritance through an actual embedded parent field.
/// Example: `impl_transient!(Derived, "Derived", Base, parent)`.
#[macro_export]
macro_rules! impl_transient {
    ($class:ty, $name:literal, $parent:ty, $field:ident) => {
        impl $crate::standard::RuntimeTyped for $class {
            fn static_type() -> &'static $crate::standard::RuntimeType {
                static DESCRIPTOR: std::sync::OnceLock<$crate::standard::RuntimeType> =
                    std::sync::OnceLock::new();
                DESCRIPTOR.get_or_init(|| {
                    $crate::standard::RuntimeType::new::<Self>(
                        $name,
                        Some(<$parent as $crate::standard::RuntimeTyped>::static_type()),
                    )
                })
            }
        }
        impl $crate::standard::Transient for $class {
            fn dynamic_type(&self) -> &'static $crate::standard::RuntimeType {
                <Self as $crate::standard::RuntimeTyped>::static_type()
            }
            fn project(&self, requested: std::any::TypeId) -> Option<&dyn std::any::Any> {
                if requested == std::any::TypeId::of::<Self>() {
                    return Some(self);
                }
                $crate::standard::Transient::project(&self.$field, requested)
            }
            fn transient_base(&self) -> &$crate::standard::StandardTransient {
                $crate::standard::Transient::transient_base(&self.$field)
            }
        }
    };
}
