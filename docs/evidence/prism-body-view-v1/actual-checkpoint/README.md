# Actual portable body projection checkpoint

Source: `272aa25b46ffd0e04312e706cce2f72e6cdcb252` on
`kernel/prism-body-view`, 2026-10-14. This private ADR0047 read projection consumes
the actual sealed resolver/publication/map fixture from `6ed91a0be` (source
`a3b345b7e`). Production body projection remains the previously reviewed
`41b0250ed` implementation. This checkpoint adds actual-operation acceptance and
ownership tests; it does not issue names, validate another body, or expose a public
application outcome.

## Frozen verification

Rust/Cargo 1.99.0, x86_64 Linux; exact compiler and kernel information is in
`platform.log.gz`. `CARGO_BUILD_JOBS=2`, isolated target
`/tmp/martensite-body-view-target`. All 49,925 tracked input files have identical
SHA-256 bookends. Evidence files in this directory were added after the run.

Executed successfully:

- TKBO all-target/all-feature tests: **1,755** (1,107 library, 646 integration,
  two example tests), including all **36** body-view tests.
- TKBO all-feature doctests: **35**.
- TKBO docs with `RUSTDOCFLAGS=-D warnings`.
- TKBO all-target/all-feature Clippy with `-D warnings`.
- Workspace `cargo fmt --all -- --check`.
- Eight pairs of independent process payloads, compared byte for byte.
- Independent operations on four worker threads and immutable shared readers,
  exercised by the body corpus; existing toolkit concurrency tests also ran.

Reproduce the matrix from this source with:

```sh
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR=/tmp/martensite-body-view-target
cargo +1.99.0 test -p rustycad-tkbo --all-targets --all-features
cargo +1.99.0 test -p rustycad-tkbo --doc --all-features
RUSTDOCFLAGS='-D warnings' cargo +1.99.0 doc -p rustycad-tkbo --no-deps --all-features
cargo +1.99.0 clippy -p rustycad-tkbo --all-targets --all-features -- -D warnings
cargo +1.99.0 fmt --all -- --check
```

`receipt.json` records exact process selectors and commands. To emit the complete
body bytes, run the exact `actual_body_process_receipt` test with
`MARTENSITE_ACTUAL_BODY_RECEIPT` set to a distinct output file in each process.
Original orchestration scripts are retained as `.py.txt`, including their
corrective continuation. The first joined-measure selector matched zero tests;
the harness rejected this, then the exact selectors from the test inventory ran
successfully in two processes. That zero-test log remains explicit. The initial
development Clippy failure (incorrect test-only member access) is also retained;
the corrected source passes the frozen strict gate.

## Independent expectations and isolation

The actual result contains 13 member definitions, 26 directed uses, three faces,
three wires, three edges, two physical vertices, six trims, six coedges, 12 local
vertex contexts, and eight stored-tolerance observations. The seam has two uses;
each closed cap rim retains separate native endpoint roles even though the
physical vertex is shared. Wire members retain `output_slot=None` and full
semantic ancestry. Caller namespaces and stored/traversal reversals are exercised.

Every captured curve/surface/trim/tolerance operand is compared with the projected
encoding. Fresh snapshots reconstruct every owned row through read methods.
Request mutation and disposal of the publication/map leave these reads unchanged;
test-only mutation of an owned body row changes its recomputed replay. Separate
curve allocations are observed directly without using addresses as identity.
The resolver fixture has already dropped live construction owners before body
projection; earlier publication tests remain the live-owner mutation gate.

Two independent processes produced these identical payloads:

| Payload | Bytes |
| --- | ---: |
| Actual complete named body | 2,260,259 |
| Historical symbolic body rows | 11,672 |
| Historical publication V1 | 2,202,990 |
| Historical symbolic semantic map | 10,571 |
| Historical frozen V1 | 89,921 |
| Ordered validation | 1,394,630 |
| Joined measures | 193,495 |
| Complete-measure scalar receipt | 206 |

The complete-body SHA-256 is
`6a114275c5c7feacd3342f1d1fe34fb4f03a974a10b707303002fe863297f0fd`.
All other digests and both complete payloads are retained. Publication, symbolic
body and symbolic-map hashes were also checked against their earlier baselines;
no baseline was replaced. The 206-byte scalar receipt is explicitly narrower than
the complete joined-measure proof.

## Retained storage and cost observations

The fixed actual workload retained **109** observable nonempty vector allocations
with **2,270,947 bytes** of vector capacity, including the **2,240,803-byte** full
semantic-map/publication binding. Copied curve scalars occupy **800 bytes**. The
dictionary entry payload is another **4,848 bytes**; its private capacity is not
observable and is excluded from the vector allocation/capacity totals. The
historical 2,202,990-byte publication receipt is a different fixture and is not
added again to the actual binding. No optional reservation receipt is included.

Thirty-two repeated body projections took **7,632,463 ns** total (about 0.239 ms
each) in this unoptimized test build. The entire observation process, including
actual fixture construction, peaked at **87,644 KiB RSS**. Full `/usr/bin/time -v`
and JSON observations are retained. These measure retained storage and observed
latency/RSS, not total transient allocator calls, lock costs, or a previous-release
whole-operation benchmark. No storage redesign is claimed.

## Acceptance limits

This completes the actual-body private read-projection gate for one admitted
single polynomial Jordan prism. It is not Kernel #10 release completion, public
semantic-version acceptance, annular/general-profile support, persistent edit
naming, mesh/application integration, or an owned public diagnostic envelope.
No new OCCT execution or numerical oracle is involved. Exact copied operands,
independent combinatorial expectations, reviewed owned baselines, and mutation
invariants are the evidence kinds.

No new ThreadSanitizer run is claimed. The previously recorded host startup
limitation remains unresolved; ordinary `Send`/`Sync` assertions and threaded
replay do not complete sanitizer acceptance. Release-scale concurrency stress,
an accepted-workload previous-release performance comparison, and the broader
public outcome gate remain separate obligations.
