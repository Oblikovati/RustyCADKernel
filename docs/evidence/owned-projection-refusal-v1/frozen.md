# Owned projection refusal V1 frozen receipt

## Source and scope

Frozen source: `9699a8adf561196aa296984ea5546dde0650476d`. ADR0056 owns only the actual post-publication Projection refusal. The bridge moves the original owner/cause/callback; other scoped families retain the identical original box. Accepted child publication is failure context, never outer success. No producer behavior changed.

Dependencies: complete-owned source `47712cfc5960d4c7d7b65b6c81b61256fe044fef`, shared key helper `2329f32515ee743eb2d6ef116c5290d3030387c7`, and shared diagnostic encoder facade `8882d47a4`. `baseline.json` and `baseline.tar.gz` retain the original and unchanged structural replay comparisons.

## Executed checks

Rust 1.99.0, Cargo 1.99.0, Linux x86_64, two build jobs, isolated worktree target. `frozen.json` records exact commands, durations, platform, environment and result counts.

- Full TKBO all-target/all-feature suite: 2006 passed in total; per-target counts: 1358 passed / 0 ignored, 646 passed / 0 ignored, 0 passed / 0 ignored, 2 passed / 0 ignored, 0 passed / 0 ignored.
- Doctests: 35 passed.
- Strict all-target/all-feature Clippy, workspace formatting check and TKBO documentation: passed.
- New focused prefix: 17 passed. Actual six post-publication checkpoints were exercised in cancellation/native modes. Independent typed fixtures cover every supported cause variant, provenance and scalar edge bits.
- The unchanged exhaustive complete-operation test exercised 8,268 checkpoints in both cancellation and native-failure modes (16,536 terminal paths).
- All 50683 tracked-file hashes were identical before and after the frozen gate.

The archive includes the complete logs and reusable `frozen/gate.py`, `frozen/replay.py`, and source bookends. Run the gate from the checkout root using `python3 target/owned-projection-failure/frozen/gate.py`. The replay runner accepts checkout, output directory and freshly built toolkit test binary as its three arguments.

## Process and thread evidence

Eight fresh processes produced four byte-identical pairs. The three existing complete/preparation/publication payloads also equal their pinned baselines. New projection refusal payload: 26,813,998 bytes. Exact payload and binary SHA-256 values are in `frozen.json`; the binary hash was unchanged before and after replay.

The focused corpus also compares four independent host-thread operations with serial bytes and concurrent immutable readers. Compile-time Send/Sync assertions cover the new owned report and exposed cause/class types. These tests do not replace ThreadSanitizer.

## Review, failures and limits

The coordinator read source `9699a8adf` for ownership/move, classification, full request plus accepted-publication envelope, and body/volume typed encoders; no blocker was found within that scope. The review did not execute tests.

Retained development failures: one baseline command selected system rustc 1.95 because PATH was not pinned (corrected Rust 1.99 command passed); an incorrect test import was corrected; strict Clippy rejected a redundant let/return and a large unboxed error, corrected by preserving unsupported arms in the same original box. Original logs remain archived; no tests or tolerances were weakened.

ThreadSanitizer remains incomplete under the previously recorded ordinary-host startup failure (`docs/evidence/prism-tsan-startup-diagnosis-2026-10-14`); no unchanged-host retry or bypass occurred. No live OCCT, native/GPU, release performance, application acceptance or complete release gate is claimed. Preparation, Resolution and Publication owned-refusal families remain incomplete. Their read-only inventory is retained as `frozen/remaining-families.md`; no follow-on implementation belongs to this candidate.

The user paused broader work. This evidence-only commit seals the completed candidate for coordinator integration and cleanup; it does not start another slice.

Archive SHA-256: `35f57c6edd5ffd662a9bce1b87f441f281aa4709a91c2d2a154af5960ee3ace9`.
