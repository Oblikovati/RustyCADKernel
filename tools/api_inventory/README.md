# OCCT API extractor (experimental)

`occt-api-extract` parses one configured OCCT header as a synthetic C++
translation unit and writes JSON Lines to stdout. It uses Clang LibTooling and
USR generation, rather than the hand-maintained API ledger. This is an
inventory input; it does not establish that the full OCCT API has been
extracted or ported.

## Build

The prototype is built against LLVM and Clang 23.1.2:

```sh
cmake -S tools/api_inventory -B target/api_inventory \
  -DLLVM_DIR=/path/to/llvm-23.1.2/lib/cmake/llvm \
  -DClang_DIR=/path/to/llvm-23.1.2/lib/cmake/clang \
  -DCMAKE_PREFIX_PATH=/path/to/llvm-23.1.2
cmake --build target/api_inventory -j4
ctest --test-dir target/api_inventory --output-on-failure
```

## Invocation

```sh
target/api_inventory/occt-api-extract \
  --source-root /path/to/OCCT/src \
  --main-header /path/to/OCCT/src/Standard/Standard.hxx \
  --emit-header /path/to/generated/header.hxx \
  -- -std=c++17 -I/path/to/OCCT/include -I/path/to/build/include
```

Compiler flags follow `--`. The wrapper should remove compiler executable,
source-file, output-file, and compile-only arguments from captured build flags.
The tool adds syntax-only mode and its Clang resource directory. A generated
main header is emitted even when it is outside `source-root`. Included source
files are emitted when they are the main header, explicitly named with
`--emit-header`, or `.lxx`, `.pxx`, or `.gxx` fragments under `source-root`.
All encountered include files are reported by `header_seen`.

## Event policy

Each output line is a JSON object with an `event` discriminator and the
`main_header` that caused it:

* `declaration`: Clang declaration kind, USR, qualified name, canonical
  signature/type, normalized identity signature, template parameter signature,
  canonical and written parameter types, per-parameter default-expression
  observations, direct and ancestor access, parent USR, implicit/macro status,
  spelling/expansion locations,
  visibility disposition, and source path. Public and protected declarations
  are candidates. Private declarations and declarations nested under a private
  type are retained with an explicit exclusion reason. Parameters, compiler
  generated declarations, template-owned record/function patterns, access
  markers, static assertions, and other non-API AST nodes are also reported as
  exclusions. Namespace/file variables and static data members are candidates.
  Dependent `UnresolvedUsingValue` declarations remain visible API obligations
  with `api_resolution=unresolved_using_value_target` and
  `callable_status=unknown` until template instantiation resolves their targets.
* `macro`: definition, replacement tokens, parameters, and location. Include
  guards are identified by Clang's completed multiple-include/header-guard
  analysis, emitted as `macro_category=include_guard`, and excluded from API
  candidates. A macro merely defined below `#ifndef` (such as a fallback
  constant macro) remains a source macro candidate.
* `header_seen`: canonical included path plus whether declarations from that
  path were emitted for this run.
* `skipped_range`: an inactive preprocessor range with start/end locations.
  Declarations inside it are not in the AST output; this event makes that
  omission visible.
* `diagnostic`: Clang severity, message, source location when available, and
  main-header context.
* `summary`: counts by encountered declaration kind and disposition. Any
  encountered declaration kind without an explicit candidate or exclusion
  policy is `unhandled_decl_kind`, listed in `unhandled_decl_kinds`, and makes
  the process exit with status 3. Parse errors fail with a nonzero status.

Declarations are source occurrences. A header declaration and its `.lxx`
definition can therefore share a USR while carrying distinct locations. A
consumer groups callable declarations by USR, kind, normalized identity
signature, and normalized template parameter signature, while other declarations
use USR and kind. All source occurrences are preserved. Clang can emit the same USR for SFINAE overloads whose dependent
return types differ; the canonical signature keeps those overloads distinct.
Parameter spelling is normalized so out-of-line definitions and redeclarations
with different parameter names share an identity. Template wrappers are
candidates; the associated pattern record or function node is retained as
`template_owned_declaration` to avoid claiming it as a second API symbol.
Default arguments are recorded per observed declaration, including inherited,
unparsed, and uninstantiated states. They do not participate in declaration
identity because defaults may appear on only one redeclaration. Merged rows
retain each distinct default observation; older extractor records carry an
explicit `default_argument_capture=unavailable` status, while non-callable
records are marked `not_applicable`. Empty template metadata means the
declaration is not a function template. Template defaults are removed from
identity only when the bounded normalizer accepts the syntax; unsupported forms
retain their raw spelling and are blocked from crosswalk credit.

## Completeness limits

This prototype is intentionally not an API-completeness proof. Extraction
depends on the exact compile flags, generated headers, platform configuration,
and selected main-header set. Each configuration sees only declarations active
in that preprocessor run; `skipped_range` records inactive regions but does not
enumerate declarations hidden inside them. A cross-platform/configuration
matrix is needed to cover those APIs. External/system headers are excluded
from declaration emission, though their paths appear as `header_seen`.
Unsupported declaration kinds stop a run, but passing the gate only means all
kinds encountered in that one AST have an explicit policy. Macro candidate
rows are not expanded into C++ declarations or Rust coverage claims. Rust item
matching, source/header reachability validation, C-binding dependency
exclusions, and global inventory reconciliation belong to the orchestration
and proof pipeline.

