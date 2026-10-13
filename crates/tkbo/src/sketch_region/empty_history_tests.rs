// SPDX-License-Identifier: GPL-2.0-only
//! A named test builder deliberately emits no leaves; mapped PrepareHistory still determines deletions.
use super::{
    arrangement::*,
    copy_inputs::CopiedSources,
    outcome::*,
    tests::{closed_cubic, sources},
};
use crate::bop_algo::{BOPAlgoBuilder, BOPAlgoPaveFiller, BuilderHooks};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

struct EmptyResultHook(BOPAlgoBuilder);
impl BuilderHooks for EmptyResultHook {
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder {
        // The fake alters result admission, not the real interference/history implementation.
        &mut self.0
    }
    fn check_data(&mut self, filler: &BOPAlgoPaveFiller) {
        // Preserve native failure reports while deliberately allowing the single source.
        self.0.options().report().merge(filler.options().report());
    }
    fn build_result(
        &mut self,
        _: ShapeEnum,
        _: &crate::bop_ds::DataStructure,
    ) -> Result<(), Failure> {
        // No output leaf is selected; real PrepareHistory must classify every copied member removed.
        Ok(())
    }
}
struct EmptyMappedHistory;
impl ArrangementExecutor for EmptyMappedHistory {
    fn execute(&mut self, copied: &CopiedSources) -> Result<BOPAlgoBuilder, ArrangementCause> {
        let mut filler = BOPAlgoPaveFiller::new();
        filler.set_arguments(&copied.arguments);
        filler.perform_internal().map_err(test_failure)?;
        let mut hook = EmptyResultHook(BOPAlgoBuilder::new());
        hook.0.set_arguments(&copied.arguments);
        hook.perform_internal1(&mut filler).map_err(test_failure)?;
        Ok(hook.0)
    }
}
fn test_failure(cause: Failure) -> ArrangementCause {
    ArrangementCause::Failure {
        phase: ArrangementPhase::Interference,
        cause,
    }
}
#[test]
fn actual_empty_container_preserves_each_deleted_original_member() {
    let original = closed_cubic();
    let result =
        arrange_with_executor(&sources(&original), &mut |_| false, &mut EmptyMappedHistory);
    let ArrangementCompletion::CompletedEmpty { deleted } = result else {
        panic!("expected mapped deleted-member coverage")
    };
    assert_eq!(deleted.len(), 3);
    assert!(deleted.iter().all(|key| key.source == RegionSourceId(42)));
    assert!(
        deleted
            .iter()
            .any(|key| key.topology_address == original.tshape().object_address().addr())
    );
}
#[test]
fn unsupported_container_history_returns_explicit_member_coverage_gap() {
    use rustycad_tkbrep::topods::{Builder, Wire};
    let original = closed_cubic();
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, &original).unwrap();
    let wire: Shape = wire.into_shape();
    let result = arrange_owned_sources(&sources(&wire), &mut |_| false);
    assert!(
        matches!(result,ArrangementCompletion::Incomplete(ArrangementCause::UnsupportedHistoryCoverage{member}) if member.source==RegionSourceId(42))
    );
}
