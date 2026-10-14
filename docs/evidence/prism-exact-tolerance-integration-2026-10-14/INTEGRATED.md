# Exact prism conformance and tolerance main integration

Main source `bb5f065290ddbe0059384970b1d2ca3fefa9d118` integrates reconciled
candidate `e9890ebc8323320164763373216b60a6e7b8130e` with all 38 candidate
crate files exact. Root verification passed **6,323 tests and 302 documentation
tests** across TKMath, TKBRep and TKBO, zero failed or ignored, strict all-target
Clippy, workspace formatting and warning-denied generated documentation.
All original 6,292 passing test names remain passing. Source/config bookends
match before and after the matrix and supplementary replay checks.

The 41 candidate/reconciliation artifact hashes and both frozen source bookends
were checked. Six independent main process executions preserve the exact
453,096-byte conformance/tolerance V2 receipt and the 89,921-byte frozen V1 and
3,629-byte tolerance-inspection V1 receipts. Full hashes and logs are retained
in `integrated-supplement.json`; commands and platform are in
`integrated-main.json`. The replay parser recognizes libtest's same-line test
prefix; no implementation or expected bytes were changed.

Accepted scope is exact represented endpoint/closure/trim conformance and explicit
stored metadata policy, bound to the immutable join. Geometry is computed once
and reused by the tolerance check. These facts alone do not publish a valid body,
complete general profiles, satisfy application acceptance or authorize release.
ThreadSanitizer remains incomplete: the ordinary-host instrumented startup
control crashed before main, and the tolerance suite was not built or executed
under TSan. No host protection or instrumentation bypass was applied.
