# ADR 0072: Ordered validation of any owned body

Status: implemented on a worktree branch (2026-10-15); public as
`rustycad_tkbo::body_check` and re-exported as `rustycad_kernel::validation`.

## Decision

`validate_body(shape, &Resolution) -> Result<ValidationReport, ResolutionRejected>`
validates any owned body (solid, comp-solid, shell, or compound of them) in four
ordered stages. It generalizes the private ADR 0014 stage checkers of
`sketch_region_validation` instead of duplicating them: `BodyKind::Any` skips the
declared-kind check, and `inspect_stage`, `inspect_self_intersection` and
`inspect_planar_edge_residual` become crate-visible. Their behaviour for the existing
planar-region and solid kinds is unchanged.

| Stage | Checks | Mode |
| --- | --- | --- |
| Topology | Shells of solids are closed two-manifolds: every non-degenerate edge has exactly two face uses in opposite directions (`FreeEdge`, `NonManifoldEdge`, `InconsistentOrientation`); free shells (sheets) check only manifoldness and orientation. Euler-Poincare per solid, `V - E + F - H = 2 (S - G)`, `H = L - F`, degenerate edges excluded (`EulerPoincare` for an odd characteristic or negative genus). Mapped BRepCheck topology statuses. | `Exact` |
| Geometry | Mapped BRepCheck with geometric controls: curves on surfaces, p-curves, vertices on curves within tolerance. | `Sampled` (curve-on-surface is sampled) |
| Self-intersection | Mapped `BOPAlgo_CheckerSI` (`Interference` names both sub-shapes). | `NativeSearch` |
| Tolerance | Valid values; vertex >= edge >= face (`ToleranceOrder`); the resolution cap (`ToleranceAboveBudget`); the certified planar curve/p-curve residual within the edge tolerance (`CurveResidualExceedsTolerance`). | `Exact` when every edge use was decided by the certificate, else `Sampled` |

A stage with a defect is `Failed`; with only coverage gaps it is `Inconclusive`
(never a pass); all later stages are `NotRun`. Checker failures become inconclusive
findings, never errors; only an invalid resolution is refused. `is_valid()` requires
four passes; `is_exact()` additionally requires exact modes, so it is false while the
self-intersection stage rests on the native search.

The structural self-check gaps that ADR 0014 records on every run (inner search and
pair completion are not observable natively) define the `NativeSearch` mode rather than
findings. Every other gap (native alerts, unscheduled face phase, level below 9,
unattempted faces) stays an inconclusive finding. `BRepCheck_Analyzer` creates no
result for compounds and comp-solids (`case TopAbs_COMPSOLID: case TopAbs_COMPOUND:
break;`), so their missing checker is not a gap.

Findings name topology by kind and 1-based `TopExp::MapShapes(root, kind)` index
(`TopologyRef`, resolvable against the root), so the report is plain owned data
(`Send + Sync`) with no live shape. The tolerance budget is the cap of the shared
`profile_extrusion::Resolution` (ADR 0068, other branch); `NoLimit` checks no budget.

## Verification

Through the facade, with real tkprim bodies: box, cylinder, sphere, cone (frustum),
torus and an annulus prism pass all four stages; the Euler genus is 0, 0, 0, 0, 1, 1
and the box census is `(V, E, F, L, S) = (8, 12, 6, 6, 1)`. Each broken body gives its
named finding in its stage and leaves the later stages `NotRun`: a box missing a face
(four `FreeEdge`, each an edge of the missing face), a box with a repeated face (four
`NonManifoldEdge { uses: 3 }`), a box with a reversed face (four
`InconsistentOrientation` on its edges), two overlapping boxes in a compound
(`Interference` at self-intersection after two passes), a vertex at tolerance 0.5
under a 1e-3 cap (`ToleranceAboveBudget` naming that vertex), and an edge above its
vertices' tolerance (two `ToleranceOrder`). Reports are identical across four threads.
Ledger unit tests cover verdict precedence and blocking, name resolution and the
rejected resolution field. The existing tkbo validation tests are unchanged.

## Limits

The geometry stage samples; the self-intersection stage is a native search, not a
certificate; residual certificates apply only to line and Bezier edges on planes.
Non-solid invariants (sheet boundaries, mixed dimensions) are limited to manifoldness
and orientation. Cancellation and per-stage cost reporting are not exposed yet.
