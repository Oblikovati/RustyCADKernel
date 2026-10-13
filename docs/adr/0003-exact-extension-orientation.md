# Exact orientation for extension topology

The analytic sketch-region contract in [issue #10] needs exact signs for supplied
coordinates. Add `tkmath::predicates::orient2d` as an extension: represent each
finite binary64 coordinate as an integer multiple of 2^-1074 and compute the
orientation determinant with exact integers. Positive means counterclockwise.
Nonfinite inputs return the first offending point, axis, and value. Signed zeros
represent the same coordinate. This decision does not change mapped OCCT APIs.

Keep `num-bigint` behind this project-owned interface, pinned with default
features disabled. Integer subtraction occurs before multiplication, so neither
rounded coordinate differences nor overflow, underflow, FMA contraction, or
host rounding mode can change a sign. The positive common scale squared does
not change orientation. The maximum finite input integer needs 2098 bits;
intermediate differences and products remain bounded by the binary64 domain.
No arbitrary coordinate magnitude limit or modelling tolerance is introduced.

Use one exact path initially. A floating filter can be added only with independent
proof that every certified sign agrees with this path, including subnormals and
nonfinite intermediate results. An extended-precision approximation alone is
not an exact predicate. Allocation cost is accepted for this first primitive;
no application performance or complete region-classification claim is made.

This starts the G1 prerequisite of the kernel improvement roadmap. It replaces
no mapped operation and adds no shape recognizer, tolerance, retry, or geometric
type assertion. The eventual region consumer must delete its diagnostic sampled
classification path under issue #10 after the full region/history/validation
corpus passes. Do not run two production classifiers as an ordered fallback.
This primitive alone does not certify analytic-curve incidences, construction
error, source ownership, bounded regions, or complete operation results.

[issue #10]: https://github.com/Oblikovati/RustyCADKernel/issues/10
