## Project identity and work selection

- Name this OCCT port **Martensite** in documentation and new code references to
  the project. Keep OCCT names when identifying upstream APIs and provenance;
  existing `rustycad-*` package identifiers remain valid until explicitly migrated.
- When choosing or resuming porting work, follow the current priorities in
  [docs/porting-plan.md](docs/porting-plan.md#current-priorities). Complete the
  [application blockers](docs/application-blockers.md) in priority order before
  broader visualization, Vulkan renderer, or exchange-format expansion.

## Toolkit completion and migration

- A toolkit's porting work is complete when its OCCT API coverage reaches 100% in
  [porting/api-inventory/METRICS.md](porting/api-inventory/METRICS.md): every
  declaration is implemented or excluded with a reason, and no row is partial.
- Migrate a completed toolkit to the [Martensite repository](../Martensite):
  - Remove the toolkit and its tests from every porting harness and oracle test
    in this repository: native probes, numeric and toolkit verifiers, parity
    ledger rows, inventory and coverage tooling.
  - Keep its tests as ordinary Rust tests with independent oracles. Do not keep
    a runtime or test dependency on OCCT.
  - Place the toolkit crate in the Martensite repository, and follow that
    repository's AGENTS.md, CONTRIBUTING.md and release process.
  - Release the toolkit only from the Martensite repository, under its governance.
- Migrate bottom-up: a complete toolkit migrates only after all its OCCT toolkit
  dependencies have migrated. Bring incomplete dependencies to 100% first.
- Toolkits that remain in this repository depend on the released Martensite
  crate of a migrated toolkit, not on a copy in this repository.

## Code style

- Functions: 4-20 lines. Split if longer.
- Files: under 500 lines. Split by responsibility.
- One thing per function, one responsibility per module (SRP).
- Names: specific and unique. Avoid `data`, `handler`, `Manager`.
  Prefer names that return <5 grep hits in the codebase.
- Types: explicit. No `any`, no `Dict`, no untyped functions.
- No code duplication. Extract shared logic into a function/module.
- Early returns over nested ifs. Max 2 levels of indentation.
- Exception messages must include the offending value and expected shape.

## Comments

- Keep your own comments. Don't strip them on refactor — they carry
  intent and provenance.
- Write WHY, not WHAT. Skip `// increment counter` above `i++`.
- Docstrings on public functions: intent + one usage example.
- Reference issue numbers / commit SHAs when a line exists because
  of a specific bug or upstream constraint.

## Tests

- Tests run with a single command: `eg: go test`.
- Every new function gets a test. Bug fixes get a regression test.
- Mock external I/O (API, DB, filesystem) with named fake classes,
  not inline stubs.
- Tests must be F.I.R.S.T: fast, independent, repeatable,
  self-validating, timely.

## Dependencies

- Inject dependencies through constructor/parameter, not global/import.
- Wrap third-party libs behind a thin interface owned by this project.

## Structure

- Follow the framework's, sdk's language's convention.
- Prefer small focused modules over god files.
- Predictable paths: controller/model/view, src/lib/test, etc.

## Formatting

- Use the language default formatter (`cargo fmt`, `golangci-lint`, `prettier`,
  `black`, `rubocop -A`). Don't discuss style beyond that.

## Logging

- Structured JSON when logging for debugging / observability.
- Plain text only for user-facing CLI output.

## Repository
- Keep good branch and worktree hygene, when work is merged delete those.


## Kernel extension and refactoring

Martensite began as an OCCT port and reached API parity; the port is history. OCCT is not an
oracle or a compatibility constraint. The GitHub milestones (Q1–X2, T1) are the work plan; each
milestone and issue states its own scope, dependencies, and acceptance criteria.

### Change and replacement

- Follow semantic versioning. A breaking change needs a version bump and changelog migration notes.
- Separate structural refactors from numerical or behavioral changes.
- Pin current behavior with owned fixtures before a refactor; a refactor passes them unchanged.
- A behavior change needs independent expected results, a kernel decision, and a new behavior version.
- Keep earlier behavior selectable for replay until a recorded migration.
- Reuse existing adaptors, builders, history modules, and mesh factories.
- Give each replacement a deletion task, corpus gate, and one-milestone migration limit.
- Prove one complete operation before extending a replacement engine.
- Do not add Go-kernel code or production fallbacks; check Go-derived work against its known defects.

### Operation result contracts

- Return completion, diagnostics, validation coverage, history, and achieved error together.
- Return typed errors with operation, entities, failure class, and cause; never only a flag or a string.
- Distinguish completed empty results from failed, cancelled, or incomplete computations.
- Validate topology, geometry consistency, self-intersection, and tolerance consistency in order.
- Select validation invariants for the declared body kind.
- Make the check mode explicit; a sampled pass is not an exact pass.
- Offer local checks of changed entities; keep the full ordered check for publication.
- Treat missing or inconclusive required checks as incomplete, never as valid.
- Keep invalid and partial geometry out of successful operation results.
- Express history as deleted, merged, created, split, or derived relations with all parents.
- Compose operation, copy, and healing history; leave document naming policy to the application.
- Perform healing explicitly on a copy and return its identity remap.

### Geometry and numerical policy

- Use one general pipeline per operation; keep conditioned fast paths inside it.
- Do not add shape-specific retry ladders or silent faceted replacements.
- Never use tessellation for modelling decisions or acceptance evidence.
- Take area and volume evidence from error-bounded integration, with its error.
- Keep spatial, parameter, angular, modelling, and approximation tolerances distinct.
- Report requested limits, effective settings (including clamps), and measured error separately.
- Tag each measurement as a certified bound, a sampled estimate, or unknown.
- Give tolerance caps explicit no-limit, report, and refuse modes; never default to no limit.
- Do not widen limits or reduce recorded error to satisfy an acceptance check.
- Use exact or filtered signs for topology decisions; keep metric tolerance tests separate.
- Compute shared incidences once and reuse their results.
- Never derive identity from returned order, coordinate rank, or proximity.
- Certify pruning bounds and root choices; report exhausted search as incomplete.

### Ownership and thread safety

- Published results are immutable, `Send`, and `Sync`; reading them takes no lock.
- Mutate only in operation-local builders, before publication.
- Make operations non-destructive by default; test input isolation after success, failure,
  cancellation, and source mutation.
- Define ownership for all geometry, topology, caches, and meshes reachable from a result.
- Do not treat `Clone`, `Arc`, locks, or `&Shape` as proof of isolation.
- Keep no thread-local or global mutable algorithm state; pass state in an operation context.
- A result must not depend on the thread, earlier work on it, or concurrent operations.
- Make operation objects `Send`; avoid `Rc` and `RefCell` in operation state.
- Give each `unsafe impl Send/Sync` a safety comment and a Miri or loom test.
- Assert `Send`/`Sync` at compile time for each public type, or document why it is thread-confined.
- Key reusable results by input revisions, options, and tolerance policy.
- Merge parallel results in a total order; no output may depend on hash iteration order.
- Measure copy, allocation, and lock costs before redesigning storage.

### Verification and completion

- Use independent oracles: closed forms, error-bounded integration, Monte Carlo membership,
  invariants, and reviewed baselines.
- Change a baseline only by a reviewed update that states the reason and behavior version.
- Require invariants, independent expected results, and generated degenerate cases for changed algorithms.
- Check per-face geometry, trims, tolerances, and history; volume alone is insufficient.
- Compare two processes byte for byte; compare multi-thread with single-thread results.
- Run concurrency stress tests and ThreadSanitizer for code that claims thread safety.
- Record revisions, options, platforms, oracle kinds, executed tests, and exclusions.
- Keep kernel acceptance and application acceptance separate.
- Benchmark identical accepted workloads against the previous release; report time and peak memory.
- Remove superseded production paths when their replacement gate passes.
- Do not weaken tests, widen tolerances, or hide failures to complete a migration.

## Integration workflow

- Use `main` as the integration branch. Merge completed worktree branches directly
  into `main`; do not create pull requests.
- The coordinating agent serializes merges, resolves conflicts, and runs the
  relevant checks. A subagent's work is complete only after its committed changes
  are integrated into `main`.
- After integration and checks, delete the completed worktree and its merged
  branch. Verify that all commits are on `main`, local changes are accounted for,
  and no active process or pending task needs the checkout or its build artifacts.
- Start new worktree branches from current `main`. Preserve unfinished work and
  report its branch, worktree, and remaining failures during handoff.
