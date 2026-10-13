//! `BOPAlgo_CheckerSI`: checks one shape for self-interference by intersecting its
//! sub-shapes with each other and collecting the interfering pairs.
#[path = "checker_si/completion.rs"]
mod completion;
#[cfg(test)]
#[path = "checker_si/completion_tests.rs"]
mod completion_tests;
#[path = "checker_si/face_self.rs"]
mod face_self;
pub(crate) use completion::{
    FaceTermination, NativeAlertObservation, SelfCheckCompletion, SelfCheckPhase,
};
use completion::{capture_failure, enter_phase};
#[path = "checker_si/post_treat.rs"]
mod post_treat;
#[path = "checker_si/solids.rs"]
mod solids;
use super::alerts::alert;
use super::{AlertIntersectionFailed, AlertMultipleArguments, BOPAlgoPaveFiller};
use crate::bop_ds::{BOPDSIteratorSI, DataStructure};
use crate::int_tools::Context;
use rustycad_tkernel::standard::{Failure, Handle};
use std::ops::{Deref, DerefMut};

/// The self-interference checker: a pave filler over one argument whose broad phase pairs
/// the argument's own sub-shapes; the interfering pairs end in `DS().Interferences()`.
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoCheckerSI;
/// let mut checker = BOPAlgoCheckerSI::new();
/// checker.perform();
/// // No argument: the checker needs exactly one.
/// assert!(checker.options().has_errors());
/// ```
pub struct BOPAlgoCheckerSI {
    filler: BOPAlgoPaveFiller,
    level_of_check: i32,
}

impl Default for BOPAlgoCheckerSI {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BOPAlgoCheckerSI {
    type Target = BOPAlgoPaveFiller;
    fn deref(&self) -> &BOPAlgoPaveFiller {
        &self.filler
    }
}

impl DerefMut for BOPAlgoCheckerSI {
    fn deref_mut(&mut self) -> &mut BOPAlgoPaveFiller {
        &mut self.filler
    }
}

impl BOPAlgoCheckerSI {
    /// A checker of every interference kind, non-destructive and without p-curve
    /// construction (`BOPAlgo_CheckerSI()`); `BOPAlgoCheckerSI::new()`.
    pub fn new() -> Self {
        let mut filler = BOPAlgoPaveFiller::new();
        filler.set_non_destructive(true);
        filler.set_avoid_build_pcurve(true);
        Self {
            filler,
            level_of_check: DataStructure::nb_interf_types() - 1,
        }
    }

    /// Limits the check to the interference kinds up to `level`: 0 V/V, 1 +V/E, 2 +E/E,
    /// 3 +V/F, 4 +E/F, 5 +F/F, 6 +V/S, 7 +E/S, 8 +F/S, 9 +S/S (the default); a level out of
    /// that range is ignored (`SetLevelOfCheck`); `checker.set_level_of_check(4)`.
    pub fn set_level_of_check(&mut self, level: i32) {
        if (0..DataStructure::nb_interf_types()).contains(&level) {
            self.level_of_check = level;
        }
    }

    /// The configured level of check; `checker.level_of_check() == 9`.
    pub fn level_of_check(&self) -> i32 {
        self.level_of_check
    }

    /// Intersects the sub-shapes of the single argument, checks the faces on
    /// self-intersection, the shapes against the solids, and records every interfering pair;
    /// any failure is reported as `AlertIntersectionFailed` (`Perform`); `checker.perform()`.
    pub fn perform(&mut self) {
        // Compatibility callers retain no new receipts or failure ownership.
        self.perform_recorded(None);
    }

    /// Observe exact execution privately; retain original diagnostics only on opt-in.
    /// Example: `let receipt = checker.perform_observed();`.
    pub(crate) fn perform_observed(&mut self) -> SelfCheckCompletion {
        let mut trace = SelfCheckCompletion::new(self.level_of_check);
        self.perform_recorded(Some(&mut trace));
        trace
    }

