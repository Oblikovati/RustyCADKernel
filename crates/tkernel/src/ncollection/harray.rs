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

//! Handle-shared arrays, corresponding to `NCollection_HArray1` and `NCollection_HArray2`.
//!
//! Natively these classes inherit both the array and `Standard_Transient`, so every
//! holder of an `occ::handle` edits the same storage. A [`Handle`](crate::standard::Handle)
//! lends only shared references, so the array sits behind a lock: [`HArray1::array1`]
//! and [`HArray1::change_array1`] return read and write guards that expose the complete
//! [`Array1`] API, exactly as the native `Array1()`/`ChangeArray1()` expose the base class.
use super::{Array1, Array2, ArrayError};
use crate::standard::{RuntimeType, RuntimeTyped, StandardTransient, Transient};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Mutex, OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// A shared, bounded one-dimensional array; `Handle::new(HArray1::<f64>::with_bounds(1, 3)?)`.
///
/// ```
/// use rustycad_tkernel::{ncollection::HArray1, standard::Handle};
/// let shared = Handle::new(HArray1::with_value(1, 3, 0.5).unwrap());
/// shared.get().unwrap().change_array1().set_value(2, 1.5).unwrap();
/// assert_eq!(*shared.get().unwrap().array1().value(2).unwrap(), 1.5);
/// ```
pub struct HArray1<T: Send + Sync + 'static> {
    base: StandardTransient,
    values: RwLock<Array1<'static, T>>,
}

/// A shared, bounded two-dimensional array; `Handle::new(HArray2::<f64>::with_bounds(1, 2, 1, 3)?)`.
///
/// ```
/// use rustycad_tkernel::ncollection::HArray2;
/// let grid = HArray2::with_value(1, 2, 1, 3, 7_i32).unwrap();
/// assert_eq!(grid.array2().upper_col(), 3);
/// ```
pub struct HArray2<T: Send + Sync + 'static> {
    base: StandardTransient,
    values: RwLock<Array2<'static, T>>,
}

impl<T: Send + Sync + 'static> Default for HArray1<T> {
    fn default() -> Self {
        Self::from_owned(Array1::default())
    }
}

impl<T: Send + Sync + 'static> HArray1<T> {
    /// Wraps an array without copying it; `HArray1::from_owned(Array1::<i32>::with_size(2))`.
    pub fn from_owned(values: Array1<'static, T>) -> Self {
        Self {
            base: StandardTransient::default(),
            values: RwLock::new(values),
        }
    }

    /// Read access to the wrapped array, as the native `Array1()`; `array.array1().len()`.
    pub fn array1(&self) -> RwLockReadGuard<'_, Array1<'static, T>> {
        self.values.read().unwrap_or_else(PoisonError::into_inner)
    }

    /// Write access seen by every handle holder, as the native `ChangeArray1()`;
    /// `array.change_array1().init(0)`.
    pub fn change_array1(&self) -> RwLockWriteGuard<'_, Array1<'static, T>> {
        self.values.write().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<T: Default + Send + Sync + 'static> HArray1<T> {
    /// Allocates default values at `lower..=upper`; `HArray1::<i32>::with_bounds(1, 3)?`.
    pub fn with_bounds(lower: i32, upper: i32) -> Result<Self, ArrayError> {
        Ok(Self::from_owned(Array1::with_bounds(lower, upper)?))
    }
}

impl<T: Default + Clone + Send + Sync + 'static> HArray1<T> {
    /// Allocates the bounds and fills every slot with `value`; `HArray1::with_value(1, 3, 0.5)?`.
    pub fn with_value(lower: i32, upper: i32, value: T) -> Result<Self, ArrayError> {
        let mut values = Array1::with_bounds(lower, upper)?;
        values.init(value);
        Ok(Self::from_owned(values))
    }

    /// The native C-array constructor: with `use_buffer` the array views the first
    /// `upper - lower + 1` values of a static buffer (detached on first write, see
    /// [`Array1::from_shared`]); without it the buffer is ignored and fresh default values
    /// are allocated, as natively. A buffer shorter than the bounds is an error instead of
    /// the native out-of-bounds read; `HArray1::from_buffer(&VALUES, 1, 3, true)?`.
    pub fn from_buffer(
        buffer: &'static [T],
        lower: i32,
        upper: i32,
        use_buffer: bool,
    ) -> Result<Self, ArrayError> {
        if !use_buffer {
            return Self::with_bounds(lower, upper);
        }
        let size = checked_extent(lower, upper)?;
        if buffer.len() < size {
            return Err(ArrayError::DimensionMismatch {
                expected: size,
                actual: buffer.len(),
            });
        }
        Ok(Self::from_owned(Array1::from_shared(
            &buffer[..size],
            lower,
        )))
    }
}

