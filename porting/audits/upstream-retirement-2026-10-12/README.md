# Explicit upstream retirement scope

ADR0002 removes OCCT-marked deprecated/obsolete APIs from required new Martensite
porting. Existing Rust implementations remain intact. The catalog binds markers
and source hashes to exact declaration IDs in the existing compiler inventory.
No behavioral proof or class completion is inferred from a scope exclusion.

`catalog.json.gz` records 500 observed retired APIs: 371 implemented mappings
preserved, 37 pre-existing exclusions, 90 previously unclaimed declarations and
two partial mappings now excluded. The ledger retains the partial mappings'
implementation paths and original proof limitations.

`compiler-observations.jsonl.gz` contains successful supplemental Clang captures;
`receipt.json` records tool/input hashes, pinned upstream revision and every
header result. Metadata-only extraction deliberately does not claim primary API
coverage. Commands use the manifest source verification and header-job machinery
in `tools/api_inventory`, with `--retirement-markers-only -fparse-all-comments`.
The host GCC13 standard-library context is explicit in the capture; the two math
exception headers also require their real Standard_Handle declaration context.

148 of 149 candidate header probes parsed. RWGltf_GltfJsonParser still failed
because of its RapidJSON dependency. Its only retirement wording describes the
obsolete glTF1 format of `myIsGltf1`, and does not retire the parser API; no
exclusion is guessed from that failed probe. Source-site joins are preferred;
unique Clang USR/kind/signature/template identities also connect declarations
across their different redeclaration sites. No name-only matching is used.
No marked compiler identities remain unmatched. This is an evidence-backed initial catalog, not a
claim that every configuration or future upstream retirement has been captured.

`rejected-candidates.json.gz` retains comments about obsolete parameters, caches,
algorithms and external functionality. Those are not API retirement evidence.
Explicit compiler attributes, API status sentences, documented deprecation
annotations and retirement groups are covered by the Rust-independent Python
classifier tests and the Clang fixture tests.
