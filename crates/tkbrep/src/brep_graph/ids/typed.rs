// Copyright (c) 2026 OPEN CASCADE SAS
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
// Rust translation of the `Typed<Kind>` templates of ModelingData/TKBRep/BRepGraph/
// BRepGraph_NodeId.hxx, BRepGraph_RefId.hxx and BRepGraphInc/BRepGraphInc_RepId.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The compile-time typed ids shared by the node, reference and representation families.
//!
//! `BRepGraph_NodeId::Typed<Kind::Face>` is [`TypedId<FaceTag>`](TypedId): a `u32` index
//! and a zero-sized kind tag. The tag names its family ([`IdFamily`]) and its raw kind,
//! so the conversions, the kind checks and the hashes are written once per family
//! (docs/topology-design.md, section 11.1).
use crate::brep_graph::BRepGraph;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// One id family: nodes, references or representations.
pub trait IdFamily: 'static {
    /// The untyped `(Kind, Index)` id of the family.
    type Untyped: Copy;
    /// The untyped id of `kind` and `index`.
    fn untyped(kind: i32, index: u32) -> Self::Untyped;
    /// `IsValidKind(static_cast<Kind>(kind))`.
    fn valid_kind(kind: i32) -> bool;
    /// `std::hash<Typed<Kind>>` of the family.
    fn typed_hash(kind: i32, index: u32) -> usize;
    /// `Typed::IsRemoved(graph)`: the untyped `IsRemoved`.
    fn is_removed(graph: &BRepGraph, kind: i32, index: u32) -> bool;
}

/// A family whose typed ids also have `operator+`, `operator-` and `IsOwned`.
pub trait OwnedFamily: IdFamily {
    /// `Typed::IsOwned(graph)`: the untyped `IsOwned`.
    fn is_owned(graph: &BRepGraph, kind: i32, index: u32) -> bool;
}

/// The zero-sized tag of one kind, the `TheKind` template argument.
pub trait KindTag: Copy + Default + fmt::Debug + Send + Sync + 'static {
    /// The id family of the kind.
    type Family: IdFamily;
    /// The enumerator value of the kind.
    const RAW_KIND: i32;
    /// The OCCT alias of the typed id (`BRepGraph_FaceId`), for diagnostics.
    const NAME: &'static str;
}

/// A provider of a dense id range, the argument of `IsValidIn`: anything with `Nb()` or
/// `Size()`; `FaceId::new(2).is_valid_in(&vec![1, 2, 3])`.
pub trait CountProvider {
    /// `Nb()` or `Size()`.
    fn count(&self) -> u32;
}

impl<T> CountProvider for [T] {
    fn count(&self) -> u32 {
        self.len() as u32
    }
}

impl<T> CountProvider for Vec<T> {
    fn count(&self) -> u32 {
        self.len() as u32
    }
}

/// `BRepGraph_NodeId::Typed<Kind>` (and the reference and representation twins): an index
/// with a compile-time kind; the default is the invalid id.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{FaceId, NodeId, NodeKind};
/// let face = FaceId::new(3);
/// assert_eq!(NodeId::from(face), NodeId::new(NodeKind::FACE, 3));
/// assert!(!FaceId::default().is_valid());
/// ```
pub struct TypedId<T> {
    /// `Index`: the slot in the per-kind vector (`u32::MAX` when invalid).
    pub index: u32,
    tag: PhantomData<T>,
}

impl<T: KindTag> TypedId<T> {
    /// `THE_START_INDEX`.
    pub const START_INDEX: u32 = 0;
    /// `THE_INVALID_INDEX`.
    pub const INVALID_INDEX: u32 = u32::MAX;

    /// `explicit Typed(uint32_t)`; `FaceId::new(4)`.
    pub const fn new(index: u32) -> Self {
        Self {
            index,
            tag: PhantomData,
        }
    }

    /// `Start()`: the first id of the dense sequence; `FaceId::start().index == 0`.
    pub const fn start() -> Self {
        Self::new(Self::START_INDEX)
    }

