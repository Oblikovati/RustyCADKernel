# ADR 0081: One measured-value type for modelling results and body properties

Status: implemented on a worktree branch (2026-10-15); integration and release are the
coordinator's.

## Context

Release 0.2.8 shipped two public types for the same idea. `modeling` (ADR 0068) reported
region areas and extrusion volumes as `Measure { value, error, kind: MeasureKind }`, where
the kind said what the error means (`CertifiedBound`, `ClosedForm`, ...). `properties`
(ADR 0073) reported `Measured { value, error_bound }` and named the integration method once
per result (`IntegrationMethod`). An application comparing `OperationResult::volume()` with
`body_properties(solid).volume` had to translate between them, and the two enums mixed
"how it was computed" with "what the bound means" differently.

## Decision

`rustycad_tkbo::measured` defines one type, re-exported by `rustycad_kernel::modeling` and
`rustycad_kernel::properties` (it lives in TKBO because the modelling seam is there and the
kernel facade depends on TKBO, not the reverse):

- `Measured { value, error_bound, method: IntegrationMethod }` and
  `MeasuredPoint { point, error_bound, method }`, with `encloses`, `evidence` and `scaled`.
- `IntegrationMethod` says how the value was computed, ordered from strongest to weakest
  (`max` combines): `CertifiedEnclosure`, `RepresentationEnclosure`, `ClosedForm`,
  `AdaptiveQuadrature`, `NativeUnbounded`.
- `Evidence` says what the bound means and is derived from the method, never stored apart:
  `Proven` (certified enclosure), `Estimated` (representation, closed form, quadrature),
  `Unbounded` (native).

`RepresentationEnclosure` is new and names what arc regions already were: the ADR 0066
certified enclosure of the kernel's Bézier representation, plus an estimate of that
representation's rounding (split points, weights). Line-only regions stay
`CertifiedEnclosure`. Body properties keep their methods (`ClosedForm`,
`AdaptiveQuadrature`, `NativeUnbounded`) and every value now carries its own method;
`BodyProperties::method` and `FaceProperties::method` remain the weakest of their values.

`OperationResult::volume()` and `PlanarRegion::area()` return `Measured`, so an extrusion
volume is directly comparable with `body_properties(result.solid()).volume`.

## Semver

Breaking for 0.2.8 callers: `profile_extrusion::{Measure, MeasureKind}` are removed (use
`Measured`, `error_bound`, `method`); struct literals of `properties::Measured` and
`MeasuredPoint` need the `method` field. The coordinator chooses the version (0.3.0 under
Cargo's 0.x rules).

## Evidence

`measured` unit tests: method order and derived evidence, scaling keeps the method and
covers the product rounding. Kernel `modeling_tests::operation_volume_is_a_body_properties_measure`:
a slot with a circular hole extruded by 1.5 gives a `RepresentationEnclosure` volume that
encloses `(8 + 3 pi / 4) 1.5`, `body_properties` of the same solid encloses it too, and the
two intervals overlap. The existing region tests check `CertifiedEnclosure` for a rectangle
and `RepresentationEnclosure` for a general arc segment; every properties test is unchanged.

## Limits

Only the volume is reported by an extrusion; its area and centroid come from
`body_properties`. `RepresentationEnclosure` is not proven for the input curves.
