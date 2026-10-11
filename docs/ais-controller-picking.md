# Martensite controller picking and orbit pivots

Martensite implements `AIS_ViewController::PickPoint`, `PickAxis`, and
`GravityPoint`, plus the default `AIS_InteractiveContext::GravityPoint` forwarding
operation, against OCCT commit `3d097a0328e71b826377d4814ab05ec3c3d23871`.
These operations use the real context selector and `V3dView` selection volumes.

Picking clears the previous highlighting pixel before traversal, preserves the
output point on a miss, and uses the first ranked result without context detection
filters. Cursor-ray projection occurs only when requested and the ranked depth is
not `Precision::IsInfinite`. Coordinate validation preserves that upstream
predicate, including its NaN behavior.

All five rotation modes are implemented. Last-pick mode truncates the retained
render-buffer start coordinates; center-pick mode divides integer window size by
two. Failed picks and empty scene bounds fall back to context gravity. Camera and
nonempty scene modes preserve the cached highlighting pixel. An unattached window
in center-pick mode raises a typed failure before clearing that pixel.

The committed asset contains 80 native snapshots of actual displayed `AIS_Point`
selection in orthographic and perspective views. It covers axis/cursor hits and
misses, projected and unprojected points, every rotation mode, and retained output
and highlight state. Rust regressions additionally cover missing windows, empty
scene fallback, and the native infinite-coordinate threshold.

Run the complete toolkit checks with:

```sh
cargo test -p rustycad-tkv3d --all-targets --locked
```

Native generation is local-only. Compile
`tools/oracle/martensite_controller_picking.cxx` with the pinned OCCT include/library
paths, linking `TKV3d`, `TKService`, `TKG3d`, `TKMath`, and `TKernel`; run its binary
twice and compare the output before replacing
`crates/tkv3d/tests/fixtures/controller-picking-native.txt`. Source and asset hashes
are recorded in `porting/runtime-evidence/controller-picking-2026-10-11/`.
Rust replay requires no OCCT dependency.

The full controller class remains partial. Public frame orchestration,
`FitAllAuto`, camera event dispatch, redraw, and XR are separate unfinished work.
