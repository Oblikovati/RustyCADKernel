// SPDX-License-Identifier: GPL-2.0-only
//! Nonempty admission is an extension hook, never a change to mapped General Fuse CheckData.
use super::{copy_inputs::*, outcome::*, relations::compose_relations};
use crate::bop_algo::{BOPAlgoBuilder, BOPAlgoPaveFiller, BuilderHooks};
use rustycad_tkernel::standard::Failure;

pub(super) struct NonemptyArrangement(pub BOPAlgoBuilder);
impl BuilderHooks for NonemptyArrangement {
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder {
        // This hook exposes the same mapped builder state; admission alone differs.
        &mut self.0
    }
    fn check_data(&mut self, filler: &BOPAlgoPaveFiller) {
        // Admission already rejected empty input; preserve every actual mapped filler error.
        self.0.options().report().merge(filler.options().report());
    }
}
pub(super) trait ArrangementExecutor {
    fn execute(&mut self, copied: &CopiedSources) -> Result<BOPAlgoBuilder, ArrangementCause>;
}
pub(super) struct MappedArrangementExecutor<'a> {
    pub(super) observation: Option<&'a mut super::partition_types::PartitionObservation>,
    pub(super) space_capture: Option<super::partition_types::PartitionSpaceCapture>,
}
impl ArrangementExecutor for MappedArrangementExecutor<'_> {
    fn execute(&mut self, copied: &CopiedSources) -> Result<BOPAlgoBuilder, ArrangementCause> {
        let mut filler = BOPAlgoPaveFiller::new();
        filler.set_arguments(&copied.arguments);
        filler.set_non_destructive(true);
        let performed = filler.perform_internal();
        if performed.is_err() {
            self.capture(copied, &filler, None);
        }
        performed.map_err(interference_failure)?;
        let mut builder = NonemptyArrangement(BOPAlgoBuilder::new());
        builder.0.set_arguments(&copied.arguments);
        builder.0.set_non_destructive(true);
        let performed = builder.perform_internal1(&mut filler);
        self.capture(copied, &filler, Some(&builder.0));
        performed.map_err(interference_failure)?;
        Ok(builder.0)
    }
}
/// Run only the owned arrangement stage; `arrange_owned_sources(&inputs,&mut |_|false)`.
/// Cancellation is observed at boundaries; mapped copy/interference calls are synchronous.
pub(crate) fn arrange_owned_sources(
    sources: &[ArrangementSource<'_>],
    cancelled: &mut dyn FnMut(ArrangementPhase) -> bool,
) -> ArrangementCompletion {
    arrange_with_executor(
        sources,
        cancelled,
        &mut MappedArrangementExecutor {
            observation: None,
            space_capture: None,
        },
    )
}
pub(super) fn arrange_with_executor(
    sources: &[ArrangementSource<'_>],
    cancelled: &mut dyn FnMut(ArrangementPhase) -> bool,
    executor: &mut dyn ArrangementExecutor,
) -> ArrangementCompletion {
    if let Err(cause) = admit_sources(sources) {
        return ArrangementCompletion::Declined(cause);
    }
    if cancelled(ArrangementPhase::Copy) {
        return ArrangementCompletion::Cancelled(ArrangementPhase::Copy);
    }
    let copied = match copy_sources(sources) {
        Ok(copied) => copied,
        Err(cause) => return reject_cause(cause),
    };
    arrange_copied(copied, cancelled, executor)
}
fn arrange_copied(
    copied: CopiedSources,
    cancelled: &mut dyn FnMut(ArrangementPhase) -> bool,
    executor: &mut dyn ArrangementExecutor,
) -> ArrangementCompletion {
    if cancelled(ArrangementPhase::Interference) {
        return ArrangementCompletion::Cancelled(ArrangementPhase::Interference);
    }
    let builder = match execute_copied(&copied, executor) {
        Ok(builder) => builder,
        Err(cause) => return reject_cause(cause),
    };
    if cancelled(ArrangementPhase::History) {
        return ArrangementCompletion::Cancelled(ArrangementPhase::History);
    }
    match compose_relations(&copied, &builder) {
        Ok(output) => output,
        Err(cause) => reject_cause(cause),
    }
}
fn reject_cause(cause: ArrangementCause) -> ArrangementCompletion {
    match cause {
        ArrangementCause::Failure { .. } | ArrangementCause::MappedReport { .. } => {
            ArrangementCompletion::Failed(cause)
        }
        _ => ArrangementCompletion::Incomplete(cause),
    }
}
fn reject_reports(builder: &BOPAlgoBuilder) -> Result<(), ArrangementCause> {
    if !builder.options().has_errors() {
        return Ok(());
    }
    Err(ArrangementCause::MappedReport {
        phase: ArrangementPhase::Interference,
        report: snapshot_report(builder.options().report()),
        affected_source: None,
    })
}
fn interference_failure(cause: Failure) -> ArrangementCause {
    ArrangementCause::Failure {
        phase: ArrangementPhase::Interference,
        cause,
    }
}

pub(super) fn snapshot_report(
    original: &rustycad_tkernel::message::Report,
) -> std::sync::Arc<rustycad_tkernel::message::Report> {
    let owned = std::sync::Arc::new(rustycad_tkernel::message::Report::new());
    // Keep actual typed alert payloads, but own their container across source report reuse.
    owned.merge(original);
    owned
}

fn execute_copied(
    copied: &CopiedSources,
    executor: &mut dyn ArrangementExecutor,
) -> Result<BOPAlgoBuilder, ArrangementCause> {
    let builder = executor.execute(copied)?;
    reject_reports(&builder)?;
    Ok(builder)
}

impl MappedArrangementExecutor<'_> {
    fn capture(
        &mut self,
        copied: &CopiedSources,
        filler: &BOPAlgoPaveFiller,
        builder: Option<&BOPAlgoBuilder>,
    ) {
        if let Some(observer) = self.observation.as_deref_mut() {
            *observer = super::partition_capture::capture_partition_with_space(
                copied,
                filler,
                builder,
                self.space_capture,
            );
        }
    }
}
