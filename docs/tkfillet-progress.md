# TKFillet progress

TKFillet (`crates/tkfillet`) ports the OCCT fillet and chamfer toolkit at
3d097a0328e71b826377d4814ab05ec3c3d23871, package by package in dependency order:
ChFiDS, Blend, BlendFunc, BRepBlend, ChFiKPart, ChFi2d, ChFi3d, BRepFilletAPI,
FilletSurf.

## Mapping rules

- Value classes (`ChFiDS_CommonPoint`, `ChFiDS_FaceInterference`, `ChFiDS_Regul`, ...)
  are plain structs. Their `Change*` accessors are `&mut` accessors.
- The transient classes that the builders share and change through several handles
  (`ChFiDS_SurfData`, `ChFiDS_Stripe`, `ChFiDS_Spine` and its subclasses) are
  `chfi_ds::Shared<T>` handles (`Arc<Mutex<T>>`). A null native handle is `None`.
  `chfi_ds::lock` gives the mutable view.
- `ChFiDS_ElSpine` is a curve adaptor: a `Handle<ElSpine>` whose state sits behind a
  lock, as every adaptor of the port.
- The inheritance `ChFiDS_Spine` -> `ChFiDS_FilSpine` / `ChFiDS_ChamfSpine` is the
  `SpineObject` trait (the virtual members) plus `Deref` to the `Spine` part.
- Output references become returned values, overloads take distinct names, and the
  native exceptions become `Failure` values with the native exception type.

## Native quirks kept

- `ChFiDS_Stripe::Reset` nullifies the first pcurve twice and keeps the last one.
- `ChFiDS_SurfData::Copy` does not copy the curve indices and their flags.
- `ChFiDS_FilSpine::SetRadius(Law_Function, IinC)` puts the law into a composite that is
  dropped: only the split flag and the radii change.
- `ChFiDS_FilSpine::IsConstant(IE)` is true and `Radius(IE)` is 0 when no radius is set.
- `ChFiDS_SurfData::Vertex(First, OnS)` and `Get2dPoints(First, OnS)` select the last
  point on the second face for an `OnS` other than 1 and 2.

## Verification

`python3 tools/verify_toolkit.py TKFillet` runs the numeric oracle
`tools/verify_tkfillet_numeric.py`: `tools/oracle/tkfillet_probe.cxx` and its Rust twin
`crates/tkfillet/examples/numeric_probe.rs` on `tools/tkfillet_corpus.py`.

| Package | Probe operations |
| --- | --- |
| ChFiDS | `fl_spine`, `fl_arc_spine`, `fl_fil_law` |
| BlendFunc | `fl_const_rad`, `fl_const_rad_inv`, `fl_evol_rad` |
