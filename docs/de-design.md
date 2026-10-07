# Data exchange framework design notes (TKDE)

This note fixes how the TKDE toolkit (`crates/tkde`, `rustycad-tkde`: package `DE`) maps to
Rust. It is the base that the exchange formats (STEP, IGES, STL, OBJ, glTF, VRML, PLY and the
OCAF formats of TKDECascade) plug into. The rules of [geom-design.md](geom-design.md) sections
1 to 4 (abstract class as trait, `Handle<dyn Trait>`, `SharedState`, exceptions as `Failure`)
and [ocaf-design.md](ocaf-design.md) sections 1 and 2 (names with the package prefix, overload
suffixes, self handles) apply; this note covers what they do not. Code comments and ledger
notes cite it as "de-design.md section N".

## 1. Classes

| OCCT | Rust |
| --- | --- |
| `DE_ConfigurationContext` | struct `DeConfigurationContext` (a transient; its map in one `SharedState`) |
| `DE_ConfigurationNode` (abstract) | trait `DeConfigurationNode: Transient` over the embedded `DeConfigurationNodeBase` (`GlobalParameters`, `myIsEnabled`, the self handle) |
| `DE_ConfigurationNode::DE_SectionGlobal` | `DeSectionGlobal` (a `Copy` value) |
| `DE_ShapeFixConfigurationNode` (abstract) | trait `DeShapeFixConfigurationNode: DeConfigurationNode` over `DeShapeFixConfigurationNodeBase` |
| `DE_ShapeFixParameters`, its `FixMode` | `DeShapeFixParameters`, `DeShapeFixParametersFixMode` |
| `DE_Provider` (abstract) | trait `DeProvider: Transient` over `DeProviderBase` (`myNode`) |
| `DE_Provider::ReadStreamNode`, `WriteStreamNode` and their lists | `DeProviderReadStreamNode<'a>`, `DeProviderWriteStreamNode<'a>`, `DeProviderReadStreamList<'a>`, `DeProviderWriteStreamList<'a>` |
| `DE_Wrapper` | struct `DeWrapper` |
| `DE_ValidationUtils` | unit struct `DeValidationUtils` |
| `DE_PluginHolder<T>`, `DE_MultiPluginHolder<T...>`, `DEPLUGIN` | `DePluginHolder<T>`, `DeMultiPluginHolder<T>` over a tuple, macro `deplugin!` |

- **Overrides that call the base.** A format node overrides `Load` and `Save` and calls
  `DE_ShapeFixConfigurationNode::Load`/`Save` (and `DE_ConfigurationNode::Load(path)` is
  hidden by the override natively). A Rust trait method cannot call the method it overrides,
  so the base bodies are also provided methods with their own names
  (`shape_fix_load`, `shape_fix_save`, `load_resource_file`); the format node's `load` and
  `save` call them where the C++ calls the base class.
- **Copy constructors** (`DE_ConfigurationNode(handle)`, `DE_ShapeFixConfigurationNode(handle)`)
  are base constructors from another node: `DeConfigurationNodeBase::copied_from`,
  `DeShapeFixConfigurationNodeBase::copied_from`.
- **Fallible members.** A member that can throw natively (through `OSD_File`, `Value`, or a
  provider override) returns `Result<_, Failure>`: `DeConfigurationContext::load*`,
  `DeConfigurationNode::load_resource_file` and `save_resource_file`, the `DeWrapper` loads,
  saves, lookups and transfers, and every `DeProvider` transfer.

## 2. Types that TKDE only forward-declares

`DE_Provider` and `DE_Wrapper` name `TDocStd_Document` (TKLCAF) and `XSControl_WorkSession`
(TKXSBase) through forward declarations: TKDE links neither toolkit and never dereferences
the handles. Rust signatures need a type:

- `occ::handle<TDocStd_Document>` is `Handle<TDocStdDocument>`; `rustycad-tkde` depends on
  `rustycad-tklcaf` for it (TKLCAF does not depend on TKDE, so there is no cycle).
