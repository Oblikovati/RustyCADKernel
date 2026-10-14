# Structural cleanup over the pinned two-loop proof

Baseline production `aa8a7b3a4`, full 46-test corpus `d8281ed1f`, six current V1
payload pins completed at `3a2ba98cf`. The cleanup separates source role lookup,
scalar captures, metadata admission, proof assembly, and test setup/assertions into
focused helpers. Existing check/callback ordering, typed refusal payloads and
classification remain unchanged. The new borrowed failure classification accessor
reuses that same classifier; downstream composition must not map causes separately.
The test fixture and authored identity mapper receive narrow prism-subtree visibility
for the existing annular-construction worker. There is no production authority constructor.

Executed Rust1.99/jobs2 on unchanged final cleanup source:

- 46 focused tests passed, including process/thread and all callback cutoffs.
- Strict TKBO all-target/all-feature Clippy passed; formatter check passed.
- All six legacy payloads exactly equal their pre-cleanup owned fixtures.
- Complete V2 payload exactly equal: 43,373 bytes,
  SHA256 `3bbc93ebcf5d76ed2ddf9fdaeddc1ab9e337b0f9294766e8429eebad6521a16e`.
- Executable-body audit: zero bodies above 20 physical lines; signatures and doc
  attributes excluded. Largest file 246 lines. The audit helper and output are retained.

Earlier intermediate source receipts retain the one wrong helper argument type
(`&Shape` instead of `&Face`) and mechanical Clippy needless-borrow/let-return
findings. These were corrected without changing proof bytes or test conclusions.
The final source digest manifest identifies the actual checked cleanup source.
Full frozen candidate/integration gates and ordinary-host sanitizer acceptance
remain distinct and are not claimed by this focused receipt.
