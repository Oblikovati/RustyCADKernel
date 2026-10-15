# TObj design notes (TKTObj)

`crates/tktobj` (`rustycad-tktobj`) ports the TObj package of OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`: typed objects of OCAF-based models. It sits on
the TKLCAF and TKCDF ports; the rules of [ocaf-design.md](ocaf-design.md) and
[geom-design.md](geom-design.md) sections 1 to 4 apply. Code comments and ledger notes cite
this note as "tobj-design.md section N".

## 1. Classes

| OCCT | Rust |
| --- | --- |
| `TObj_Object` | trait `TObjObject` (virtual members, native bodies as provided methods) over `TObjObjectBase`; non-virtual members in the extension traits `TObjObjectState` (labels, names, flags, father), `TObjObjectData` (the protected value tools) and `TObjObjectArrays` (the protected array tools); statics `GetObj`, `Detach(label)` on `TObjObjectBase` |
| `TObj_Partition` | trait `TObjPartition` over `TObjPartitionBase`; the concrete `TObj_Partition` is `TObjStandardPartition`; statics `Create`, `GetPartition` on `TObjStandardPartition`; its `TObj_Object` overrides come from `tobj_partition_object_members!` |
| `TObj_HiddenPartition` | `TObjHiddenPartition` |
| `TObj_Model` | trait `TObjModel` over `TObjModelBase` (`NewEmpty` required); non-virtual members in `TObjModelState`; statics `GetDocumentModel`, `SetNewName` on `TObjModelBase` |
| `TObj_Application` | trait `TObjApplication: TDocStdApplication` over `TObjApplicationBase`; the concrete class is `TObjStandardApplication` with `GetInstance` |
| `TObj_ObjectIterator` | trait `TObjObjectIterator` (`More`, `Next`, `Value` through `&self`); the empty base class is `TObjStandardObjectIterator` |
| `TObj_LabelIterator` | trait `TObjLabelIterator` over `TObjLabelIteratorBase`; `tobj_label_iterator_members!` fills `More`, `Next`, `Value` |
| `TObj_OcafObjectIterator`, `TObj_ReferenceIterator`, `TObj_SequenceIterator`, `TObj_ModelIterator` | same names, `TObj` prefix |
| `TObj_CheckModel` | `TObjCheckModel`, embedding the `Message_Algorithm` port with its own runtime type (message keys follow `TObj_CheckModel`) |
| `TObj_Persistence` | `TObjPersistence` (a registration, unregistered on `Drop`) and trait `TObjPersistenceFactory` (`New`) |
| `TObj_Assistant` | `TObjAssistant`, a class of statics |
| `TObj_TObject`, `TObj_TModel`, `TObj_TNameContainer`, `TObj_TReference`, `TObj_TXYZ`, `TObj_TIntSparseArray` | `TObjTObject`, `TObjTModel`, `TObjTNameContainer`, `TObjTReference`, `TObjTXyz`, `TObjTIntSparseArray` |
| `TObj_DeletingMode` | `TObjDeletingMode` |
| nested enums (`TypeFlags`, `ObjectState`, `DataTag`, `ChildTag`, `RefTag`) | unit structs of constants (`TObjTypeFlags`, `TObjObjectStateFlags`, `TObjObjectDataTag`, ...) |

A getter named `Get` is `get_object`, `get_map` or `get_xyz`, because `Handle::get` would
shadow it. `occ::handle<Standard_Type>` arguments are `Option<&'static RuntimeType>`.
`Message_Messenger` handles are `Arc<Messenger>`, as in the TKernel port.

## 2. Self handles and construction

`TObjObjectBase` and `TObjModelBase` keep a weak self handle like the OCAF bases
(ocaf-design.md section 2). A native constructor body that uses `this` runs once the object
is in its handle: `TObjObjectBase::attach(&object, set_name)` is the body of
`TObj_Object(theLabel, theSetName)` (it binds the handle, puts the `TObj_TObject` on the
label and names the object from its partition). `TObjObjectBase::new(label)` alone is the
persistence constructor `TObj_Object(const TObj_Persistence*, theLabel)`.

`~TObj_Model` closes the model; `TObjModelBase`'s `Drop` runs the base `Close`, as a native
destructor dispatches to the base class. A model referenced by its `TObj_TModel` attribute
lives until it is closed, as natively.

## 3. Statics

`TObj_Assistant`, the `TObj_Persistence` map and `TObj_Application::GetInstance` are native
function-local statics; they are process-wide mutex-guarded values here. A lock is never
held while a model, an object or a persistence tool is called. The native GTests run one
by one; the Rust fixtures serialize too.

## 4. Persistence

The native `IMPLEMENT_TOBJOCAF_PERSISTENCE` registers each type at static initialization.
Rust has no static constructors: the registry registers `TObj_Partition` and
`TObj_HiddenPartition` when it is first used, and an application registers its own types
with `TObjPersistence::register_static` (or a scoped `TObjPersistence::new`) before a clone
or a retrieval needs them. `DumpTypes` lists the names in the order of the ported
`NCollection_DataMap`.

## 5. Native file formats

`BinTObjDrivers` and `XmlTObjDrivers` (TKBinTObj, TKXmlTObj) are out of scope (no OCCT
native file formats, user decision 2026-09). `TObj_Model::SaveAs` and `Load` run their
native bodies over the drivers of the application's format; without one, `SaveAs` reports
`TObj_Appl_SDriverFailure` and returns false, a file load fails with the retrieval error,
and a stream load initializes an empty model. The two stream GTests that register the
BinTObj format are native-only ledger rows; the native tests run and pass.

## 6. Native behaviour kept

- `TObj_TIntSparseArray::SetValue` never calls `Backup`: a command that changes only the
  array leaves its label unflagged, the optimized `TDF_Data::CommitTransaction` skips it,
  and the command records no undo. With another attribute backed up on the label the
  array's own delta undoes and redoes.
- `TObj_SequenceIterator::More` (a const member) moves past objects of another type, and a
  null entry ends the iteration.
- `TObj_Partition::GetNewName` moves the last index only past names that were taken.
- A clone keeps no name when the source name is registered already.
- `TObj_Model::Save` on a model that was never saved fails with `Standard_NoSuchObject`
  (`TDocStd_Document::GetPath` of an unstored document).
- `TObj_ReferenceIterator` keeps the filtered object of a skipped reference as its current
  object until the next match.

Not kept: `TObj_SequenceIterator::Value` past the end raises `Standard_OutOfRange` natively
(a null sequence crashes); the port returns a null handle, because the iterator interface
returns a handle. Null-handle dereferences that crash natively are `Standard_NullObject`
failures.
