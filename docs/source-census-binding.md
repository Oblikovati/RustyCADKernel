# Martensite source-bound census and endpoint measurements

ADR0014's diagnostic census now has an opt-in source-bound entry point,
`enumerate_source_bound_census`. Its `SourceConformancePolicy` separately declares
original-plane and construction distance limits and its own root-work policy.
The legacy `enumerate_boundary_census` has no implicit source policy and preserves
its original cancellation/pair order. Source work and successful incidence work
remain separately observable; failed incidence work is still an explicit missing
operation obligation.

The bound source inspection owns scalar snapshots and complete original keys.
Binding generates the required occurrence-parent matrix from the current inputs,
not from whatever rows the report supplies. It rejects missing, extra, duplicate,
reordered or unrepresented originals/rows, changed coefficient bits, weights,
native/basis trims, support and placement factors, full original keys, copied
occurrence identities, requested policy, stage limits and work receipts. All
three ordered stages must return `Within`. Passing some rows never removes a
global obligation. A complete binding removes only `SourceConversionAndPlane`,
the request-limited representation obligation.

This numerical conformance does not prove that a nonzero representation error
preserves original contact topology. Canonical contact identity, complete mapped
partitions, pair/self coverage, search, self-intersection and tolerance remain
independent. No healthy region/body operation or native class completion is
introduced.

The shared `PartitionOccurrenceIdentity::capture` reads the actual cumulative
copied topology address/location/orientation. It does not recover objects from
an address. Captured constructions retain that identity. The shared
`capture_space_representation` and `same_space_representation` interfaces let an
opt-in DS reader capture and compare actual live 3D coefficients without a
second scalar-capture implementation. Consumers adopt these through checked main.

Contact coverage binds the incidence source's complete curve/line descriptors,
intervals, domain/sign policy, root settings and requested world error to both
actual inputs. IDs only locate a unique physical producer; reused physical
occurrences still require identical descriptors. Orphan reports, changed sources
and duplicate IDs retain visible unbound contact records rather than an empty
pass.

Native parameter evidence uses the actual captured pcurve kind: Bézier identity
or exact line endpoint transport. The exact dyadic incidence bracket is lifted
into native parameters before comparison with the actual `BRepTool::parameter`
value. Rounded `(t-a)/(b-a)` normalization is removed. Typed evidence retains the
native bracket and supplied parameter; missing/stale maps and nonfinite values
are incomplete. Approximate mapped split values remain unchanged and are never
snapped. Singleton equality is a measurement, not canonical contact identity.

Endpoint missing states separately retain representation, incidence-source,
curve-parameter, second-axis parameter, canonical contact, 3D error composition
and partition gaps. The previous ordinary floating world projection remains a
provisional observation at unchanged bounds. Generated-parent history and shared
incident vertices remain necessary; proximity cannot repair missing identity.
Representation descriptors are checked again on contact inspection so a later
input mutation cannot silently reuse a stale successful attachment.

The corpus includes full and partial matrices, duplicate/unrepresented rows,
bit-level and physical mutations, shared/reversed/located originals, separate
policies and spent work, cancellation/retry, nonzero plane errors, orphan sources,
actual native curved split fixtures and typed missing parameter maps. Rust
regression replay requires no OCCT process. GPU submission and mapped geometry
algorithms are unchanged.

The census retains the caller source policy independently of the report.
`current_obligations()` revalidates that attachment against current descriptors
and reinstates representation incompleteness after later input/policy mutation;
the original recorded missing list remains historical evidence.

The census retains the actual construction support face solely to recapture
current copied curve, stored pcurve and plane/support correspondence. Native
aliases may mutate those fields while topology identity and cached scalar bits
remain unchanged. Current checks recapture the admitted representations and
reject mismatch or typed capture failure. They do not mutate geometry. The
original immutable scalar inspection remains valid historical evidence after
such mutation; it cannot by itself certify the current BRep body. Retaining or
cloning a face is not an input-isolation proof. Original source coefficients
remain the operation's historical pre-copy snapshot, not a claim about a later
mutated caller object.
