# Martensite circle datums and adaptive curve presentations

`AisCircle` ports both `AIS_Circle` constructors, the circle/parameter accessors,
geometry and parameter setters, datum identity, filled sensitivity, and the four
appearance overrides from pinned OCCT `3d097a0`. Full circles retain exact circle
sensitivity; arcs use the native twelve-segment sensitive polygon, including its
intermediate tangent points and optional filled-sector center. Selection callbacks
ignore the requested mode. Owners have priority zero and retain the actual datum.
Changing geometry, parameters or filled sensitivity does not schedule a recompute;
call the context's redisplay/update path as with native OCCT.

A span of either +2π or −2π in the parameter constructor selects full-circle
behavior. Setting either endpoint later forces arc behavior even for a full turn.
Sensitive geometry is a snapshot, while the retained Geom circle remains live.
Appearance changes reuse the line datum implementation and update existing groups.
The native stale own-width behavior remains; circle `UnsetWidth` deliberately omits
the aspect-synchronization notification present in `AIS_Line`.

The prerequisite `StdPrsDeflectionCurve` ports all five Add and four Match overloads.
Drawing respects relative/absolute chordal deviation and angular deviation through
the existing Martensite GCPnts tangential sampler. It samples each C1 interval and
preserves repeated join points. Line endpoints append before primitive allocation;
other curves allocate before appending. Draw-disabled overloads do not create
Graphic3d groups or arrows. Drawer overloads set the line aspect and place arrows
at the actual trimmed endpoint. Explicit point/deviation overloads preserve aspects.

Automatic infinite bounds use native geometric expansion, with a 100,000-iteration
termination guard specific to the deflection presentation family. Explicit ranges
retain their overload-specific sentinel handling. Match preserves native Manhattan
endpoint proximity, segment matching and the separate circle-step rule. These
choices intentionally retain observable upstream behavior instead of substituting
a different tessellator or closest-point algorithm.

Circle presentations temporarily use deviation coefficient `1e-5`, restoring the
previous value only on success, as native OCCT does. The absolute-deviation policy
remains effective. The numeric oracle covers analytic curves, Bezier and C1-split
B-splines, all public overloads, appearance mutations, selection modes and failures.
There are 26 compiler declaration mappings across these two classes.

## Verification

```sh
cargo test -p rustycad-tkv3d -p martensite-vulkan --all-targets --locked
python3 tools/verify_tkv3d_numeric.py --source ../OCCT --native-build target/occt-all-full
DISPLAY=:1 python3 tools/verify_vulkan.py --source ../OCCT \
  --build target/occt-all-full --output target/vulkan-circle.json \
  --captures target/vulkan-circle-captures \
  --performance-baseline porting/runtime-evidence/vulkan-circle-baseline-2026-10-10.json
```

Nineteen AIS modes span circles/arcs, filled selection, reversed full-turn
parameters, live color/width changes, geometry replacement/mutation, trim setters,
absolute deviation, erase/redisplay and selection/detection. Six camera/extent
configurations run on lavapipe and native AMD. The `circle-datum-updates` release
workload includes changing radius, adaptive presentation rebuilding, selection
updates and GPU redraw/readback, with 20 warmup and 100 measured samples. Its first
distribution is collected separately from accepted performance baselines.

Strict raster comparisons remain visible in the receipts. The two previously
identified AMD infinite-line pixel-row special cases remain deferred and nonblocking.

## Worktree verification

The [full worktree receipt](../porting/runtime-evidence/vulkan-circle-worktree-2026-10-10.json)
records 1,062 strict captures per profile: 688 pass on lavapipe and 608 on AMD.
All 114 new circle captures pass on both profiles, and all 948 earlier statuses
remain unchanged. Both profiles pass 18 runtime checks with zero validation
errors. The eight accepted timing workloads remain within their existing budgets.
The circle workload is initially collected at median/p95 0.737/0.924 ms on
lavapipe and 0.509/0.678 ms on AMD; independent main validation is still pending.

The complete TKV3d native oracle passes 154,984 numeric comparisons plus the
noexcept-abort check. TKV3d and Vulkan all-target tests pass (487 tests combined),
as do all-target Clippy and 31 Python harness tests. This evidence precedes main
integration and does not claim complete renderer parity.

## Main integration

The [main receipt](../porting/runtime-evidence/vulkan-circle-main-2026-10-10.json)
independently passes all 114 circle captures and 18 runtime checks per profile.
All 8,640 raw buffers match retained main artifacts: 7,728 from previous slices
and 912 new circle color/depth buffers. The complete native oracle, 487 Rust
tests, 31 Python tests and Clippy also pass on main.

Six initially unresolved mappings required the compiler inventory’s explicit
trait-qualified method paths. After correcting them, the fresh inventory resolves
all 26 new declarations, with 51,662 done, 974 partial, 23,405 open and zero
unresolved ledger rows. This remains incomplete coverage of the 76,041-API scope.

The initial main timing run exceeded AMD window/circle budgets while a corrected
inventory job was running. Those failures remain in its receipt. A single
[repeat after inventory completed](../porting/runtime-evidence/vulkan-circle-repeat-performance-2026-10-10.json)
passes all nine accepted workload budgets on both profiles, including the new
circle baseline. Eight other workloads remain unmatched. The completed circle
worktree and its build cache are deleted, never archived; the integration cleanup
receipt records ancestry, local-change, artifact and process checks.
