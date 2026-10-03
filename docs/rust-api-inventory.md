# Rust public API inventory

`tools/normalize_rustdoc_inventory.py` turns the pinned format 61 rustdoc JSON
capture into a path based Rust declaration inventory. Run it from the workspace
root after a complete capture:

```sh
python3 tools/capture_rustdoc_json.py verify
python3 tools/normalize_rustdoc_inventory.py
```

The default output is `target/api-proof/rust-api-inventory.json`. Use
`--capture`, `--workspace`, and `--output` to select another capture, its exact
source checkout, and destination. The command verifies the capture completeness
markers and each artifact's recorded digest, byte size, rustdoc format version,
and indexed item count. It also recomputes each crate's source input digest;
normalization fails if those sources do not match the capture record.

The raw capture's `verify` command compares the exact recorded Git revision and
build environment as well as content hashes. Run it before committing a snapshot.
`porting.py status` instead checks the recorded source, ledger, artifact and tool
content hashes, so a later documentation-only commit does not invalidate those
content checks.

Each declaration has a stable Rust path, kind, normalized rustdoc signature,
source file and source hash, and a declaration hash. The inventory includes
reachable public modules and re-exports, nominal declarations, public struct
and union fields, enum variants and their fields, trait members, inherent
members, and non-synthetic trait implementation members. Resolved item IDs in
signatures become paths. Repeated paths are permitted for overload-like Rust
surfaces and are distinguished by signature, implementation context, and
declaration hash.

The versioned visibility policy includes every reachable public item present in
rustdoc JSON, including items marked `doc(hidden)`; the normalizer does not
silently exclude them. A glob re-export exposes members in the importing
namespace: `pub use hidden::*` does not expose a `hidden` module. Rustdoc
canonical defining paths are used inside signatures, but do not grant public
reachability through private modules. A checked-in nightly format-61 compiler
fixture covers nested and crate-root glob exports. Trait implementation selectors
use reachable trait names, including renamed and cross-crate re-exports; the
canonical compiler trait path stays in `impl_context`. Implementations whose
trait belongs to a captured crate but has no reachable export are withheld with
an explicit `unreachable-trait` gap. External uncaptured traits retain their
rustdoc identity and remain subject to the external dependency boundary. A
second compiler fixture covers private defining modules and public trait aliases.

Capture provenance includes the workspace revision, tool versions, input and
artifact hashes, selected feature policy, and each crate's feature selection.
The normalizer also requires the checked source tree to match every package
source digest recorded at capture time.

Macro bodies are represented by a digest and byte count. Unresolved item links
and unclassified raw rustdoc item kinds are reported per crate. Neither these
artifacts nor this normalizer establish Rust-to-OCCT API parity: the output sets
`cross_language_completeness_claim` to false, and the capture revision and tool
fingerprints remain attached for provenance.
