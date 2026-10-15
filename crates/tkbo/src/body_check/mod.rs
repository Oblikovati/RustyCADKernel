// SPDX-License-Identifier: GPL-2.0-only
//! Martensite ADR 0072: ordered validation of any owned B-rep body.
//!
//! [`validate_body`] composes the ADR 0014 stage checkers (generalized from the private
//! sketch-region validation) with exact topology and tolerance checks, in this order:
//!
//! 1. **Topology**: closed two-manifold shells with opposite edge uses for solids, the
//!    Euler-Poincare census, and the mapped BRepCheck topology statuses (exact).
//! 2. **Geometry**: curves on surfaces, p-curves, vertices on curves (mapped BRepCheck;
//!    curve-on-surface is sampled, so the mode is `Sampled`).
//! 3. **Self-intersection**: the mapped BOPAlgo_CheckerSI run (`NativeSearch`).
//! 4. **Tolerance**: valid values, vertex >= edge >= face, the resolution's cap, and the
//!    certified planar curve/p-curve residual where it applies.
//!
//! A stage that fails or is inconclusive leaves every later stage `NotRun`.
mod census;
mod ledger;
mod naming;
mod native;
mod report;
mod tolerance;

#[cfg(test)]
mod ledger_tests;

pub use ledger::ValidationReport;
pub use report::{
    CheckMode, EulerCensus, Finding, FindingKind, Severity, StageReport, StageVerdict, TopologyRef,
    ValidationStage,
};

use crate::profile_extrusion::{Resolution, ToleranceCap};
use crate::sketch_region_validation::ValidationStage as CheckerStage;
use ledger::Ledger;
use naming::TopologyNames;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// A resolution field that is not finite and positive: its name and value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolutionRejected {
    /// The offending field, e.g. `"cap"`.
    pub field: &'static str,
    /// Its value.
    pub value: f64,
}

/// Validates `shape` in four ordered stages under `resolution` (its tolerance cap is the
/// tolerance budget). Checker failures become inconclusive findings, never errors.
///
/// ```
/// use rustycad_tkbo::body_check::{validate_body, CheckMode, StageVerdict, ValidationStage};
/// use rustycad_tkbo::profile_extrusion::{DisplayTolerance, Resolution, ToleranceCap};
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
///
/// let display = DisplayTolerance { chordal: 1e-3, angular: 0.1 };
/// let resolution = Resolution::new(1e-6, 1e-7, 1e-9, display, ToleranceCap::Refuse(1e-5));
/// let mut maker = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0).unwrap();
/// let report = validate_body(maker.shape().unwrap(), &resolution).unwrap();
/// assert!(report.is_valid());
/// assert_eq!(report.census[0].genus, Some(0));
/// let search = report.stage(ValidationStage::SelfIntersection);
/// assert_eq!((search.verdict, search.mode), (StageVerdict::Passed, CheckMode::NativeSearch));
/// ```
pub fn validate_body(
    shape: &Shape,
    resolution: &Resolution,
) -> Result<ValidationReport, ResolutionRejected> {
    let budget = tolerance_budget(resolution)?;
    let names = match TopologyNames::of(shape) {
        Ok(names) => names,
        Err(failure) => return Ok(unindexable(shape, &failure, budget)),
    };
    let root = names.name(shape);
    let ledger = Ledger::new();
    let mut stages = Stages {
        shape,
        names: &names,
        root,
        ledger,
    };
    let (passed, census) = stages.topology();
    if passed && stages.geometry() && stages.self_intersection() {
        stages.tolerance(budget);
    }
    Ok(stages.ledger.finish(census, budget))
}

/// The admitted resolution's cap as the tolerance budget (`NoLimit` checks none).
fn tolerance_budget(resolution: &Resolution) -> Result<Option<f64>, ResolutionRejected> {
    resolution
        .admit()
        .map_err(|(field, value)| ResolutionRejected { field, value })?;
    Ok(match resolution.cap {
        ToleranceCap::NoLimit => None,
        ToleranceCap::Report(limit) | ToleranceCap::Refuse(limit) => Some(limit),
    })
}

/// A root whose sub-shapes cannot be indexed: topology inconclusive, the rest not run.
fn unindexable(shape: &Shape, failure: &Failure, budget: Option<f64>) -> ValidationReport {
    let kind = shape.shape_type().unwrap_or(ShapeEnum::Shape);
    let mut ledger = Ledger::new();
    ledger.record_failure(
        ValidationStage::Topology,
        TopologyRef { kind, index: 1 },
        failure,
    );
    ledger.finish(Vec::new(), budget)
}

/// The ordered stage runs over one indexed body.
struct Stages<'a> {
    shape: &'a Shape,
    names: &'a TopologyNames,
    root: TopologyRef,
    ledger: Ledger,
}

impl Stages<'_> {
    /// Census and mapped topology: whether the stage passed, and every computed census.
    fn topology(&mut self) -> (bool, Vec<EulerCensus>) {
        let (mut findings, census) = match census::topology_census(self.shape, self.names) {
            Ok(found) => found,
            Err(failure) => {
                self.ledger
                    .record_failure(ValidationStage::Topology, self.root, &failure);
                return (false, Vec::new());
            }
        };
        findings.extend(native::checker_findings(
            self.shape,
            CheckerStage::Topology,
            self.names,
        ));
        let passed = self
            .ledger
            .record(ValidationStage::Topology, CheckMode::Exact, findings);
        (passed, census)
    }

    fn geometry(&mut self) -> bool {
        let findings = native::checker_findings(self.shape, CheckerStage::Geometry, self.names);
        self.ledger
            .record(ValidationStage::Geometry, CheckMode::Sampled, findings)
    }

    fn self_intersection(&mut self) -> bool {
        let findings = native::self_intersection_findings(self.shape, self.names);
        self.ledger.record(
            ValidationStage::SelfIntersection,
            CheckMode::NativeSearch,
            findings,
        )
    }

    fn tolerance(&mut self, budget: Option<f64>) {
        match tolerance::tolerance_findings(self.names, budget) {
            Ok((findings, mode)) => self
                .ledger
                .record(ValidationStage::Tolerance, mode, findings),
            Err(failure) => {
                self.ledger
                    .record_failure(ValidationStage::Tolerance, self.root, &failure);
                false
            }
        };
    }
}
