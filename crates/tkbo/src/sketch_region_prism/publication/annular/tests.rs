// SPDX-License-Identifier: GPL-2.0-only
//! Independent expectations: four faces (two caps, two sides), six original uses all continued,
//! two intermediate wire retirements, every child bound to the same sealed join bytes.
use super::*;
use crate::sketch_region_prism::exact_tolerance::StoredToleranceCap;
use crate::sketch_region_prism::frozen::annular_history::fixture::{
    AnnularHistoryCase, fresh_embedding,
};
use crate::sketch_region_prism::frozen::annular_orientation::fixture::join_options;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::metric_bounds::*;

pub(in crate::sketch_region_prism) fn options() -> AnnularPublicationOptions {
    let stored_cap = StoredToleranceCap::refuse(1.).unwrap();
    AnnularPublicationOptions {
        version: 1,
        join: join_options(),
        validation: AnnularValidationOptions {
            version: 2,
            stored_cap,
        },
        history: PrismHistoryOptions::annular_v2(),
        measures: measure_options(),
    }
}
fn measure_options() -> AnnularMeasureOptions {
    let (mut side_area, mut total_area) = (
        PolynomialSideOptions::default(),
        EnclosureSumOptions::default(),
    );
    (
        side_area.absolute_error_limit,
        total_area.absolute_error_limit,
    ) = (0.01, 0.021);
    AnnularMeasureOptions {
        version: 2,
        cap_area: Default::default(),
        signed_sweep: Default::default(),
        side_area,
        maximum_side_subdivisions: 8192,
        total_area,
    }
}
struct Recorder<F>(Vec<PrismPublicationCheckpoint>, F);
impl<F: FnMut(PrismPublicationCheckpoint) -> Result<bool, Failure> + Send> PrismPublicationProgress
    for Recorder<F>
{
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        self.0.push(point);
        (self.1)(point)
    }
}
/// Outcome summary: `Ok(replay)` or `Err((phase, class))`, plus every observed checkpoint phase.
type Published = (
    Result<Vec<u8>, (PrismPublicationPhase, PrismPublicationClass)>,
    Vec<PrismPublicationPhase>,
);
fn publish(
    case: &AnnularHistoryCase,
    requested: AnnularPublicationOptions,
    stop: impl FnMut(PrismPublicationCheckpoint) -> Result<bool, Failure> + Send,
) -> Published {
    let mut recorder = Recorder(Vec::new(), stop);
    let outcome = case.with_region(|region| {
        let request = AnnularPublicationRequest {
            region,
            receipt: &case.receipt,
            bindings: &case.publication,
            copied: &case.names.copied,
            historical: &case.names.historical,
            requested,
        };
        let embedding = fresh_embedding(region, &case.receipt);
        let result = publish_annular_prism(&request, embedding, &mut recorder);
        result.map(|r| accepted(&r)).map_err(|f| (f.phase, f.class))
    });
    (outcome, recorder.0.iter().map(|p| p.phase).collect())
}
/// Closed form: the authored cubic loop (0,0),(4,0),(0,4),(0,0) bounds area 12/5; the inner
/// loop is scaled by 1/2, so each annular cap has area 12/5 * 3/4 = 9/5 and the sweep by
/// (0,0,2) has signed magnitude 18/5.
fn assert_closed_form(measures: &AnnularPrismMeasures) {
    let contains = |[lo, hi]: [f64; 2], value: f64| lo <= value && value <= hi;
    let caps = measures
        .faces()
        .iter()
        .filter(|f| matches!(f.measure, AnnularFaceMeasure::Cap(_)));
    let caps: Vec<_> = caps.map(|f| f.measure.enclosure()).collect();
    assert_eq!(caps.len(), 2);
    assert!(caps.iter().all(|e| contains(*e, 1.8)), "{caps:?}");
    let sweep = measures.signed_sweep().enclosure();
    assert!(contains(sweep, 3.6) || contains(sweep, -3.6), "{sweep:?}");
}
fn accepted(result: &PublishedAnnularPrism) -> Vec<u8> {
    assert_closed_form(result.measures());
    let join = result.join();
    assert!(result.validation().binds(join));
    assert!(result.history().binds(join));
    assert!(result.measures().matches(join));
    assert_eq!(join.faces().len(), 4);
    assert_eq!(result.history().continued().len(), 6);
    assert_eq!(result.history().retired_wires().len(), 2);
    assert!(result.history().deleted_originals().is_empty());
    assert_eq!(result.measures().faces().len(), 4);
    result.replay_bytes().to_vec()
}
#[test]
fn one_source_binding_publishes_validation_history_and_measures_in_order() {
    let case = AnnularHistoryCase::nested();
    let (outcome, phases) = publish(&case, options(), |_| Ok(false));
    let replay = outcome.unwrap_or_else(|e| panic!("refused {e:?}"));
    assert_eq!(phases.last(), Some(&PrismPublicationPhase::Commit));
    let order: Vec<_> = phases
        .windows(2)
        .filter(|w| w[0] != w[1])
        .map(|w| w[1])
        .collect();
    use PrismPublicationPhase::*;
    assert_eq!(order, [Validation, History, Measures, Commit]);
    assert_eq!(phases[0], Join);
    let (again, _) = publish(&AnnularHistoryCase::nested(), options(), |_| Ok(false));
    assert_eq!(again.unwrap(), replay);
}
#[test]
fn every_phase_cancellation_and_failure_refuses_without_a_body_and_allows_reuse() {
    use PrismPublicationPhase::*;
    let case = AnnularHistoryCase::nested();
    for phase in [Join, Validation, History, Measures, Commit] {
        let (outcome, _) = publish(&case, options(), |p| Ok(p.phase == phase));
        assert_eq!(
            outcome.unwrap_err(),
            (phase, PrismPublicationClass::Cancelled)
        );
        let fail = |p: PrismPublicationCheckpoint| match p.phase == phase {
            true => Err(Failure::new("fixture callback failure")),
            false => Ok(false),
        };
        let (outcome, _) = publish(&case, options(), fail);
        assert_eq!(outcome.unwrap_err(), (phase, PrismPublicationClass::Failed));
    }
    assert!(publish(&case, options(), |_| Ok(false)).0.is_ok());
}
#[test]
fn unsupported_version_and_history_refusal_keep_their_phase() {
    let mut case = AnnularHistoryCase::nested();
    let mut requested = options();
    requested.version = 2;
    let refused = publish(&case, requested, |_| Ok(false)).0.unwrap_err();
    assert_eq!(
        refused,
        (
            PrismPublicationPhase::Admission,
            PrismPublicationClass::Incomplete
        )
    );
    case.names.historical.scopes.pop();
    let refused = publish(&case, options(), |_| Ok(false)).0.unwrap_err();
    assert_eq!(
        refused,
        (
            PrismPublicationPhase::History,
            PrismPublicationClass::Incomplete
        )
    );
}
