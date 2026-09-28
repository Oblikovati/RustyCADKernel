// Copyright (c) 2001-2024 OPEN CASCADE SAS
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

//! `NCollection_HSequence`: a sequence shared through handles.
//!
//! OCCT derives the class from both `NCollection_Sequence` and `Standard_Transient`, and
//! every handle holder edits the one sequence. Rust handles share `&` access only, so the
//! sequence sits behind a lock and is lent to closures (`sequence`, `change_sequence`).
use super::Sequence;
use crate::standard::{Failure, Handle, RuntimeType, RuntimeTyped, StandardTransient, Transient};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Mutex, OnceLock, PoisonError, RwLock},
};
/// Handle-shared sequence; `let items = Handle::new(HSequence::<i32>::default()); items.get().unwrap().append(1);`.
pub struct HSequence<T> {
    base: StandardTransient,
    items: RwLock<Sequence<T>>,
}
impl<T> Default for HSequence<T> {
    fn default() -> Self {
        Self {
            base: StandardTransient::default(),
            items: RwLock::new(Sequence::default()),
        }
    }
}
impl<T: Clone> HSequence<T> {
    /// Copies `source` and keeps its allocator, as the C++ copy of the base sequence does;
    /// `HSequence::from_sequence(&values)`.
    pub fn from_sequence(source: &Sequence<T>) -> Self {
        Self {
            base: StandardTransient::default(),
            items: RwLock::new(source.clone()),
        }
    }
    /// Appends and empties `source` (OCCT `Append(SequenceType&)`); `items.append_sequence(&mut more)`.
    pub fn append_sequence(&self, source: &mut Sequence<T>) {
        self.change_sequence(|items| items.append_sequence(source));
    }
}
impl<T> HSequence<T> {
    /// Lends the wrapped sequence for reading, as `Sequence()`; `items.sequence(|s| s.len())`.
    pub fn sequence<R>(&self, read: impl FnOnce(&Sequence<T>) -> R) -> R {
        read(&self.items.read().unwrap_or_else(PoisonError::into_inner))
    }
    /// Lends the wrapped sequence for editing, as `ChangeSequence()`;
    /// `items.change_sequence(|s| s.reverse())`.
    pub fn change_sequence<R>(&self, edit: impl FnOnce(&mut Sequence<T>) -> R) -> R {
        edit(&mut self.items.write().unwrap_or_else(PoisonError::into_inner))
    }
    /// Appends one item; `items.append(42)`.
    pub fn append(&self, value: T) {
        self.change_sequence(|items| {
            items.append(value);
        });
    }
}
impl<T: Clone + Send + Sync + 'static> HSequence<T> {
    /// Moves every item of the handled sequence to the end of this one, leaving it empty
    /// (OCCT `Append(const handle<T>&)`); appending a sequence to itself does nothing and a
    /// null handle reports `Standard_NullObject`; `items.append_handle(&others)?`.
    pub fn append_handle(&self, other: &Handle<HSequence<T>>) -> Result<(), Failure> {
        let Some(source) = other.get() else {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "NCollection_HSequence::Append null handle; expected a handled sequence",
                "",
            ));
        };
        if std::ptr::eq(source, self) {
            return Ok(());
        }
        source.change_sequence(|moved| self.append_sequence(moved));
        Ok(())
    }
}
/// One descriptor per item type, all named `NCollection_HSequence` like the C++ template's
/// `DEFINE_STANDARD_RTTI_INLINE`; a `static` in a generic function would be shared by every
/// instantiation, so the descriptors are interned by `TypeId`.
impl<T: Send + Sync + 'static> RuntimeTyped for HSequence<T> {
    fn static_type() -> &'static RuntimeType {
        static DESCRIPTORS: OnceLock<Mutex<HashMap<TypeId, &'static RuntimeType>>> =
            OnceLock::new();
        let mut known = DESCRIPTORS
            .get_or_init(Mutex::default)
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        known.entry(TypeId::of::<Self>()).or_insert_with(|| {
            Box::leak(Box::new(RuntimeType::new::<Self>(
                "NCollection_HSequence",
                Some(StandardTransient::static_type()),
            )))
        })
    }
}
impl<T: Send + Sync + 'static> Transient for HSequence<T> {
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        if requested == TypeId::of::<Self>() {
            return Some(self);
        }
        Transient::project(&self.base, requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        &self.base
    }
}
