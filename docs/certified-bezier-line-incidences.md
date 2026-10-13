# Source-bound Bézier/line incidence certificates

Martensite's `rustycad_tkmath::predicates::roots::bezier_line::certify_bezier_line_incidences`
extends the existing exact projection, root isolation and isolated-sign pipeline.
It does not replace mapped OCCT solvers or complete application blocker #10.

The query borrows controls, optional rational weights, the original Bernstein
parameter interval, and two line points. The report owns those exact binary64
inputs and all requested policies. Occurrence keys identify supplied sources;
they are not geometry-confidence assertions. Every selected contact references
an index in the complete supporting-line root isolation. A closed-segment query
retains exact endpoint-sign membership for every supporting root. Supporting-line
queries retain all roots and skip segment-only conversion and sign stages.
Coincident curves are declined rather than returned as an empty success.

`RootIsolationOptions.max_degree` still admits the reduced normal and denominator
polynomials. Version 1 `BezierDomainSignOptions` separately admits longitudinal
conversion: either an explicit degree cap or the retained source basis degree.
The report records actual normal/denominator, longitudinal conversion and derived
sign-product degrees. `domain_sign_applied()` distinguishes skipped stages from
exact zero sign polynomials; supporting queries report no applied cap or sign
degrees. Policy version is still validated, while an unused degree cap cannot
reject otherwise certified supporting roots. Multiplication by the denominator may increase sign-product
degree; this matches the existing directed-sign conversion convention. An
unsupported policy version is invalid; insufficient domain admission is incomplete.
The existing line/ray operations retain their original degree policy.

## Root and construction certificates

The denominator has a complete empty root isolation over the original parameter
interval. The normal isolation accounts for all roots, including endpoints and
even multiplicities. Selected roots are refined by re-isolating the same original
normal polynomial within their certified one-root brackets. Refinement must
preserve exactly one root. No point residual is relabelled as a root-error bound.

Homogeneous coordinates and weights are evaluated using exact rational interval
de Casteljau arithmetic. The denominator interval must separate from zero before
reciprocal multiplication. This supports the original basis also on extrapolated
parameter intervals. Exact comparison against the ordered finite binary64 lattice
constructs outward coordinate bounds, including subnormal values. Overflow or an
unresolved denominator requests refinement; it does not fabricate a point.

For each coordinate, the two adjacent binary64 values enclosing the exact
interval midpoint compete using exact maximum distance to both exact endpoints.
The lower numeric candidate wins exact ties. The sum of squared axis radii bounds
the Euclidean world error; exact comparison chooses its smallest outward binary64
square root. Acceptance compares exact squared error with the exact square of
the caller's positive finite world limit. Limits are never widened. A singleton
root can still fail construction: `ConstructionResolutionLimit` preserves the
request, unchanged effective limit, last enclosure and last root. It makes no
universal assertion about all finite-point representability.

## Work, failures and ownership

The subdivision budget covers denominator isolation, initial normal isolation,
and every subsequent world-construction root refinement together. Endpoint
Sturm–Tarski sign evaluation does not subdivide. Exact integer arithmetic,
interval evaluation and directed binary64 searches are not bounded by the
subdivision count; the report does not claim an arithmetic-operation budget.
Cancellation is polled through the existing algebra and interval evaluation.
Complete empty, invalid, declined, cancelled and incomplete outcomes remain
distinct. Failure reports preserve typed original causes and owned source inputs;
no partial contact list is returned as success.

Rust regressions cover independent rational parameter/world polynomials,
negative weights, repeated roots, closed endpoints, extrapolation, generated
rotation/reflection/scale/reversal/permutation, cancellation and repaired reuse,
source mutation after success/failure/cancellation, and explicit domain policy.
Ordinary OCCT fixtures supplement this evidence; they do not certify completeness
or construction error. Attributed assets live under
`crates/tkmath/tests/exact_roots/bezier_line/incidence_assets` and require no OCCT.

Run the operation regressions and accepted benchmark-input verification with:

```sh
cargo test -p rustycad-tkmath --all-features --lib --test toolkit --bench bezier_incidence
```

The benchmark defaults to functional verification, with no timings. An explicit
`--measure` measures the complete query, acceptance checks and result destruction
for six identical accepted workloads, twenty warmups and one hundred samples.
Timing and process peak RSS require a separately approved quiet-host run; neither
failed workloads nor an uncertified residual-only solver form an equivalent
performance baseline. No benchmark baseline has been accepted.

Remaining scope includes coincident-overlap representation, other admitted curve
families, reusable wire/arrangement certificates, and complete application
incidence consumption. These are not covered by this operation's completion.
