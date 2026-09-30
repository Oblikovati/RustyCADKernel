// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_RefId.hxx and the RefId
// members of BRepGraph.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_RefId`: the `(Kind, Index)` address of a reference entry.
use super::hashing::{combine, hash_i32, hash_u32};
use super::typed::{CountProvider, IdFamily, KindTag, OwnedFamily, TypedId};
use crate::brep_graph::BRepGraph;
use crate::brep_graph_inc::RefTag;

/// `BRepGraph_RefId::Kind`: a transparent `int` with one constant per enumerator.
///
/// ```
/// use rustycad_tkbrep::brep_graph::RefKind;
/// assert!(RefKind::is_topology_ref_kind(RefKind::CHILD));
/// assert!(!RefKind::is_topology_ref_kind(RefKind::OCCURRENCE));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefKind(pub i32);

impl RefKind {
    /// `Kind::Shell`: usage of a shell definition.
    pub const SHELL: Self = Self(0);
    /// `Kind::Face`.
    pub const FACE: Self = Self(1);
    /// `Kind::Wire`.
    pub const WIRE: Self = Self(2);
    /// `Kind::Vertex`.
    pub const VERTEX: Self = Self(3);
    /// `Kind::Solid`.
    pub const SOLID: Self = Self(4);
    /// `Kind::Child`: a compound child of any kind.
    pub const CHILD: Self = Self(5);
    /// `Kind::Occurrence`.
    pub const OCCURRENCE: Self = Self(6);

    /// `IsValidKind(theKind)`.
    pub fn is_valid_kind(kind: Self) -> bool {
        (0..=6).contains(&kind.0)
    }

    /// `IsTopologyRefKind(theKind)`: Shell through Child.
    pub fn is_topology_ref_kind(kind: Self) -> bool {
        Self::is_valid_kind(kind) && kind >= Self::SHELL && kind <= Self::CHILD
    }
}

/// `BRepGraph_RefId`: a lightweight typed index into a per-kind reference vector; the
/// default is invalid (`Kind::Shell`, index `u32::MAX`).
///
/// ```
/// use rustycad_tkbrep::brep_graph::{RefId, RefKind};
/// let id = RefId::new(RefKind::WIRE, 1);
/// assert!(id.is_valid() && !RefId::default().is_valid());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefId {
    /// `RefKind`.
    pub kind: RefKind,
    /// `Index`.
    pub index: u32,
}

impl Default for RefId {
    fn default() -> Self {
        Self::new(RefKind::SHELL, Self::INVALID_INDEX)
    }
}

impl RefId {
    /// `THE_START_INDEX`.
    pub const START_INDEX: u32 = 0;
    /// `THE_INVALID_INDEX`.
    pub const INVALID_INDEX: u32 = u32::MAX;

    /// `BRepGraph_RefId(theKind, theIdx)`.
    pub const fn new(kind: RefKind, index: u32) -> Self {
        Self { kind, index }
    }

    /// `Start(theKind)`.
    pub const fn start(kind: RefKind) -> Self {
        Self::new(kind, Self::START_INDEX)
    }

    /// `Invalid(theKind)`.
    pub const fn invalid(kind: RefKind) -> Self {
        Self::new(kind, Self::INVALID_INDEX)
    }

    /// `IsValid()`.
    pub fn is_valid(self) -> bool {
        RefKind::is_valid_kind(self.kind) && self.index != Self::INVALID_INDEX
    }

    /// `IsValid(theMaxCount)`.
    pub fn is_valid_for(self, max_count: u32) -> bool {
        self.is_valid() && self.index < max_count
    }

    /// `IsValidIn(theProvider)`.
    pub fn is_valid_in(self, provider: &(impl CountProvider + ?Sized)) -> bool {
        self.is_valid_for(provider.count())
    }

    /// `++id`.
    pub fn pre_increment(&mut self) -> Self {
        self.index = self.index.wrapping_add(1);
        *self
    }

    /// `id++`.
    pub fn post_increment(&mut self) -> Self {
        let previous = *self;
        self.index = self.index.wrapping_add(1);
        previous
    }

    /// `operator+(theOffset)`.
    pub fn offset(self, offset: u32) -> Self {
        Self::new(self.kind, self.index.wrapping_add(offset))
    }

    /// `operator-(theOffset)`.
    pub fn retreat(self, offset: u32) -> Self {
        Self::new(self.kind, self.index.wrapping_sub(offset))
    }

    /// `std::hash<BRepGraph_RefId>`.
    pub fn hash_code(self) -> usize {
        combine(&[hash_i32(self.kind.0), hash_u32(self.index)])
    }

    /// `Visit(theRefId, theFunc)`: calls the visitor with the typed id of the kind; an
    /// unsupported kind reaches the native fallback, the invalid shell reference id.
    pub fn visit<V: RefVisitor>(self, visitor: V) -> V::Output {
        use crate::brep_graph as g;
        match self.kind {
            RefKind::SHELL => visitor.visit(g::ShellRefId::from_ref_id(self)),
            RefKind::FACE => visitor.visit(g::FaceRefId::from_ref_id(self)),
            RefKind::WIRE => visitor.visit(g::WireRefId::from_ref_id(self)),
            RefKind::VERTEX => visitor.visit(g::VertexRefId::from_ref_id(self)),
            RefKind::SOLID => visitor.visit(g::SolidRefId::from_ref_id(self)),
            RefKind::CHILD => visitor.visit(g::ChildRefId::from_ref_id(self)),
            RefKind::OCCURRENCE => visitor.visit(g::OccurrenceRefId::from_ref_id(self)),
            _ => visitor.visit(g::ShellRefId::invalid()),
        }
    }

