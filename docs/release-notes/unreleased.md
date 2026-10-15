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

## Visualization: TKV3d API completed

- `AisInteractiveContext::{set_object_deviation_coefficient,
  set_object_deviation_angle, set_object_iso_on_triangulation}` port the
  per-object `SetDeviationCoefficient`, `SetDeviationAngle` and
  `IsoOnTriangulation` overloads.
- The protected `AIS_Shape` drawer helpers are public as
  `rustycad_tkv3d::ais::set_shape_drawer_{color,material,transparency,width}`,
  and the protected `PrsDim_EllipseRadiusDimension` constructor body as
  `rustycad_tkv3d::prs_dim::init_ellipse_radius`.
- `AisViewController` re-exports the listener `press_mouse_button`,
  `release_mouse_button`, `pressed_mouse_buttons`, `last_mouse_flags`,
  `last_mouse_position` and `has_touch_points` as inherent methods.
- The parity ledger claims the AIS_Manipulator, PrsDim, AIS_ViewController,
  AIS_InteractiveContext, AIS_Shape and remaining AIS declarations: TKV3d API
  done 3235 -> 5118 of 5172; 37 rows stay partial with their reasons and 17
  class records wait for the row generator.
