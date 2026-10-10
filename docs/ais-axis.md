# Martensite AIS axis datums

Martensite implements `AIS_Axis`, `AIS_TypeOfAxis` and both
`DsgPrs_XYZAxisPresentation::Add` overloads from OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. These add 29 explicit public API
mappings. Private presentation and selection overrides implement the native
virtual behavior without claiming private helpers as new public APIs.

All four constructors are available: retained `Geom_Line`, `Geom_Axis1Placement`,
coordinate-frame `Geom_Axis2Placement` plus axis kind, and a ray with optional
unit-derived length. The port injects `UnitsApi`; no process-global units service
is introduced. Coordinate-axis construction catches conversion failure and uses
the native 0.1 fallback. Line/ray conversion failures propagate.

Axis selection snapshots endpoints when geometry is assigned. Changing units or
mutating the retained line does not refresh those endpoints until the setter runs.
The line presentation still consumes live curve geometry. Each requested selection
mode produces one segment owner with priority 3 and the actual axis identity.
The datum signature is 2; only display mode zero is accepted. Native infinite-state
behavior, including positive-length rays, is retained.

Coordinate-axis fields cache direction, length, label and a line-aspect handle.
`SetTypeOfAxis` changes only the classification. `SetAxis2Placement` rebuilds the
fields, while `SetComponent`/`SetAxis1Placement` switch presentation kind without
discarding the retained frame handle. Aspect changes synchronize existing groups.
Negative widths do nothing; zero first resets widths and then reaches the native
failing zero-width setter. Positive widths do not assign `myOwnWidth`. Unsetting
color resets line and datum colors but leaves the stored drawer color unchanged.
`SetDisplayAspect` replaces the cached line aspect and propagates its color.

The presentation builder emits the segment, actual arrow geometry and an optional
`Graphic3d_Text`. One overload inherits the line aspect for arrow and text; the
other retains separate supplied aspects. Arrow length is one tenth of axis length.
The native tiny label height is retained by generic groups and normalized to 16
by the rendering driver. Empty labels emit no text; the explicit text-aspect
transition still occurs in the second overload.

Native construction with an initially unknown coordinate-axis kind reads
uninitialized scalar/label fields, as does enabling a ray's never-initialized label.
Martensite initializes those fields to zero/empty instead of reproducing undefined
memory reads. Native differential cases cover the defined transitions, including
changing an initialized coordinate axis to `Unknown`.

## Rendering and evidence

Axis labels exercise unblended font coverage inherited from line aspects.
The native standard font shader always rejects alpha at or below 0.285, regardless
of the aspect cutoff. Only `MaskBlend` enables coverage blending; other alpha modes
use the unblended text pipeline for the currently supported opaque text colors.
CPU and GPU regressions cover the fixed cutoff. This also corrects the earlier
text slice's use of the aspect cutoff; its default fixtures used 0.285.

Eighteen axis modes cover X/Y/Z, infinite line, axis-one placement, explicit/default
ray lengths, labels disabled, appearance and width resets, display-aspect replacement,
geometry mutation, erase/redisplay, selection and detection. Five text modes add
alpha-mode and cutoff coverage. Together they expand the matrix from 690 to 828
captures per device. Strict native color/depth failures remain recorded; this slice
does not relax tolerances or claim complete raster parity.

`ais-axis-appearance` measures color/width updates, retained line/arrow/label draws,
font layout, Vulkan upload/submission and readback. Like the other workloads it
runs in release mode with 20 warmups and 100 samples on lavapipe and native AMD.
Its three draws retain the axis label. A new accepted timing baseline is not implied.

Run `cargo test -p rustycad-tkv3d --lib --tests --locked` for toolkit tests,
`python3 tools/verify_tkv3d_numeric.py --source ../OCCT --native-build target/occt-all-full`
for the native oracle, and
`python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full --output target/vulkan-axis.json`
for rendering captures, runtime checks and timings on both device profiles.
The two previously deferred AMD infinite-line pixel-row differences remain
nonblocking special cases; their diagnostic work has not resumed.

The [worktree renderer receipt](../porting/runtime-evidence/vulkan-ais-axis-worktree-2026-10-10.json)
records 94 Vulkan all-target tests, 15 runtime checks per profile and zero validation
errors. All 367 TKV3d tests, 28 Python harness tests and Clippy pass. The
[native numeric receipt](../porting/runtime-evidence/ais-axis-numeric-2026-10-10.json)
records 151,496 comparisons, including 610 new axis/presentation requests.

Lavapipe passes 526/828 strict captures (36/108 new axis captures); native AMD
passes 451/828 (30/108 new axis captures). The 30 added text-alpha captures per
profile retain strict raster differences. All 690 previous capture statuses are
unchanged on each device. The six established performance budgets pass; seven
newer workloads remain explicitly unmatched. Worktree axis median/p95 is
0.439/0.487 ms on lavapipe and 0.351/0.507 ms on native AMD.

The [main integration receipt](../porting/integration/2026-10-10/ais-axis.json)
records verification at `95db0d773`. Main reproduces the worktree results, including
all [6,624 raw color/depth buffers](../porting/runtime-evidence/ais-axis-main-buffer-audit-2026-10-10.json).
Main axis median/p95 is 0.496/0.619 ms on lavapipe and 0.361/0.465 ms on native AMD.
The [cleanup receipt](../porting/integration/2026-10-10/ais-axis-cleanup.json)
confirms deletion of the merged worktree, branch and 27,696,985,198 bytes of
build/probe artifacts after checking ancestry, local changes and active processes.
No checkout or build artifacts were archived.

The refreshed compiler inventory resolves all 29 new mappings uniquely, with zero
unresolved ledger rows. The in-scope totals are 51,621 done, 974 partial and 23,446
open out of 76,041 declarations. All 31 Rust crate artifacts were captured and
verified at the integration revision above; this does not establish full port
completion or renderer raster parity.