    fn perform_recorded(&mut self, mut trace: Option<&mut SelfCheckCompletion>) {
        let arguments = self.filler.arguments().extent();
        if arguments != 1 {
            self.filler
                .options()
                .add_error(&alert(AlertMultipleArguments));
            finish_rejected(trace, arguments, self.filler.options());
            return;
        }
        if let Err(failure) = self.perform_checks(trace.as_deref_mut()) {
            capture_failure(trace.as_deref_mut(), failure);
            self.filler
                .options()
                .add_error(&alert(AlertIntersectionFailed));
        }
        if let Some(trace) = trace {
            trace.returned = true;
            trace.native_alerts = self.filler.options().has_errors();
            trace.alerts = observed_alerts(self.filler.options());
        }
    }

    fn perform_checks(
        &mut self,
        mut trace: Option<&mut SelfCheckCompletion>,
    ) -> Result<(), Failure> {
        let level = self.level_of_check;
        self.filler
            .perform_with_initializer(|filler| initialize_checker(filler, level));
        enter_phase(trace.as_deref_mut(), SelfCheckPhase::FacePolicy);
        self.check_face_self_intersection(trace.as_deref_mut())?;
        enter_phase(trace.as_deref_mut(), SelfCheckPhase::SolidPairs);
        self.perform_solid_checks()?;
        if self.filler.options().has_errors() {
            return Ok(());
        }
        enter_phase(trace, SelfCheckPhase::PostTreat);
        self.post_treat()
    }

    fn perform_solid_checks(&mut self) -> Result<(), Failure> {
        let stages: [CheckStage; 4] = [
            Self::perform_vz,
            Self::perform_ez,
            Self::perform_fz,
            Self::perform_zz,
        ];
        for stage in stages {
            if !self.filler.options().has_errors() {
                stage(self)?;
            }
        }
        Ok(())
    }

    /// The data structure, which `perform` must have created.
    fn structure_mut(&mut self) -> Result<&mut DataStructure, Failure> {
        self.filler
            .data_structure_mut()
            .ok_or_else(missing_structure)
    }
}

/// One check run after the intersection of the sub-shapes.
type CheckStage = fn(&mut BOPAlgoCheckerSI) -> Result<(), Failure>;

/// The failure of a stage that runs without the data structure `Init` creates.
fn missing_structure() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BOPAlgo_CheckerSI: data structure is None; expected Init to have created it",
        "",
    )
}

/// `BOPAlgo_CheckerSI::Init`: unlike the base `Init` it neither validates the arguments nor
/// derives the non-destructive mode, and its iterator pairs sub-shapes of the same argument
/// and keeps only the kinds up to the level of check.
fn initialize_checker(filler: &mut BOPAlgoPaveFiller, level: i32) -> Result<(), Failure> {
    filler.clear();
    filler.initialize_data_structure()?;
    let context = Handle::new(Context::default());
    let mut iterator = BOPDSIteratorSI::new();
    let structure = filler.data_structure().ok_or_else(missing_structure)?;
    let options = filler.options();
    iterator.prepare_with_context(
        structure,
        context.get(),
        options.use_obb(),
        options.fuzzy_value(),
    )?;
    iterator.update_by_level_of_check(level);
    filler.install_context_and_iterator(context, iterator.into_iterator());
    Ok(())
}

fn finish_rejected(
    trace: Option<&mut SelfCheckCompletion>,
    arguments: i32,
    options: &super::Options,
) {
    if let Some(trace) = trace {
        trace.rejected_arguments = Some(arguments);
        trace.returned = true;
        trace.native_alerts = true;
        trace.alerts = observed_alerts(options);
    }
}

fn observed_alerts(options: &super::Options) -> Vec<NativeAlertObservation> {
    use rustycad_tkernel::message::Gravity;
    options
        .report()
        .get_alerts(Gravity::Fail)
        .into_iter()
        .map(|original| NativeAlertObservation { original })
        .collect()
}
