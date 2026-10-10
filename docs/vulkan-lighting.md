# Martensite opaque surface lighting

Martensite now renders retained Graphic3d triangles with the legacy PhongFacet,
Gouraud and Phong algorithms in Vulkan 1.3. The source oracle is OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`, particularly
`OpenGl_ShaderManager::PushLightSourceState`, `Graphic3d_ShaderManager` and
`OpenGl_Material`. This is a partial renderer capability, not full legacy renderer
parity or API-completion credit.

Normals, materials and enabled lights are snapshotted at redraw. Missing normals
fall back to PhongFacet for Gouraud/Phong, matching native surface policy.
Inverse-transpose model matrices transform nodal normals; separate inverse view
matrices supply world-space eye vectors and headlights. Directional, point and
spot lights preserve native attenuation, range, cone and concentration behavior.
Ambient intensity is applied, while legacy Phong ignores non-ambient intensity.
The native exact-axis spotlight behavior is preserved.

The std430 lighting record contains a 304-byte header followed by 80 bytes per
source. Linear material coefficients feed an sRGB attachment. Gouraud computes
separate front/back vertex colors; Phong computes interpolated fragment lighting;
PhongFacet derives geometric normals from fragment position derivatives. Vertex
color multiplication, distinguished back materials, front-only vertex coloring
and emission ordering follow native shader policy. Lit clipped shaders share the
existing clip-chain storage at descriptor set 1. Pipelines are cached lazily so
ordinary unlit initialization does not construct the new lighting variants.

Stencil mask passes force unlit rendering. PBR models, transparent surface
materials, shadows, lit normal-bearing lines/points and lit retained capping
materials remain unsupported. Requested enabled shadow casters produce an error
when shadow maps are enabled. Disabled casters and disabled shadow rendering are
accepted. Default shading resolved to Unlit rejects unsupported distinguished or
front-only back-color policy rather than silently ignoring it.

## Functional evidence

```sh
cargo test -p martensite-vulkan --all-targets --locked
cargo clippy -p martensite-vulkan --all-targets --locked -- -D warnings
DISPLAY=:1 python3 tools/verify_vulkan.py --source ../OCCT \
  --build target/occt-all-full --case-family lighting \
  --output target/vulkan-lighting.json --captures target/vulkan-lighting-captures
```

The focused command recompiles and checks all 20 SPIR-V variants for Vulkan 1.3,
builds the native oracle, runs all CPU tests, the existing 19 Vulkan runtime
checks, and 15 analytic lighting scenes on both lavapipe and the native GPU.
Each device checks 61,440 analytic RGBA/depth pixels; validation errors are zero.
The native matrix covers 21 lighting modes over six size/projection/translation
configurations, including live material/light/normal edits, clipping, rotated
headlights, missing normals, front/back faces and vertex colors.

| Profile | Exact native captures | Strict failures |
| --- | ---: | ---: |
| lavapipe, llvmpipe LLVM 20.1.2 | 124/126 | 2 |
| AMD Radeon 8060S, RADV STRIX_HALO | 64/126 | 62 |

Both lavapipe failures change one color pixel in a perspective scene. AMD failures
include sparse Phong/Gouraud differences and facet scenes with up to 418 changed
color pixels. These causes are not established by this receipt. Depth comparisons
stay within the existing 1e-6 tolerance. The exact-color gate remains red; no
threshold or expected failure was added. Raw RGBA/depth, PNGs and separate color
and depth diff masks are retained alongside the receipts.

Mutable native buffers begin at full capacity. The paired fixture fills vertex
slots, creates mutable attributes and invalidates the buffer after a normal edit;
using AddVertex or editing a static VBO would test a different native contract.

## Performance evidence and remaining work

The harness now runs 21 release workloads, including `lighting-updates`, with
100 samples and 20 warmup frames, validation disabled. The new workload includes
retained normal/light updates, scene extraction, submission, fence wait and
color/depth readback. Its baseline is collected, not accepted, while native parity
is failing. The previous 20 workloads were compared against freshly rebuilt
`main` on the same devices and driver enumeration.

The first before/after comparison exceeded several timing budgets. Reversing the
measurement order passed all 20 existing AMD workloads and retained one lavapipe
p95 failure: single-triangle 0.908 ms against a 0.885 ms limit. Both raw timing
sets are retained. This evidence does not establish a performance regression or
prove its absence; the timing gate remains open. No existing budget was relaxed.

Receipts and captures: `porting/runtime-evidence/vulkan-lighting-2026-10-11/`.
Continue native precision/behavior comparisons, shadow maps, PBR, section
materials and lighting for other primitive families. Global inventory completion
remains false.