impl<T: Clone + Send + Sync + 'static> HArray1<T> {
    /// Copies an array and its bounds into a new shared array; `HArray1::from_array(&values)`.
    pub fn from_array(values: &Array1<'_, T>) -> Self {
        Self::from_owned(owned_copy(values))
    }
}

impl<T: Send + Sync + 'static> HArray2<T> {
    /// Wraps a rectangle without copying it; `HArray2::from_owned(Array2::<i32>::with_size(2, 2))`.
    pub fn from_owned(values: Array2<'static, T>) -> Self {
        Self {
            base: StandardTransient::default(),
            values: RwLock::new(values),
        }
    }

    /// Read access to the wrapped rectangle, as the native `Array2()`; `grid.array2().len()`.
    pub fn array2(&self) -> RwLockReadGuard<'_, Array2<'static, T>> {
        self.values.read().unwrap_or_else(PoisonError::into_inner)
    }

    /// Write access seen by every handle holder, as the native `ChangeArray2()`;
    /// `grid.change_array2().init(0)`.
    pub fn change_array2(&self) -> RwLockWriteGuard<'_, Array2<'static, T>> {
        self.values.write().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<T: Default + Send + Sync + 'static> HArray2<T> {
    /// Allocates default values in the inclusive rectangle; `HArray2::<i32>::with_bounds(1, 2, 1, 3)?`.
    pub fn with_bounds(rl: i32, ru: i32, cl: i32, cu: i32) -> Result<Self, ArrayError> {
        Ok(Self::from_owned(Array2::with_bounds(rl, ru, cl, cu)?))
    }
}

impl<T: Default + Clone + Send + Sync + 'static> HArray2<T> {
    /// Allocates the rectangle and fills it with `value`; `HArray2::with_value(1, 2, 1, 3, 0)?`.
    pub fn with_value(rl: i32, ru: i32, cl: i32, cu: i32, value: T) -> Result<Self, ArrayError> {
        let mut values = Array2::with_bounds(rl, ru, cl, cu)?;
        values.init(value);
        Ok(Self::from_owned(values))
    }
}

impl<T: Clone + Send + Sync + 'static> HArray2<T> {
    /// Copies a rectangle and its bounds into a new shared array; `HArray2::from_array(&grid)`.
    pub fn from_array(values: &Array2<'_, T>) -> Self {
        let mut copy = Array2::default();
        copy.assign(values);
        Self::from_owned(copy)
    }
}

// A copy that owns its storage even when `values` views a borrowed buffer, as the
// native copy constructor always allocates.
fn owned_copy<T: Clone>(values: &Array1<'_, T>) -> Array1<'static, T> {
    let mut copy = Array1::default();
    copy.assign(values);
    copy
}

fn checked_extent(lower: i32, upper: i32) -> Result<usize, ArrayError> {
    let size = i64::from(upper) - i64::from(lower) + 1;
    if size < 0 {
        return Err(ArrayError::Bounds { lower, upper });
    }
    Ok(size as usize)
}

// `DEFINE_STANDARD_RTTI_INLINE(NCollection_HArray1, Standard_Transient)` names every
// instantiation "NCollection_HArray1". A Rust generic cannot own a `static`, so each
// instantiation's descriptor is created once, leaked, and cached by its TypeId.
fn generic_static_type<C: 'static>(name: &'static str) -> &'static RuntimeType {
    static REGISTRY: OnceLock<Mutex<HashMap<TypeId, &'static RuntimeType>>> = OnceLock::new();
    let registry = REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    let mut types = registry.lock().unwrap_or_else(PoisonError::into_inner);
    types.entry(TypeId::of::<C>()).or_insert_with(|| {
        let parent = Some(<StandardTransient as RuntimeTyped>::static_type());
        Box::leak(Box::new(RuntimeType::new::<C>(name, parent)))
    })
}

macro_rules! impl_generic_transient {
    ($class:ident, $name:literal) => {
        impl<T: Send + Sync + 'static> RuntimeTyped for $class<T> {
            fn static_type() -> &'static RuntimeType {
                generic_static_type::<Self>($name)
            }
        }
        impl<T: Send + Sync + 'static> Transient for $class<T> {
            fn dynamic_type(&self) -> &'static RuntimeType {
                Self::static_type()
            }
            fn project(&self, requested: TypeId) -> Option<&dyn Any> {
                if requested == TypeId::of::<Self>() {
                    return Some(self);
                }
                self.base.project(requested)
            }
            fn transient_base(&self) -> &StandardTransient {
                &self.base
            }
        }
    };
}
impl_generic_transient!(HArray1, "NCollection_HArray1");
impl_generic_transient!(HArray2, "NCollection_HArray2");
