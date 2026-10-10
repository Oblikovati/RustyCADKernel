# Martensite AIS line datums

`rustycad_tkv3d::ais::AisLine` implements the finite and infinite `AIS_Line`
object from OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`. Fifteen public
class/constructor/member declarations have explicit compiler-ID mappings.
Presentations use `StdPrsCurve`, and mode-zero selection builds real
`Select3DSensitiveSegment` entities with priority-five owners. Other selection
modes remain empty as in native OCCT.

Constructors take an `Arc<SharedState<UnitsApi>>` alongside the geometry. Infinite
selection calls `any_to_ls(250000, b"mm")` on that live context each time. Changes
between SI and MDTV therefore affect existing objects. UnitsAPI's private resource
catalogs now use shared lock-protected storage, preserving their aliasing while
allowing the context to be injected into thread-safe AIS objects. Existing unit
conversion/resource tests and a cross-thread live-system regression cover this.

Geometry setters retain point/line handles without scheduling redisplay or changing
the object's infinite-state flag. Finite presentation rebuilds its cached line;
selection reads the current endpoint geometry. Coincident points raise the native
construction error before replacing that cached line.

Appearance calls replace or synchronize aspects in existing groups. The four
`PrsMgrPresentableObject` color/width virtual methods now return `Result<(), Failure>`
so derived native exceptions propagate through base handles and material updates.
Base implementations still apply their original mutations and return success.
`AIS_Line` assigns its narrowed own width before a failing setter/constructor;
`UnsetWidth` deliberately retains a nonzero own width. These quirks are covered.

## Native lifetime defect and oracle control

The pinned native `replaceWithNewLineAspect` binds `anAspectOld` to the handle
inside the current `Prs3d_LineAspect`, replaces that wrapper, then uses the borrowed
handle to construct its replacement map. Without another owner of the wrapper,
this accesses freed storage. A minimized multi-request sequence crashes in
`NCollection_BaseMap::destroy`, called from `AIS_Line::UnsetColor`.

The [reproducer](../porting/runtime-evidence/ais-line-diagnostics-2026-10-10/native-aspect-lifetime-reproducer.txt)
and [debugger stack](../porting/runtime-evidence/ais-line-diagnostics-2026-10-10/native-aspect-lifetime-backtrace.txt)
retain the original failure. The native fixture now retains the current wrapper
across each mutation; no upstream source, operation or expected value is changed.
This guard removes the crash and permits comparison of the intended aspect-map
behavior. Reference counts are not part of this fixture's output. Martensite copies
the handle before replacement and needs no caller lifetime guard.

The corpus omits one undefined sequence on an unlinked drawer: negative SetWidth,
SetColor, UnsetColor, UnsetWidth. The first two calls fail and leave no line aspect;
native UnsetColor then dereferences null. It is not treated as a successful parity
case. All valid single and combined sequences remain in the corpus.

## Functional and performance checks

Five CPU regressions cover object identity, live units, retained geometry, empty
selection modes, failure state and inherited appearance. The new native corpus
contains 596 multi-operation requests; the complete TKV3d oracle passes 149,514
comparisons of exact values, exceptions, sensitive owners and presentation buffers.

Eleven GPU modes cover finite/infinite lines, live color/width changes, restoration,
endpoint changes, erase/redisplay, selection and dynamic detection. Six viewport/camera configurations
run on both lavapipe and native AMD with unchanged strict color/depth comparisons.
The runtime harness also picks the actual line through the AIS selector.
`ais-line-appearance` measures live color/width mutation plus Vulkan redraw/readback,
with 20 warmups and 100 retained samples per device alongside existing workloads.
The established six-workload baseline remains unchanged; new workloads have no
automatically accepted baseline.

```sh
cargo test -p rustycad-tkv3d --all-targets
python3 tools/verify_tkv3d_numeric.py --source ../OCCT --native-build target/occt-all-full
python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --output target/ais-line-gpu.json --captures target/ais-line-captures \
  --performance-baseline porting/runtime-evidence/vulkan-surfaces-baseline-2026-10-10.json
```

This completes the line object, not AIS or the renderer. Point primitives and
additional built-in presentations, native window presentation and retained
rasterization parity failures remain on the visualization-first plan.

Vulkan shadow presentations now retain the original structure's live geometry,
while using their own visibility, transform and color highlight. Nested shadows
flatten to the original parent as in native OCCT. Four CPU regressions cover
geometry lifetime, transform snapshots, live persistence and highlight restoration; the real AIS
pick path exercises immediate highlights on both devices. Bounding-box highlights,
transparent highlights and general structure connections remain unsupported.

The worktree matrix passes 435/438 captures on lavapipe and 391/438 on AMD,
including all selected/detected line cases. The earlier three software and
45 host failures remain. Two new host infinite-line cases have 28 differing
pixels: both renders contain 43 yellow pixels within identical bounds, but
AMD OpenGL shifts 14 covered pixels one row relative to AMD Vulkan and both
software captures. The cause remains unresolved; strict comparison stays failed.
All six established performance budgets pass. Four newer workloads, including
`ais-line-appearance`, still have no accepted baseline.
