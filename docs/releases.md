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
python3 tools/release_packages.py --tag v0.1.0
```

The command derives the package inventory from Cargo metadata. Cargo creates
normalized archives. The command extracts them into a new temporary workspace,
restores the packaged test support, adds local registry patches for the full
package set, and runs all-target/all-feature tests plus doctests with the owner
lockfile. No original crate path is used by that test workspace. Output appears
under `target/releases/vX.Y.Z` only after those tests pass.

Cargo packaging uses `--no-verify --exclude-lockfile` because the kernel's
sibling packages are not on crates.io. This does not omit verification: the
separate extracted-workspace test is mandatory and uses the delivered owner
lockfile. An individual crate archive is not a standalone installation of the
full kernel or its oracle corpus.

Run tool tests with:

```sh
python3 tools/run_tool_tests.py --jobs 2 --pattern 'test_release_*.py'
```

Python 3.12 or later is required. Linux native prerequisites include pkg-config,
Tcl/Tk 8.6 development files, FreeType, Fontconfig, X11, and the Vulkan loader and
software driver. The workflow installs them before the archive tests.

## Publication

`.github/workflows/release.yml` runs on `v*` tags or manual dispatch on a tag.
It requires the complete reusable kernel CI workflow first, then packages and
tests the extracted files. Only the final publication job has write permission.
It verifies checksums, creates a draft with all assets, and publishes the draft
as a stable release. An existing release causes an error; assets are not replaced.
A failed native, oracle, package, or documentation check prevents publication.

Creating these tools does not certify all kernel capabilities or publish a
release. The application pin remains empty until the owner release passes.
Track this work in [issue #3](https://github.com/Oblikovati/RustyCADKernel/issues/3).
