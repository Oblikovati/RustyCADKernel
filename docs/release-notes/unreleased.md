# Unreleased

Next target: **0.3.0** (breaking public API changes since 0.2.8). Workspace
version changes and publication remain with the release coordinator.

## Breaking: modelling seam follow-ups (ADRs 0081-0084)

- One measured-value type, `rustycad_tkbo::measured::Measured { value,
  error_bound, method }`, is shared by `modeling` and `properties`.
  `modeling::{Measure, MeasureKind}` are removed. `Evidence` (Proven /
  Estimated / Unbounded) and `IntegrationMethod::{CertifiedEnclosure,
  RepresentationEnclosure}` are new.
- `OperationResult::validation()` returns the public `validate_body` report;
  `self_intersection_certificate()` is new. `PlanarRegion::validation()` returns
  `RegionValidation`.
- `build_planar_regions` admits full ellipses, elliptic arcs
  (`ProfileCurve::EllipticArc`) and clamped polynomial or rational B-splines
  (`Spline` gains `degree`, `weights` and `knots`).
- Regions that share an edge and loops that touch at a vertex are admitted.
  New named declines: `TangentBranches`, `PinchedRegion`, `BridgeCurve`,
  `InvalidCurve`, `EndOffCurve`. Hole loops now run clockwise, as in the face.

## Breaking: certified incidences for periodic splines and offsets (ADR 0080)

Native periodic B-spline knot vectors unwrap exactly through `KnotSequence`.
Line, circle, ellipse and spline offsets are certified as sources and targets
where an implicit partner exists, or refused by name (`OffsetPair`,
`OffsetBasis`, `OffsetRadius`, `OffsetEllipseAxis`, `OffsetContinuity`,
`OffsetSingular`, `PeriodicKnotShift`, `PeriodicShape`). Spline-source
extension stays refused until the application defines a rule.
