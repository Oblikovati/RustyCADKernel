# Martensite captured winding certificates

`CapturedBezierWireStore` is an operation-local mathematical certificate store.
It certifies the same exact closed-wire winding membership as the existing
`classify_bezier_wire`; it does not certify valid faces or completed region solids.
The compatibility classifier retains its per-wire request budget, typed causes,
cancellation polls, exact closure and half-open endpoint rules.

Create one store with the full requested `RootIsolationOptions` and ray. Capture
controls, optional weights and the closed basis interval using
`CapturedBezierWireInput::capture`, then supply current occurrence IDs and reversal
in `CapturedBezierWireOccurrence`. Scalar capture owns independent values before
callbacks. The physical-source token must be assigned by the owning operation
from its actual captured representation/placement; an integer or topology address
alone proves neither input isolation nor persistent identity. Original trim-to-basis
provenance belongs to that owning operation, not a fabricated topology dependency
inside TKMath.

Admission compares exact coefficient/weight/interval bits and physical token.
The ray and every requested policy field remain fixed within a store. Reversed
traversal and occurrence ID are assembly inputs, never proof keys. A distinct
physical source with identical coefficients remains a distinct admission. Changed
payloads miss even when a caller reuses the token; invalid inputs never produce
successful certificates.

A cold miss preserves preparation, endpoint join, numerator evaluation, then
contribution append. Denominator failure therefore precedes an open join, while
an open join precedes later numerator failure. Only a successful segment capture
is admitted; final wire closure and later segments can still fail. Completed
segment proofs survive those failures. Reuse reverses actual exact endpoints and
contributions and supplies the current IDs for boundary attribution. It does not
simulate skipped solver cancellation polls; the real assembly polls remain.

`CaptureWorkReceipt` distinguishes newly executed subdivisions, operation total,
reused certificates and historical certificate cost. Cold captures use the actual
remaining budget. Successful hits perform zero new subdivisions, even at budget
zero. Failed or cancelled attempts retain all completed splits without refund.
The existing isolator emits accounting events only after its original successful
split increment; partial midpoint arithmetic is not labelled a completed split.
The original public errors and successful isolation counters are unchanged.

`store.proofs()` borrows immutable constructor-owned evidence: actual denominator
isolation, supporting roots and longitudinal signs, aligned local contributions,
or the actual coincident directed branch. Zero normal/directed polynomials have
an explicit branch tag. No fabricated empty isolation replaces a missing proof.
A boundary short-circuit retains aligned `None` entries for contributions that
were not evaluated. Each proof records its actual effective capture policy.

Run the Rust regression corpus with:

```sh
cargo test -p rustycad-tkmath --all-targets
```

The captured corpus covers reversed cubic and polygon reuse, independent known
winding/tangent outcomes, cold cancellation/cause equivalence, exact subnormal and
rational closure gaps, signed weights/poles, payload/policy invalidation, failure
budget accounting and prior-proof survival. No ordinary test launches OCCT.

The `captured_winding` benchmark verifies three accepted operations by default.
Only `--measure --mode=baseline` or `--measure --mode=captured` requests 20 warmups
and 100 raw samples. Each operation includes capture/assembly/checks/result and
store destruction; baseline and captured modes use identical geometry, requested
options and expected classifications. Actual newly performed and historical proof
work remain distinct. Benchmark-only allocation-call observation delegates every
allocation operation unchanged to `System`; there is no production allocator or
ownership change. Peak RSS is measured externally with `/usr/bin/time`; it is
process memory, not GPU memory. Timing acceptance remains pending coordinator GO.

This is a kernel extension acceptance slice, with no new OCCT API/class completion
claim. Candidate construction, complete shared-boundary incidence and body checks
remain the application operation's separate obligations. The deferred camera,
chain, Iterator and canonical lifetime task remains preserved in its own checkout.