- `occ::handle<XSControl_WorkSession>&` is `&mut DeWorkSession`, an alias of
  `Handle<StandardTransient>`, because TKXSBase has no Rust crate yet. TKDE only tests it for
  null and passes it on, so the alias keeps the native behaviour; a provider casts it to the
  work session class with `cast`. Once the work session is ported the alias becomes its
  handle and the providers keep their code.
- `std::istream&` and `std::ostream&` in the stream nodes are `&mut dyn IStream` and
  `&mut dyn OStream`, the TKernel stream ports with the libstdc++ state bits, so the
  `fail()` checks of `DE_ValidationUtils` and the position restore of
  `CreateContentBuffer` behave as natively.

## 3. The global wrapper

`DE_Wrapper::GlobalWrapper()` is a process-wide wrapper that the plugin holders of every
format library register into when the library is loaded, and that DRAW and applications read.
The port keeps it, because removing it would change what a registration is observable from:

- `DeWrapper::global_wrapper()` returns the handle stored in a process-wide
  `RwLock<Handle<DeWrapper>>`, created on first use as an empty wrapper;
  `DeWrapper::set_global_wrapper` replaces it, a null handle is ignored, as natively.
- `DeWrapper::global_load_mutex()` is the process-wide `Mutex<()>` the plugin holders take
  while they register and unregister.
- `DePluginHolder<T>` creates a `T`, registers it into the global wrapper under the mutex and
  unregisters it when dropped. `deplugin!(name: A, B)` defines a `name()` function that keeps
  one `DeMultiPluginHolder<(A, B)>` alive for the life of the process, the native static of
  `PLUGINFACTORY` (Rust has no shared-library entry point to export it from).

Everything else takes the wrapper as a value: a test or an application that wants an isolated
registry creates its own `DeWrapper`, as native code does with `new DE_Wrapper()`.

## 4. Registry order

The format map is the native `NCollection_DataMap<TCollection_AsciiString,
NCollection_IndexedDataMap<TCollection_AsciiString, occ::handle<DE_ConfigurationNode>>>`, kept
as the TKernel `DataMap` and `IndexedDataMap` ports. Their bucket layout, string hash and
head insertion are native, so the iteration order of the formats (which decides which format
a lookup tries first and the order of `Save`) and the vendor order inside a format (the
priority) are the native ones. `tools/oracle/tkde_configuration_probe.cxx` records both from
the native library and `crates/tkde/tests/de_native_oracle.rs` compares them.

- `Bind` keeps the first node of a format and vendor (`IndexedDataMap::Add` returns the old
  index and does not replace the node) and still returns `true`.
- `UnBind` removes with `RemoveKey`, which moves the last vendor into the hole, as natively.
- `ChangePriority` copies the vendor map, rebuilds it in priority order and rebinds it.
- `Copy` is the implicit C++ copy constructor: the maps are copied (`DataMap` copy keeps the
  native rechaining) and the nodes are shared, `KeepUpdates` is kept. The constructor from a
  handle binds every node of the source one by one and, as natively, writes its own `false`
  `KeepUpdates` into the source.
- A lookup walks a snapshot of the nodes in map order and calls the node's virtual members
  outside the wrapper's lock: a node that changes the wrapper during a lookup does not
  deadlock (natively it would invalidate the iterator).

## 5. Native quirks kept

- `DE_ConfigurationContext::LoadStr` raises `Standard_OutOfRange` on a line of blanks
  between two lines (`WhatKindOfLine` reads `Value(0)`); the parameters read before it stay.
- `LoadFile` of a file that cannot be opened reports a failure and returns `true`;
  `DE_ConfigurationNode::Load()` with the default empty path raises `Standard_ProgramError`
  from `OSD_File::Open`.
- A value that ends with `:` loses it (`a : b:` gives `b`); a value `\ x` keeps the space.
- `GetInteger` accepts `1e-3` (an integer prefix and no `.`): 1. `GetReal` reads hexadecimal
  (`0x10` is 16) through `Strtod`.
- `DE_ShapeFixConfigurationNode::Load` casts any integer to `FixMode`, so `FixMode` is a
  newtype over `i32` with the three named values as constants.
- `DE_ShapeFixConfigurationNode::Save` keeps the native comment typos and the misplaced `!`
  inside two comment lines.
