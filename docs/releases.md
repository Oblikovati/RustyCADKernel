# Martensite packages for Oblikovati

The kernel repository tests and releases its own packages. The application
consumes a stable release and runs application integration tests. It must not
build a release from an uncommitted sibling checkout.

## Consumer contract

For workspace version `X.Y.Z`, publish one stable GitHub release named `vX.Y.Z`
at the tested commit. Keep the tag and release assets immutable. Include:

- `NAME-X.Y.Z.crate` for every workspace package, including all transitive
  `rustycad-*` and `martensite-*` packages. The current workspace has 31 packages.
- `rust-toolchain.toml`, the toolchain used outside the source checkout.
- `LICENSE`, the kernel license text.
- `Cargo.lock`, the owner lockfile used by the package verification workspace.
- `martensite-test-support.tar.gz`, the shared oracle files and parity mapping
  used by the package tests. Tests retain their original relative paths.
- `martensite-release.json`, with version, commit, package hashes, and validation.
- `SHA256SUMS`, including the release manifest and each delivered artifact.

Oblikovati's `release.toml` records repository `Oblikovati/RustyCADKernel`,
version `X.Y.Z`, and the full tested commit SHA. Its Cargo dependencies use that
Git revision. Cargo resolves the kernel's internal path dependencies inside
that fetched revision. All internal dependency versions must match the workspace
release. GitHub assets provide the tested source packages; they are not a Cargo
registry. This process does not publish to crates.io.

## Owner release check

On a clean checkout at the release tag, use the pinned Rust toolchain and run:

```sh
python3 tools/release_packages.py --tag v0.1.1
```

The command derives the package inventory from Cargo metadata. Cargo creates
normalized archives. The command extracts them into a new temporary workspace,
restores the packaged test support, adds local registry patches for the full
package set, and runs all-target tests plus doctests with the owner lockfile
in both default and all-feature configurations. No original crate path is used
by that test workspace. Output appears
under `target/releases/vX.Y.Z` only after those tests pass.

Cargo packaging uses `--no-verify --exclude-lockfile` because the kernel's
sibling packages are not on crates.io. This does not omit verification: the
separate extracted-workspace test is mandatory and uses the delivered owner
lockfile. Test workspaces omit debug symbols to fit hosted-runner disk limits;
debug assertions and optimization levels are unchanged. Incremental caches are
disabled, and each feature profile clears only its private target directory. An
individual crate archive is not a standalone installation of the
full kernel or its oracle corpus.

Run tool tests with:

```sh
python3 tools/run_tool_tests.py --jobs 2 --pattern 'test_release_*.py'
```

Python 3.12 or later is required. Linux native prerequisites include pkg-config,
Tcl/Tk 8.6 development files, FreeType, Fontconfig, X11, and the Vulkan loader
and
software driver. The workflow installs them before the archive tests.

## Publication

`.github/workflows/release.yml` runs on `v*` tags or manual dispatch on a tag.
It requires the complete reusable kernel CI workflow first, then packages and
tests the extracted files. Only the final publication job has write permission.
It verifies checksums, creates a draft with all assets, and publishes the draft
as a stable release. An existing release causes an error; assets are not
replaced.
A failed native, oracle, package, or documentation check prevents publication.

