# Same-body prism proof: main integration

Integrated revision: `31fa1b8b2bba48c49929efeea7497ff285b8e0f3`.
Reviewed source: `93fac56f229fc299080f38581bdf3d5a82acbcfd`; branch evidence tip `c6aaacad3`.

- Full TKBO: **1,384 tests and 35 documentation tests passed**, none failed or ignored.
- Strict all-target/all-feature Clippy, formatting and warning-denied documentation passed.
- All 33 changed crate files exactly match the reviewed source.
- Every tracked crate/configuration hash stayed unchanged through the matrix, replay and documentation runs.
- All 1384 recorded candidate test names remain present in the integrated passing run.
- Verified 84 sealed candidate artifacts and both original/structural source bookends.

## Independent process replay

Two fresh processes per row matched each other and the frozen candidate exactly.

| Complete payload | Bytes | SHA-256 |
|---|---:|---|
| join | 193080 | `de1f88a80716a820eddc03254425f0400b5743e425126e3d43bf843ae2908871` |
| frozen_v1 | 89921 | `88e28942e1fc6ad301a90f5f07624962d842f0262bde1377103cd3470a871fb4` |
| complete_corrected | 207 | `1943b5f23e744fc9c2c4b90c379f5422e7f927eea92912dd2056f65529d7f6e1` |
| cap_subset | 319 | `ad4b31d4e8075751c5f0bdd4bda83d859c7f7545a8c0d4848206c947dd56f9a3` |

## Scope and remaining gates

This integrates a reusable, immutable same-body proof binding. It preserves
historical diagnostic completion/validation states. It does not publish a valid
body, complete source history, general profiles, application acceptance or a
release. Joined orientation, ordered validation, exact conformance/tolerance,
complete history and source-bound measures are separate unfinished work.

The affected changes are in TKBO; unchanged dependency gates remain historical
evidence. TSan acceptance remains incomplete after the previously recorded host
startup failure; this integration makes no sanitizer or full API/test parity claim.

Commands, complete compressed outputs, source hashes, artifact verification and
all eight process payloads are retained beside this note. No OCCT oracle or
sanitizer workaround was executed.