    /// `IsRemoved(theGraph)`: soft-removed; false for an invalid id.
    pub fn is_removed(self, graph: &BRepGraph) -> bool {
        if !self.is_valid() {
            return false;
        }
        self.visit(RefRemovedProbe(graph))
    }

    /// `IsOwned(theGraph)`: false for an invalid id.
    pub fn is_owned(self, graph: &BRepGraph) -> bool {
        if !self.is_valid() {
            return false;
        }
        self.visit(RefOwnedProbe(graph))
    }
}

/// The generic callable of `BRepGraph_RefId::Visit`.
pub trait RefVisitor {
    /// The callable's result.
    type Output;
    /// Called with the typed id of the visited kind.
    fn visit<T: RefTag>(self, id: TypedId<T>) -> Self::Output;
}

struct RefRemovedProbe<'g>(&'g BRepGraph);

impl RefVisitor for RefRemovedProbe<'_> {
    type Output = bool;
    fn visit<T: RefTag>(self, id: TypedId<T>) -> bool {
        self.0.inc_storage().is_removed(id)
    }
}

struct RefOwnedProbe<'g>(&'g BRepGraph);

impl RefVisitor for RefOwnedProbe<'_> {
    type Output = bool;
    fn visit<T: RefTag>(self, id: TypedId<T>) -> bool {
        self.0.inc_storage().is_owned(id)
    }
}

/// The reference id family of [`TypedId`].
#[derive(Clone, Copy, Debug)]
pub struct RefFamily;

impl IdFamily for RefFamily {
    type Untyped = RefId;
    fn untyped(kind: i32, index: u32) -> RefId {
        RefId::new(RefKind(kind), index)
    }
    fn valid_kind(kind: i32) -> bool {
        RefKind::is_valid_kind(RefKind(kind))
    }
    fn typed_hash(kind: i32, index: u32) -> usize {
        RefId::new(RefKind(kind), index).hash_code()
    }
    fn is_removed(graph: &BRepGraph, kind: i32, index: u32) -> bool {
        RefId::new(RefKind(kind), index).is_removed(graph)
    }
}

impl OwnedFamily for RefFamily {
    fn is_owned(graph: &BRepGraph, kind: i32, index: u32) -> bool {
        RefId::new(RefKind(kind), index).is_owned(graph)
    }
}

impl<T: KindTag<Family = RefFamily>> TypedId<T> {
    /// `FromRefId(theRefId)`: another kind or an invalid id gives the invalid typed id.
    pub fn from_ref_id(id: RefId) -> Self {
        if id.kind.0 != T::RAW_KIND || !id.is_valid() {
            return Self::invalid();
        }
        Self::new(id.index)
    }
}

impl<T: KindTag<Family = RefFamily>> From<TypedId<T>> for RefId {
    fn from(id: TypedId<T>) -> Self {
        RefId::new(RefKind(T::RAW_KIND), id.index)
    }
}

impl<T: KindTag<Family = RefFamily>> PartialEq<RefId> for TypedId<T> {
    fn eq(&self, other: &RefId) -> bool {
        other.kind.0 == T::RAW_KIND && other.index == self.index
    }
}

impl<T: KindTag<Family = RefFamily>> PartialEq<TypedId<T>> for RefId {
    fn eq(&self, other: &TypedId<T>) -> bool {
        other == self
    }
}

macro_rules! ref_tags {
    ($($(#[$doc:meta])* $tag:ident, $alias:ident, $kind:ident, $name:literal;)*) => {$(
        #[doc = concat!("The kind tag of `", $name, "`.")]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $tag;
        impl KindTag for $tag {
            type Family = RefFamily;
            const RAW_KIND: i32 = RefKind::$kind.0;
            const NAME: &'static str = $name;
        }
        $(#[$doc])*
        pub type $alias = TypedId<$tag>;
    )*};
}

ref_tags! {
    /// `BRepGraph_ShellRefId`.
    ShellRefTag, ShellRefId, SHELL, "BRepGraph_ShellRefId";
    /// `BRepGraph_FaceRefId`.
    FaceRefTag, FaceRefId, FACE, "BRepGraph_FaceRefId";
    /// `BRepGraph_WireRefId`.
    WireRefTag, WireRefId, WIRE, "BRepGraph_WireRefId";
    /// `BRepGraph_VertexRefId`.
    VertexRefTag, VertexRefId, VERTEX, "BRepGraph_VertexRefId";
    /// `BRepGraph_SolidRefId`.
    SolidRefTag, SolidRefId, SOLID, "BRepGraph_SolidRefId";
    /// `BRepGraph_ChildRefId`.
    ChildRefTag, ChildRefId, CHILD, "BRepGraph_ChildRefId";
    /// `BRepGraph_OccurrenceRefId`.
    OccurrenceRefTag, OccurrenceRefId, OCCURRENCE, "BRepGraph_OccurrenceRefId";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_ref_ids_convert_and_compare() {
        let wire = WireRefId::new(3);
        let untyped: RefId = wire.into();
        assert_eq!(untyped, RefId::new(RefKind::WIRE, 3));
        assert_eq!(wire, untyped);
        assert!(!FaceRefId::from_ref_id(untyped).is_valid());
        assert!(!RefKind::is_valid_kind(RefKind(7)));
        assert!(RefId::start(RefKind::FACE) < RefId::start(RefKind::WIRE));
        assert_eq!(wire.hash_code(), untyped.hash_code());
    }
}
