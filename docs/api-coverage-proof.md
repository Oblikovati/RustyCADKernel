# OCCT API coverage proof

`porting/parity.json` is a manually curated mapping ledger. Its 27,139 API rows are
useful work records, but they are not an authoritative upstream symbol inventory.
`api_inventory_complete` must remain `false` until an independent inventory is
generated from both projects and the comparison gate passes.

## Current compiler inventory

The configured scan observes **110,270 public/protected C++ declaration
candidates**, plus 6,526 macro candidates tracked separately. It accounts for
all 6,316 source-header paths and the generated version header:

| Header disposition | Count |
| --- | ---: |
| Standalone parse succeeded | 5,633 |
| Parsed in a real OCCT translation-unit context | 260 |
| Included fragments observed | 384 |
| Parsing/context gaps | 12 |
| No configured owning toolkit | 28 |

All 384 `.lxx` fragments are observed. Context successes retain their original
standalone failure, source translation unit, flags, and hashes. Six toolkit
configurations remain absent; OpenGL/GLES share physical headers, so per-toolkit
ownership totals overlap and do not prove the absent GLES configuration.
There are 2,651 conflicting-declaration observations and 1,094 skipped preprocessor
ranges retained for review. The observed declaration count is a lower bound on
the cross-platform surface, not a denominator for a full-port percentage.

The legacy-test snapshot separately accounts for 8,648 GoogleTest declarations:
7,393 exact Rust mappings and 1,255 unmapped, with zero unknown or duplicate
mappings. Its runtime status remains unverified until verifier evidence is
joined; DRAW candidates are also explicit unverified obligations.

See the [snapshot summary](../porting/api-inventory/completeness.json) for
compiler-checked mapping counts, per-toolkit gaps, input hashes, and the exact
Rust capture provenance. Test-declaration mappings remain a separate ledger;
API correspondence does not establish matching runtime behavior.

## Source boundary prototype

`tools/extract_occt_public_headers.py` walks the pinned OCCT CMake graph from
`src/MODULES.cmake` through each module's `TOOLKITS.cmake`, then each toolkit's
`PACKAGES.cmake`, package `FILES.cmake`, and toolkit-root `FILES.cmake`. It follows
the installed-header suffix filter in `COLLECT_AND_INSTALL_OCCT_HEADER_FILES`,
records source hashes and all owners of shared packages, and records
`Standard_Version.hxx` through its template source. Its checked-in output is
`porting/api-header-manifest.json`.

The manifest intentionally says `declaration_inventory_complete: false` and
`coverage_proof: false`. A header list does not enumerate declarations. Reproduce
or check it with:

```sh
python3 tools/extract_occt_public_headers.py --source ../OCCT
python3 tools/extract_occt_public_headers.py --source ../OCCT --check
```

The extractor requires the baseline OCCT revision and a clean checkout. The
generated JSON preserves the full toolkit/package/source ownership and records
`EXTERNLIB.cmake` names. External-library declarations are outside OCCT's source
boundary; OCCT-owned wrappers around C libraries remain in scope.

This is the all-module source candidate set, independent of which optional
modules a particular host can configure. The generated version header is
represented by its template and generator, not by a configured output hash. Both
facts must be included in the eventual compiler inventory configuration.

## Required declaration inventory

The proof gate still needs two independent compiler inventories:

1. **OCCT C++:** run a pinned Clang AST frontend over every installed header from
   the manifest. Use the fully configured all-module OCCT include paths, defines,
   generated headers, and external development headers. Emit one row per explicit
   public declaration, including public types, aliases, enum values, data fields,
   functions, constructors, operators, and templates. Each row needs a stable
   qualified identity and canonical signature, kind, owning toolkit, source path,
   source location, source hash, and compiler/configuration fingerprint. Filter
   declarations by OCCT header source location, not by toolkit dependency names.
2. **Rust:** extract each workspace crate's reachable public items from rustdoc
   JSON with a pinned nightly compiler. Include public re-exports and public
   members, but exclude private and `doc(hidden)` items only under an explicit
   versioned policy. Record crate, public path, item kind, signature, and source
   location/hash.

Join the inventories to `parity.json` by upstream declaration identity. The gate
must fail on an unparsed header, a Clang diagnostic, unsupported declaration
kind, duplicate/ambiguous identity, missing Rust mapping, stale mapping, or
unjustified exclusion. Allowed exclusions need a reason code and source evidence;
`EXTERNLIB` alone is not a reason to exclude OCCT declarations. No declarations
may be silently dropped because they are templates, conditional, deprecated,
generated by an OCCT macro, or difficult to map.

## Local proof-tool bootstrap

`tools/bootstrap_api_proof.sh` pins LLVM 23.1.2 and `nightly-2026-09-25` in
`porting/api-proof-toolchain.json`. The script places the LLVM archive and
unpacked binaries in the user's cache, installs the Rust nightly through
rustup, and configures OCCT at the pinned source revision with all seven
manifest modules enabled. The result is written under `target/api-proof/` and
includes an all-module `compile_commands.json` plus a configuration record
binding it to the OCCT revision and toolchain configuration.

Run the steps independently to inspect setup cost and output:

```sh
tools/bootstrap_api_proof.sh install
tools/bootstrap_api_proof.sh configure
tools/bootstrap_api_proof.sh verify
tools/probe_occt_header_ast.py \
  --source ../OCCT \
  --compile-commands target/api-proof/occt-build/compile_commands.json \
  --header src/FoundationClasses/TKernel/Standard/Standard_Real.hxx \
  --output target/api-proof/probes/Standard_Real.ast.json
tools/capture_rustdoc_json.py capture
tools/capture_rustdoc_json.py verify
```

