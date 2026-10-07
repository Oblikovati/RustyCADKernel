# OCAF design notes (TKCDF, TKLCAF)

This note fixes how the OCAF base toolkits map to Rust: TKCDF (`crates/tkcdf`,
`rustycad-tkcdf`: packages CDM, PCDM, CDF, LDOM) and TKLCAF (`crates/tklcaf`,
`rustycad-tklcaf`: packages TDF, TDataStd, TFunction, TDocStd). Both depend only on
TKernel. The rules of [geom-design.md](geom-design.md) sections 1 to 4 (abstract class as
trait, `Handle<dyn Trait>`, `SharedState`, exceptions as `Failure`) apply; this note covers
what they do not. Code comments and ledger notes cite it as "ocaf-design.md section N".

## 1. Toolkit-wide rules

- **Modules and names.** Each package is one module in snake case (`TDataStd` is
  `tdata_std`, `TDocStd` is `tdoc_std`). A class keeps its package prefix in its Rust name
  (`TDF_Label` is `TdfLabel`, `CDM_Document` is `CdmDocument`), because the packages reuse
  short names (`TDF_Reference`, `CDM_Reference`, `PCDM_Reference`).
- **Package classes** (`TDF`, `PCDM`) are unit structs with associated functions.
- **Overloads** get a suffix that names the distinguishing argument
  (`TDF_TagSource::Set(label)` is `set`, `TDF_TagSource::Set(int)` is `set_value`).
- **Exceptions** are `Result<_, Failure>` with the native exception type and the offending
  value in the message.
- **Native file formats are not ported** (user decision 2026-09): the concrete storage
  and retrieval drivers of the Bin, Xml and Std formats, the `Storage_*` based header
  reader-writer `PCDM_ReadWriter_1`, the file-format probe of `PCDM_ReadWriter::FileFormat`
  and `PCDM::FileDriverType`, and LDOM (the XML DOM that only `PCDM_DOMHeaderParser` and the
  Xml format toolkits use). The abstract driver interfaces (`PCDM_Reader`, `PCDM_Writer`,
  `PCDM_RetrievalDriver`, `PCDM_StorageDriver`, `PCDM_ReadWriter`) are ported, so that an
  exchange-format driver can plug into the application.

## 2. Self handles (`this` converted to a handle)

OCCT converts `this` into a handle wherever an object registers itself: `CDM_Document`
creates `CDM_Reference(this, ...)`, `TDF_Attribute::Backup` stores `backup->myNext = this`,
`TDF_Attribute::DeltaOnAddition` returns `new TDF_DeltaOnAddition(this)`. The intrusive
reference count makes that legal in C++. A Rust `Handle` is an `Arc`, which cannot be made
from `&self`.

Each base struct that needs it (`CdmDocumentBase`, `TdfAttributeBase`, `TdfData`) keeps a
`WeakHandle` to its own object, set once by `bind_self` when the first owning handle is
made. The constructors that return handles (`TdfData::new`, `TdocStdDocument::new`) bind
it. An attribute is also bound when it is attached to a label (`TdfLabel::add_attribute`
takes the handle), which covers every native use of `this` on attributes: they all need an
attached attribute. `this()` on an unbound object returns `Standard_NullObject`, the error
a null handle would give natively.

## 3. Label tree ownership

- **TDF_LabelNode** is `TdfLabelNode`, shared as `Arc<TdfLabelNode>`, with its mutable
  fields in one `SharedState`: the father is a `Weak`, the first child and the next brother
  are `Arc`s (a node owns its children), the last-found-child cache is a `Weak`, and the
  data framework is a `WeakHandle<TdfData>`. Tag and the flags-and-depth word keep their
  native bit layout (`TDF_LabelNodeImportMsk`...).
- **TDF_Label** is the value `TdfLabel`, an `Option<Arc<TdfLabelNode>>`. Equality and
  hashing use the node address, as the native pointer compare and `std::hash<TDF_Label>`.
  `HasLowerNode`/`HasGreaterNode` compare addresses, as natively; their answers, and the
  iteration order of maps of labels, depend on allocation addresses in both
  implementations.
- **Lifetime.** Natively the nodes live in the `NCollection_IncAllocator` of `TDF_Data` and
  die with it; a label that outlives its data dangles. In Rust a label keeps its node
  alive, and `TdfData`'s `Drop` runs the native `Destroy` (abort, forget the root
  attributes, clear the entry table) and then clears every node: children, attribute list
  and father. That also breaks the reference cycles that attributes holding labels create.
  A label that outlived its data is a detached node: `data()` is null, `is_root()` is true
  and it has no child and no attribute.
