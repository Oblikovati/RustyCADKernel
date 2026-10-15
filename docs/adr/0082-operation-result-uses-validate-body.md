# ADR 0082: OperationResult validation is the public body report

Status: implemented on a worktree branch (2026-10-15); integration and release are the
coordinator's.

## Context

ADR 0068 gave `OperationResult` its own small `ValidationReport` (four `StageStatus` values
and a self-intersection certificate) computed by the private ADR 0014 stages. ADR 0072 then
published `validate_body`, whose `ValidationReport` has typed findings named by stable
topology indices, check modes, the Euler census and the tolerance budget. The two reports
had one name and different contents: an application could not compare an extrusion's
validation with `validate_body` of the same solid, and the extrusion carried no census.

## Decision

`extrude` validates its solid with `body_check::validate_body` and stores that report:
`OperationResult::validation()` returns `&body_check::ValidationReport` (re-exported as
`modeling::ValidationReport`; the same type as `validation::ValidationReport`). Its findings
name topology by `TopExp::MapShapes` index, which is the order of `OperationResult::faces`,
`edges` and `vertices` and of `BodyTopology` (ADR 0076).

- A `Refuse` cap is the body-check budget. A tolerance stage that failed only on
  `ToleranceAboveBudget` keeps the operation's `ToleranceOverBudget` decline at the
  `Tolerance` stage; any other non-pass is `ValidationFailed` at the `Validation` stage with
  the first non-passing stage and its rendered findings. Inconclusive stages decline too.
- A `Report` cap is a finding, not a defect (ADR 0068): the body check runs without a budget
  (`tolerance_budget == None`) and the operation records `ToleranceAboveReportLimit`.
- The self-intersection stage of the report is `NativeSearch`. The proof that makes the
  pass exact for an extrusion moved to `OperationResult::self_intersection_certificate()`
  (`AffineSweepOfCertifiedRegion`).

Region faces are not bodies (`validate_body` refuses a face root), so `PlanarRegion` keeps
the ADR 0014 planar-region stages; their summary type is renamed `RegionValidation` so that
`modeling::ValidationReport` names only the public body report.

## Semver

Breaking for 0.2.8 callers: `OperationResult::validation()` returns a reference to the body
report; `self_intersection_evidence` moved to `self_intersection_certificate()`;
`PlanarRegion::validation()` returns `RegionValidation`.

## Evidence

`profile_extrusion::extrusion::operation_validation_is_the_public_body_report`: for a box, a
cylinder and a tube the stored report equals `validate_body(result.solid())` exactly, every
stage passed, topology is `Exact`, self-intersection `NativeSearch`, the census genus is
0, 0, 1 and the budget is the cap. `report_cap_keeps_a_valid_report_without_budget` and the
existing over-budget test cover both cap kinds. All 30 seam tests pass.

## Limits

Region faces keep the private stage summary. The decline still renders findings as text;
callers who need typed findings can run `validate_body` on a result they keep.
