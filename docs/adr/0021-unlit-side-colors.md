# Programmable unlit side colors, behavior version 2

Status: bounded V2 implementation with independent CPU/GPU checks; full renderer
acceptance and migration remain open (two existing OIT tests report synchronization
hazards, pending coordinating-agent baseline classification).

Martensite's V1 programmable renderer rejects untextured unlit surfaces when
material distinction or front-only vertex color is requested. This is pinned
by `crates/vulkan/tests/fixtures/unlit-side-color/legacy-v1.txt`, captured from
the Rust behavior at `eb63c6843`, and by the existing viewer regression.

V2 is an additive, explicit `RenderUnlitFaceColorPolicy::SideColorsV2` opt-in.
All existing driver constructors keep `LegacyV1`, including its rejection
messages. Neither the public lighting-model enum nor workspace version changes.
The driver owns its policy immutably; each submitted draw owns its color values.

## Color and alpha decision

This decision covers programmable, untextured triangle surfaces. Unlit colors
are linear, unpremultiplied interior RGBA; Phong material coefficients and lights
do not participate. The existing front vertex/bound-color precedence is retained.

| Vertex color array | Back vertex colors allowed | Front | Back |
| --- | --- | --- | --- |
| Present | Yes | Interpolated vertex RGBA | Interpolated vertex RGBA |
| Present | No | Interpolated vertex RGBA | Resolved back interior RGBA |
| Absent | Either | Existing resolved front/bound RGBA | Back interior RGBA when distinguished; otherwise existing front/bound RGBA |

For front-only vertex colors with distinction disabled, the resolved back color
is the front interior RGBA. For distinction enabled it is the back interior
RGBA. In particular, **without a color array, the front-only flag has no effect**;
distinction alone selects the back interior color. This last rule is a named
Martensite decision, not evidence supplied by the three historical shader tests.

For newly accepted V2 combinations, BlendAuto routes a draw to the deferred
blend pass if either rendered side can have alpha below one. Mask and MaskBlend
keep the existing strict `alpha < cutoff` test, applied to the selected side
after any mesh-edge composition. Opaque retains its explicit nonblend policy;
Blend retains its explicit deferred policy. Existing V1-supported inputs retain
their current pass routing and output. Culling and mirrored winding continue
through the existing raster pipeline. The routing check excludes a culled side.
Color highlighting replaces both sides. The existing RGB-only framebuffer write
mask retains opaque destination alpha; side alpha affects blending and masking
without changing that capture contract.

Textures, lines, markers, fixed-function lighting and cap construction retain
their current behavior. A V2/fixed-function policy combination must be rejected,
not silently converted to programmable lighting. No second intersection,
lighting, or renderer engine is introduced.

## Shader transport and ownership

An owned optional back RGBA uses the existing untextured surface auxiliary
vec4 vertex slot, with an explicit push-constant flag. It is not a texture UV.
The slot may carry RGBA only for an ordinary untextured triangle draw with no
glyph/marker/line state, no lighting state and no texture binding (including an
unavailable or binding-only texture). Polygon-edge draws, cap construction and explicit replacement by lighting clear
this state.
The existing surface, clipping, mesh-edge and alpha/OIT shader families consume
it. The 128-byte mesh-edge push range does not grow. Old supported draw encoding
and every existing expected asset remain unchanged.

## Evidence and migration gate

CPU checks must cover V1 exact replay, V2 flag/array/model combinations, source
mutation after snapshot, alpha routing/cutoffs, packing and pipeline-cache
identity. GPU checks must independently assert front/back primary colors and
alpha, winding/mirroring, clipping, mesh-edge composition and preservation of
old accepted scenes on lavapipe and the native GPU. GPU evidence must identify
device/revision/options and measured errors. No live OCCT run is required.

The three `Graphic3d_ShaderManager_Test.cxx` cases assert OpenGL program identity
and generated source snippets. These Vulkan behavior checks do not translate
those assertions and supply no 1:1 legacy mapping credit. Their mapping status
remains unresolved independently.

Open task `UNLIT-V2-MIGRATE`: after the full CPU/GPU corpus gate passes, record
the default-policy migration and remove the superseded V1 rejection path and
its selector from production, retaining the owned historical fixture for replay
with the recorded release. The renderer coordinator owns this task and must name
the owning renderer milestone before default activation; deletion is due no later
than the end of that milestone (at most one milestone of overlap). V2 is not activated by default here; no
GitHub milestone, issue completion or migration is claimed by this document.
Any later removal of the public V1 selector needs the appropriate semantic
version change and release migration notes.
