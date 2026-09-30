// Rust addition beside the record accessors of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checked, `Result`-returning forms of the record accessors.
//!
//! Natively `Vertex()`, `ChangeEdge()`, `ShellRef()`, `EdgeCurve3DRep()` and the
//! relation accessors read `NCollection_DynamicArray::Value`, which is `noexcept` and
//! does not check the index: an id past the store is undefined behaviour. The fast
//! Rust accessors keep that contract and panic; the forms here return
//! Standard_OutOfRange with the offending id and the valid range instead. Every
//! checked form goes through [`check_typed_index`], the one range check of the crate
//! (docs/topology-design.md, section 11.14).
use super::Storage;
use super::tags::{NodeTag, RefTag, RepTag, StoredTag};
use crate::brep_graph::{KindTag, TypedId};
use rustycad_tkernel::standard::Failure;
use std::fmt;

/// The slot of an index inside `0..nb`, else Standard_OutOfRange naming the id kind,
/// the index and the valid range. The kind name is formatted only on failure, so the
/// check costs one comparison on the fast path.
///
/// ```
/// use rustycad_tkbrep::brep_graph_inc::check_index;
/// assert_eq!(check_index("BRepGraph_FaceId", 2, 6).unwrap(), 2);
/// assert!(check_index("BRepGraph_FaceId", 6, 6).is_err());
/// ```
pub fn check_index(kind_name: impl fmt::Display, index: u32, nb: u32) -> Result<usize, Failure> {
    if index < nb {
        return Ok(index as usize);
    }
    Err(Failure::out_of_range(&format!(
        "BRepGraph: {kind_name} index {index} is out of range; expected 0..{nb}"
    )))
}

/// [`check_index`] for an untyped id, named by its family and raw kind.
///
/// ```
/// use rustycad_tkbrep::brep_graph_inc::check_untyped_index;
/// let failure = check_untyped_index("BRepGraph_RefId", 1, 4, 0).unwrap_err();
/// assert!(failure.message().contains("BRepGraph_RefId of kind 1 index 4"));
/// ```
pub fn check_untyped_index(
    family: &'static str,
    raw_kind: i32,
    index: u32,
    nb: u32,
) -> Result<usize, Failure> {
    check_index(UntypedKindName { family, raw_kind }, index, nb)
}

// The diagnostic name of an untyped id: its family and raw kind.
struct UntypedKindName {
    family: &'static str,
    raw_kind: i32,
}

impl fmt::Display for UntypedKindName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} of kind {}", self.family, self.raw_kind)
    }
}

/// [`check_index`] for a typed id, named by its OCCT alias.
///
/// ```
/// use rustycad_tkbrep::brep_graph::FaceId;
/// use rustycad_tkbrep::brep_graph_inc::check_typed_index;
/// let failure = check_typed_index(FaceId::new(9), 6).unwrap_err();
/// assert!(failure.message().contains("BRepGraph_FaceId index 9"));
/// ```
pub fn check_typed_index<T: KindTag>(id: TypedId<T>, nb: u32) -> Result<usize, Failure> {
    check_index(T::NAME, id.index, nb)
}

impl Storage {
    /// The slot of a typed id inside its store (removed slots included), else
    /// Standard_OutOfRange.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::EdgeId;
    /// use rustycad_tkbrep::brep_graph_inc::Storage;
    /// let mut storage = Storage::new();
    /// let edge = storage.append_edge();
    /// assert_eq!(storage.check_id(edge).unwrap(), 0);
    /// assert!(storage.check_id(EdgeId::new(1)).is_err());
    /// ```
    pub fn check_id<T: StoredTag>(&self, id: TypedId<T>) -> Result<usize, Failure> {
        use super::stores::RemovalPlanes;
        check_typed_index(id, T::store(self).count())
    }

    /// The checked [`def`](Self::def): Standard_OutOfRange past the store.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::VertexId;
    /// use rustycad_tkbrep::brep_graph_inc::Storage;
    /// assert!(Storage::new().try_def(VertexId::new(0)).is_err());
    /// ```
    pub fn try_def<T: NodeTag>(&self, id: TypedId<T>) -> Result<&T::Def, Failure> {
        let slot = self.check_id(id)?;
        Ok(&T::store(self).items[slot])
    }

    /// The checked [`change_def`](Self::change_def).
    pub fn try_change_def<T: NodeTag>(&mut self, id: TypedId<T>) -> Result<&mut T::Def, Failure> {
        let slot = self.check_id(id)?;
        Ok(&mut T::store_mut(self).items[slot])
    }

    /// The checked [`entry`](Self::entry).
    pub fn try_entry<T: RefTag>(&self, id: TypedId<T>) -> Result<&T::Entry, Failure> {
        let slot = self.check_id(id)?;
        Ok(&T::store(self).items[slot])
    }

    /// The checked [`change_entry`](Self::change_entry).
    pub fn try_change_entry<T: RefTag>(
        &mut self,
        id: TypedId<T>,
    ) -> Result<&mut T::Entry, Failure> {
        let slot = self.check_id(id)?;
        Ok(&mut T::store_mut(self).items[slot])
    }

    /// The checked [`record`](Self::record).
    pub fn try_record<T: RepTag>(&self, id: TypedId<T>) -> Result<&T::Record, Failure> {
        let slot = self.check_id(id)?;
        Ok(&T::store(self).items[slot])
    }

    /// The checked [`change_record`](Self::change_record).
    pub fn try_change_record<T: RepTag>(
        &mut self,
        id: TypedId<T>,
    ) -> Result<&mut T::Record, Failure> {
        let slot = self.check_id(id)?;
        Ok(&mut T::store_mut(self).items[slot])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::{EdgeCurve3DRepId, FaceId, ShellRefId};

    #[test]
    fn checked_forms_name_the_id_and_the_range() {
        let mut storage = Storage::new();
        let face = storage.append_face();
        storage.try_change_def(face).unwrap().tolerance = 0.25;
        assert_eq!(storage.try_def(face).unwrap().tolerance, 0.25);
        let failure = storage.try_def(FaceId::new(3)).unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_OutOfRange");
        assert!(
            failure
                .message()
                .contains("BRepGraph_FaceId index 3 is out of range; expected 0..1")
        );
        assert!(storage.try_change_def(FaceId::invalid()).is_err());
    }

    #[test]
    fn checked_entries_and_records_refuse_ids_past_the_store() {
        let mut storage = Storage::new();
        let reference = storage.append_shell_ref();
        assert!(storage.try_entry(reference).is_ok());
        assert!(storage.try_change_entry(ShellRefId::new(1)).is_err());
        let curve = storage.append_edge_curve3d_rep();
        storage.try_change_record(curve).unwrap().param_last = 2.0;
        assert_eq!(storage.try_record(curve).unwrap().param_last, 2.0);
        assert!(storage.try_record(EdgeCurve3DRepId::new(5)).is_err());
    }
}
