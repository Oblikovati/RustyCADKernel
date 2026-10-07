# TKV3d design notes

This note fixes how the OCCT toolkit TKV3d maps to Rust (`crates/tkv3d`, `rustycad-tkv3d`:
packages BRepFont, Select3D, SelectBasics, SelectMgr, Prs3d, StdPrs, StdSelect, DsgPrs, PrsDim,
PrsMgr, AIS and V3d). The rules of [geom-design.md](geom-design.md) sections 1 to 4,
[ocaf-design.md](ocaf-design.md) section 1 and [visualization-design.md](visualization-design.md)
apply: an abstract class is a trait over an embedded base, `Handle<dyn Trait>` holds it, the
fields that any handle may change sit in one `SharedState`, exceptions are `Failure` values with
the offending value, one snake-case module per package, the package prefix stays in each Rust
name. Code comments and ledger notes cite this note as "v3d-design.md section N".

## 1. Presentations and the driver

`Prs3d_Presentation` is a typedef of `Graphic3d_Structure`, so a presentation is a
`Handle<dyn Graphic3dStructureSource>`. Nothing in TKV3d talks to OpenGL: V3d and PrsMgr reach the
graphic driver only through the TKService traits (`Graphic3dGraphicDriver`, `Graphic3dCView`,
`Graphic3dCStructure`, `Graphic3dGroup`). The tests run on the TKService null driver
(`rustycad_tkservice::null_driver::NullGraphicDriver`); the Vulkan 1.3 driver plugs in later
behind the same traits.

## 2. Selecting volumes

`SelectBasics_SelectingVolumeManager` is the trait `SelectBasicsSelectingVolumeManager`. Its
overloads that differ only by the last argument get one name each: the pick-result form keeps the
native name (`overlaps_box`), the `bool* theInside` form takes the suffix `_inside` and an
`Option<&mut bool>` (`None` is the native null pointer), and the depth-less point test is
`overlaps_point_only`. The deprecated `Overlaps` aliases are the same trait methods.

## 3. Drawer link fallback

`Prs3d_Drawer` getters that fall back to the link drawer read their own fields, release the
lock and then ask the link. A drawer linked to itself therefore cannot deadlock, where the native
code recurses without end. The uninitialized native fields (`myPreviousDeviationCoefficient`,
`myPreviousDeviationAngle` until the first set, `Prs3d_ArrowAspect::myIsZoomable` in two
constructors) start at 0 and `true` in the port; their values are only observable through
`DumpJson` natively, where they are garbage.

## 4. Numeric probes

The numeric code (quadric tools, arrows, frustums, sensitive entities, view and camera
operations) is compared bit for bit with the native toolkit by
`tools/oracle/tkv3d_probe.cxx` and its Rust twin `crates/tkv3d/examples/numeric_probe.rs`
through `tools/verify_tkv3d_numeric.py`, on the request corpus of `tools/tkv3d_corpus.py`.
The native float steps of `Prs3d_ToolQuadric::FillArray` (`1.0f / n`) are kept.
