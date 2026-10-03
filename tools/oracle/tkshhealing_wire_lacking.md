# Wire missing-connection checks

The companion fixture executes unchanged OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871` algorithms from the native modeling
build. Nine rows cover CheckLacking's vertex/p-curve failures, surface tolerance,
reversed tangents, caller precision, and bulk status accumulation.

Each named row reports `found DONE1 DONE2 FAIL1 FAIL2 FAIL3 MaxDistance2d
MaxDistance3d aggregate aggregate_DONE aggregate_FAIL maximum_after_CheckConnected
cleared_OK`. Rust parses the native floating-point output and compares exact
values. The early small-gap return deliberately retains a **squared** 2D maximum,
and CheckConnected changes the 3D minimum without overwriting the maximum.
These details preserve current native behavior.

A tenth row checks CheckGaps3d after a previously measurable curve loses its
3D representation. Failed pairs must not contribute stale distances; the final
minimum and maximum both become zero. An eleventh native row checks that Gap2d preserves the distinction
between missing face/surface (FAIL1) and a loaded face with no ready wire (OK).

Run the Rust regressions:

```sh
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_lacking
```

Build the native fixture with the same flags and libraries as
[the singular-edge fixture](tkshhealing_wire_singular_checks.md), substituting
`tkshhealing_wire_lacking.cxx` for its source. These fixtures supplement the
legacy suite and do not establish whole-toolkit parity.
