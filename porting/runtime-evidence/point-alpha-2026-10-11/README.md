# Martensite point alpha local evidence

Built-in unlit native points and marker sprites now preserve OCCT `3d097a0` alpha
policy, submission order and layer depth writes. The paired corpus has 21 modes,
six view configurations and two explicitly selected local devices. No upstream
API completion credit is claimed by this renderer slice.

- `receipt.json.gz`: final source hashes, actual runtime output, all 252 strict
  capture results, both complete 37-workload timing distributions and process/load
  snapshots. Source fingerprints remain unchanged across captures and timings.
- `initial-validation.json.gz`: final functional/capture evidence before timings.
- `captures/{lavapipe,host}`: every raw OpenGL/Vulkan RGBA and depth buffer; selected
  64×64 orthographic view/color/depth PNGs include every mode.
- `capture-sha256.json.gz`: byte fingerprints for committed raw captures and views.
- `exploratory-color-precision/receipt.json.gz`: original non-endpoint RGB custom
  sprite strict comparison, exact native/Rust fixture source copies and raw pairs.
- `*-performance.log`: all raw 100-sample/20-warmup optimized workload records.
- `cpu.log`, `clippy.log`, `python.log`, build/capture/asset logs: local checks.

Both devices pass complete offscreen acceptance, analytic point alpha/depth checks,
clipped and camera-persistent point submission, lighting acceptance and all 21
native golden replays without OCCT. The committed native assets under
`crates/vulkan/tests/fixtures/point-alpha` contain 84 buffers. Every asset was
regenerated twice with byte-identical output and its own device-bound provenance.
The 254 Rust all-target tests, 23 bounded Python Vulkan test modules, Clippy with
warnings denied, formatting, canonical toolkit generation and all checked-in
SPIR-V/source comparisons passed.

Native AMD passes 126/126 strict comparisons. Lavapipe passes 123/126; the exact
0.1 aspect-alpha cutoff under perspective leaves three strict failures at
64×64 offset0, 96×48 offset2 and 48×96 offset−3. Native software interpolation
crosses the shader's fixed threshold, while Vulkan discards. No gate is weakened.

The exploratory custom RGB sprite passes all six AMD views but differs by one
byte in 12 pixels per lavapipe view because of sRGB texture decoding precision.
The original strict evidence and exact fixture source are retained. RGB endpoints
isolate the final user-sprite alpha fixture; colored BALL fixtures separately
exercise sRGB decoding. This does not complete software sprite color parity.

The `point-alpha-updates` release workload measures retained opacity/alpha-mode
mutation, V3d extraction, Vulkan submission, fence wait and color/depth readback.
Median/p95 are 0.3588185/0.455052 ms on lavapipe and 0.298151/0.412052 ms on AMD.
The historical ten-workload lavapipe budgets pass unchanged at 1.25 median/1.5 p95.
Native historical device/probe identity differs, so no native regression pass is
claimed. The new point-alpha baseline remains unaccepted.

The host is shared: unrelated `oblikovati-build gate`/coverage compiler activity
was observed before timing. Team builds paused and active external compilation
drained before this 10.5-second interval; snapshots show only stopped FitAll
compiler processes at both ends. These observations qualify timing evidence and
are not a promise that the host was completely idle between snapshots.

Native comparison is local-only during porting; Windows/macOS checks are deferred.
Normal-bearing annotation material restrictions, custom shaders, bounded/indexed
native sprite draw-path differences and multisample alpha-to-coverage remain
outside this implemented slice. Surface, line and text alpha policies remain
independent. No pixel-row, endpoint or precision retuning is included.