The current compiler pin targets Linux x86_64. Its official zstd release
archive is 1,183,587,483 bytes compressed; the extracted LLVM tree occupies
about 12.4 GB in this environment. The pinned nightly installation occupies
about 601 MB.
CMake configuration may also require platform development headers for optional
OCCT features. This environment currently configures 5,692 translation units
covering 67 of 75 manifest toolkits. CMake omits `TKIVtk` and `TKIVtkDraw`
because VTK is unavailable, and `TKDEGLTF` and `TKXSDRAWGLTF` because RapidJSON
is unavailable. `TKD3DHost`, `TKD3DHostTest`, `TKOpenGles`, and
`TKOpenGlesTest` are absent from the Linux configuration due to platform-specific
toolkit availability. `verify` fails while any manifest toolkit lacks compile
commands. These absences are inventory failures, not API exclusions.

The bootstrap only creates compiler flags and configuration evidence; the
single-header probe below captures raw AST output but cannot form a complete
declaration inventory. The proof still needs declaration normalization across
all 6,316 headers, rustdoc item normalization for every crate, and an
ambiguity-free crosswalk to the API mapping ledger before any completeness flag
may change.
On this host, parsing `Standard_Real.hxx` completed without Clang diagnostics
and produced a 37.7 MB raw AST with 703,520 nodes (45,312 `*Decl` nodes). Those
counts include transitive headers and are only a toolchain smoke measurement.
`tools/capture_rustdoc_json.py` enumerates workspace packages through Cargo
metadata and captures one JSON artifact for each library target. Cargo's
`rustdoc` command does not offer workspace selection, so it invokes
`cargo rustdoc --package ... --lib` once per crate with `--all-features`. The capture
records package source and artifact hashes, Cargo metadata/lockfile, Rust
toolchain versions, active build flags, and configuration file fingerprints.
It verifies that every workspace package has exactly one library target and
that every JSON artifact has a valid root, index, and pinned format version 61.
In this workspace it
captured all 17 library crates with 96,272 combined indexed items.

Those JSON files are raw compiler outputs, not a normalized public API
inventory. The capture does not yet filter private or `doc(hidden)` items,
normalize signatures, account for each independent feature combination or
target platform, or crosswalk Rust items to OCCT declarations. Its manifest
keeps `normalized_public_inventory` false.

The Rust normalization stage is documented in
[`rust-api-inventory.md`](rust-api-inventory.md). It expands public re-exports
across the matching captured crates and keeps `doc(hidden)` public items.
The current compressed snapshot and its capture revision are published under
[`porting/api-inventory`](../porting/api-inventory/README.md), alongside the
C++ declarations, crosswalk, hashes, and unresolved obligations. Earlier raw
rustdoc and smoke AST counts above are bootstrap measurements, not current
completeness figures.

## Completeness limitations

The bootstrap currently provides Clang and a pinned nightly rustdoc compiler,
but its Linux CMake configuration does not cover every manifest toolkit. Compiler inventories and a conservative crosswalk now exist, with explicit
configuration, parsing, identity, and mapping gaps. They cannot currently
validate 100% API coverage. The existing
`tools/audit_api_rows.py` is explicitly heuristic and cannot substitute for the
compiler-backed stage.

The C++ declaration universe can vary with platform macros and optional external
headers, so a Linux compile configuration alone cannot prove cross-platform API
completeness. The manifest also records a generated-header template rather than
the generated file; its configured content and the exact CMake options must be
captured alongside any AST output.

Before enabling the proof gate, configure OCCT with every in-scope module/toolkit
and generated header, regenerate and normalize rustdoc JSON for the current
workspace, validate the schema against representative templates, macros,
overloads, and re-exports, then run the extractor over all 6,316 installed
source headers and account for every declaration kind. The gate must
also bind output to the OCCT SHA, header hashes, tool versions, CMake config, and
Rust crate sources so source or configuration changes invalidate the result.

## Conservative declaration crosswalk

`tools/api_coverage_report.py` joins the C++ declaration stream, normalized Rust
public paths, and `parity.json`. It resolves a row only when the parsed upstream
qualified name and parameter types select exactly one C++ declaration, and its
Rust target path selects exactly one public Rust item. Bare names are accepted
only when they identify one declaration in the row's toolkit. No substring,
similarity, or guessed overload matches are used.

The report distinguishes implemented, excluded, partial, and upstream-unimplemented
rows from missing or ambiguous C++/Rust targets. It lists unclaimed C++ API
declarations separately and carries per-toolkit totals plus all crosswalk rows.
Any missing header/configuration, unsupported inventory item, unresolved row,
duplicate claim, partial mapping, or unclaimed declaration keeps the declaration
crosswalk proof false. The report always sets `behavioral_equivalence_proven` to
false: declaration identity does not establish 1:1 behavior or test parity.
The report also treats Clang `unsupported_records` and rustdoc
`unclassified_kinds.counts` as explicit proof gaps, alongside unresolved links.

With the generated artifacts in place, run:

```sh
python3 tools/api_coverage_report.py \
  --cpp porting/api-inventory/occt-declarations.jsonl.gz \
  --cpp-manifest porting/api-inventory/occt-manifest.json \
  --rust target/api-proof/rust-api-inventory.json \
  --output target/api-proof/api-coverage-report.json
```

The C++ declaration IDs are compiler-generated identities; the parity strings
remain manual aliases. A report row retains both IDs and the exact matched Rust
path so alias corrections can be reviewed. Macro candidates are not C++ API
declarations and are not counted as matches. If an excluded parity row has no
declaration candidate, it stays unresolved until the source can be identified
without treating a macro definition elsewhere as evidence for that invocation.