Creating these tools does not certify all kernel capabilities or publish a
release. The application pin remains empty until the owner release passes.
Track this work in
[issue #3](https://github.com/Oblikovati/RustyCADKernel/issues/3).

## Hosted runner prerequisites

The formatting check uses `tools/check_rust_format.py` to check every workspace
target with edition-specific, bounded rustfmt commands. Windows could not launch
the original aggregate formatter command at `v0.1.0` (`os error 206`); batching
preserves the full check while staying below the process argument limit.
The Vulkan runtime and performance checks run under Xvfb with the lavapipe ICD,
so native-window tests have an actual X server even on headless hosted runners.
Neither workaround skips a test or relaxes a renderer comparison.

The Linux workspace and coverage jobs install the Tcl/Tk and visualization
development libraries explicitly. macOS uses Homebrew `tcl-tk@8` with local
versioned pkg-config aliases, alongside FreeType and Fontconfig. This repairs the
missing `tcl8.6` prerequisite observed in Dependabot run `38073783006`; it does not
accept or merge the dependency update. Hosted verification of these fixes is pending.

The oracle tool tests now use portable bounded evidence. The private-return
receipt v2 replays the same compiler captures and an exact, hash-checked copy of
the pinned upstream header, so CI does not require the capture host's absolute
checkout path. The OSD interface receipt v2 uses the verified current TKernel
rustdoc capture after an unrelated re-export changed its containing source file.
The v1 receipts and all identity, visibility, and source-hash checks remain intact;
these updates grant no additional API coverage.

X11 window APIs, examples, and their tests are Linux-specific, matching the
existing Xlib dependency and display implementation. Other window and offscreen
checks remain enabled on macOS and Windows. The matrix completes every platform
even when another fails, so a macOS failure no longer cancels Windows diagnostics.

Windows Tcl/Tk prerequisites are built with MSVC from the pinned upstream 8.6.18
commits in the workflow. The installer supplies matching import-library metadata
and runtime DLL/library paths. The Tcl build script preserves native path-list
separators, and header checks use Clang on Windows while retaining the positive
and negative ABI checks. Tcl header tests pass locally; hosted Windows linking
and execution remain a required release gate.

The Windows cross-check also exposed C `long`/wide-character assumptions:
FreeType point sizes now reject overflow before conversion, glyph comparisons
preserve the native width, and Vulkan rejects an XID that cannot fit its ABI.
Test fixtures encode Windows wide characters as UTF-16 and apply their existing
platform guards consistently. No renderer parity threshold was relaxed.


Workspace tests and coverage are split into six deterministic package groups on
hosted runners. Run `python3 tools/ci_test_shards.py --shard 0 --count 6 --runner
test` to reproduce one group (indices 0–5). Cargo metadata discovers every member;
target counts balance the groups without excluding any crate, feature, or doctest.
Formatting, full-workspace Clippy, and documentation checks run once per platform.
Run `cargo test-all` locally for the original whole-workspace invocation.

Run `38075760896` exhausted Ubuntu disk while linking the workspace tests; its
coverage runner also reached only 5 MB free. Debug symbols and incremental builds
were already disabled. Each coverage group now deletes its ordinary build before
instrumentation, retaining nextest XML and uploading its LCOV report as
`coverage-and-tests-<index>`. All six reports constitute the workspace coverage
set; no individual group's percentage represents the entire workspace.

Extracted release archives are checked one package at a time, with both default
and all features, including all targets and doctests. The temporary build is
deleted after each package/profile pair, and release builds use the same disabled
debug-symbol/incremental settings as CI. Package contents and test coverage are
unchanged.

## Platform geometry fixtures

The torus Boolean and cylinder seam tests compare exact native bits on the tested
host. macOS ARM64 uses its own independently captured coordinates and parameters;
its libm results differ from the Linux captures. The canonical native comparison
uses `-ffp-contract=off` with Clang/GCC and `/fp:precise` with MSVC, matching Rust's
separate multiply and add operations. This is a compiler policy, not a numerical
tolerance: every recorded numeric and topology field remains exact.

Run `38119937683` proves all 56 macOS torus rows agree under that policy. The
unmodified default Clang contraction capture in `38118858849` differs on nine
numeric rows and remains recorded; bitwise agreement with that compiler policy
is unproven. Both macOS cylinder captures produce node hash `5c0ce88cea863cc7`,
matching Rust. Raw captures, compiler configuration, hashes, and original failing
test results are retained in
`porting/runtime-evidence/release-ci-2026-10-11/`.

Run `38120792912` independently verifies all 56 Windows x64 torus rows and the
cylinder hash `8051bf23619e245b` against pinned OCCT with MSVC `/fp:precise`.
The Windows torus fixture keeps native CRT CRLF bytes; its raw SHA-256 is
`308525d257049fe0d6f9680232b3d10ca16d12725c28916b0306fce23dcb343d`.
Original Linux-fixture failures are retained alongside both native captures.

Locale tests preserve host libc behavior: Darwin's C locale accepts bytes through
`0xff`, while the Linux C locale rejects non-ASCII bytes. Unrepresentable-character
regressions use `€` so both hosts exercise the native conversion error. DRAW output
comparisons reverse Tcl console transport once and retain the literal CRLF in
Win32 loader messages. Neither case changes the production locale or loader API.

## Regenerating host oracle assets

During porting, generate host fixtures with the manual
`native-platform-oracle.yml` workflow.
It builds pinned OCCT on each requested runner, captures the C++ probe output
directly, and uploads `geometry-oracle-<runner>` artifacts with source revisions,
compiler configuration, raw-output hashes, and separate Martensite test logs.
Native output supplies the expected files; Martensite output never supplies them.

For example:

```sh
gh workflow run native-platform-oracle.yml --ref main \
  -f platforms='["macos-latest","windows-latest"]' -f fp_contract=off -f capture=geometry
gh run download RUN_ID --name geometry-oracle-macos-latest --dir target/oracle-import/macos
gh run download RUN_ID --name geometry-oracle-windows-latest --dir target/oracle-import/windows
```

After checking the receipt hashes and comparing native and Martensite results,
commit `platform-geometry/<probe>.expected` under
`crates/tkbo/tests/fixtures/tkbo_<probe>.<host>.expected`. The cylinder capture becomes
`crates/tkmesh/tests/fixtures/tkmesh_cylinder_nodes.<host>.expected`. Keep the original native
bytes, including Windows CRLF, and retain the receipt and diagnostic logs.
Integrate the assets into `main` with their test changes. Normal CI reads these
committed assets; it does not regenerate expectations from the code under test.
During porting, regenerate when the pinned upstream revision, probe inputs, or
compiler policy changes. This workflow and its receipt-integrity tool tests are
temporary porting infrastructure. The Rust behavioral tests and static assets
remain when the OCCT harnesses are removed; subsequent changes follow
[Martensite's independent test policy](porting-plan.md#independent-rust-tests-and-oracle-retirement).

Run `38123916241` confirms all four Boolean snapshots (2,176 rows per host) and
all three cylinder-face node and triangle hashes independently agree on macOS
ARM64 and Windows x64. Those native captures now supply the committed fixtures.
The original failures against Linux fixtures remain in the evidence directory.
Tests still compare every logical row and its count, or every node/triangle hash,
exactly. These baselines are owned by the Rust test crates and included directly
in their Cargo archives. Removing the OCCT generators and porting receipts does
not remove these behavior tests or their fixtures. Legacy test resources still
enter the extracted-package test-support bundle until their ownership is migrated.
