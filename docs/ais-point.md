# Martensite AIS point datums and Vulkan sprites

`rustycad_tkv3d::ais::AisPoint` ports the native `AIS_Point` geometry, appearance,
selection and presentation behavior at OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`StdPrsToolPoint` supplies live coordinates to the existing generic `Prs3dPoint`;
`StdPrsPoint` names that specialization. Seventeen declarations, including the
protected presentation hook, have explicit compiler-ID mappings. `vertex()` builds a fresh topological
vertex from the current geometric point.

The component handle stays live. Replacing it does not schedule redisplay.
Selection creates a sensitive point with a priority-ten owner in every requested
mode, including negative modes. Display accepts zero and minus 99; the latter
uses the static highlight drawer's marker, as upstream does. Separate default
static and dynamic drawers retain native colors, scale and layer settings.

Point appearance follows linked drawer values unless color, width or marker
is owned. Native `SetWidth` only stores the width; a later marker/color mutation
refreshes the point aspect. Mutations preserve native partial state when an
invalid scale raises an exception. Replacing an aspect updates retained groups.

Six CPU regressions cover these behaviors. A 1,372-request addition to the native
corpus covers linked/unlinked drawers, marker types, valid and invalid widths,
mutation sequences, geometry replacement, selection owners, vertex creation,
coordinate extraction and point matching. The complete TKV3d oracle passes
150,886 comparisons, including exact finite values and exception traces.

## Vulkan rendering and tests

The renderer now lowers point arrays to indexed `POINT_LIST` submissions with
screen-space size. Native `Graphic3dMarkerImage` standard and user sprites provide
nearest-sampled pixels in a fragment storage buffer. Highlighting uses the native
alpha sprite. Fragment alpha rejection and blending follow the native marker path.
Device feature, point-size and storage-buffer limits are checked before submission.

Twenty-four capture modes cover standard markers, an asymmetric user bitmap,
live color/scale, restoration, component changes, erase/redisplay, selection,
detection and a 1,024-point batch. Six camera/viewport configurations run on both
lavapipe and the native host. CPU tests prove the batch stays in one draw after
its construction scope ends. Runtime tests require nonempty output for visible
modes, actual AIS picking, and no Vulkan validation errors.

`point-sprites` measures the same 1,024-point batch at 256 by 256 pixels, with
20 warmups and 100 release-mode redraw/readback samples. It has no accepted
performance baseline yet; established workload budgets remain unchanged.

Point rendering is **partial**. The strict capture oracle retains raster placement,
marker color and highlight discrepancies as failures. Initial captures show an
exact asymmetric user bitmap and moved point, alongside pixel-boundary placement
differences for centered standard markers. These are recorded separately from the
previous AMD infinite-line special cases. They do not block continued feature
porting, and no pixel tolerance or performance baseline is loosened to hide them.

Run the complete rendering checks with:

```sh
python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --output target/vulkan-ais-point.json --captures target/vulkan-ais-point-captures \
  --performance-baseline porting/runtime-evidence/vulkan-surfaces-baseline-2026-10-10.json
```

## Main integration evidence

Merge `93724656d` passes 361 TKV3d tests, 81 Vulkan tests, 28 Python harness tests,
Clippy with warnings denied, and all 150,886 native numeric comparisons. Both
devices pass 13 runtime checks with zero validation errors. Existing capture
statuses and all six established timing budgets are unchanged.

| Profile | Strict captures | New point captures | 1,024-point median / p95 |
| --- | ---: | ---: | ---: |
| lavapipe | 484 / 582 | 49 / 144 | 0.872 / 1.031 ms |
| native AMD | 415 / 582 | 24 / 144 | 0.370 / 0.630 ms |

The [main receipt](../porting/runtime-evidence/vulkan-ais-point-main-2026-10-10.json)
retains every strict failure. The 95 software and 120 native point failures remain
unresolved renderer parity gaps; the earlier two AMD AIS line special cases stay
separate and deferred. Five workloads, including point sprites, still have no
accepted timing baseline. These outcomes do not claim renderer completion.

All 4,656 raw buffers match the pre-integration worktree captures byte for byte.
The [integration record](../porting/integration/2026-10-10/ais-point.json) links
that audit and the cleanup receipt. The completed point worktree, merged branch
and 25,962,254,227-byte checkout/build cache were deleted, without archiving.

The refreshed inventory records 51,592 done, 974 partial and 23,475 open APIs
out of 76,041 in scope, with zero unresolved ledger rows. Full port completion
and renderer parity remain unproven.
