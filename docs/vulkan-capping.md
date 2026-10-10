# Martensite stencil capping

Martensite renders opaque Unlit, PhongFacet, Gouraud and Phong sections of closed retained groups through the
existing `Graphic3d_ClipPlane` interface. The reference is OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`, principally
`OpenGl_CappingAlgo.cxx`, `OpenGl_CappingPlaneResource.cxx`,
`OpenGl_Group.cxx` and `OpenGl_ShaderManager.cxx`.

## Implemented behavior

A capped frame uses a D32-float/S8 attachment and stencil-compatible pipelines.
Ordinary frames retain D32-float attachments. Stencil pipelines are created on
first use; unsupported combined attachment formats fail explicitly before
pipeline creation. The depth readback copies only the float depth aspect.

After ordinary clipped geometry, each section plane generates an odd/even mask
from closed groups. Mask draws disable color writes, depth testing and depth
writes. A new batch clears stencil, then each covered fragment inverts the low
bit. The section surface tests that bit for equality to one and uses `LESS` depth
comparison, respecting the layer's depth policy. Subsequent ordinary draws
restore their layer policy and `LEQUAL` comparison.

The plane uses OCCT's twelve homogeneous vertices: four finite centers and eight
zero-weight directions. Its orientation replaces the model matrix and retains
structure persistence. No finite oversized polygon approximates the plane.
The native cap-mask path calls `Group::Render` directly, bypassing the ordinary
group-transform/persistence wrapper. Martensite preserves that observable quirk:
mask geometry uses the structure transform and persistence, while the ordinary
geometry uses its group transform too. Initial failing transformed-group captures
are retained alongside the corrected results.

For a union chain, the mask sees only the selected original sub-plane. The cap
sees each other member reversed as a separate intersected halfspace; unrelated
chains retain their ordinary union semantics. Only the head's enabled/capping
flags select the chain. Existing local overrides, persistent-anchor policy and
bounding-box optimizations apply before the cap tasks are built.

Plane appearance accumulates closed groups within each structure. Selecting an
object appearance property creates separate group masks. Shadow geometry keeps
its own structure boundary, so coincident source/instance geometry does not
cancel through a combined parity mask. Object material inheritance copies the
front material and interior RGB, preserving the cap's alpha and shading model.
Texture/shader inheritance is represented, but unsupported texture/shader
rendering still returns an error.

Opaque automatic face culling now follows closed-group state and is disabled
while fragment clipping is active. Explicit front/back culling and view overrides
are respected. Mirrored structure winding is corrected for ordinary rendering;
the native cap pass resets that correction. These states are dynamic Vulkan
pipeline state and are covered by device checks and native captures.

Lit caps use OCCT's constant local normal `(0, -1, 0)` and the plane orientation
as ModelWorld. They retain structure persistence for the eye transform, resolve
`Default` from the live view, and snapshot the inherited material and view lights.
Stencil masks remain unlit. PBR, cap textures and custom shaders remain open.

## Verification

The original 25 native scene modes span six projection/extent/translation cases on each
of lavapipe and the AMD Radeon 8060S. They include open/disabled/uncapped controls,
plane changes, union/intersection chains, disabled children, object appearance,
overlapping groups, group transforms/persistence, persistent structures, shadow
instances, explicit culling and mirrored geometry.

| Profile | Strict native comparisons | Remaining differences |
| --- | ---: | --- |
| lavapipe | 148 / 150 | Two overlapping-group perspective cases, 18 and 30 color pixels |
| AMD Radeon 8060S | 135 / 150 | Fifteen perspective cases with identical RGBA and depth differences up to 6.44e-6 |

The acceptance threshold remains exact RGBA and depth error at most 1e-6.
Failures are retained, not converted into passing tolerances. The AMD failures
include uncapped controls, transformed groups, explicit front culling and
overlapping groups. The lavapipe differences occur in the overlap parity boundary.
These observations do not prove a single cause for every remaining difference.

The complete regression replay compares 2,700 previously integrated captures
and all 300 cap captures against their retained buffers. All 3,000 color/depth
buffer pairs, plus the applicable native-window buffers, are unchanged after final
source cleanup. All sixteen shader variants pass regeneration/validation.
The renderer has 144 passing Rust test executions and 37 passing Python harness
tests; Clippy with warnings denied and repository Rust formatting pass.

`capping_acceptance` independently checks odd/even masks, batch resets,
color/depth isolation, restored ordinary depth comparison and face culling on
both devices with Vulkan validation. The same checks run in
`offscreen_acceptance`. Device-independent tests cover retained plane mutation,
chain filtering, group boundaries, immutable snapshots and material inheritance.

```sh
cargo test -p martensite-vulkan --all-targets --locked
cargo run -p martensite-vulkan --example capping_acceptance --locked
DISPLAY=:1 python3 tools/verify_vulkan.py --help
```

The release `capping-plane-updates` workload alternates a section equation on a
256×256 closed cube. It measures extraction, the ordinary/mask/surface draws,
uploads, submission, fence wait and color/depth readback with validation disabled.
All ten accepted workload budgets pass in all eight fixed-order rounds
(`main`, change, change, `main` on each profile), using 100 samples after 20 warmup
frames. The new cap workload's median/p95 times are 0.720/0.878 and 0.729/0.882 ms
on lavapipe, and 0.397/0.562 and 0.396/0.532 ms on AMD. Its baseline remains
unaccepted; the full 20-workload timing gate retains ten unmatched workloads.
An initial stale main executable was rejected by the workload-count guard before
comparison; main was rebuilt and the complete experiment restarted.

Evidence: [native captures](../porting/runtime-evidence/vulkan-capping-native-2026-10-10.json),
[complete buffer regression audit](../porting/runtime-evidence/vulkan-capping-regression-2026-10-10.json),
and [controlled performance](../porting/runtime-evidence/vulkan-capping-performance-2026-10-10.json).
Raw captures are retained under `target/vulkan-capping-captures` in the main
checkout; no completed worktree or its build cache is archived.

## Remaining scope

Capping is partial. Lit materials, hatching, textures, custom shaders,
transparency, additional section/layer/window combinations, strict boundary
parity and other platform configurations remain required. General group stencil
commands are distinct from the internal cap mask and remain unsupported.
No renderer capability family is complete, and this work adds no OCCT API
inventory credit for renderer replacement internals.

The [main integration receipt](../porting/runtime-evidence/vulkan-capping-main-2026-10-10.json)
confirms all 10,857 verification input fingerprints match, all 144 renderer test
executions pass, and both devices pass the nineteen runtime checks plus fifteen
analytic clipping scenes with zero validation errors. The completed capping
worktree and branch were deleted, reclaiming approximately 14 GiB of build cache.

## Lit-section verification

Four paired fixtures add the Unlit-view ambient override and directional lighting for PhongFacet, Gouraud and Phong,
bringing the capping matrix to 29 modes and 174 comparisons per device. Run:

```sh
DISPLAY=:1 python3 tools/verify_vulkan.py --source ../OCCT \
  --build target/occt-all-full --case-family capping \
  --output target/lit-capping.json --captures target/lit-capping-captures
```

The release `lit-capping-plane-updates` workload measures the same 256×256 moving
section with Phong shading. The harness requires this workload alongside existing
timings; exact native RGBA and the 1e-6 depth threshold remain unchanged.
