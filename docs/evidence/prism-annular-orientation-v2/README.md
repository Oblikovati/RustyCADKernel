# Conditional annular orientation V2 candidate

Implementation source `db8706a75`; corrected evidence runner `473166a75`.
ADR0054 declares the private conditional scope. This is candidate evidence;
coordinator integration and application/release acceptance are separate.

## Executed checks

Rust 1.99.0, two Cargo build jobs, isolated target recorded in `final/receipt.json`.
The frozen source passed **2,054 tests**: 1,406 library, 646 integration, and two
example tests; **35 doctests**; strict all-target/all-feature Clippy; documentation
build; workspace formatter. All **50,861 tracked inputs** match before and after.
The 21 focused orientation tests include actual construction for independent
outer/inner stored reversals, traversal reversals, reflection, negative sweep,
literal 4-face/12-trim directions, typed refusals, every cancellation checkpoint,
source mutation/input drop, independent operation replay and four concurrent
readers performing eight proof/replay operations each.

The original runner then encountered an evidence-harness error: the legacy joined
orientation selector omitted `tests::replay_tests`, selected zero tests, and had no
payload to parse. `final/runner.log.gz` retains that failure; it is not a kernel
failure or a successful replay claim. Evidence-only commit `473166a75` corrects
that one selector and changes no crate, Cargo or test inputs.

A redundant full rerun was stopped at coordinator instruction. Its explicit
termination record, interrupted-test log and source bookends are retained under
`final-corrected/`; they are **not** counted as completed acceptance evidence.
The corrected replay-only supplement reuses the exact gate commands and existing
binary, records binary/harness/driver hashes, verifies an empty crate/Cargo diff
since the completed matrix, and has independent equal tracked-input bookends.

## Full process payloads

Every row below was emitted by two separate processes. All bytes match, and the
three V1 rows additionally match the earlier retained structural baselines.

| Payload | Bytes | SHA-256 |
| --- | ---: | --- |
| Orientation V1 | 752 | `67df03c3156f98f17484f304d3a67078534778fa6fbe87882c8234ef5ac50f31` |
| Joined orientation V1 | 215,177 | `d642f0f81b03da8689cd7930d2fa27b269faec055d9629bdb507422bcd321968` |
| Publication V1 | 2,202,990 | `accba93e30c455d9f888a96c5b70f684a1dbacbbf3afec9076de3544c7a13107` |
| Annular join V2 | 536,690 | `7daf5bf9b081a784b1225dd2bc1989d0b1a5f131e44d378abeb898c849cdd1b1` |
| Annular orientation V2 | 546,491 | `6c72bbddce37a78eae326d338031abe8b4436b2411c4b44c2631b685390abc64` |

Compressed complete payloads and logs are retained in `replay-supplement/`.
`artifact-sha256.json` seals retained artifacts; source manifests seal every
tracked input used by the completed matrix and supplement. Development failures
(including the inherited pre-cleanup Clippy finding) remain in `development/`.
The shared-signs directory separately pins V1 before structural extraction.

## Reproduction and exclusions

From a clean checkout of the recorded source with the pinned toolchain:

```sh
python3 docs/evidence/prism-annular-orientation-v2/gate.py /tmp/annular-orientation-check
```

`replay_supplement.py` documents the exact harness-only repair invocation. It
expects this task's retained test executable path; `gate.py` discovers its current
executable from the full test log instead. No full matrix was inferred from a
focused run or from payload hashes.

Independent expected signs use analytic cubic derivatives/material-side vectors
and exact rational Jacobian receipts retained in `independent/`. The earlier
positively scaled direction receipt and later actual half-scale/unit-direction
receipt are both preserved with their distinct operands. These are local sign
checks atop the existing globally bound domain theorem, not a new embedding proof.

No new OCCT execution, GPU execution, numerical integration, storage redesign,
release benchmark or public validity promotion is claimed. Unchanged TKMath and
TKBRep sources were not independently rebuilt/tested as standalone toolkits for
this TKBO-only slice. ThreadSanitizer was not executed for this suite; the retained
host startup-control failure in `prism-tsan-startup-diagnosis-2026-10-14` remains
incomplete and ordinary threaded tests do not waive that release requirement.
