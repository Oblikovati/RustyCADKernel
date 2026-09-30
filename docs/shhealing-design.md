# TKShHealing design notes

This note fixes how the TKShHealing port (`crates/tkshhealing`, see
[tkshhealing-progress.md](tkshhealing-progress.md)) maps C++ constructs that the earlier
design notes ([geom-design.md](geom-design.md), [topology-design.md](topology-design.md),
[geomalgo-design.md](geomalgo-design.md), [topalgo-design.md](topalgo-design.md)) do not
cover. Each package adds its section before it codes the construct, and cites it by its
number in code comments and ledger notes.

## 1. Toolkit-wide rules

- **Modules and names.** Each package is one module in snake case (`ShapeExtend` is
  `shape_extend`, `SHMessage` is `sh_message`). A class keeps its package prefix in its
  Rust name without the underscore (`ShapeExtend_WireData` is `ShapeExtendWireData`),
  because the Shape Healing packages reuse the same short names (`ShapeAnalysis_Edge`,
  `ShapeBuild_Edge`, `ShapeFix_Edge`). Package functions (`ShapeExtend::EncodeStatus`)
  are associated functions of a unit struct named after the package.
- **Enumerations** keep the native order, so `value as i32` is the native value, with a
  `from_index` for the probes (`ShapeExtendStatus::from_index`).
- **Status fields.** Many tools keep an `int` bit field of `ShapeExtend_Status` flags and
  answer `Status(ShapeExtend_Status)` with `ShapeExtend::DecodeStatus`; the Rust field is
  the same `i32`, set with `ShapeExtend::encode_status`, so that the bits are the native
  bits.
- **Transient classes and handles** follow geom-design.md sections 2 and 3: a class
  derived from `Standard_Transient` is used through `Handle`, keeps its mutable fields in
  one shared state, and registers its RTTI name. A class that derives from a Transient
  class of a lower toolkit (for example `ShapeBuild_ReShape` from `BRepTools_ReShape`)
  gets its own section before it is coded.
- **Exceptions** become `Result<_, Failure>` errors that carry the native exception type
  and the offending value (geom-design.md section 4).
- **Generic C++ code.** The toolkit has no `.gxx` template, and its only `.pxx` files are
  the compiled message resources of SHMessage (data, not code); an `.lxx` file
  holds the inline members of one class and is ported with that class.
