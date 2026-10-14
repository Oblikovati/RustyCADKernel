# Private prism tolerance inspection V1 evidence

Source revision: `edf1de5ff4eaa68d5fbc03b1f30dfe909825b0c1`.
Base: `bdb88a547`. Kernel decision: [ADR0030](../../adr/0030-private-prism-tolerance-inspection.md).
This is private evidence collection for RustyCADKernel issue #10 and Martensite C1
/ issue #13. It publishes no body and changes no existing ordered coverage.

## Executed checks

With Rust/Cargo 1.99.0 and `CARGO_BUILD_JOBS=2`:

- 6,078 TKMath/TKBRep/TKBO tests passed; none failed or ignored.
- 302 doctests passed; none failed or ignored.
- All-target/all-feature Clippy with `-D warnings` and workspace formatting passed.
- All 15,757 tracked build-source inputs had identical before/after SHA256 hashes.
- Independent processes produced identical 3,629-byte arithmetic receipts:
  `a3bbc53e1b5be36f87331ff1319ce1e47b078447b694844d21f004abdaaaf8a7`.
- The existing full frozen projection replay remains 89,921 identical bytes:
  `88e28942e1fc6ad301a90f5f07624962d842f0262bde1377103cd3470a871fb4`.

From the repository root, the complete matrix is reproducible with:

```sh
python3 docs/evidence/prism-tolerance-inspection-v1-2026-10-14/run-matrix.py
```

The script overwrites only this evidence directory's check logs and manifests.
Exact commands, exit codes, durations and platform details are in `receipt.json`
and `checks.json`. `candidate-source-sha256.json` identifies changed crate files
for integration review. `source-before.json.gz` and `source-after.json.gz` cover
all tracked crate inputs, Cargo manifests/lock, toolchain and configuration.

## What is proved

The authored source-bound closed cubic extruded by `[0,0,2]` retains 26 directed
contexts, 8 physical metadata entities, 6 directed trims and 6 physical endpoint
queries with all 12 directed vertex uses. Raw values, effective accessor values,
requested raw caps, requested/effective world budgets and exact residual reports
remain separate. All exact-zero fixture endpoint and trim bounds are zero.

Raw metadata `1/8` against explicit cap `1/16` records Report or Refuse while
geometric zero remains unchanged. Invalid actual raw values are rejected with
their entity and original bits. Owned endpoint offsets of `1/8` independently
exceed budgets `0` and `0.1`, and prove squared upper bound `1/64` at `0.125`.
Placement-order, invalid parameter, missing/duplicate incidence, stale metadata,
source mutation and cancellation fixtures preserve required refusals and causes.

The earlier mutation draft was intentionally retained in the evidence. Changing
a retained cap vertex also changes its PrismCopy closure owner; the exact shared
source gate refuses before endpoint inspection. The final test pins that typed
refusal rather than bypassing it. Independent metric arithmetic uses owned
operands and makes no body admission claim. Clippy initially identified large
error payloads; original causes were boxed without suppressing the lint.

## Limits and next gate

Every inspection retains an unresolved stored-tolerance/placement-semantics gap.
It admits no aggregate tolerance certificate. Refused metadata and exceeded or
incomplete metric decisions can appear in a captured diagnostic report; they do
not authorize a valid body. `Within` is a certified bound and need not be a tight
maximum. The original incomplete receipt and compatibility accessor floors are
unchanged.

The process receipt covers arithmetic results and selected scalar operands and
counts, excluding process-local topology addresses. It is not a full body
serializer. Independent worker comparison and compile-time Send/Sync checks do
not claim general thread safety; no new sanitizer or benchmark claim is made.
No live OCCT, tessellation decision, application acceptance or release-completion
claim is involved. Scalar endpoint queries have a constant mapped domain and
complete with zero bisections; the inspector does not introduce a new root-search
budget or certify arbitrary nonzero tolerance transport.

The concrete follow-on ordered consistency and immutable publication obligations
are recorded in [next-policy-composition.md](next-policy-composition.md). That
work requires a new explicit decision/version after integration and cleanup.
