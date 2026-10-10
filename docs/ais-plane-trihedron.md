# Martensite plane trihedrons

Martensite implements `AIS_PlaneTrihedron` from OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`, adding 15 explicit public API mappings.
It builds on the ported AIS line/point objects and labeled DsgPrs axis presentation.
The constructor accepts a retained `GeomPlane` and an injected `UnitsApi` service.
Axis length starts at 100 mm converted to local units; conversion failures propagate.

The datum has signature 4 and accepts display mode zero. Its own infinite flag
starts false; computing a presentation marks the structure infinite. Both displayed
axes use the X line aspect, with independent arrow and text aspects. The X and Y
stored lengths remain independent when callers change the datum aspect directly.
Presentation and selection use `Position().Ax2()`, while the fresh line accessors
use the plane's stored X/Y axes, preserving native indirect-frame behavior.

`XAxis`, `YAxis` and `Position` allocate fresh objects from the live plane on every
call. The two new lines always start royal blue. Selection retains the three child
objects allocated during construction: mode 0 creates two segments sharing a
priority-5 owner for the whole trihedron; mode 1 creates a priority-8 origin point
owned by the original child point; mode 2 creates two priority-7 segment owners for
the original child lines. Sensitive geometry still follows the current plane and
lengths. Replacing or mutating the plane does not refresh those child objects.
Mode -1 clears existing selection entities; other unsupported modes leave them alone.

`SetLength` writes all three stored lengths and schedules presentation updates.
Plane and label setters do not schedule updates. `SetColor` updates the drawer and
both axis line aspects and synchronizes live presentation aspects; it does not
recolor the constructor's child objects or freshly returned lines. Labels retain
`AsciiString` bytes, with C-string truncation at the first NUL when presented.
The shared DsgPrs builder now accepts byte slices as well as UTF-8 strings so this
path does not introduce a lossy string conversion.

## Verification

Run `cargo test -p rustycad-tkv3d --lib --tests --locked` for CPU tests and
`python3 tools/verify_tkv3d_numeric.py --source ../OCCT --native-build target/occt-all-full`
for the native oracle. The oracle adds 210 deterministic requests over three frames,
two unit systems, five lengths, and seven operation sequences. They exercise
selection geometry and owner state, fresh child accessors, appearance, labels,
plane replacement/mutation, unequal lengths, and actual presentation primitives.

The Vulkan harness adds 14 modes across six camera/viewport configurations on
lavapipe and the native host GPU. Scenes cover indirect frames, lengths, unequal
axes, color, independent arrow/text styling, label replacement/removal, live geometry,
erase/redisplay, selection, and detection. Run
`python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full --output target/vulkan-plane.json`
for captures, runtime checks and timings.

The `plane-trihedron-updates` performance workload measures color and length
changes, AIS redisplay, six retained line/arrow/label draws, Vulkan submission and
readback at 256 by 256 pixels. It uses release builds, 20 warmups and 100 samples.
An unmatched baseline remains an explicit acceptance failure. Strict raster
thresholds are unchanged; the two AMD infinite-line pixel-row cases remain deferred.

The CPU suite passes 373 TKV3d tests, including six new plane datum tests. The
Vulkan all-target suite passes 95 tests; all 28 Python harness tests and Clippy
(`--all-targets -- -D warnings`) pass. The
[native numeric receipt](../porting/runtime-evidence/plane-trihedron-numeric-2026-10-10.json)
records 151,706 exact comparisons, including the 210 new requests. Native and Rust
noexcept selection-clear failure checks both abort as expected.

The [worktree renderer receipt](../porting/runtime-evidence/vulkan-plane-trihedron-worktree-2026-10-10.json)
records 16 runtime checks per profile with zero validation errors. Lavapipe passes
538/912 strict captures and native AMD 458/912; 12/84 and 7/84 of the added captures
pass respectively. All 828 earlier capture statuses are unchanged. The six
established timing budgets pass, while eight newer workloads have no accepted
baseline. The new plane workload's median/p95 is 0.555/0.674 ms on lavapipe and
0.456/0.630 ms on AMD. Overall strict renderer acceptance remains failed.

The [main integration receipt](../porting/integration/2026-10-10/plane-trihedron.json)
records verification at `6c5978905`. Main repeats the native, CPU and Clippy checks,
and all [7,296 raw color/depth buffers](../porting/runtime-evidence/plane-trihedron-main-buffer-audit-2026-10-10.json)
match the worktree byte for byte. Main plane workload median/p95 is 0.548/0.696 ms
on lavapipe and 0.481/0.626 ms on AMD. The
[cleanup receipt](../porting/integration/2026-10-10/plane-trihedron-cleanup.json)
confirms deletion of the merged worktree, branch and 27,351,297,902 bytes of build
and probe artifacts, after checking ancestry, local changes and active processes.
Nothing was archived.

The refreshed compiler inventory uniquely resolves all 15 new mappings and reports
zero unresolved ledger rows. The defined scope now contains 51,636 done, 974 partial
and 23,431 open APIs out of 76,041 declarations. All 31 Rust crate artifacts were
captured and verified at the integration revision above. Full port completion
and complete renderer parity remain unproven.
