# Integrated prism topology, orientation and point-query adapters

Tested main source: `d59cbce3cff316151890483233defb673269213e`.
The coordinator serialized these merges, then checked their combined source:

| Work | Tested candidate | Main integration |
|---|---|---|
| Ordered capture V2 and topology | `9262459aa` | `d59cbce3c` |
| Conditional face/trim orientation | `c23deb36f` | `fa178d8f5` |
| Structural raw-point query adapters | `ccfaad005` | ancestor of `d59cbce3c` |

The full affected TKBO all-target/all-feature suite passed **1,358 tests** and
**35 doctests**, with zero failures or ignored tests. Clippy, formatting and
documentation with warnings denied passed. Checks used Rust 1.99.0 and two build
jobs. All 27 topology, 15 orientation and 3 adapter candidate crate files match
exactly. Every tracked crate/configuration input hash remained unchanged across
the suite, ten replay processes and documentation check.

No original candidate TKBO test is missing. Relative to the reconciled topology
candidate, 24 tests were added (22 orientation and 2 query adapter tests).
Relative to the original orientation TKBO suite, 106 were added; exact names are
in `integrated-test-preservation.json`. Unchanged dependency suites are covered
by the earlier 6,138-test/302-doctest TKMath/TKBRep/TKBO integration at
`a3245eace`; this combined root run is explicitly TKBO-only.

## Independent checks and replay

Root verified all sealed payloads in the four original/reconciled candidate
archives: 70 + 32 orientation and 45 + 22 topology files. The two orientation
archive manifests are also protected by their outer SHA256SUMS. Root reran the
independent orientation algebra verifier: universal coefficient identities and
425,088 finite side/cap sign checks reproduce the retained receipt exactly.
The verifier and raw algebra tool requests/results are retained in
`integrated-orientation-algebra.tar.gz`.

Two independent root processes per receipt match their candidate bytes:

| Receipt | Bytes | Scope |
|---|---:|---|
| Topology | 96,088 | Complete certificate bound to its frozen projection |
| Orientation | 752 | Portable scalar conclusions; excludes operation-local identities |
| Embedding | 2,095 | Portable theorem conclusions; not a complete body serializer |
| Legacy frozen V1 | 89,921 | Full unchanged diagnostic projection |
| Copy lineage | 167,821 | Full unchanged owned copy/sweep observations |

All hashes, commands and logs are in `integrated-replay.json`. Candidate and root
suites also exercise independent thread replay. No replay record with only
scalar conclusions establishes persistent source identity or full publication.

## Meaning and remaining obligations

Topology verifies the closed combinatorial manifold conditions, treating each
one-wire face as an abstract 2-cell. The separate same-body geometry join must
supply actual cap Jordan disks and the side chart's embedding. Ordered capture
V2 retains graph wire traversal separately from containment insertion order;
V1 remains selectable and byte-identical. Orientation checks outward face and
induced trim directions only under the fresh embedding hypotheses. The point
adapters preserve exact source maps and raw local-point placement operands;
they do not implement the pending complete exact tolerance certificate.

ThreadSanitizer acceptance remains incomplete: the ordinary-host instrumented
startup controls/executable fail before test output, and the topology sanitizer
suite was not executed. No bypass was used. Full ordered body publication,
complete source history, general Kernel #10 profiles, persistent split identity,
application adoption and owner release/package gates remain open. No full
API/test parity or controlled performance acceptance is claimed.