- **Null labels.** The native inline accessors (`Tag`, `Father`, `IsRoot`, `IsImported`,
  `HasChild`, `MayBeModified`, `AttributesModified`, `Data`) dereference the node without a
  check and crash on a null label. The Rust accessors return the value of an empty root
  instead (tag 0, null father, `false`, null data); the members that throw natively
  (`Depth`, `FindChild`, `NbChildren`, `FindAttribute`, ...) return `Standard_NullObject`
  as natively. The members that mutate through a null node (`AddAttribute`,
  `ForgetAttribute`, `ResumeAttribute`, `ForgetAllAttributes`) return `Standard_NullObject`.
- `TDF_Data::LabelNodeAllocator` (the `NCollection_IncAllocator` of the nodes) has no Rust
  counterpart: nodes are `Arc`s. It is excluded in the ledger.

## 4. Attributes

- **TDF_Attribute** is the trait `TdfAttribute: Transient`. Its pure virtual members are
  required methods (`id`, `restore`, `new_empty`, `paste`), the virtual ones are provided
  methods with the native default body, and the non-virtual members are provided methods
  that no implementation overrides. The private fields (`myLabelNode`, `myTransaction`,
  `mySavedTransaction`, `myFlags`, `myNext`, `myBackup`) live in the embedded
  `TdfAttributeBase` (one `SharedState`), reached through the required accessor
  `attribute_base()`.
- **Attribute list.** The attributes of a label keep the native singly linked list through
  `myNext`, because `TDF_AttributeIterator` relies on it: a removed attribute keeps its
  `myNext`, so an iterator that stands on it continues. A backup keeps `myNext` pointing to
  the attribute it backs up, a strong handle, as natively; `RemoveBackup` and
  `TDF_Data::Destroy` break that cycle as natively.
- **Down casts.** `occ::down_cast<TDF_TagSource>(att)` is `att.cast::<TdfTagSource>()`; a
  concrete attribute registers `dyn TdfAttribute` as an interface
  (`impl_transient!(..., interfaces: [dyn TdfAttribute])`). The template
  `FindAttribute<T>` is `find_attribute_as::<T>`.
- **Callbacks** (`AfterAddition`, `BeforeRemoval`, `BeforeForget`, `AfterResume`,
  `BeforeUndo`, `AfterUndo`, `AfterRetrieval`, `BeforeCommitTransaction`) and `Restore`,
  `Paste` return `Result<_, Failure>`, because overrides in the derived toolkits throw.
- **Locks.** A member never holds its own lock while it calls another attribute, a label
  node or the data framework (geom-design.md section 3): every native call chain here is
  re-entrant (`Backup` calls `BackupCopy`, which calls `Restore` on a new attribute that
  reads this one).

## 5. Transactions and deltas

- **TDF_Data** is `TdfData`, a transient with its counters, time stack and entry table in
  one `SharedState`. Its private members that `TDF_Transaction` reaches as a friend
  (`OpenTransaction`, `CommitTransaction`, `CommitUntilTransaction`, `AbortTransaction`,
  `AbortUntilTransaction`) are `pub(crate)`; the commit walks the tree exactly as the
  native recursive `CommitTransaction(label, delta, withDelta)`, including the
  `TDF_DATA_COMMIT_OPTIMIZED` shortcuts on the `MayBeModified`/`AttributesModified` flags.
- **TDF_Delta** keeps its attribute deltas in a `Vec` in native order; `FixOrder` moves
  the removal deltas to the end before an undo, as natively. `TDF_Delta::Labels` fills an
  `NCollection_Map` of labels and copies it in map order, as natively.
- **TDF_AttributeDelta** is the trait `TdfAttributeDelta: Transient` over the embedded
  `TdfAttributeDeltaBase` (attribute and label). The abstract `TDF_DeltaOnRemoval` and
  `TDF_DeltaOnModification` (subclassed by TDataStd and TNaming) are traits
  `TdfDeltaOnRemoval` and `TdfDeltaOnModification`; `TDF_DeltaOnAddition`,
  `TDF_DeltaOnForget` and `TDF_DeltaOnResume` are structs.
- **TDF_Transaction** aborts in its destructor: the Rust `Drop` does the same and drops a
  failure, since a destructor cannot report one.

## 6. Documents and applications (CDM, CDF, TDocStd)

- `CDM_Document`, `CDM_Application`, `CDF_Application`, `TDocStd_Application` are traits
  over embedded bases (`CdmDocumentBase`, `CdmApplicationBase`, ...), with the virtual
  members as provided methods. `TDocStd_Document` is a struct that implements
  `CdmDocument`.
- **Raw back pointers** (`CDM_Reference::myFromDocument`, `CDM_MetaData::myDocument`) are
  `WeakHandle`s; the native pointer can dangle, the weak handle then gives a null handle.
- **Resource_Manager** is not a transient in the TKernel port. `occ::handle<Resource_Manager>`
  is `Arc<SharedState<ResourceManager>>` (`SharedResourceManager`): shared and mutable
  through every holder, as the native handle.
- **MetaDataLookUpTable** returns a reference to the application's map natively. The Rust
  member returns the write guard of the map; a caller must drop it before it calls back
  into the application.
