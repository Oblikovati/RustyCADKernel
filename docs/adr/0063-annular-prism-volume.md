# ADR 0063: Annular prism physical volume

Status: private implementation on a worktree branch (2026-10-15).

## Decision

`frozen::annular_volume::certify_annular_prism_volume(join, orientation, measures,
version, cancel)` turns the annular measures' certified signed sweep integral into
a physical-volume enclosure with its unchanged certified error. It adds no
integration algorithm, tessellation or sampled sign.

The annular sweep integrates both copied boundaries after material
normalization: each curve is reversed exactly when `native_winding *
material_side < 0`, with material side `+1` for the outer and `-1` for the
inner boundary. Green's theorem over that normalized pair gives the positive
annulus area in native coordinates; the placed sweep Jacobian multiplies it.
The exact sign of the integral is therefore the exact placed-Jacobian sign
already certified by `AnnularOrientationCertificate::jacobian_sign`.

Admission requires the orientation certificate and the measures to bind the
same complete join bytes, and the measures' boundary directions to equal the
certified directions exactly. Sign evidence must have Jacobian sign `+/-1`,
native windings `+/-1` and material sides `(+1, -1)`; anything else is a typed
`SignFactors` incompleteness. Transport reuses the ADR0044 isometry
(`[lo, hi] -> [-hi, -lo]`, representative negated, error unchanged) and its
contradiction check. Cancellation at the begin/finish checkpoints yields no
volume.

## Evidence and replay

`Martensite.AnnularPrismVolume/v1` records the sign, both boundary directions,
enclosure, representative, error bound and the complete join, orientation and
measure receipts. Tests use independent closed forms: the cubic loop bounds
12/5, the inner copy scales by 1/2, so the annulus area is 9/5 and the volume
for `|h| = 2` is 18/5. Every orientation variant (stored and traversed
reversal, reflection, negative sweep, both combined, reversed request order)
contains 18/5 with `|representative - 18/5| <=` the certified error, and both
exact signs are exercised. A second independent route checks consistency with
twice each certified world cap-area enclosure. Foreign orientation, version
and both cancellation checkpoints refuse; replay is identical across
operations and threads.

## Limits

The interpretation is conditional on the two nested closed polynomial loops of
ADR0061. It is not yet composed into the publication receipt or a public
outcome, and covers no general profile.