## Whole-checkout capture and reconciliation

The header manifest is the input boundary. Capture uses each header's owning
CMake toolkit flags and fingerprints the pinned source, compiler binary and
libraries, generated version header, compilation database, SDK headers, and
collector. Successful cached probes are reusable only with the same inputs and
unchanged artifact bytes. Failed probes remain in the header table.

```sh
python3 tools/capture_occt_inventory.py \
  --source /path/to/OCCT \
  --database /path/to/configured/compile_commands.json \
  --extractor target/api_inventory/occt-api-extract --jobs 8
python3 tools/merge_occt_inventory.py \
  --capture target/api-proof/occt-inventory
PYTHONPATH=tools python3 -m unittest discover -s tools/tests
```

`--limit N` runs a development subset and leaves every other header pending.
It cannot produce a complete inventory. `.lxx` fragments are tracked through
successful containing-header probes. Generated headers have explicit generated
identities. Shared headers retain every toolkit owner, including owners absent
from the current configuration.

The merger writes deterministic compressed tables under
`porting/api-inventory/`: declarations, macro candidates, header coverage,
diagnostics, unsupported records, skipped preprocessor ranges, and contributing
files. The manifest records counts and SHA256 values. C++ USR, declaration
kind identify non-callable declarations; callable identities also include
normalized identity and template parameter signatures. All observed source
sites and raw signature variants are retained. A
source-TU definition without a header declaration site is retained as an
excluded implementation-only declaration. A missing USR uses a visibly
unverified location identity. Signature or visibility
conflicts remain explicit blockers. Macro definitions are a separate table and
must not silently enter the declaration coverage numerator.

A successful standalone header parse proves only that configured header scope.
The inventory does not yet resolve every standalone-header failure, inactive
preprocessor branch, platform-specific toolkit, macro mapping, or behavioral
obligation. Therefore `coverage_proof` and `declaration_inventory_complete`
remain false. A ledger mapping alone is not compiler correspondence or test
parity evidence.

## Auditing capture gaps

`retry_occt_header_context.py` retries headers included directly by real CMake
translation units, plus short chains of generated or external wrapper headers.
It stops at the first OCCT source header; it does not discover arbitrary
OCCT-header-to-header inclusion chains. The retry table preserves compiler
flags, source hashes, and probe artifacts:

```sh
python3 tools/retry_occt_header_context.py \
  --source /path/to/OCCT \
  --database /path/to/configured/compile_commands.json \
  --extractor target/api_inventory/occt-api-extract \
  --capture target/api-proof/occt-inventory/capture-manifest.json \
  --output target/api-proof/occt-context-retries
```

Retry artifacts are recorded relative to the retry manifest directory. The
merger also accepts prior working-directory-relative artifact paths and
absolute artifact paths.

The checked-in
`porting/api-inventory/occt-transitive-context-plan.json` records 21 contexts
found by following actual OCCT header include chains: sixteen prior successful
probes and five prior BVH failures whose artifacts did see the target header.
The Windows-only `WNT_Dword.hxx` attempt is omitted because the old artifact did
not see that guarded header. The plan contains relative header/TU paths and
input hashes, not machine-specific absolute paths. Create or validate a plan
from the original context evidence:

```sh
python3 tools/replay_occt_header_context.py plan \
  --retries /path/to/context-retry-manifest.json \
  --capture target/api-proof/occt-inventory/capture-manifest.json \
  --source /path/to/OCCT \
  --database /path/to/configured/compile_commands.json \
  --extractor target/api_inventory/occt-api-extract \
  --output porting/api-inventory/occt-transitive-context-plan.json
```

Replay planned contexts against a capture and extractor whose hashes match.
Headers already parsed in that standalone capture are verified by artifact
hash and target-header observation, then recorded separately as
`already_parsed`; the remaining contexts are rerun into a fresh output
directory. Old failure status stays as provenance; only a new run's success
and target-header observation can mark a retry passed. Existing evidence is
never overwritten:

```sh
python3 tools/replay_occt_header_context.py run \
  --plan porting/api-inventory/occt-transitive-context-plan.json \
  --capture target/api-proof/new-occt-inventory/capture-manifest.json \
  --source /path/to/OCCT \
  --database /path/to/configured/compile_commands.json \
  --extractor target/api_inventory/occt-api-extract \
  --output target/api-proof/context-replay
```

The v2 capture parsed five of the planned BVH headers standalone; the other 16
contexts reran successfully with the v2 extractor. The replay manifest records
the five standalone results separately, so they are not submitted as retries
against already parsed capture rows.

`gap_audit.py` records unresolved platform/configuration gaps and validates the
retry artifacts without changing original capture evidence. Its report keeps
direct and transitive retry counts separate and always sets `coverage_proof`
to false:

```sh
PYTHONPATH=. python3 -m tools.api_inventory.gap_audit \
  --capture target/api-proof/occt-inventory/capture-manifest.json \
  --retries /path/to/context-retry-manifest.json \
  --merged-manifest porting/api-inventory/occt-manifest.json \
  --source /path/to/OCCT \
  --output porting/api-inventory/occt-capture-gap-audit.json
```
