# Frozen prism topology v1 / ordered capture V2 evidence

## Provenance and scope

Base `1978079a46f48a948d5185d7228c0af12df24c8f`; baseline fixture commit
`1eff9bed1`; behavior-preserving shared graph extraction `a1e4bb957`; final
source revision `77978c638`. This is the private exact combinatorial prerequisite
for RustyCADKernel#10, specified by [ADR0033](../../adr/0033-private-frozen-prism-topology.md).
No valid body, geometric outwardness, embedding, application or release
acceptance is claimed. No live OCCT, GPU, native desktop, tessellation oracle,
new graph owner or geometry evaluator was used.

`replay.py` is the reusable gate, invoked from a checkout with Rust 1.99 installed:

```sh
python3 docs/evidence/frozen-prism-topology-v1-2026-10-14/replay.py target/prism-topology-v1/final
```

It runs all TKBO targets/features, doctests, documentation with warnings denied,
Clippy with warnings denied, package formatting, two independent process replays
(each also checks eight ordinary host threads), and the legacy frozen replay.
All commands use jobs2. Source bookends cover tracked files outside this evidence
directory; final command/results and exact replay hashes are in `summary.json`.
Raw command logs and source snapshots are retained in `candidate-raw-evidence.tar.gz` (original frozen run) and the final reconciled archive.

The original frozen run passed 591 library + 646 toolkit + 2 example tests,
35 doctests, docs/Clippy/fmt and both process replay checks. Its 96,088-byte
certificate SHA-256 is `852bd440732f89512ab35e27c2f13b70d00b9af818c3dc3266122b3bee75a9a7`.
The later failed-input isolation assertion and main reconciliation are recorded
separately; the original receipt is not relabeled.

## Independent expectations and retained red evidence

The owned actual cubic extrusion has V2/E3/coedges6/wires3/faces3/shell1/solid1:
one coedge per loop cap, and two uses of the same side seam in one face. The
baseline pins those counts and opposite cumulative edge orientations. It does
not label these observations an existing valid-solid proof.

The first uncommitted certificate prototype wrongly assumed containment order
was traversal order. Its actual-prism tests failed at a cyclic corner (17 passed,
7 failed); a minimized probe retained the exact side insertion order and endpoints:

| Stored use | Local direction | Start role | End role |
|---|---|---|---|
| Generated seam | Forward | First | Last |
| Generated seam | Reversed | Last | First |
| First loop | Reversed | First | First |
| Last loop | Forward | Last | Last |

The graph already retained a complete traversal but V1 discarded that sequence
when sorting coedges by stable identity. The approved V2 preserves it separately,
without rewriting source containment or adding an ordering algorithm. The
certificate requires a complete bijection to original uses and independently
checks all cyclic corners; missing, duplicate, foreign and wrong-order rows fail.
V1 missing explicit traversal is incomplete. No original failure was relabeled
as a passing old baseline; the prototype is diagnostic evidence without a frozen
source revision claim. Named regression tests pin both the old insertion table
and the new actual graph traversal.

Independent finite cell-word fixtures supply a tetrahedral sphere, two one-edge
disks, pinched/disconnected complexes and a torus. Expected closure, opposite
use multiplicity, vertex-link connectivity and Euler characteristic come from
those authored identities and words, not sampled geometry. All 16 tetrahedron
face-orientation masks have exactly two consistently oriented outcomes. Whole
shell reversal passes; one-face reversal fails. Failure/cancellation never returns
a partial certificate. Every observed checkpoint is exercised for cancellation
and callback error, including commit; callback source containment mutation also
refuses V2 capture. Owned input isolation and exact source binding are exercised.

## Legacy replay and exclusions

The 89,921-byte V1 capture SHA-256 remains
`88e28942e1fc6ad301a90f5f07624962d842f0262bde1377103cd3470a871fb4`.
Pre/post structural logs compare byte for byte; final replay checks the same
owned bytes. V1's existing population status gate already refuses partial explorer
or tolerance ordering (`SuccessWithWarnings`); a V1-positive/partial-order case
was not fabricated. V2-only traversal extraction adds no new V1 gate.

Development compiler/lint failures and the initial non-rustup cargo invocation
are retained alongside successful reruns, with no acceptance attributed to those
failures. The initial tool selector was corrected to explicit toolchain PATH;
typed callback failure storage was boxed after Clippy's large-error diagnostic.

[ThreadSanitizer](TSAN.md) remains incomplete: a fresh instrumented startup control
crashed before main; the actual topology sanitizer suite was not built/executed.
No unsafe Send/Sync implementation or sanitizer bypass is present. Ordinary
threaded certificate replay is limited to immutable frozen inputs and operation-local
progress; it does not certify concurrent mutation of native geometry aliases.

Host load was shared and high. Command wall times are diagnostic only. No prior
release implements this exact accepted certificate workload, so no identical
accepted-workload release performance comparison is available. This evidence
makes no performance acceptance claim and does not authorize publication.
