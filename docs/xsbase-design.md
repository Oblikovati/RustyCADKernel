# XSBase design notes (TKXSBase)

This note fixes how TKXSBase, the foundation of the exchange formats (STEP, IGES and the
DE wrappers), maps to Rust. The port is `crates/tkxsbase` (`rustycad-tkxsbase`) with one
module per package: `moni_tool`, `xs_message`, `interface`, `transfer`, `if_graph`,
`if_select`, `transfer_brep`, `xs_algo`, `xs_control` and `lib_ctl`. It depends on the
crates of its native EXTERNLIB (TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep,
TKTopAlgo, TKShHealing). The rules of [geom-design.md](geom-design.md) sections 1 to 4
(abstract class as trait, `Handle<dyn Trait>`, `SharedState`, exceptions as `Failure`)
and [ocaf-design.md](ocaf-design.md) section 1 (names keep the package prefix, overload
suffixes, package classes as unit structs) apply. Code comments and ledger notes cite
this note as "xsbase-design.md section N".

## 1. Entities and the interface model

- **Entities** are any transient: `occ::handle<Standard_Transient>` is
  `Handle<StandardTransient>`. A format toolkit (StepData, IGESData) casts it to its own
  entity classes. Identity (the native pointer compare and the pointer hash of
  `NCollection_IndexedMap<occ::handle<Standard_Transient>>`) is the handle identity.
- **Interface_InterfaceModel** is the trait `InterfaceModel: Transient`. Its pure virtual
  members (`ClearLabels`, `ClearHeader`, `GetFromAnother`, `NewEmptyModel`, `DumpHeader`,
  `PrintLabel`, `StringLabel`) are required methods; the virtual ones are provided methods
  with the native body; the non-virtual ones are provided methods that no model
  overrides. The private fields (the indexed map of entities, the report maps, the
  syntactic and semantic checks, the dispatch flag, the category string and the GTool)
  live in the embedded `InterfaceModelBase` (one `SharedState`), reached through the
  required accessor `model_base()`. Entity numbers are 1-based as natively.
- **Interface_Protocol**, **Interface_GeneralModule**, **Interface_ReaderModule** are
  traits over the same pattern; a protocol names its resources (other protocols) and
  maps an entity to a case number (`CaseNumber`, `TypeNumber`).
- **Interface_Check** keeps its six native message lists (fails, warnings, infos, each
  with the final and the original text) as `Option<Vec<Handle<HAsciiString>>>`, so the
  native "list not created yet" state is kept; **Interface_CheckIterator** keeps the
  native sequence of checks with their entity numbers.
- **Interface_EntityIterator** is a value with the native shared list and cursor
  (`occ::handle<NCollection_HSequence>` and `occ::handle<Interface_IntVal>`): copies share
  them, so `Start`, `Next` and `More` on a copy move the original cursor as natively.
  Its virtual `Start` is the trait `EntityIteratorStart`, which the graph iterators of
  `Interface_Graph` override.
- **Interface_ParamSet**, **Interface_FileReaderData**, **Interface_FileParameter** keep
  the native 1-based parameter numbering and the native growth of their arrays.
- **Interface_Static** derives from `MoniTool_TypedValue`: the struct embeds a
  `MoniToolTypedValue` and registers it as its RTTI parent (ocaf-design.md section 7,
  derived classes by embedding).

## 2. Generic libraries (LibCtl) keyed by protocol

Natively `LibCtl_Library` is a template instantiated per (module, protocol) pair
(`Interface_GeneralLib`, `Interface_ReaderLib`, and in the format toolkits
`StepData_WriterLib`, `IGESData_SpecificLib`, ...). Each instance has a process-wide
linked list of `(module, protocol)` global nodes, filled by `SetGlobal` when a format
toolkit initialises (`StepAP214::Init`). A library object built from a protocol collects
the global nodes whose protocol has the same dynamic type as the protocol or one of its
resources, recursively, and `Select` dispatches an entity to the first module whose
protocol gives it a positive case number.

- `lib_ctl::Library<K: LibraryKind>` is the one generic port of the template.
  `LibraryKind` names the module and protocol trait objects of an instance. The native
  `*_GlobalNodeOf*Lib` and `*_NodeOf*Lib` classes are the generic `GlobalNode<K>` and
  `Node<K>` handles (linked lists, as natively, so the dispatch order is the native
  registration order).
- **Global state.** The native global list is a static per instance. The port keeps the
  list in a `LibraryRegistry<K>` value. Each `LibraryKind` gives its process-wide
  registry (a `OnceLock`) for the static members (`SetGlobal`, the constructors); the
  `*_in` variants take an explicit registry, so tests and embedders can build a library
  over a private registry without touching process state. The process-wide registries,
  the static table of `Interface_Static`, the active protocol of `Interface_Protocol`,
  the model templates of `Interface_InterfaceModel`, the timers of `MoniTool_Timer` and
  the typed-value library of `MoniTool_TypedValue` are the only global mutable state of
  the toolkit, all because the native API exposes them as statics; each is a `RwLock`
  behind a `OnceLock`.

## 3. The transfer process

- `Transfer_ProcessForTransient` and `Transfer_ProcessForFinder` are two instances of
  the native template `Transfer_TransferProcess`. The port writes the algorithm once, as
  the generic `TransferProcess<K: TransferProcessKind>`, over a kind
  (`TransferTransientKind`: `Handle<StandardTransient>` with identity hashing;
  `TransferFinderKind`: `Handle<dyn TransferFinder>` with the `GetHashCode`/`Equates`
  hashing of `Transfer_FindHasher`). The actor and the iterator templates are generic
  over the same kind. The binder map is the native `NCollection_IndexedDataMap`, so
  `MapIndex`, `Mapped` and the iteration of the results keep the native numbering.
