# Martensite porting plan

Martensite ports the defined OCCT scope to Rust. The
[compiler inventory](../porting/api-inventory/README.md) records measured API
coverage; toolkit notes and runtime receipts record behavioral evidence.

## Current priorities

User decision, 2026-10-12: APIs explicitly marked deprecated or obsolete by OCCT
do not require new ports. Keep existing implementations for now and record
declaration-specific upstream evidence for exclusions. See the
[retired API scope decision](adr/0002-exclude-upstream-retired-apis.md).

User decision, 2026-10-10: prioritize visualization and the new Vulkan renderer
above exchange-format porting. This execution order supersedes the earlier
base-toolkit-first scheduling in `porting/base-toolkit-priority.json` and the
original phase order below. Dependencies still need correct implementations;
complete the particular prerequisite that blocks a visualization or renderer slice.

User decision, 2026-10-11: verify porting locally on this Linux host and defer
Windows/macOS testing. Run the affected toolkit suites locally; rendering changes
retain local lavapipe and native-host functional captures and performance checks.
Existing Linux CI remains enabled. New porting work is kept local: user decision
on 2026-10-11, stop pushing to the remote. Integrate into local `main`, test locally,
and delete completed merged worktrees and branches. Deferred platform checks remain part of eventual
validation and do not remove APIs or behavior from the defined porting scope.

User decision, 2026-10-11: parallel porting uses at most six agents, including
the coordinating agent. Independent slices use separate checkouts; the coordinator
serializes local `main` integration, checks and deletion of completed worktrees.

