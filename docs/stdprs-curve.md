# Martensite standard curve presentations

`rustycad_tkv3d::std_prs::StdPrsCurve` implements the eight `StdPrs_Curve`
Add/Match overloads from OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The ledger maps the class and each overload by compiler declaration ID. Match
coordinates are grouped into `Point3`; fallible operations return `Failure`.

The port preserves automatic geometric expansion of infinite bounds, explicit
sentinel clamping, two-point lines, B-spline knot-scaled discretisation and
append-only point output. Drawer-only Add overloads install the line aspect and
optionally draw the arrow even when curve drawing is disabled. Point-output
variants do neither. The arrow uses the adaptor's last parameter, including when
the explicit drawing range ends elsewhere. Matching preserves the native
Manhattan endpoint metric, strict projected-segment threshold and circle sampling
that can overshoot the supplied range. These native behaviors are deliberate.

## Validation

Five CPU regressions exercise these distinctions using real geometry and a named
recording Graphic3d driver. The native TKV3d corpus adds 6,376 cases across all
eight overloads, finite and infinite lines, circles, ellipses, Bézier and B-spline
curves. The full oracle passes 148,918 comparisons of numeric values, exceptions,
points, aspect callbacks and primitive buffers. Fresh drawers in both fixtures
call `SetupOwnDefaults` before using their initially null aspects.

Seven GPU modes exercise those five finite curve types, arrows and collection
without drawing. Six camera/viewport configurations run on each device. All 42
new lavapipe cases pass; native AMD passes 35/42. Two circle cases have four
coverage differences each, and five arrow cases have small depth differences.
The CPU presentation buffers match OCCT; the exact cause of these additional
renderer differences remains unproven. No tolerance or acceptance gate is relaxed.
The existing 3 lavapipe and 38 AMD line/persistence failures remain, giving
369/372 and 327/372 passing captures respectively. Eleven GPU runtime checks per
profile pass with no validation errors.

`curve-presentations` rebuilds 32 B-spline presentations each frame, including
sampling, group replacement, extraction, Vulkan submission and readback. The
worktree run measured median 2.469 ms on lavapipe and 2.152 ms on AMD. All six
established workload budgets passed. Curve, line and persistence workloads lack
accepted baselines, so the overall strict renderer gate remains failed.

Run the CPU suite with `cargo test -p rustycad-tkv3d --all-targets`. Native checks:

```sh
python3 tools/verify_tkv3d_numeric.py --source ../OCCT --native-build target/occt-all-full
python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --output target/stdprs-gpu.json --captures target/stdprs-captures \
  --performance-baseline porting/runtime-evidence/vulkan-surfaces-baseline-2026-10-10.json
```

The [worktree receipt](../porting/runtime-evidence/vulkan-stdprs-worktree-2026-10-10.json)
retains measurements and failures. Final integration evidence is recorded in
`porting/integration/2026-10-10/stdprs-curve.json`.

This completes the standard curve overload family, not the StdPrs package or
renderer capability family. `AIS_Line` can now use this prerequisite; its object
API and selection integration remain to be ported. Point primitives, additional
presentations and native window rendering also remain priorities.