- The concrete `TransferProcessForTransient` / `TransferProcessForFinder` embed the
  generic body and dereference to it. The virtual members (`PrintTrace`, `CheckNum`)
  are the trait `TransferProcessOps<K>`; the body keeps a weak handle to its owner
  (bound by the constructors, which return handles), dispatches its own calls through
  it, and passes it to the actors as the native code passes `this`.
  `Transfer_TransientProcess` and `Transfer_FinderProcess` embed their instance (the
  RTTI parent), add the model and graph members and override the virtual members.
- **Transfer_Binder** is the trait `TransferBinder: Transient` over the embedded
  `TransferBinderBase` (status, execution status, check, next result); the concrete
  binders (`Transfer_SimpleBinderOfTransient`, `Transfer_VoidBinder`,
  `Transfer_MultipleBinder`, `Transfer_TransientListBinder`,
  `TransferBRep_ShapeBinder`...) are structs.
- **Actors** (`Transfer_ActorOfProcessForTransient` and its subclasses) are the trait
  `TransferActorOps<K>` with the native virtual members (`Recognize`, `Transferring`) as
  provided methods; the chain of actors (`SetNext`, `IsLast`) lives in the embedded
  `TransferActor<K>`. `Transfer_ActorOfTransientProcess` adds the trait
  `TransferActorOfTransientProcessOps` (`Transfer`, `TransferTransient`); its
  `Transferring` override is the function `transferring_by_transient_process`, which a
  derived actor calls from its own `transferring` (the same for the finder actors).
- The native stream buffers of the messenger send their text when they go out of
  scope; the port collects the text and sends it at the same point, so the trace
  lines keep the native order.
- **Exceptions.** The native process catches `Standard_Failure` around an actor when
  `ErrorHandle` is set and records it in the binder check; the port does the same with
  the `Failure` an actor returns. `Transfer_TransferDeadLoop` and
  `Transfer_TransferFailure` are `Failure` errors with these exception names.

## 4. Selections and the work session (IFSelect, XSControl)

`IFSelect_Selection`, `IFSelect_Dispatch`, `IFSelect_Modifier`, `IFSelect_Signature` and
their abstract subclasses are traits over embedded bases, with one struct per concrete
class. `IFSelect_WorkSession` and `XSControl_WorkSession` keep their native item tables
(`NCollection_IndexedDataMap` of items and names), so item numbers and the order of
`ItemNames` are the native ones. The command interpreter (`IFSelect_SessionPilot`,
`IFSelect_Act`, the `*_Functions` packages) reads commands as text and writes plain text
to a `std::io::Write`, as the native `std::cout` output.

The work session details:

- The items are a vector of (item, attribute) pairs in ident order (the native indexed
  map); the names are the TKernel `DataMap`, so `ItemNames` keeps the native order.
- `ClearData` is the only virtual member: `IFSelectWorkSessionOps::clear_data`. A
  derived session (`XSControl_WorkSession`) embeds `IFSelectWorkSession` and calls
  `bind_self` with its handle, so that `SetModel` and `ClearData(1)` call the override.
- The native static `errhand` is a per-thread flag (the native sessions run on one
  thread; a process-wide flag would let a guarded call of one thread change the error
  handling of the others). A guarded call (native `try`
  around the call with `errhand` cleared) reports the error and gives an empty result
  when the errors are handled, else returns the error. The native defect of the
  sending calls is kept: after a successful guarded sending, `errhand` stays cleared.
- The editors and the edit forms are built as handles (the editor receives its form as
  `this`): `IFSelectEditorBase::bind_self` records the handle of an editor.

The XSControl details:

- `XSControl_Controller` is abstract: the trait `XSControlController` over the embedded
  `XSControlControllerBase`. A norm calls `bind_self` with its handle, because `Record`
  registers `this` in the process-wide table of the norms.
- `XSControl_Vars` has only virtual members: they are `XSControlVarsOps` methods. The
  in-out names (`const char*&`) are `&mut String`.
- `XSControlWorkSession::new` returns the handle of the session, bound to its
  IFSelect base (so `SetModel` calls the overridden `ClearData`) and given to the
  controller in `Customise`. The native global mutex of `SelectNorm`, `NewModel` and
  `TransferWriteShape` is a process-wide `Mutex`.
- The transfer reader's `Context` is a write guard on the map: drop it before the next
  call on the reader.

XSAlgo (the shape processing) needs two items from outside TKXSBase:

- `DE_ShapeFixParameters` is a TKDE header that TKXSBase includes without linking TKDE
  (TKDE does not depend on TKXSBase). TKDE is not ported, so `xs_algo` defines it as
  `DEShapeFixParameters`; its parity rows stay with TKDE.
- The cascade length unit of `UnitsMethods` is a process global natively. The TKernel
  port keeps it per `UnitsMethods` instance, so `xs_algo_units()` is the process-wide
  instance of the exchange formats.

## 5. Native quirks kept

- `MoniTool_AttrList::SameAttributes` copies the attribute map: its documentation says
  the lists are shared, but the native assignment copies the map (the attribute objects
  are shared).
- Every native map that a member iterates (`NCollection_DataMap`) is the TKernel
  `DataMap`, so the iteration order is the native bucket order.
