# Independent and ordinary incidence fixtures

`parameter-roots.json`, `denominator-roots.json`, and `world_x-roots.json` are
unaltered structured outputs from Jacobian `polynomial.roots.isolate` over exact
QQ coefficients. The supplied polynomials are respectively 6t²−6t+1,
−2t²+2t+1, and 64x²−64x+13. These certify roots of those polynomials;
they do not independently certify the whole source-to-world construction.
Rust tests establish that linkage and check endpoint signs of the independently
eliminated world polynomial. Denominator root-freedom is also checked by the
existing shared exact isolator over the actual source interval.

`occt-ordinary.tsv` is the byte-identical output of two pinned OCCT CPU runs.
Its native tool, compiler command, headers/libraries and SHA-256 hashes are in
`porting/runtime-evidence/certified-bezier-incidences-2026-10-13/native-provenance.json`.
Ordinary native intersection parameters/points supplement the exact certificate
checks; they do not supply a completeness or error bound. Rust replay uses no OCCT.

`accepted.tsv` supplies all six functional benchmark workloads. Query policy is
parameter width 0.001, root degree 32, total subdivision budget 4096, version 1
source-basis domain-sign admission, and world error 1e-8. All queries must complete
and match the recorded contact count before any sample is retained. Timings
include full query, acceptance checks and result destruction; peak memory is
whole-process RSS, including corpus preparation. Benchmark verification without
`--measure` emits no timing samples.

The exact dyadic root t=.5/world x=1/3 regression separately checks binary64
neighbors 0x3fd5555555555555 and 0x3fd5555555555556. Their exact distances from
1/3 are 1/(3·2^54) and 1/(3·2^53), respectively. Consequently 3e-17 is achievable
by the nearest candidate and 1e-20 is not for this exact world point. This is a
specific independently determined case, not a universal representability claim.
