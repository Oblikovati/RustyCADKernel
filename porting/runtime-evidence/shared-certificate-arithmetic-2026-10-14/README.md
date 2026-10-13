# Shared certificate arithmetic extraction

Base revision: `300b9c50543209f56df05636789b161ef7283497`.
Exact ratio, interval de Casteljau and directed binary64 rounding helpers move
from the private line-incidence namespace into a private roots arithmetic module.
This prevents a second construction-error implementation for paired curves.
Numeric expressions and all provenance comments are unchanged. Equivalence
normalizes only private visibility/import paths, whitespace and one formatter
trailing comma in the ExactRatio constructor parameter list. No mapped solver,
public line/ray query, result, limit, error or callback order changes.

Executed gates: all 26 TKMath library tests; 83 existing Bézier line/ray/winding/
wire toolkit regressions; six accepted incidence benchmark-input verifications
without timings; all-target/all-feature Clippy with warnings denied. Each passed.
This is an independent structural checkpoint, not a paired-system root proof,
C++ API credit or whole application blocker #10 completion.
