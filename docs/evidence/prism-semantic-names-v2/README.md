# Semantic dictionary V2 aggregate extension

Source `e3865d02e` adds only Partition/CandidateCopy Aggregate and AggregateUse roles, with literal tokens 1058, 1059, 1314, 1315. The independent generator extends the prior reviewed literal oracle; it does not import or parse production Rust. Full V1 fixture and 10,571-byte symbolic map remain unchanged.

All 40 naming and 671 prism tests passed, strict all-target Clippy and formatting passed, and complete input source hashes match. Two processes and four threads produce identical V2 envelopes. The initial 39-test log predates a test-only function split; the final gate covers 40 tests. No test failure was hidden or expectation weakened.

The legacy request entry still authors V1. Only the new resolver/whole-operation path may explicitly opt into V2, before lookup/construction as described in ADR0042. Symbols alone prove no actual stage correspondence. This evidence claims no public body or complete owned operation.
