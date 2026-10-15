# OCAF design notes (TKCDF, TKLCAF, TKCAF, TKVCAF)

This note fixes how the OCAF toolkits map to Rust: TKCDF (`crates/tkcdf`,
`rustycad-tkcdf`: packages CDM, PCDM, CDF, LDOM) and TKLCAF (`crates/tklcaf`,
`rustycad-tklcaf`: packages TDF, TDataStd, TFunction, TDocStd), which depend only on
TKernel, and TKCAF (`crates/tkcaf`, `rustycad-tkcaf`: packages TNaming, TDataXtd, AppStd),
which adds the modeling toolkits (sections 10 and 11). The rules of [geom-design.md](geom-design.md) sections 1 to 4 (abstract class as
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

## 7. TDataStd attributes over collections

- **Shared arrays.** `occ::handle<NCollection_HArray1<T>>` is `Handle<HArray1<T>>`: the
  attribute and every caller of `Array()` share one storage, as natively. A null native
  handle is a null `Handle`; the inline accessors (`Lower`, `Upper`, `Length`, `Value`)
  answer the native neutral values for it (`RealFirst()` for `TDataStd_RealArray::Value`,
  `-1` for `TDataStd_ByteArray::Upper`).
- **One array core.** `TDataStd_IntegerArray`, `RealArray`, `ExtStringArray` and
  `ByteArray` repeat the same code natively. Their state is one `ArrayState<T>` (array,
  delta flag, user ID) and their members call the generic functions of
  `tdata_std/array_core.rs`; each class keeps its own struct, RTTI name, GUID and the few
  native differences, which are explicit parameters (the null guard of
  `TDataStd_RealArray::ChangeArray`, the order of `SetID` and `Init` in `SetAttr`, the
  `Dump` text).
- **Array deltas.** `TDataStd_DeltaOnModificationOf{Int,Real,Byte,ExtString}Array` are
  four structs over one generic `ArrayDeltaState<T>`: the constructor records the changed
  indexes and old values and then drops the backup's array (`RemoveArray`), and `Apply`
  writes them back, as natively.
- **Unchecked indexes.** `NCollection_Array1::Value` does not check its index in a release
  build. The port checks it and returns `Standard_OutOfRange`.
- **Native null dereferences** (`ChangeArray` with a null array, `ChangeArray` of an
  attribute with no array whose bounds match) are `Standard_NullObject` errors.
- **Derived classes by embedding.** A native class derived from a concrete attribute
  (`TDataStd_Relation` from `TDataStd_Expression`) embeds the base struct and registers it
  as its RTTI parent, so its handle casts to the base. A base member that natively calls a
  virtual function on `this` (`Backup` calls the virtual `BackupCopy`) goes through the
  attribute's own handle, so the derived override runs.
- **Derived-attribute registry.** `TDF_DerivedAttribute` keeps its prototypes in an
  `NCollection_DataMap` with `Standard_CStringHasher`, so `Attributes` lists them in the
  native map order. The TKLCAF registrations (the static initializers of
  `IMPLEMENT_DERIVED_ATTRIBUTE`) run once before the first query.
- **Tree nodes.** The raw `TDataStd_TreeNode*` links are weak handles: the labels own the
  nodes. The structure members need the node's own handle, bound when the node is added
  to a label; a node never added to a label cannot be linked.

## 8. Function mechanism (TFunction)

- **TFunction_Driver** is the trait `TFunctionDriver: Transient` over the embedded
  `TFunctionDriverBase` (the label). `Execute` is the required method; `Validate`,
  `MustExecute`, `Arguments` and `Results` are provided methods with the native bodies.
  `Execute(occ::handle<TFunction_Logbook>&)` takes `&mut Handle<TFunctionLogbook>`.
- **TFunction_DriverTable::Get** is the process-wide table, a `OnceLock` handle; the
  per-thread tables are a `Vec` of maps indexed from 1, grown as `AddDriver` asks.
- **Logbook, GraphNode, Scope** keep their `NCollection_Map` and `NCollection_DoubleMap`
  fields (native iteration order); getters that natively return a reference to a map
  return a copy.
- **TFunction_IFunction** and **TFunction_Iterator** are plain values; their native
  exceptions (`Standard_NoSuchObject` for a missing function, driver or graph node) are
  `Failure` errors, and the native null dereferences (`GetMaxNbThreads` walks the next
  functions without a scope) are `Standard_NullObject`.

## 9. LDOM (the light XML DOM of TKCDF)

LDOM is a general XML reader, DOM and writer. The native XML storage drivers (TKXml*) and
the XML header reader of PCDM build on it, and those are not ported; LDOM itself is, since
other code (the QABugs DRAW commands) parses XML with it.

- **Memory.** `LDOM_MemManager` owns every node of a document in one arena: a node is a
  `LdomNodeId` (an index), and the native `LDOM_BasicNode*` links (`mySibling`, `myFirstChild`,
  `myRootElement`) are `Option<NodeId>`. The native memory blocks hold raw bytes; the port
  keeps the strings as reference-counted byte strings, so `Allocate` has no untyped
  counterpart. The hash table of tag and attribute names keeps the native 256 buckets and
  CRC-16 hash, and a hashed name carries its hash index (natively the byte before it).
- **Strings.** `LDOMBasicString` is an enum over the native string types. Its `operator==`
  compares the type and the pointer (an `Arc` identity), `equals` compares the contents, and
  copying an `LDOM_AsciiFree` string allocates a new copy, as natively.
- **Basic nodes.** `LDOM_BasicNode`, `LDOM_BasicElement`, `LDOM_BasicAttribute` and
  `LDOM_BasicText` are views (document, node id) over the arena; their members, including
  the protected ones `LDOM_Element` and the parser use, keep the native list surgery
  (attributes after the children in one sibling chain, `GetSibling` skipping and unlinking
  null nodes, `RemoveAttribute` finding but not removing an attribute).
- **DOM values.** `LDOM_Node` and its value subclasses (`LDOM_Element`, `LDOM_Attr`,
  `LDOM_CharacterData`, `LDOM_Text`, `LDOM_Comment`, `LDOM_CDATASection`) are structs that
  embed the node and dereference to it; the cached last child (`myLastChild`, written by
  `const` members natively) is a `Cell`.
- **Streams.** `Standard_IStream` is `std::io::Read` behind `LdomIStream`, which keeps the
  C++ eof and fail flags and `gcount`, so a short `read` or a full `getline` ends the input
  as natively. `Standard_OStream` is `std::io::Write`; `LDOM_OSStream` is the chunked string
  buffer of `LDOM_SBuffer`.
- **Parser callbacks.** The virtual `LDOMParser::startElement`/`endElement` are the trait
  `LdomParserHooks`; `parse` uses the default hooks.
- **Native crashes.** Where the native code dereferences a null result, the port reports an
  error instead: an invalid character reference (`Decode` returns null) fails the record or
  the parse with a message, and an embedded NUL byte at the end of the input, on which the
  native reader loops forever, is skipped. Every other quirk is kept, including the text
  record `>` after a CDATA section and the attribute search of `getAttributeNode` that
  starts after the first attribute; `tests/ldom_native_oracle.rs` compares the port with
  the output of a native program.

## 10. TNaming: the evolution graph

TNaming records how shapes evolve. A `TNaming_NamedShape` holds a list of `TNaming_Node`s,
one per (old shape, new shape) pair; the root attribute `TNaming_UsedShapes` maps each
shape (`TopTools_ShapeMapHasher`, so orientation is ignored) to a `TNaming_RefShape`, the
head of the chain of nodes that use that shape. Natively all of this is raw pointers with
explicit `new`/`delete`.

- **Nodes and ref shapes** are `Arc<TNamingNode>` and `Arc<TNamingRefShape>` with their
  fields in one `SharedState`; a native pointer is `Option<Arc<_>>` (`TNamingNodePtr`,
  `TNamingRefShapePtr`) and pointer equality is `Arc::ptr_eq` (two null pointers are
  equal, as natively). The chain links (`nextSameAttribute`, `nextSameOld`,
  `nextSameNew`, `myOld`, `myNew`, `FirstUse`) are strong, so a walk never meets a freed
  node. The native `delete` of a ref shape is `release`, which drops its first use (a ref
  shape's first use points back to it); the `delete` of a node drops its links to the other
  nodes of the shape chains, which breaks the remaining cycles, and keeps its shapes,
  attribute and attribute link readable. Natively an iterator that stands on a deleted node
  reads freed memory (`TNaming::Transform` iterates a label while its builder clears the
  named shape, which is safe only when the builder backs it up, in a later transaction);
  the port reads what `RemoveNode` left in the node instead. `myAtt` is a
  `WeakHandle<TNamingNamedShape>`: `BackupCopy` and `Restore` move the nodes from one
  attribute to the other and re-point it, as natively.
- **Clear on destruction.** `~TNaming_NamedShape` calls `Clear`, which unlinks the nodes
  from the shape chains through the root's map. The Rust `Drop` does the same. An attribute
  with a null label (a removed attribute, a backup after `RemoveBackup`, a label freed by
  `TDF_Data::Destroy`) is left alone, as natively. `TDF_LabelNode::Destroy` therefore drops
  the old node state after releasing the node's lock, so that the destructor sees the reset
  node instead of deadlocking on it.
- **The map** is a `DataMap<Shape, Arc<TNamingRefShape>, ShapeMapHasher>` in the
  attribute's `SharedState`; `Map()` returns its write guard, which a caller drops before it
  calls back into the framework. Native quirks kept: `Delete` binds a new null-shape ref
  shape each time (the previous one stays reachable from its nodes only), and `RemoveNode`
  unbinds the key of the shape, whichever ref shape is bound to it.
- **Iterators.** `TNaming_Iterator`, `TNaming_NewShapeIterator`, `TNaming_OldShapeIterator`
  and `TNaming_SameShapeIterator` keep the current node. The constructors from a label whose
  root has no `TNaming_UsedShapes` leave the native iterator uninitialized; the port starts
  at the end. The copy constructors of the new- and old-shape iterators descend (or ascend)
  to the evolutions of the current shape; they are `from_new_shape_iterator` and
  `from_old_shape_iterator`, and the structs do not implement `Clone`. `Next` and the
  accessors at the end are `Standard_NoMoreObject`/`Standard_NoSuchObject` errors (some are
  crashes natively), and a node whose attribute is gone is `Standard_NullObject`.
- **Overloads.** `TNaming_Builder::Generated(new)` is `generated`, `Generated(old, new)` is
  `generated_from`; the iterator constructors are `new` (a named shape), `from_label`,
  `from_label_in_transaction`, `from_used_shapes`, `from_used_shapes_in_transaction` and
  `from_iterator`. `TNaming_NamedShape::Get` is an inherent method that a `Handle` hides
  (`Handle::get`); call it as `TNamingNamedShape::get(&named)`.
- **DumpJson** of a node or a ref shape with a negative depth recurses without end natively
  (a ref shape's first use points back to it) and overflows the stack; the port returns an
  error for a graph link dumped at a negative depth.
- **TNaming_Name** is a value (`Clone`); `TNaming_Naming::GetName` returns a copy and
  `ChangeName` the write guard of the name. The native constructor leaves the shape type and
  the orientation uninitialized; the OCCT allocator clears memory, so they read
  `TopAbs_COMPOUND` and `TopAbs_FORWARD`, which the port uses.

## 11. TDataXtd: the extended attributes

- **Construction attributes.** `TDataXtd_Point`, `Axis`, `Plane`, `Placement` and `Shape`
  derive from `TDataStd_GenericEmpty` and are declared with the
  `rustycad_tklcaf::tdata_std_empty_attribute!` macro. Their geometry is the
  `TNaming_NamedShape` of their label; `TDataXtd_Geometry` reads it back. The `Set`
  overloads are `set` (find or add), `set_point`, `set_line` and `set_plane`.
- **Derived-attribute registration.** Natively `IMPLEMENT_DERIVED_ATTRIBUTE` registers the
  five construction attributes when the library loads. Rust has no load-time initializer,
  so the constructor of each of them calls `TDataXtd::register_derived_attributes`, which
  registers all five once, in the native link order. The guard is an `AtomicBool`, not a
  `Once`: the registry builds its prototypes with these constructors, which call back into
  the registration.
- **TDataXtd_Pattern** is the trait `TDataXtdPattern` (`PatternID`, `NbTrsfs`,
  `ComputeTrsfs`); its static `GetID` is the provided `TDataXtdPattern::get_id` (on
  concrete types only), which is also the `ID` of `TDataXtd_PatternStd`. `ComputeTrsfs` fills a slice from its start; a slice
  shorter than `NbTrsfs` is `Standard_OutOfRange`. `NbTrsfs` multiplies by
  `NbInstances2` whenever it is set, also for a linear pattern, as natively.
- **OCC2932 checks.** The setters of `Constraint`, `PatternStd` and `Position` do not back
  up an unchanged value. Natively they dereference the new handle; the port reads a null
  new handle as a change. The four-geometry `TDataXtd_Constraint::Set` compares the third
  stored geometry with the fourth given one, as natively.
- **TDataXtd_Presentation** holds a `Quantity_NameOfColor` as `NamedColor`. The old-enum
  conversions are in `presentation_colors`. An old value that gives a negative color is
  `Standard_OutOfRange` (natively an invalid enumerator).
- **TDataXtd_Triangulation** covers the `Poly_Triangulation` methods so that each change
  is backed up. `Restore` and `Paste` install a copy of the mesh. The methods on an
  attribute without a mesh are `Standard_NullObject` errors (natively a null dereference).

## 12. AppStd

`AppStd_Application` is `AppStdApplication`, a `TDocStd_Application` built like
`TDocStdStandardApplication` (`new` returns its bound handle). Its `ResourcesName` is
`Standard`. Its `DumpJson` is an inherent method: the native one hides the non-virtual
`TDocStd_Application::DumpJson`, and it dumps that one as its base class.

## 13. TPrsStd: the presentation attributes (TKVCAF)

TKVCAF (`crates/tkvcaf`, `rustycad-tkvcaf`) joins OCAF to the AIS of TKV3d.

- **Attributes.** `TPrsStd_AISViewer` (root attribute holding the `AIS_InteractiveContext`)
  and `TPrsStd_AISPresentation` (the interactive object of a label) are `TdfAttribute`
  structs; the context and the object (`myAIS`) live in a `SharedState`, which no member
  holds while it calls the context, a driver or the data. The settings of a presentation
  (color, material, transparency, width, modes) live in the label's `TDataXtd_Presentation`;
  `getData` is `get_data` and a label without data is `Standard_NoMoreObject`. The
  presentation sets itself as the owner of its object through its self handle (section 2).
- **Drivers.** `TPrsStd_Driver` is the trait `TPrsStdDriver` (`update(label, &mut ais)`);
  the six standard drivers are stateless structs. `TPrsStd_DriverTable::Get` is a
  process-wide `OnceLock` handle filled with the standard drivers, as the native static.
  Drivers that natively build an `AIS_Axis` or an `AIS_Line` take the process-wide
  `UnitsAPI` of TKV3d (`prs_dim_units_api`).
- **Native quirks kept.** `SetMaterial` tests `HasOwnMode` (not `HasOwnMaterial`) before
  its early return; `SetWidth` does not build a missing object; `AISErase` on a detached
  label does nothing; `TPrsStd_ConstraintDriver` places a dimension (not a
  `PrsDim_Relation`) at a `TDataXtd_Position` through a null pointer natively, which is a
  `Standard_NullObject` error here.
- **Constraint tools.** Every value-returning `ComputeX` starts from a null object, as
  natively. The deprecated in/out overloads are excluded (ADR 0002), so the branches of
  `computeXImpl` that run only on an existing object are not ported. `ComputeTextAndValue`
  formats `%g` with the `Standard_CString` `sprintf` and converts through the current
  units; `ComputeRound` returns null when `PrsDim_RadiusDimension` fails (the native
  `OCC_CATCH_SIGNALS` block); the other exceptions propagate as `Failure` errors.

TKXCAF, the XDE document on top of these toolkits, has its own note:
[xcaf-design.md](xcaf-design.md). The one rule it adds here: a `Backup` that the native
reaches through a virtual call of a derived attribute goes through the attribute's own
handle (`backup_as_dynamic` in `tdata_std::support`, used by `TDataStd_Real::Set`), so the
delta records the most derived type.
