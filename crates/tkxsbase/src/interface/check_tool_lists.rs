// Rust port of DataExchange/TKXSBase/Interface/Interface_CheckTool.cxx (CompleteCheckList,
// CheckList, AnalyseCheckList, VerifyCheckList, WarningCheckList)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The check lists of an `Interface_CheckTool`. Each native list runs its entities in a
//! `try` block that restarts after the failing entity; here each entity is one step whose
//! failure is recorded (`raisecheck`) before the next one. The running check and entity
//! keep their native reuse between entities (a report's check can be cleared by the next
//! entity, as natively).
use super::check_tool::{ERROR_HANDLING, live_model, raise_check};
use super::{
    InterfaceCheck, InterfaceCheckIterator, InterfaceCheckTool, InterfaceInterfaceModel,
    InterfaceModelEntities, InterfaceModelReports,
};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use std::sync::atomic::Ordering;

/// The running state of a list: the result, the current check and entity.
struct ListRun {
    res: InterfaceCheckIterator,
    ach: Handle<InterfaceCheck>,
    ent: Handle<StandardTransient>,
}

impl InterfaceCheckTool {
    /// Runs `step` on every entity; a failing step is recorded in the running check,
    /// which is listed, and `fail_bits` are set in the status.
    fn run_list(
        &mut self,
        model: &Handle<dyn InterfaceInterfaceModel>,
        res: InterfaceCheckIterator,
        fail_bits: i32,
        mut step: impl FnMut(
            &mut Self,
            &dyn InterfaceInterfaceModel,
            i32,
            &mut ListRun,
        ) -> Result<(), Failure>,
    ) -> Result<InterfaceCheckIterator, Failure> {
        let live = live_model(model)?;
        ERROR_HANDLING.store(false, Ordering::SeqCst);
        let mut run = ListRun {
            res,
            ach: Handle::new(InterfaceCheck::new()),
            ent: Handle::default(),
        };
        for i in 1..=live.nb_entities() {
            if let Err(failure) = step(self, live, i, &mut run) {
                raise_check(failure, &mut run.ach)?;
                run.res.add(&run.ach, i);
                self.thestat |= fail_bits;
                run.ach = Handle::new(InterfaceCheck::new());
            }
        }
        Ok(run.res)
    }
    /// A result list on the model of the tool.
    fn new_result(&self) -> (Handle<dyn InterfaceInterfaceModel>, InterfaceCheckIterator) {
        let model = self.theshare.model();
        let mut res = InterfaceCheckIterator::new();
        res.set_model(&model);
        (model, res)
    }
    /// The semantic check of entity `i` into `ach`: its module check, or the semantic
    /// check the model recorded.
    fn semantic_check(
        &self,
        model: &dyn InterfaceInterfaceModel,
        i: i32,
        ent: &Handle<StandardTransient>,
        ach: &mut Handle<InterfaceCheck>,
        merge: bool,
    ) -> Result<(), Failure> {
        if !model.has_semantic_checks() {
            return self.fill_check(ent, &self.theshare, ach);
        }
        if merge {
            ach.get_messages(&model.check(i, false));
        } else {
            *ach = model.check(i, false);
        }
        Ok(())
    }
    /// The global check (static, then of the protocol, then dynamic) of the model.
    fn global_checked(&self, model: &dyn InterfaceInterfaceModel) -> Handle<InterfaceCheck> {
        let mut globch = model.global_check(true);
        if let Some(protocol) = model.protocol().get() {
            protocol.global_check(&self.theshare.graph(), &mut globch);
        }
        model.verify_check(&mut globch);
        globch
    }
    /// `CompleteCheckList`: the global check and every entity check with messages.
    pub fn complete_check_list(&mut self) -> Result<InterfaceCheckIterator, Failure> {
        self.thestat = 3;
        let (model, mut res) = self.new_result();
        let globch = self.global_checked(live_model(&model)?);
        if globch.has_failed() || globch.has_warnings() {
            res.add(&globch, 0);
        }
        if globch.has_failed() {
            self.thestat |= 12;
        }
        self.run_list(&model, res, 12, |tool, live, i, run| {
            run.ach.clear();
            run.ent = live.value(i)?;
            run.ach.set_entity(&run.ent);
            if live.is_report_entity(i, false) {
                run.ach = live.report_entity(i, false).check(); // INCLUT Unknown
                if run.ach.has_failed() {
                    run.res.add(&run.ach, i); // FAIL : no semantic Check
                    run.ach = Handle::new(InterfaceCheck::new());
                    tool.thestat |= 12;
                    return Ok(());
                }
            }
            let ent = run.ent.clone();
            tool.semantic_check(live, i, &ent, &mut run.ach, true)?;
            if run.ach.has_failed() || run.ach.has_warnings() {
                run.res.add(&run.ach, i);
                run.ach = Handle::new(InterfaceCheck::new());
            }
            Ok(())
        })
    }
    /// `CheckList`: the global and entity checks with fails.
    pub fn check_list(&mut self) -> Result<InterfaceCheckIterator, Failure> {
        self.thestat = 3;
        let (model, mut res) = self.new_result();
        let globch = self.global_checked(live_model(&model)?);
        if globch.has_failed() {
            self.thestat |= 12;
            res.add(&globch, 0);
        }
        self.run_list(&model, res, 12, |tool, live, i, run| {
            if live.is_report_entity(i, false) {
                run.ach = live.report_entity(i, false).check();
            } else {
                run.ent = live.value(i)?;
                run.ach.clear();
                run.ach.set_entity(&run.ent);
                let ent = run.ent.clone();
                tool.semantic_check(live, i, &ent, &mut run.ach, false)?;
            }
            if run.ach.has_failed() {
                tool.thestat |= 12;
                run.res.add(&run.ach, i);
            }
            Ok(())
        })
    }
    /// `AnalyseCheckList`: the checks of the report entities (the reading).
    pub fn analyse_check_list(&mut self) -> Result<InterfaceCheckIterator, Failure> {
        self.thestat = 2;
        let (model, res) = self.new_result();
        self.run_list(&model, res, 8, |tool, live, i, run| {
            if !live.is_report_entity(i, false) {
                return Ok(());
            }
            run.ach = live.report_entity(i, false).check();
            if run.ach.has_failed() || run.ach.has_warnings() {
                tool.thestat |= 8;
                run.res.add(&run.ach, i);
            }
            Ok(())
        })
    }
    /// `VerifyCheckList`: the semantic checks of the entities not in error.
    pub fn verify_check_list(&mut self) -> Result<InterfaceCheckIterator, Failure> {
        self.thestat = 1;
        let (model, res) = self.new_result();
        self.run_list(&model, res, 4, |tool, live, i, run| {
            if live.is_error_entity(i) {
                return Ok(());
            }
            run.ent = live.value(i)?;
            run.ach.clear();
            run.ach.set_entity(&run.ent);
            let ent = run.ent.clone();
            tool.semantic_check(live, i, &ent, &mut run.ach, false)?;
            if run.ach.has_failed() || run.ach.has_warnings() {
                tool.thestat |= 4;
                run.res.add(&run.ach, i);
            }
            Ok(())
        })
    }
    /// `WarningCheckList`: the checks with warnings and no fail. Native quirk kept: the
    /// running check gets the entity of the previous step before its own.
    pub fn warning_check_list(&mut self) -> Result<InterfaceCheckIterator, Failure> {
        self.thestat = 3;
        let (model, res) = self.new_result();
        self.run_list(&model, res, 12, |tool, live, i, run| {
            run.ach.clear();
            run.ach.set_entity(&run.ent);
            if live.is_report_entity(i, false) {
                let rep = live.report_entity(i, false);
                if rep.is_error() {
                    tool.thestat |= 12;
                    return Ok(());
                }
                run.ach = rep.check();
            }
            run.ent = live.value(i)?;
            let ent = run.ent.clone();
            tool.semantic_check(live, i, &ent, &mut run.ach, false)?;
            if run.ach.has_failed() {
                tool.thestat |= 12;
            } else if run.ach.has_warnings() {
                run.res.add(&run.ach, i);
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_norm::{fake_chain, fake_entity};
    use super::super::{
        InterfaceCheckStatus, InterfaceGraph, InterfaceHGraph, InterfaceReportEntity,
    };
    use super::*;

    // OCCT 3d097a0 keeps errh process-wide; independent tests must not race its mode.
    fn run_check_tool_child(name: &str) -> bool {
        const CASE_ENV: &str = "RUSTYCAD_CHECK_TOOL_CHILD_CASE";
        if std::env::var(CASE_ENV).ok().as_deref() == Some(name) {
            return false;
        }
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", name, "--test-threads=1", "--nocapture"])
            .env(CASE_ENV, name)
            .output()
            .unwrap();
        assert!(
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains("test result: ok. 1 passed;"),
            "isolated case {name} failed: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        true
    }

    #[test]
    fn lists_of_the_fake_model() {
        if run_check_tool_child("interface::check_tool_lists::tests::lists_of_the_fake_model") {
            return;
        }
        let (model, _) = fake_chain();
        let mut tool = InterfaceCheckTool::new(&model).unwrap();
        // a and b share nothing: warnings.
        let complete = tool.complete_check_list().unwrap();
        assert_eq!(complete.items().len(), 2);
        assert_eq!(complete.status(), InterfaceCheckStatus::CheckWarning);
        assert!(tool.check_list().unwrap().items().is_empty());
        assert_eq!(tool.verify_check_list().unwrap().items().len(), 2);
        assert_eq!(tool.warning_check_list().unwrap().items().len(), 2);
        assert!(tool.analyse_check_list().unwrap().items().is_empty());
        tool.check_success(true).unwrap();
        assert_eq!(tool.check(1).unwrap().nb_warnings(), 1);
        assert_eq!(tool.unknown_entities().unwrap().nb_entities(), 0);
        let mut out = Vec::new();
        tool.print_check(&tool.check(1).unwrap(), &mut out).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            " Warning Messages : 1 :\nno shared entity\n"
        );
        let mut list = Vec::new();
        tool.print_list(&complete, &mut list).unwrap();
        assert!(!list.is_empty());
    }

    #[test]
    fn failures_and_reports() {
        if run_check_tool_child("interface::check_tool_lists::tests::failures_and_reports") {
            return;
        }
        let (model, ents) = fake_chain();
        model.add_entity(&fake_entity("raise", &[])).unwrap();
        let mut tool =
            InterfaceCheckTool::from_graph(&InterfaceGraph::new(&model, true).unwrap()).unwrap();
        let checks = tool.check_list().unwrap();
        assert_eq!(checks.items().len(), 1);
        let text = checks.check(5).c_fail(1, true).unwrap();
        assert_eq!(
            text.as_bytes(),
            b"** Exception Raised during Check : Standard_Failure **"
        );
        assert!(tool.check_success(false).is_err());
        let check = Handle::new(InterfaceCheck::new());
        check.add_fail_text("bad", "");
        model
            .set_report_entity(
                1,
                &Handle::new(InterfaceReportEntity::with_check(&check, &ents[0])),
            )
            .unwrap();
        let hgraph = Handle::new(InterfaceHGraph::new(&model, true).unwrap());
        let mut by_hgraph = InterfaceCheckTool::from_h_graph(&hgraph).unwrap();
        assert_eq!(by_hgraph.analyse_check_list().unwrap().items().len(), 1);
        let err = by_hgraph.check_success(true).err().unwrap();
        assert_eq!(err.exception_type(), "Interface_CheckFailure");
        let with_protocol = InterfaceCheckTool::with_protocol(&model, &model.protocol()).unwrap();
        assert_eq!(with_protocol.check(5).unwrap().nb_fails(), 1);
    }
}
