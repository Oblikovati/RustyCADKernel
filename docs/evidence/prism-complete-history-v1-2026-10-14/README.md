# Complete prism history baseline

Basis `e6955e6db466ae677956ad82537134fb97793b22`. Before structural changes,
297 prism tests passed, with exact unchanged V1 frozen and lineage replay bytes.
`baseline.json` pins toolchain, commands, platform and source bookends.
`owned-expectations.json` states independent one-operation incidence expectations;
it does not grant history completeness or body validity.

## Structural exhaustive sweep queries

The additive private query receipt distinguishes NotReached from Captured, and
retains generated lists, both cap results (including null), and true/false native
retirement for every actual copied-member query. Legacy output construction and
serializers are unchanged. `retention.json` records 303 prism tests and exact
unchanged lineage/frozen replay.

The first new test incorrectly assumed that native wire retirement implies null
cap results; its failure is retained in `retention-null-cap-assumption-red.log.gz`.
`make_prism.rs::is_deleted` means absence of a used generated swept shape and is
independent of cap images. The corrected structural invariant keeps both cap
continuations together with the retired wire and empty generated list. This
corrects an unpinned draft assertion; no owned baseline expectation changed.
