// Copyright (c) 2021 OPEN CASCADE SAS
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

//! Shared storage for Poly_ArrayOfNodes and Poly_ArrayOfUVNodes.
use rustycad_tkernel::{
    ncollection::{AliasedArray, ArrayValue},
    standard::Failure,
};
/// A runtime-precision coordinate array; `NodeArray::<3>::new(4)?`.
#[derive(Clone)]
pub struct NodeArray<'a, const N: usize> {
    storage: AliasedArray<'a>,
}
/// Three-dimensional mesh nodes; `ArrayOfNodes::new(3)?`.
pub type ArrayOfNodes<'a> = NodeArray<'a, 3>;
/// Two-dimensional parametric mesh nodes; `ArrayOfUvNodes::new(3)?`.
pub type ArrayOfUvNodes<'a> = NodeArray<'a, 2>;
impl<const N: usize> Default for NodeArray<'_, N> {
    fn default() -> Self {
        Self {
            storage: AliasedArray::empty(N * size_of::<f64>()).unwrap(),
        }
    }
}
impl<'a, const N: usize> NodeArray<'a, N>
where
    [f64; N]: ArrayValue,
    [f32; N]: ArrayValue,
{
    /// Allocates double-precision nodes; `ArrayOfNodes::new(2)?`.
    pub fn new(length: usize) -> Result<Self, Failure> {
        Ok(Self {
            storage: AliasedArray::new(N * size_of::<f64>(), length)?,
        })
    }
    /// Borrows double coordinates without copying; `ArrayOfNodes::from_double_slice(&mut [[0.0;3];2])?`.
    pub fn from_double_slice(nodes: &'a mut [[f64; N]]) -> Result<Self, Failure> {
        Ok(Self {
            storage: AliasedArray::from_slice(nodes)?,
        })
    }
    /// Borrows single coordinates without copying; `ArrayOfNodes::from_float_slice(&mut [[0.0_f32;3];2])?`.
    pub fn from_float_slice(nodes: &'a mut [[f32; N]]) -> Result<Self, Failure> {
        Ok(Self {
            storage: AliasedArray::from_slice(nodes)?,
        })
    }
    /// Reports the stored element precision; `nodes.is_double_precision()`.
    pub fn is_double_precision(&self) -> bool {
        self.storage.stride() == N * size_of::<f64>()
    }
    /// Changes precision only before allocation; `nodes.set_double_precision(false)?`.
    pub fn set_double_precision(&mut self, double: bool) -> Result<(), Failure> {
        if !self.storage.is_empty() {
            return Err(self.failure(
                "Standard_ProgramError",
                "SetDoublePrecision() should be called before allocation",
            ));
        }
        self.storage = AliasedArray::empty(
            N * if double {
                size_of::<f64>()
            } else {
                size_of::<f32>()
            },
        )?;
        Ok(())
    }
    /// Returns the number of nodes; `nodes.len()`.
    pub fn len(&self) -> usize {
        self.storage.len()
    }
    /// Tests the unallocated state; `nodes.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
    /// Reports ownership of the node buffer; `nodes.is_deletable()`.
    pub fn is_deletable(&self) -> bool {
        self.storage.is_deletable()
    }
    /// Resizes with optional preservation of existing coordinates; `nodes.resize(5,true)?`.
    pub fn resize(&mut self, length: usize, copy: bool) -> Result<(), Failure> {
        self.storage.resize(length, copy)
    }
    /// Copies equal-length arrays, converting precision when needed; `target.assign(&source)?`.
    pub fn assign(&mut self, source: &Self) -> Result<(), Failure> {
        if self.is_double_precision() == source.is_double_precision() {
            return self.storage.assign(&source.storage);
        }
        if self.len() != source.len() {
            return Err(self.failure(
                "Standard_DimensionMismatch",
                "Assign(), arrays have different sizes",
            ));
        }
        for index in 0..self.len() {
            self.set_coordinates(index, source.coordinates(index)?)?;
        }
        Ok(())
    }
    /// Takes the source's storage and precision, as `Move`. Natively the source keeps a
    /// non-owning view of the same bytes; Rust ownership forbids that alias, so the source
    /// is left empty with its precision; `target.move_from(&mut source)`.
    pub fn move_from(&mut self, source: &mut Self) {
        let mut empty = Self::default();
        // An unallocated array accepts either precision.
        let _ = empty.set_double_precision(source.is_double_precision());
        *self = std::mem::replace(source, empty);
    }
    /// Reads coordinates widened to double; `nodes.coordinates(0)?`.
    pub fn coordinates(&self, index: usize) -> Result<[f64; N], Failure> {
        if self.is_double_precision() {
            return Ok(*self.storage.value::<[f64; N]>(index)?);
        }
        Ok(self.storage.value::<[f32; N]>(index)?.map(f64::from))
    }
    /// Stores coordinates with the native float cast when applicable; `nodes.set_coordinates(0,[0.0;3])?`.
    pub fn set_coordinates(&mut self, index: usize, point: [f64; N]) -> Result<(), Failure> {
        if self.is_double_precision() {
            *self.storage.change_value(index)? = point;
        } else {
            *self.storage.change_value(index)? = point.map(|coordinate| coordinate as f32);
        }
        Ok(())
    }
    fn failure(&self, kind: &'static str, message: &str) -> Failure {
        let class = if N == 3 {
            "Poly_ArrayOfNodes"
        } else {
            "Poly_ArrayOfUVNodes"
        };
        Failure::with_type(kind, &format!("{class}::{message}"), "")
    }
}