User decision, 2026-10-12: use [acadrust](https://github.com/hakanaktt/acadrust)
for Martensite DWG format support through a Martensite-owned adapter. See the
[DWG backend decision](adr/0001-use-acadrust-for-dwg.md). Implement this with the
exchange-format work that follows the visualization and Vulkan renderer priorities.

1. Keep the integrated TKV3d AIS selection/highlighting behavior covered, including
   camera-dependent selection, native comparisons and regression tests.
2. Continue Martensite's Vulkan renderer through the existing Graphic3d interfaces.
   Match the legacy OCCT OpenGL renderer's capabilities using Vulkan 1.3; keep
   additional renderer features outside the current scope. Build a tested offscreen
   rendering path before broadening unrelated porting. The initial offscreen path
   now runs through V3d views and AIS display/erase presentations. AIS lifecycle,
   bulk and selected-object transitions, removal, status queries and per-view affinity
   have native comparisons. Structure/group transform persistence is integrated as
   partial. The 34 native AMD capture differences are traced to rasterizer precision
   ([controlled evidence](vulkan-raster-precision.md)); retain the raw failures and
   separate diagnostic gate before claiming bitwise parity or accepting its timing
   baseline. Segments/polylines, widths and shader patterns now reach Vulkan;
   [line evidence](vulkan-lines.md) records remaining gradient/depth parity gaps
   and the new performance workload. The [standard curve presentation family](stdprs-curve.md)
   now supplies all eight Add/Match overloads, native numeric comparisons and
   GPU/performance fixtures. [AIS_Line](ais-line.md) now consumes this path, with
   live appearance, unit-aware selection and GPU/performance coverage. User decision
   on 2026-10-10: defer the two AMD infinite-line pixel-row differences as known
   special cases; keep their evidence and continue feature work. [AIS_Point](ais-point.md)
   now adds live point datums, native selection and a batched Vulkan sprite path
   with functional and performance coverage. Point raster/color parity remains partial.
   [Vulkan text](vulkan-text.md) now renders retained font glyphs and oriented labels;
   [Shadow text](vulkan-shadow-text.md), [retained text face culling](vulkan-text-culling.md),
   [Subtitle backgrounds](vulkan-subtitle-text.md), and [Decal outlines](vulkan-decal-text.md)
   are integrated with native
   fixtures, both-device captures and optimized workloads. Strict text raster
   parity and broader text capabilities remain partial. [Blend/XOR](vulkan-blend-text.md)
   adds real logic operations and explicit native pixel text, with quantified color
   conversion/raster limitations retained.
   [AIS_Axis and labeled axis presentations](ais-axis.md) now add coordinate datums,
   native selection snapshots and inherited text-aspect rendering.
   [Plane presentation builders](plane-presentations.md) now preserve native edges,
   isolines, normal arrows and datum polygon submissions. [AIS plane datums](ais-plane.md)
   add native geometry, cached triangulation selection, size/color ownership and minimum
   persistence. Vulkan quadrangle lowering, unordered surface blending, device captures
   and plane-update timings are integrated. Strict raster/depth and timing gates remain
   open; the unsafe upstream XYZ constructor is explicitly partial.
   [Plane trihedrons](ais-plane-trihedron.md) now add labeled XY frames, child-owner
   selection, native comparisons and geometry-update performance coverage.
   [Native X11 presentation](vulkan-window-presentation.md) now adds swapchains,
   resize/rebind/remap checks, actual window captures and presentation timings.
   [Circle datums and adaptive deflection curves](ais-circle.md) add circular/arc
   construction geometry, filled/boundary selection and live geometry-update workloads.
   [Retained clipping](vulkan-clipping.md) adds live plane chains, persistent-anchor
   and bounding-box policy, view frustum culling, native capture controls and a
   measured plane-update workload. [Homogeneous vertex input](vulkan-homogeneous-vertices.md)
   preserves full position weights for the infinite planes needed by capping.
   [Stencil capping](vulkan-capping.md) now builds closed-group masks and infinite
   section surfaces, including native chain, instance and persistence policy.
   Opaque Phong-family section materials now share live material/light extraction;
   textures, custom section shaders and strict clipping-boundary parity remain open.
   [Shader material records](vulkan-materials.md) now match native coefficient
   packing. [Opaque Phong-family lighting](vulkan-lighting.md) now adds live normals,
   directional/point/spot/head lights, front/back materials and clipped shaders;
   strict native capture and performance gates remain open.
   [AIS animation timelines](v3d-design.md#ais-animation-timelines) now provide
   native child scheduling and timer lifecycle. [Camera animation](v3d-design.md#ais-camera-animation)
   adds live camera interpolation and deferred redraw. [Object animation](v3d-design.md#ais-object-animation)
   adds exact local transformation interpolation, context selection updates and
   immediate-layer Z-fit invalidation with native and GPU/performance fixtures.
   [Ordered group flipping](vulkan-group-flipping.md) now preserves annotation
   brackets, live orientation, lighting/clipping transforms and native cap filtering.
   [AIS text labels](ais-text-label.md) add native font-measured annotation placement,
   sensitive rectangles, retained aspect changes and viewer persistence transitions.
   [Point clouds](ais-point-cloud.md) now preserve aliased/copied buffers, cached
   bounds, application geometry callbacks and forced subset-owner highlighting,
   with native comparisons and both GPU profiles. [Nodal marker lighting](vulkan-lit-markers.md)
   now supports Gouraud/Phong points, sprite coverage and clipping. Retain strict
   point/line raster gates alongside live-normal update timings; continue remaining
   point material/alpha policies. [Nodal line lighting](vulkan-lit-lines.md) now adds
   Gouraud/Phong segments with clipping, native solid fallback for requested patterns,
   live-normal snapshots and both-device functional/performance coverage. Strict
   endpoint and color interpolation parity remains open.
   [Translucent surface lighting](vulkan-lit-transparency.md) now combines the three
   Phong-family algorithms with material opacity, distinguished back faces,
   deferred pass ordering and live opaque/translucent transitions.
   [Retained transparency](vulkan-oit-contract.md) now implements unordered blending,
   weighted OIT and dual depth peeling with live parameters, context alpha state,
   multisample fallback, layers and capping. Both-device functional replay and
   35-workload performance measurements are recorded with the implementation.
   Keep the documented texel, sprite, far-plane, high-sample and text driver
   differences explicit; full raster parity and API completion are not claimed.
   Continue remaining alpha-mask and cap/annotation policies alongside these
   retained diagnostic gates.
   [AIS trihedrons](ais-trihedron.md) now provide cached wire/shaded datum parts,
   native owner selection and protected application subclass hooks. Vulkan polygon
   offsets support the shaded geometry with both-device runtime checks; strict
   shaded selection, image parity and performance-baseline gates remain open.
   [Rubber-band overlays](ais-rubber-band.md) now preserve live screen-point sequences,
   frontier-constrained polygon filling, reusable geometry and application subclass hooks,
   with paired native captures and polygon-update timings.
   [AIS view input buffers](ais-view-input-buffer.md) now preserve all public gesture
   records, native defaults and reset/copy semantics through committed oracle assets.
   [AIS navigation dependencies](ais-view-navigation.md) now preserve camera policies,
   mouse gestures and six-axis walking state with native boundary/flag/copy fixtures.
   [AIS view controller input](ais-view-controller.md) now queues native selection,
   zoom and Z rotation, with public policies and UI-only navigation cancellation.
   [Keyboard navigation](ais-view-controller-keyboard.md) now preserves frame
   timing, all six axes, stance/thrust/release semantics and focus-loss reset.
   [Click and touch dispatch](ais-view-controller-pointer.md) now preserves animation
   ownership, selection bindings and native touch transitions.
   [Window-listener integration](ais-view-controller-window-listener.md) now exposes
   the complete controller callback interface with typed mouse/touch failures.
   [UI/render queue transfer](ais-view-controller-transfer.md) preserves pending
   requests, allocator ownership and drag priority across frames.
   [Camera frustums](ais-camera-frustum.md) add live geometry, appearance and both
   selection modes. [Primitive-array initialization](sensitive-primitive-initialization.md)
   now preserves all five overloads and native retained/failed initialization state.
   [Light-source binding](ais-light-source-binding.md) adds properties and world/local
   transforms. [Light presentations](ais-light-source-presentation.md) now add
   concrete construction, geometry, selection, directional highlighting and dragging,
   with native fixtures and both-device captures and optimized timings. Classes remain
   partial. Vulkan Shadow-style light labels now render; strict named-light capture
   differences and native-default-layer parity remain open.
   [Color scales](ais-color-scale.md) now preserve native palette policies, labels,
   layout and real presentation submissions. [Controller picking](ais-controller-picking.md)
   and [selection-aware fitting](ais-view-controller-fit.md) now use real retained
   geometry and cameras. [Touch synthesis](ais-view-controller-gestures.md) preserves
   native contact-count transitions and integer pan boundaries.
   [Primitive matching](sensitive-primitive-matching.md) now has native differential
   and ownership/lifecycle fixtures; traversal regressions preserve callback ordering.
   [Native point alpha](vulkan-point-alpha.md) adds native point/sprite alpha policies
   and committed GPU oracle buffers. Exact-cutoff software differences and additional
   sprite/material capabilities remain open.
   [Native line alpha](vulkan-line-alpha.md) now preserves forced blending in draw
   order with depth writes. Committed native buffers support Rust-only GPU replay;
   strict software endpoint parity and timing-baseline acceptance remain open.
   Continue its remaining dispatch alongside the remaining presentation
   and legacy renderer capabilities
   (see [current evidence](vulkan-renderer.md)).
   Keep functional captures and performance regression workloads together on both
   lavapipe and the native host GPU.
   Advance TKService/TKV3d functionality needed by this path alongside it; Vulkan
   work does not wait for every visualization API to be ported.
3. Expand interactive visualization, presentations and TKMeshVS, and the DRAW
   viewer commands needed to exercise them. Add renderer capabilities through
   tested scenes and observable viewer workflows.
4. Continue other dependency-ready porting after these priorities. Exchange-format
   expansion (STEP, IGES, mesh formats and exchange wrappers) is lower priority;
   address it earlier only when a concrete prioritized workflow needs it or an
   existing regression needs repair. Preserve unfinished exchange work.

See [visualization design](visualization-design.md#6-vulkan-implementation-sequence)
for the renderer sequence and acceptance checks. Exchange formats remain in the
API denominator. Reprioritization changes scheduling, not scope or proof standards.

## Objective and scope

Port the in-scope pinned OCCT API and its observable behavior to Martensite.
Follow [scope.json](../porting/scope.json) for toolkit replacements and exclusions
across visualization, application framework, data exchange and DRAW tools. Translate
upstream unit and regression tests 1:1 into Rust and use the
pinned C++ implementation as the behavioral oracle. Third-party C/C++ libraries
used by OCCT may remain external dependencies; port their OCCT adapters and retain
the capabilities they enable. Linking the OCCT kernel to provide a missing Rust
operation does not count as implementing that operation.

### Independent Rust tests and oracle retirement

User decision, 2026-10-11: OCCT is a temporary oracle during porting. Martensite
will evolve independently after the port; upstream compatibility is the initial
behavioral baseline, not a permanent dependency or evolution policy.

Port each upstream test's setup, inputs, assertions, cleanup, platform variants
and tolerances into executable Rust tests. A native comparison receipt does not
replace that translation. Rust behavioral tests must run from the Martensite
checkout or extracted release packages without an OCCT installation, source tree,
build, executable, or OCCT-linked C++ subprocess. Static expected files and test
resources belong to the Rust test crates and remain usable independently; retain
their attribution and capture provenance. Keep permanent test resources outside
removable oracle-tool directories; migrate existing shared fixtures before
retiring their harnesses.

Keep C++ probes, capture generators, live differential comparisons and upstream
inventory checks in removable porting infrastructure. Use them to establish
baselines and diagnose differences while porting. Ordinary Rust tests consume
committed assets rather than regenerating expectations from the implementation
under test. Porting-only receipt checks are not the behavioral regression suite.

Before declaring the port complete, remove the OCCT harnesses and source-fetch,
build and comparison jobs, then run the full Rust functional, rendering and
performance suite and extracted-package checks without OCCT available. Preserve
the translated test coverage and static baselines. Subsequent behavior changes
follow Martensite's own reviewed requirements; OCCT revision metadata becomes
historical provenance. Third-party libraries used through Martensite adapters
remain separate from this OCCT dependency boundary.

Rust naming, ownership and error syntax may differ from C++. Each upstream API
must still have an explicit equivalent with the same capabilities and observable
contract. C++ ABI compatibility is not implied by a Rust port. Convenience APIs can
be added after the mapped compatibility API passes its parity tests.

Baseline: OCCT **8.1.0-dev1**, commit
`3d097a0328e71b826377d4814ab05ec3c3d23871`. This is a development checkout, not a
released OCCT version. Freeze it for this port. Any baseline update during porting
requires review; completing the port does not establish an obligation to follow
later OCCT changes. The existing sibling checkout is clean and remains unmodified.

## Measured source scope

The generated [inventory](../porting/inventory.json) currently contains:

| Module | Toolkits | Work covered |
| --- | ---: | --- |
| FoundationClasses | 2 | Runtime, collections, units, tolerances, numerical algorithms |
| ModelingData | 4 | 2D/3D geometry, geometry construction, BRep representation |
| ModelingAlgorithms | 14 | Topology, primitives, booleans, meshing, healing, features, helix |
| Visualization | 7 | Services, interactive display, selection, OpenGL/GLES, VTK, D3D |
| ApplicationFramework | 13 | Documents, attributes, undo/redo, binary/XML persistence |
| DataExchange | 14 | STEP, IGES, mesh formats, XCAF, readers/writers |
| Draw | 21 | Interpreter, modeling/viewer commands, QA and exchange test tools |
| **Total** | **75** | **410 package entries, including shared package references** |

Discovery found 7,216 header candidates, 561 GoogleTest source files, 8,648
GoogleTest declaration candidates, 16,698 DRAW case candidates and 18,309 total
regression/support files. These are source counts, not API symbol or runtime test
counts. The inventory also retains data/resource paths and toolkit dependency lists.

The source build uses `src/MODULES.cmake` and per-module `TOOLKITS.cmake`;
`adm/MODULES` omits `TKHelix` at this revision. Shared OpenGL/GLES packages need
separate configuration coverage even when they share source files.

## Definition of done: 100% means every item

The following are release gates, to be automated in Phase 0 and extended with every
port. Passing scaffold CI does not satisfy these gates.

1. **API coverage is 100%.** Every public type, callable overload, constructor,
   operator, constant, enum/value, public field, public template contract and
   extension point has an explicit Rust mapping. Include inline/header-only code,
   inherited accessible interfaces and supported specializations,
   generated headers, command APIs and configuration/platform-dependent APIs.
   Protected hooks used by subclasses are part of the extension contract. Audit
   public macros and classify observable functionality separately from C++ syntax.
   APIs explicitly marked deprecated or obsolete by OCCT are excluded from required
   new ports under [ADR0002](adr/0002-exclude-upstream-retired-apis.md); retain their
   existing Rust implementations and upstream exclusion evidence.
2. **Upstream test translation is 100%.** Each upstream unit/regression case maps
   to a runnable Rust test with the same setup, inputs, assertions, cleanup and
   tolerance. Parameterized and typed instances, DRAW setup scripts, data and
   platform variants remain traceable. No fixture is silently replaced or dropped.
3. **Behavioral comparisons pass for every mapped capability.** Cover success,
   failure, degeneracy, tolerance boundaries and state changes. Add comparisons
   for APIs without upstream tests. Test status/errors, outputs, topology and
   side effects, not merely whether execution completes.
4. **All required configurations pass.** Run the union of supported OS, build,
   precision, graphics and optional native dependency profiles. A feature disabled
   in default CI still remains in the denominator. Missing dependencies, skipped
   tests and absent datasets are pending work, not passes.
5. **No unresolved parity failures or implementation placeholders.** A stub,
   unconditional success, ignored assertion, blanket expected failure, or call into
   OCCT for production behavior cannot count as a port. Preserve oracle failures
   as evidence; do not silently normalize an upstream bug into a passing result.
6. **Maintainability and distribution checks pass.** Rust formatting, linting,
   tests, docs and dependency checks pass; translated code/test notices and native
   dependency requirements are recorded. Benchmark representative workloads and
   report regressions before release; numerical parity cannot be traded for speed.

API coverage = verified mapped API items / total authoritative in-scope API items.
Test translation coverage = verified translated cases / total upstream cases in the
configuration union. Report both globally and per toolkit. An item counts as
verified only with passing evidence on every applicable profile at the current
Rust revision. Partial mappings do not count as complete.

The denominator excludes the internals of third-party dependency libraries, as
requested, but includes OCCT's wrappers around them. Internal private OCCT helpers
can be reorganized as long as behavior survives. Any other scope removal requires
an explicit scope decision; it must not be disguised as a successful mapping.
Track source/branch coverage independently. Neither finite tests nor a coverage
percentage mathematically proves equivalence for every possible input; these gates
provide auditable evidence of the required contract.

## Architecture and translation rules

Start with a facade (`rustycad-kernel`) and a test support crate (`rustycad-parity`).
Add production crates by OCCT toolkit or a justified group of tightly coupled
packages as work begins. Keep package/type traceability within each crate. Derive
build order from the recorded toolkit dependency graph; resolve cycles before
creating Cargo dependencies. Do not create 75 empty crates and call them ports.

Use `f64` where OCCT uses `Standard_Real`. Preserve integer widths, index bounds,
1-based and arbitrary-bound collections, ordering, tolerance constants, transforms,
units and serialization conventions. Treat handle identity, shared mutation,
reference lifetime, topology orientation/location and copy behavior as contracts.
Choose Rust representations after tests establish those semantics; an automatic
`Vec`/`Arc`/`Result` substitution can change observable behavior.

Translate exceptions into documented error variants while preserving when failures
occur and whether partial mutations occur. Capture null-handle cases, invalid
inputs, cancellation, progress callbacks, RTTI/downcasting and subclass dispatch.
Document language-specific substitutions in the API mapping. Use generics/traits
for template and extension contracts, with compile-time examples in addition to
runtime comparisons. Do not broaden tolerances to make a new implementation pass.

Keep the C++ oracle outside production crates. It may use C++ glue and third-party
bindings. Put retained native integrations behind small, optional adapter crates;
keep platform checks explicit. Avoid a mandatory OCCT build for `cargo test`.

### Reusing Rust packages

Evaluate mature candidates such as `nalgebra` for fixed-size vectors/matrices and
`faer` for larger dense systems. The [research note](tooling-research.md) records
current versions and alternatives (`glam`, `ndarray`, `approx`, `proptest`). Select
one implementation for a need, rather than adding overlapping math stacks now.

For each substitution, compare indexing/layout, multiplication order, transforms,
eigensolver conventions, convergence, singular behavior, rounding, infinities,
NaNs, signed zero and deterministic output against OCCT. Preserve OCCT-facing types
and adapters so dependency choices do not leak into the public compatibility API.
Keep OCCT algorithms where a library changes observable results. Record version,
features, license, MSRV and parity evidence before accepting a dependency.

## Execution phases and exit criteria

### Phase 0 — authoritative scope and oracle infrastructure

The current change supplies Rust tooling and a source-level inventory. Complete the
following before claiming API percentages or marking a first toolkit complete:

- Extract declarations with Clang using OCCT's actual compile commands, installed
  and generated headers, and a matrix of preprocessor configurations. Include
  header-only APIs that no `.cxx` file happens to instantiate. Combine AST results
  with a manual audit of macros, aliases, template contracts and DRAW commands.
- Give each item a stable ID: baseline, toolkit/package, qualified C++ name,
  canonical signature, overload/template discriminator and applicable profiles.
  Deduplicate inherited declarations without losing their exposure in derived APIs.
  Store source location and configuration evidence. Fail on unclassified headers.
- Build reproducible oracle profiles. Start with FoundationClasses on Linux, then
  expand to full modules on Linux, Windows and macOS and the relevant graphical
  backends. Pin C++ compiler, dependencies, flags, locale, thread settings and data.
- Enumerate actual GoogleTest runtime names with `--gtest_list_tests`, including
  typed/parameterized cases and disabled cases. Enumerate DRAW suites/grids/cases
  using its runner. Reconcile runtime discovery with source discovery; unexplained
  differences block inventory completeness. Track unavailable platform cases.
- Hash datasets and generated fixtures and retain provenance. Identify external
  datasets referenced by tests, obtain them where available and record missing data
  as blockers. Keep large/binary artifacts in versioned artifact storage with hashes.
- Implement the process-based oracle protocol below, its Rust driver and artifact
  retention. Add mapping validators and a completion command that exits nonzero for
  missing mappings, stale evidence, pending profiles, ignored tests or divergences.

**Exit:** reviewed API/test denominators, reproducible foundation oracle, validated
mapping schema, and one end-to-end differential test proving the pipeline works.

### Phase 1 — FoundationClasses (`TKernel`, `TKMath`)

Begin with `Standard`, `Precision`, errors, handles and bounded collections; follow
with strings, units, quantities, resources, threading and storage. Port `gp` and
math packages in their actual dependency order, including solvers and numerical
utilities. Use small vertical slices: translate upstream tests, establish oracle
results, implement Rust behavior, then add boundary/differential cases.

**Exit:** all foundation API mappings and tests pass, including lifetime/indexing,
concurrency/error behavior and numerical edge cases. Do not defer difficult OS or
storage APIs and still label the toolkit complete.

### Phase 2 — ModelingData

Port `TKG2d`, `TKG3d`, `TKGeomBase`, and `TKBRep` following their dependency graph.
Cover curves/surfaces, evaluation/derivatives, analytic and spline representations,
adaptors, topology and representation sharing. Verify degenerate knots, periodicity,
parameter domains, orientation/location and shape identity.

**Exit:** geometry and topology data contracts match the oracle; each upstream test
is translated, including representation and mutation tests.

### Phase 3 — ModelingAlgorithms

Port `TKGeomAlgo` and `TKTopAlgo`, then primitives and the graph-ordered boolean,
healing, meshing, HLR, fillet, offset, feature, helix and expression toolkits.
Use representative compounds and pathological/degenerate geometry early; avoid
building only happy-path primitives before confronting boolean and healing contracts.

**Exit:** all 14 toolkits have complete mappings and regression translations;
shape validity, topology, tolerances, failure modes and deterministic behavior match.

### ApplicationFramework and DataExchange — lower scheduling priority

Once their dependencies are available, port documents, attributes, transactions,
undo/redo and XCAF; then in-scope exchange formats and wrappers. Native OCCT
persistence formats excluded by `scope.json` remain excluded. Schedule new exchange
work according to the current priorities above, with verified dependencies.
Cover assemblies, references, names, layers, materials, colors, units, versioning,
malformed files and round trips in both directions.

**Exit:** document state and metadata match; persisted/exchanged files interoperate
with the pinned OCCT readers and writers. Byte equality is required where contractual;
otherwise use a documented semantic comparator without discarding meaningful data.

### Visualization, Vulkan and DRAW — current scheduling priority

Advance services, display, selection and the new Vulkan 1.3 renderer in bounded
slices. Vulkan replaces the OCCT OpenGL/GLES drivers under the approved scope
decision. Follow `scope.json` for other backend decisions, including undecided VTK
integration. Use FreeType, Tcl/Tk and platform libraries behind project-owned
interfaces where needed. Port in-scope OCCT integration behavior and DRAW commands;
a Rust test translation alone does not replace the DRAW API.

Use controlled rendering scenes and fixed camera, viewport, font, graphics driver
and framebuffer settings. Compare selection and scene state exactly where possible;
use explicitly reviewed image tolerances for raster variation. Cover headless and
interactive/platform behavior in suitable environments.

**Exit:** every in-scope backend/profile, public command and corresponding regression case
has passing evidence; no permanent graphics skips are hidden in completion totals.

### Phase 6 — full parity audit and release readiness

Run the complete configuration union against the frozen baseline, reconcile all
API/test records and audit every exclusion. Exercise combined workflows across
modeling, documents, exchange and display. Run stress, allocation and concurrency
checks and benchmark the representative corpus. Publish per-toolkit API and test
coverage plus divergences, source coverage and performance results.

**Exit:** all definition-of-done gates pass. Only then declare the port complete.
Estimate calendar delivery after measuring throughput on foundation and at least
one difficult geometry/algorithm slice; source file count is not a reliable estimate.

## Oracle protocol and comparison policy

Use separate C++ and Rust executables consuming the same versioned, deterministic
requests. Each request carries case ID, operation/API ID, fixed seed and inputs;
responses carry success/error status, typed results and observable state. Keep object
handles in a test-session registry so sequences can exercise shared identity and
mutation. A schema must encode special floating-point values and integer widths
without JSON number loss; encode `f64` bit patterns when exact values are needed.

Every fixture records OCCT revision, compiler/build/dependency profile, platform,
input hashes, output schema version, tolerance/comparator version and provenance.
Record crashes, timeouts, stderr and exit codes as failures with reproducible inputs.
A missing oracle executable in a live porting comparison must fail explicitly;
it must never skip and pass. Standalone Rust behavioral tests use committed
fixtures and must not require that executable.

| Result | Default comparison |
| --- | --- |
| Enums, flags, counts, statuses, discrete indices | Exact |
| Scalars/vectors/matrices | Exact where contractual; otherwise the upstream assertion's tolerance |
| NaNs, infinities, signed zero | Explicit API-specific semantics, bit checks where needed |
| Shapes | Type, orientation, placement, connectivity, sharing, tolerances and geometric properties |
| Unordered collections | Canonicalize only if ordering is genuinely unspecified |
| Files/documents | Bytes where deterministic; otherwise schema and full semantic content |
| Images | Controlled renderer plus reviewed image metrics and thresholds |
| Stateful calls | Result plus all observable mutations, callbacks and error timing |

Mass/volume alone cannot establish shape equivalence. Store enough structure to
detect topology and orientation differences. Canonicalization must not erase an
observable contract. Avoid fast-math and unreviewed fused-operation changes.

The included `Tolerance` helper supports finite nonnegative absolute/relative
bounds and defined special-value handling. It has no default tolerance. Its tests
exercise harness behavior only. Extend comparison tools as their real callers need
them; keep their tests separate from translated upstream cases.

## Per-change workflow and traceability

1. Select a dependency-ready package/API slice and its upstream tests.
2. Preserve original test IDs in Rust test names or mapping metadata; reference the
   upstream path and assertion locations. Do not copy OCCT copyright headers.
3. Run the C++ tests and capture oracle cases, including error and boundary inputs.
4. Translate the tests before implementing the Rust slice. Keep assertion counts,
   fixture setup and tolerances traceable. Splitting/merging tests is allowed only
   with a complete explicit mapping of each upstream case and assertion.
5. Implement the API, compare it with the oracle and add cases where upstream
   coverage is weak. Property tests supplement the translated suite.
6. Update mappings and per-profile evidence. Run Rust checks and the relevant
   differential suites. Review numerical and state semantics before optimizing.

The Phase 0 ledger will extend `porting/parity.json`. API rows need stable upstream
ID/signature, source, Rust item, applicable profiles, translated and added tests,
status, divergences and evidence artifacts. Test rows need upstream runtime ID,
source, fixture hashes, Rust test ID, preserved assertions/tolerances and result
per profile. Status transitions are `pending → mapped → implemented → verified`;
regressions or stale artifacts remove verified status. Skips retain an explicit
reason and remain incomplete. Reject duplicate IDs, dangling mappings and evidence
from a different baseline/Rust revision. Completeness flags require audited artifacts,
not merely changing a boolean.

## Provenance and project decisions

Retain the repository's existing GPLv2 license for the bootstrap. OCCT source/test
translations must retain their upstream notices, attribution and accompanying OCCT
license/exception texts. Review distribution terms before publishing translated
code and adding native dependencies. Translated TKernel implementation and test files now retain their OCCT notices;
the accompanying upstream license texts are under `licenses/`. The inventory also
records paths and discovery metadata.

Decisions fixed now: frozen upstream commit; toolkit scope in `scope.json`; Rust 2024;
pinned stable tools; native third-party libraries may remain; no production OCCT
fallback; API/test coverage distinct from line coverage; no premature math dependency.
Representation choices, platform profile details and numeric substitutions are made
per slice with oracle evidence and recorded alongside the mapping.

Mouse button and pointer-motion dispatch now has committed local native oracle
fixtures and Rust regressions; see [Martensite mouse dispatch](ais-view-controller-mouse.md).
[3D mouse controller routing](ais-view-controller-space-mouse.md) now uses shared
HID dispatch with committed native snapshots. Camera/window application remains open.