    /// `Invalid()`; `FaceId::invalid()`.
    pub const fn invalid() -> Self {
        Self::new(Self::INVALID_INDEX)
    }

    /// `IsValid()`: a supported kind and an allocated index; `FaceId::new(0).is_valid()`.
    pub fn is_valid(self) -> bool {
        T::Family::valid_kind(T::RAW_KIND) && self.index != Self::INVALID_INDEX
    }

    /// `IsValid(theMaxCount)`: valid and below `max_count`; `FaceId::new(2).is_valid_for(3)`.
    pub fn is_valid_for(self, max_count: u32) -> bool {
        self.is_valid() && self.index < max_count
    }

    /// `IsValidIn(theProvider)`: valid within the provider's `Nb()`/`Size()`.
    pub fn is_valid_in(self, provider: &(impl CountProvider + ?Sized)) -> bool {
        self.is_valid_for(provider.count())
    }

    /// `operator BRepGraph_NodeId()` (or the reference and representation id).
    pub fn untyped(self) -> <T::Family as IdFamily>::Untyped {
        T::Family::untyped(T::RAW_KIND, self.index)
    }

    /// `++id`: advances the index and returns the new id (the index wraps as the native
    /// `uint32_t` does; the debug assertion on an invalid id is not compiled natively).
    pub fn pre_increment(&mut self) -> Self {
        self.index = self.index.wrapping_add(1);
        *self
    }

    /// `id++`: advances the index and returns the previous id.
    pub fn post_increment(&mut self) -> Self {
        let previous = *self;
        self.index = self.index.wrapping_add(1);
        previous
    }

    /// `std::hash<Typed<Kind>>`.
    pub fn hash_code(self) -> usize {
        T::Family::typed_hash(T::RAW_KIND, self.index)
    }

    /// `IsRemoved(theGraph)`: soft-removed in the graph.
    pub fn is_removed(self, graph: &BRepGraph) -> bool {
        T::Family::is_removed(graph, T::RAW_KIND, self.index)
    }
}

impl<T: KindTag> TypedId<T>
where
    T::Family: OwnedFamily,
{
    /// `operator+(offset)`: the id `offset` slots later (wrapping as `uint32_t`).
    pub fn offset(self, offset: u32) -> Self {
        Self::new(self.index.wrapping_add(offset))
    }

    /// `operator-(offset)`: the id `offset` slots earlier (wrapping as `uint32_t`; the
    /// underflow assertion is debug-only natively).
    pub fn retreat(self, offset: u32) -> Self {
        Self::new(self.index.wrapping_sub(offset))
    }

    /// `IsOwned(theGraph)`: the item has an active owner in the graph.
    pub fn is_owned(self, graph: &BRepGraph) -> bool {
        T::Family::is_owned(graph, T::RAW_KIND, self.index)
    }
}

impl<T: KindTag> Default for TypedId<T> {
    fn default() -> Self {
        Self::invalid()
    }
}

impl<T> Clone for TypedId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TypedId<T> {}

impl<T> PartialEq for TypedId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for TypedId<T> {}

impl<T> PartialOrd for TypedId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for TypedId<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Hash for TypedId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T: KindTag> fmt::Debug for TypedId<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}({})", T::NAME, self.index)
    }
}

#[cfg(test)]
mod tests {
    use crate::brep_graph::{EdgeCurve3DRepId, FaceId, ShellRefId};

    #[test]
    fn typed_ids_step_and_compare() {
        let mut face = FaceId::start();
        assert_eq!(face.pre_increment(), FaceId::new(1));
        assert_eq!(face.post_increment(), FaceId::new(1));
        assert_eq!(face, FaceId::new(2));
        assert_eq!(face.offset(3).retreat(1), FaceId::new(4));
        assert!(FaceId::new(1) < FaceId::new(2));
        assert!(face.is_valid_in(&vec![0; 3]));
        assert!(!face.is_valid_in(&[0; 2][..]));
        assert_eq!(
            format!("{:?}", ShellRefId::new(5)),
            "BRepGraph_ShellRefId(5)"
        );
        assert_eq!(EdgeCurve3DRepId::new(9).hash_code(), 9);
    }
}
