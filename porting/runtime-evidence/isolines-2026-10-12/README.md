# Martensite isoline native evidence

Pinned upstream: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Feature base: checked main `c03a97d3e`; local Linux only. OCCT is a local oracle,
never a Rust test dependency or new CI requirement. Existing root native
libraries were reused; no OCCT rebuild was performed.

The ten generators and committed fixtures contain 179 native rows. Each ran
twice with byte-identical output; `native-runs.json` records executable,
generator, output hashes and exact compile commands. Shared oracle headers,
compiled native dependencies and focused pinned upstream sources have separate
hash receipts. Actual named driver objects capture submitted primitive buffers,
bounds and group callbacks; these CPU recordings are not raster captures.

| Native corpus | Rows | Behavior |
| --- | ---: | --- |
| surface | 30 | Real plane/cylinder/sphere/torus/cone/spline/trimmed/annulus topology, seeds and shared outputs |
| triangulation | 18 | XYZ/UV crossings, insertion order, nonlinear length fitting and locations |
| rface | 52 | Borrowed adaptor reload/reset, reversed face, missing-pcurve repair, Internal/External skips |
| adaptor | 2 | Genuine inherited virtual type query, checked partial failure and repair |
| references | 33 | Scalar/sequence aliases, initialized inline storage, IEEE comparison and GNU stable order |
| face paths | 30 | All presentation overloads, missing-mesh fallback, exact offset and differing locations |
| faults | 6 | Valid/fault/repair on retained presentations for checked invalid node indices |
| live inputs | 2 | Actual surface callback replaces surface/location and appends parameters |
| parameter edges | 4 | Null-surface no-write, infinity, negative count and closure-failure writes |
| surface slot | 2 | Const handle-slot replacement retaining old/new actual application objects |

The immutable surface-handle convenience has no upstream overload credit.
The corresponding row targets `add_on_surface_references`, which retains a
live caller handle slot. Its dispatch interface shares the real BRep runtime
descriptor and actual embedded core; no new native ancestor is fabricated.
Native BRep derivative evaluations are final and forward to the real core.
Unrelated Rust panic payloads resume across the native checked-failure catch.

The fragment contains 27 unique pinned native/compiler bindings: full eighteen
captured defined-visible Isolines declarations and nine bounded ToolRFace
construction/traversal declarations. ToolRFace remains partial without Edge
and allocation surfaces. Private helpers and parent families receive no credits.
The compiler capture uses pinned nightly `nightly-2026-09-25`, format61,
all-features and private-item policy. `scoped-capture.json` explicitly marks
only TKV3d and does not claim whole-workspace raw capture completeness. Scoped
normalization calls the production reachable/public and package/artifact-hash
logic; the capture loader is bounded to that one declared package. Production
`verify_api_claims.py` independently verifies each canonical path, source hash,
pinned visible declaration and uniqueness. Per-binding receipts stay below
500 lines; raw compiler and normalized artifacts are compressed with hashes.

Run Rust behavior checks without OCCT:

```sh
cargo test -p rustycad-tkv3d --all-targets
```

Final checks and hashes are linked through `artifact-hashes.json`. Graphical
functional/performance proof belongs to the real AIS_TexturedShape/wireframe
caller integration. No renderer baseline is accepted by this CPU port slice.
Native dangling pointers, uninitialized scalar parameters, signed-overflow
counts and null dereferences are undefined and are not fabricated as behavior.
