# Source-bound directed Bézier incidence positions, behavior version 1

## Decision and scope

Add an explicitly selected directed-incidence operation to the existing exact
Bézier/line pipeline. This is a new behavior contract, version **1**, for the
certified trim/extension prerequisite under the current application blockers.
It does not replace any existing operation or establish nearest-hit ordering.
The legacy `certify_bezier_line_incidences` remains selectable and unchanged.
In particular, its `Support` queries still skip endpoint-sign admission.

The new operation binds five endpoint positions to the owned input coefficients,
complete support isolation and constructed contacts: before origin, at origin,
interior, at terminal, beyond terminal. Position index is the support root index,
including roots excluded by a Segment query. Root order is curve parameter order,
never a claim about distance along the line. Occurrence IDs identify supplied
sources; they do not infer geometric identity from order or proximity.

## Exact evidence and shared work

Reuse the existing denominator certification, exact homogeneous projection,
complete normal-root isolation and Sturm–Tarski signs on those same brackets.
For direction d=B-A, denominator D and longitudinal numerator L, origin position
uses sign(LD), and terminal position uses sign((L-|d|²D)D). Since D is certified
nonzero and |d|² is strictly positive, these are the exact endpoint comparisons
without division, rounded world points or approximate root substitution. The
five allowed sign pairs are (--), (0-), (+-), (+0), (++). An inconsistent pair
or cardinality becomes typed incomplete failure rather than successful evidence.

The new operation explicitly requires both endpoint signs for Support as well
as Segment. It preserves the requested root limit, domain-sign limit, subdivision
budget and world error; it never widens them. The shared constructor still selects
contacts by the original domain and reports conservative world error. A required
stage failure returns owned typed failure, never a successful partial list.

## Independent expectations

The retained rational fixture has controls (0,-1),(1,2) and weights (2,1).
Its homogeneous numerators are Nx=t, Ny=4t-2, with D=2-t. At t=1/2,
Ny=0 and Nx/D=1/3. The adjacent binary64 values have exact fractions
6004799503160661/18014398509481984 and
3002399751580331/9007199254740992. Their differences from 1/3 are
+1/54043195528445952 and -1/27021597764222976, respectively.
Jacobian `polynomial.map.evaluate` independently checks the normal zero and
both rational differences; the Bernstein-to-polynomial derivation above is
explicit reasoning, not a claim that that tool verified the implementation.
A constructed binary64 point cannot equal 1/3 and must not establish equality.

Closed-form dyadic roots, endpoint equalities, line reversal, weight sign
reversal, repeated roots, generated endpoint placements and root-index alignment
supply additional owned expectations. Current behavior is captured before the
shared preparation refactor and retained byte-for-byte.

## Versioning, migration and exclusions

Version 1 is selected by the new query wrapper; unsupported versions are invalid
with the requested version retained. Existing query and report types, signatures,
failure variants and selection semantics stay available. There is no workspace
version edit or migration of existing callers in this slice. A future incompatible
contract needs a new behavior version, a package semantic version change and
migration notes; replay must retain version 1 until a recorded migration.

There is no replacement engine or deletion task: this exposes retained exact
comparisons from the existing pipeline. No TKBO consumer, ray solver, whole-wire
or region acceptance, overlap continuum, distance sorting, exchange work, GPU,
performance claim or new native oracle is included. Published evidence owns all
reachable inputs, has immutable borrowed access, and uses no locks/global state.
Compile-time Send/Sync checks and ordinary thread/process replay are bounded
verification only. Ordinary-host ThreadSanitizer acceptance remains incomplete:
startup controls already failed as recorded in
`docs/evidence/native-display-lifecycle-v1-2026-10-14/TSAN.md`; no bypass is used.
