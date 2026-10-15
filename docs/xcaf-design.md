# XCAF design notes (TKXCAF)

This note fixes how TKXCAF, the extended data exchange (XDE) document, maps to Rust
(`crates/tkxcaf`, `rustycad-tkxcaf`: packages XCAFDoc, XCAFDimTolObjects, XCAFNoteObjects,
XCAFView, XCAFPrs, XCAFApp). TKXCAF sits on the OCAF toolkits, so the rules of
[ocaf-design.md](ocaf-design.md) apply (attributes as `TdfAttribute` structs with their
fields in `SharedState`, static `Set`/`Get` members as associated functions, exceptions as
`Failure`); this note covers what they do not. Code comments and ledger notes cite it as
"xcaf-design.md".

Native file formats (BinXCAF, XmlXCAF and the persistence drivers) are not ported; a
document's storage format is only a name.

## 1. Attributes and tools

- **Plain attributes.** `XCAFDoc_Color`, `XCAFDoc_Location`, `XCAFDoc_Material`,
  `XCAFDoc_DimTol`, `XCAFDoc_Centroid`, `XCAFDoc_GraphNode`, `XCAFDoc_LengthUnit`,
  `XCAFDoc_ShapeMapTool`, `XCAFDoc_VisMaterial`, `XCAFDoc_AssemblyItemRef` and
  `XCAFDoc_VisMaterialTool` are `TdfAttribute` structs. Their static `Set(label, ...)` finds
  the attribute of the label or adds a new one first in its list, as the native default
  `AddAttribute(A)` (`append = false`).
- **Derived attributes.** `XCAFDoc_Area` and `XCAFDoc_Volume` embed a `TDataStd_Real`, which
  is their RTTI parent, so their handles cast to `TDataStdReal`. `TDataStd_Real::Set` backs
  up through the attribute's own handle (ocaf-design.md section 4), so a delta records the
  derived type, as the native virtual `BackupCopy`.
- **Tools.** The tools derived from `TDataStd_GenericEmpty` (document, shape, color, layer,
  material, GD&T, notes, view and clipping-plane tools, `XCAFDoc_Dimension`,
  `XCAFDoc_GeomTolerance`, `XCAFDoc_View`) share one macro (`generic_empty_attribute!`) that
  declares the struct, its RTTI, the empty `Restore`/`Paste` and the `DumpJson` body. The
  shape tool caches of a tool (`myShapeTool`) are a `SharedState` field.
- **Registration.** `IMPLEMENT_DERIVED_ATTRIBUTE` runs at library load natively; Rust has no
  load-time initializer, so the constructor of every derived attribute calls
  `register_derived_attributes`, which registers the sixteen derived attributes once (the
  namespaced ones with their `"xcaf"` type names).
- **Notes.** `XCAFDoc_Note` is the trait `XCAFDocNote` (its fields in `XCAFDocNoteFields`);
  `XCAFDocNoteMembers` holds the non-virtual members and the static `Get`/`IsMine` are the
  functions `xcaf_doc_note_get`/`xcaf_doc_note_is_mine`. `XCAFDoc_NoteBalloon` embeds
  `XCAFDoc_NoteComment`, its RTTI parent.
- **GD&T and views.** The dimension, tolerance, datum and view attributes keep their
  description objects in child labels; `GetObject` creates the children it reads, as
  natively (`FindChild(tag)` with the default `create = true`).

## 2. Overloads and out-parameters

Native overloads get distinct Rust names: the shape overloads of the tools take a `shape_`
or `_of_shape` name, the RGB (`Quantity_Color`) overloads of the color tool an `_rgb` suffix,
the ASCII-name overloads an `_ascii` suffix, and the `XCAFDoc_AssemblyItemId` and label
overloads of the notes tool `_label`. A `bool` with out-parameters becomes an `Option` (for
found / not found) or the out-parameter stays a `&mut` when the native appends to a caller's
sequence. The parity ledger maps each overload.

## 3. Presentation (XCAFPrs) and the application (XCAFApp)

- `XCAFPrs_AISObject` implements the TKV3d traits of `AIS_ColoredShape` over an
  `AisColoredShapeBase`, with the public `ais_colored_shape_*` bodies for the inherited
  virtuals. The protected `AIS_Shape::setMaterial` is crate-private in TKV3d; its body is
  repeated (four calls on public drawer members). TKV3d does not expose an iteration of
  `myShapeColors`, so `SetMaterial` visits the drawers `DispatchStyles` bound (recorded by
  the object); custom drawers added through `CustomAspects` by the caller are not visited.
- `XCAFPrs_Texture` embeds a `Graphic3d_Texture2D` (its RTTI parent) and overrides
  `GetImage`/`GetCompressedImage` with the image of its `Image_Texture`.
- `XCAFApp_Application` implements the TDocStd application traits, overriding
  `ResourcesName`, `InitDocument` (the document tool of the main label) and `DumpJson`; its
  constructor registers the `XCAFPrs_Driver` in the process-wide `TPrsStd_DriverTable`.
  `GetApplication` is a process-wide `OnceLock`.

## 4. Native quirks kept

- `XCAFDoc_ViewTool::SetView` (five sequences) sets the note node of the view to the
  clipping-plane GUID and links each annotation through the note of the same index
  (`Standard_OutOfRange` past the notes); emptied clipping-plane fathers forget their GD&T
  attribute.
- `XCAFDoc_AssemblyItemId::IsDirectChild` compares with the other path length minus one, so
  it never holds; `ToString` of a null item is `Standard_OutOfRange`.
- `XCAFDoc_Note::Dump` prints an empty author and time stamp, and the placeholders for set
  ones; `XCAFDoc_NoteBinData::Dump` prints the MIME type as the title.
- `XCAFDoc_DimTolTool::FindDimTol` and `FindDatum` compare the name strings by object, not by
  text. `XCAFDoc_LayerTool` shape overloads search through `myShapeTool` without the lazy
  lookup.
- `XCAFDoc_NotesTool::RemoveAll*Notes` loops forever natively when a father of the item is
  not a note; it raises `Standard_ProgramError` here.
- `XCAFDoc_DocumentTool::GetLengthUnit` and the attributes' `DumpJson` follow the notes of
  the ported classes (`XCAFDoc_LengthUnit` dumps the intended values where the native
  reads undefined memory; `XCAFDoc_DimTol`/`XCAFDoc_Datum` dump pointers).
