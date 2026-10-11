# Martensite AIS color scales

`AisColorScale` implements the defined public and protected behavior of
`AIS_ColorScale` at OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Create it with `AisColorScale::new()?`, retain a range and pixel dimensions with
`set_range(min,max)` and `set_size(width,height)`, then display in mode zero.
Its constructor installs opaque, unlit white shading. Native selection computation
intentionally contributes no sensitive entities.

The presentation submits actual indexed, per-vertex colored triangles, a frame
polyline and anchored text through existing Graphic3d interfaces. Discrete ramps,
automatic four-segment smooth ramps and custom half-endpoint smooth ramps preserve
native vertex order, integer boundaries, packed colors, group ordering and text
placement. Auto/user label lists, empty labels, border/middle placement, reversed
intervals, each label-position policy and native symmetric label filtering are
retained. Title position is retained as a property, although native Compute does
not consult it. Setters retain their native behavior of not requesting redisplay.

Without an interactive context text metrics default to width 10, ascent 1 and
descent 1. With a context, the first active view's driver supplies metrics, narrowed
by integer truncation. Drawers copy inherited text properties when creating an own
text aspect, then use the scale foreground, left/bottom alignment and zoomable text.
Labels are submitted from UTF-16 through the project's Unicode string conversion.

Number labels use the typed C printf formatter. Custom sequence getters return
read guards or explicit output copies; release guards before changing the scale.
One-based setters resize missing entries and append when the index is nonpositive.
Native custom-label out-of-range calls throw because its bounds condition uses OR;
Martensite preserves errors rather than silently returning empty text.

Automatic instance FindColor retains the native default blue/red lookup even
when configured hue endpoints differ. Static overloads support custom HLS ramps.
Failed static lookup preserves output; failed instance lookup resets it. Reversal,
logarithmic spacing and smooth drawing do not alter native lookup. Uniform palettes
use CIE Lch maximum chroma, native DeltaE2000 spacing and single-precision RGB
interpolation, including full-circle and reverse hue sweeps.

Undefined upstream boundaries have explicit safe behavior. A missing inherited
text aspect uses Martensite's default instead of the native null dereference.
A context without active views and a lone border label return typed failures.
A zero-distance uniform palette returns an error instead of looping forever.
Nonfinite hue does not enter the native endless normalization loops. Invalid typed
printf formats fail instead of invoking a mismatched C variadic call; formatting
uses growable storage instead of the native fixed 1024-byte buffer. These cases
are regression tested and do not establish compatibility with undefined execution.

`tools/oracle/martensite_color_scale.cxx` records actual native presentation calls
using a local recording driver. Two native executions produced byte-identical
760 snapshots: 84 numerical records and 32 presentation cases containing 676
submission/group records. Committed split fixtures and their provenance JSON make
Rust replay independent of OCCT libraries. The recording presentation preserves
the real structure's identities and captures submitted vertex/index buffers and
text. Native execution initializes its inherited text aspect to satisfy the native
precondition; no native GL or GPU context is needed.

The task ledger fragment claims only 73 non-macro, API-visible declarations in the
pinned scope. Private helpers, fields and unused native SizeHint receive no credit.
Runtime type machinery follows the existing transient macro and has a descriptor
regression; shared generated inventory is owned by the coordinating agent.
Renderer submission interfaces are unchanged, so this class's evidence covers
native geometry/layout/submission rather than new renderer device precision claims.

Run all functional checks with `cargo test -p rustycad-tkv3d --all-targets --locked`.
Local linting uses `cargo clippy -p rustycad-tkv3d --all-targets --locked -- -D warnings`.
