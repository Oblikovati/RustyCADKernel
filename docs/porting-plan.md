# OCCT → Rust porting plan

Execution has reached TKMath. See [TKernel progress](tkernel-progress.md) and
[TKMath progress](tkmath-progress.md) for mapped declarations, executable oracle
evidence, and outstanding toolkit acceptance work.

## Objective and scope

Port the entire pinned OCCT API and its observable behavior to Rust. Preserve all
seven modules, including visualization, application framework, data exchange and
DRAW tools. Translate upstream unit and regression tests 1:1 into Rust and use the
pinned C++ implementation as the behavioral oracle. Third-party C/C++ libraries
used by OCCT may remain external dependencies; port their OCCT adapters and retain
the capabilities they enable. Linking the OCCT kernel to provide a missing Rust
operation does not count as implementing that operation.

Rust naming, ownership and error syntax may differ from C++. Each upstream API
must still have an explicit equivalent with the same capabilities and observable
contract. C++ ABI compatibility is not implied by a Rust port. Convenience APIs can
be added after the mapped compatibility API passes its parity tests.

Baseline: OCCT **8.1.0-dev1**, commit
`3d097a0328e71b826377d4814ab05ec3c3d23871`. This is a development checkout, not a
released OCCT version. Freeze it for this port; handle later OCCT changes through
reviewed baseline updates. The existing sibling checkout is clean and remains unmodified.

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
   inherited accessible interfaces, supported specializations, deprecated APIs,
   generated headers, command APIs and configuration/platform-dependent APIs.
   Protected hooks used by subclasses are part of the extension contract. Audit
   public macros and classify observable functionality separately from C++ syntax.
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

### Phase 4 — ApplicationFramework and DataExchange

Once their dependencies are available, port documents, attributes, transactions,
undo/redo, persistence and XCAF; then all exchange formats and wrappers. Work can
proceed alongside remaining algorithms only when each required dependency passes.
Cover assemblies, references, names, layers, materials, colors, units, versioning,
malformed files and round trips in both directions.

**Exit:** document state and metadata match; persisted/exchanged files interoperate
with the pinned OCCT readers and writers. Byte equality is required where contractual;
otherwise use a documented semantic comparator without discarding meaningful data.

### Phase 5 — Visualization and DRAW

Start required test-facing commands early, then finish all services, display,
selection and graphics backends. Reuse native OpenGL/GLES, FreeType, Tcl/Tk, VTK,
platform and other external libraries where needed. Port OCCT integration behavior
and all DRAW commands; a Rust test translation alone does not replace the DRAW API.

Use controlled rendering scenes and fixed camera, viewport, font, graphics driver
and framebuffer settings. Compare selection and scene state exactly where possible;
use explicitly reviewed image tolerances for raster variation. Cover headless and
interactive/platform behavior in suitable environments.

**Exit:** every backend/profile, public command and corresponding regression case
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
A missing oracle executable must fail explicitly; it must never skip and pass.

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

Decisions fixed now: frozen upstream commit; all modules in scope; Rust 2024;
pinned stable tools; native third-party libraries may remain; no production OCCT
fallback; API/test coverage distinct from line coverage; no premature math dependency.
Representation choices, platform profile details and numeric substitutions are made
per slice with oracle evidence and recorded alongside the mapping.
