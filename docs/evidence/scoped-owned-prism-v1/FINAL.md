# Scoped owned polynomial prism composition — final candidate evidence

Source `47712cfc5960d4c7d7b65b6c81b61256fe044fef`; production composition
`fd80d59a82ed6761220c21c846f7d13d454d4f79`; separate structural preparation
extraction `35940247542cbdfd76a71cc69274ff506bc3b7e4`; base `4e75cb9da`.
The last source commit adds tests and test-only registrations. The initial actual
success and unchanged preparation baseline are retained in [CHECKPOINT.md](CHECKPOINT.md).

## Scope

One private operation now accepts an owned polynomial request, authors V2 semantic
names before construction, prepares actual geometry, resolves complete actual
correspondence, publishes through ordered validation and history, and returns the
same accepted publication with its portable body, reservation coverage and physical
volume. It uses existing geometry and proof engines. One outer callback sequence
covers the complete operation; child phase/ordinal and exact failures stay separate.

The higher-ranked finishing callback keeps original preparation, resolver and
publication owners alive until inspection finishes. Projection refusals retain the
accepted private publication as failure context; completed intermediate projection
children are not retained. This is an internal scoped operation, **not** the final
owned public error interface. General analytic profiles, holes, application C1
acceptance and the remaining public failure families remain separate release gates.

## Executed checks

All 1,957 affected TKBO tests and 35 doctests passed. Strict all-target/all-feature
Clippy, formatter and warning-denied rustdoc passed. All 12,997 tracked
Rust/TOML/lock/toolchain source hashes are identical before and after the gate.

The build, suite and tooling commands use `/home/vmiguel/.cargo/bin/rustup run 1.99.0 cargo` and
`CARGO_TARGET_DIR=target/complete-owned-build`. Compiler jobs are 2. The full test
command is `test -p rustycad-tkbo --all-targets --all-features -j2 -- --test-threads=2`.
Other commands are `test -p rustycad-tkbo --doc --all-features -j2`,
`clippy -p rustycad-tkbo --all-targets --all-features -j2 -- -D warnings`,
`fmt --all -- --check`, and
`doc -p rustycad-tkbo --all-features --no-deps -j2` with `RUSTDOCFLAGS=-D warnings`.
The archived context records Linux/toolchain and exact source hashes. Dedicated
replay runs invoke the frozen compiled unit-test executable with exact filters.

The exhaustive corpus runs the full operation afresh for each of 8,268 checkpoints,
with cancellation and native failure independently: 16,536 terminal paths, followed
by successful reuse. It checks each complete callback prefix, original local cause,
exact request and first-stop precedence. Additional tests force ordinal exhaustion,
resolver native failure with unchanged child cancellation, final Commit refusal,
empty/multiple refusal, eight pole/traversal/extrusion reversals, degenerates,
NaN/signed-zero policies, caller/revision isolation and source mutation.

Eight independent operation threads and eight shared immutable readers equal the
serial bytes. Ten additional exact-filter process executions produce five equal
pairs: the whole result, typed preparation V1, publication V1, publication V2 and
complete history V1. The complete result is 9,583,467 bytes, SHA-256
`09c3de2d8d60981dacc60240f9a7d945f833177b514f64ab495e5b45dfefd00e`.
The two complete callback traces also match. Process timings include test startup
and receipt I/O; they are not a previous-release performance benchmark.

## Independent expectations

[verify_envelope.py](verify_envelope.py) parses the complete framing without Rust
imports. It checks exact full-child cross-bindings, literal request poles, plane
and extrusion, caller dictionary bits and the outer trace distribution. For x=12t(1-t)^2 and y=12t²(1-t), Green integration gives
area 12/5; extrusion height 2 gives volume 24/5. Python rational arithmetic checks
both cap enclosures, volume enclosure, achieved error and unchanged cap policies.
The existing independent literal reservation checker verifies 101 reservations:
73 applied, 28 inapplicable, with 96 named-field applications. Geometry comprises
13 definitions, 26 directed uses, three faces, six trims and eight tolerance rows.
These scalar checks supplement the existing ordered validation, full trims and
history in the same accepted publication; volume alone is not acceptance.

Reproduction from extracted archive files:

```sh
python3 oracle/verify_envelope.py frozen/exact-complete-0.bin frozen/exact-trace-0.txt independent-envelope.json coverage-child.log
python3 oracle/verify_actual_reservations.py coverage-child.log independent-reservations.json
```

## Retained diagnostic attempts and exclusions

The expanded test fixture initially used wrong field/import names; the compiler
failure and correction are retained. A substring replay filter matched both V1 and
V2; it was replaced by a fully qualified `--exact` filter with no fixture changes.
The offline checker initially assumed the library max-curves default 64 rather
than the explicit owned fixture limit 1; the corrected expectation and original
failed checker are retained, with unchanged production and closed-form result.
A filename-only freeze probe initially omitted the test-only context registration;
its exact suffix check is recorded. The earlier unfinished development exhaustive
run was terminated and superseded by this frozen full gate; it is diagnostic-only.

An additional unchanged archived ThreadSanitizer startup control was executed under
the ordinary inherited environment: instrumented control SIGSEGV before main,
ordinary control exit 0 with its main marker. This redundant attempt is recorded
as diagnostic-only. No actual operation TSan suite was built or executed. The prior
archive/hash and fresh logs remain retained; no host protections or environment
instrumentation were bypassed. **Ordinary-host TSan acceptance remains incomplete.**
No further unchanged startup attempts were made after the coordinator instruction.

No native OCCT runs, new native reference captures, tessellation acceptance, tolerance
widening, hidden retry or current public-valid-body claim is part of this evidence.
The receipt and per-file archive manifest are in [final/receipt.json](final/receipt.json).
