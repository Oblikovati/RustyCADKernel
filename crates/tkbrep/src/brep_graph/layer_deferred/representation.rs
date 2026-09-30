// Rust translation of BRepGraph_LayerDeferred::RepresentationKind, Representation,
// Entry and Entry::RepresentationStorage
// (ModelingData/TKBRep/BRepGraph/BRepGraph_LayerDeferred.hxx and .cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The records of the deferred layer: postponed representations per graph item.
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;

/// `BRepGraph_LayerDeferred::RepresentationKind`: the representation category.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RepresentationKind {
    /// `Unknown`.
    #[default]
    Unknown,
    /// `Geometry`.
    Geometry,
    /// `Mesh`.
    Mesh,
    /// `Topology`.
    Topology,
    /// `Assembly`.
    Assembly,
    /// `Parametric`.
    Parametric,
}

/// `Representation::THE_INVALID_SOURCE_INDEX`.
pub const INVALID_SOURCE_INDEX: u32 = u32::MAX;

/// `BRepGraph_LayerDeferred::Representation`: one postponed representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeferredRepresentation {
    /// `Kind`.
    pub kind: RepresentationKind,
    /// `Role`.
    pub role: u32,
    /// `Name`.
    pub name: AsciiString,
    /// `SourceIndex`: `INVALID_SOURCE_INDEX` when unset.
    pub source_index: u32,
}

impl Default for DeferredRepresentation {
    fn default() -> Self {
        Self {
            kind: RepresentationKind::Unknown,
            role: 0,
            name: AsciiString::default(),
            source_index: INVALID_SOURCE_INDEX,
        }
    }
}

/// `Entry::RepresentationStorage::THE_MAX_REPRESENTATIONS_PER_ITEM`.
pub const MAX_REPRESENTATIONS_PER_ITEM: usize = 8;

/// `Entry::RepresentationStorage`: at most eight representations per item (a fixed
/// inline array natively; a vector bounded by the same limit here).
///
/// ```
/// use rustycad_tkbrep::brep_graph::layer_deferred::{DeferredRepresentation, RepresentationStorage};
/// let mut storage = RepresentationStorage::default();
/// storage.append(&DeferredRepresentation::default()).unwrap();
/// assert_eq!(storage.size(), 1);
/// assert!(storage.value(1).is_err());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RepresentationStorage {
    items: Vec<DeferredRepresentation>,
}

impl RepresentationStorage {
    /// `Size()`.
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// `IsEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// `ContainsKind(theKind)`: a representation of the kind is stored.
    pub fn contains_kind(&self, kind: RepresentationKind) -> bool {
        self.items.iter().any(|item| item.kind == kind)
    }

    /// `Value(theIndex)`: Standard_ProgramError out of range (`Standard_ASSERT_RAISE`).
    pub fn value(&self, index: usize) -> Result<&DeferredRepresentation, Failure> {
        let size = self.items.len();
        self.items
            .get(index)
            .ok_or_else(|| out_of_range(index, size))
    }

    /// `ChangeValue(theIndex)`: Standard_ProgramError out of range.
    pub fn change_value(&mut self, index: usize) -> Result<&mut DeferredRepresentation, Failure> {
        let size = self.items.len();
        self.items
            .get_mut(index)
            .ok_or_else(|| out_of_range(index, size))
    }

    /// `First()`: Standard_ProgramError when empty.
    pub fn first(&self) -> Result<&DeferredRepresentation, Failure> {
        self.items.first().ok_or_else(|| {
            Failure::with_type(
                "Standard_ProgramError",
                "*** ERROR: ASSERT: Deferred representation list is empty (expected at least one \
                 representation)",
                "",
            )
        })
    }

    /// `Append(theRepresentation)`: Standard_ProgramError when eight are stored.
    pub fn append(&mut self, representation: &DeferredRepresentation) -> Result<(), Failure> {
        if self.items.len() >= MAX_REPRESENTATIONS_PER_ITEM {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                &format!(
                    "*** ERROR: ASSERT: Too many deferred representations for one graph item \
                     ({} stored, at most {MAX_REPRESENTATIONS_PER_ITEM})",
                    self.items.len()
                ),
                "",
            ));
        }
        self.items.push(representation.clone());
        Ok(())
    }

    /// `Clear()`.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// The stored representations in order.
    pub fn as_slice(&self) -> &[DeferredRepresentation] {
        &self.items
    }
}

fn out_of_range(index: usize, size: usize) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!(
            "*** ERROR: ASSERT: Deferred representation index is out of range ({index}, expected \
             below {size})"
        ),
        "",
    )
}

/// `BRepGraph_LayerDeferred::Entry`: the deferred ownership record of one item.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeferredEntry {
    /// `Provider`: the loader that owns the item.
    pub provider: AsciiString,
    /// `SourceKey`: the provider's key of the item.
    pub source_key: AsciiString,
    /// `Representations`.
    pub representations: RepresentationStorage,
}
