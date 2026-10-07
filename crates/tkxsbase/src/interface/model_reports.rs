// Rust port of DataExchange/TKXSBase/Interface/Interface_InterfaceModel.cxx (reports and
// checks) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The non-virtual members of `Interface_InterfaceModel` on reports and checks: the
//! syntactic reports (`thereports`), the semantic ones (`therepch`) and the global checks.
use super::errors::interface_mismatch;
use super::interface_model::grow_reports;
use super::{
    InterfaceCheck, InterfaceCheckIterator, InterfaceDataState, InterfaceEntityIterator,
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceReportEntity,
};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use std::sync::OnceLock;

/// `nulch()`: the shared empty check returned for an entity without report.
fn null_check() -> Handle<InterfaceCheck> {
    static NULL_CHECK: OnceLock<Handle<InterfaceCheck>> = OnceLock::new();
    NULL_CHECK
        .get_or_init(|| Handle::new(InterfaceCheck::new()))
        .clone()
}

/// The non-virtual report and check members of `Interface_InterfaceModel`.
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceInterfaceModel, InterfaceModelReports};
/// fn loaded(model: &dyn InterfaceInterfaceModel) -> bool { !model.is_error_entity(1) }
/// ```
pub trait InterfaceModelReports: InterfaceInterfaceModel {
    /// `EntityState(num)`: the load state (syntactic report) then the data state
    /// (semantic report) of an entity.
    fn entity_state(&self, num: i32) -> InterfaceDataState {
        let (syntactic, semantic) = {
            let state = self.model_base().read();
            (
                state.thereports.seek(&num).cloned(),
                state.therepch.seek(&num).cloned(),
            )
        };
        let semantic_state = |rep: Handle<StandardTransient>| {
            let error = rep
                .cast::<InterfaceReportEntity>()
                .get()
                .is_some_and(InterfaceReportEntity::is_error);
            if error {
                InterfaceDataState::DataFail
            } else {
                InterfaceDataState::DataWarning
            }
        };
        let Some(found) = syntactic else {
            return semantic.map_or(InterfaceDataState::StateOK, semantic_state);
        };
        let rep = found.cast::<InterfaceReportEntity>();
        let Some(live) = rep.get() else {
            return InterfaceDataState::StateUnknown;
        };
        if live.is_unknown() {
            return InterfaceDataState::StateUnknown;
        }
        if live.has_new_content() {
            return InterfaceDataState::StateUnloaded;
        }
        if live.is_error() {
            return InterfaceDataState::LoadFail;
        }
        semantic.map_or(InterfaceDataState::LoadWarning, semantic_state)
    }
    /// `IsReportEntity(num, semantic = false)`.
    fn is_report_entity(&self, num: i32, semantic: bool) -> bool {
        let state = self.model_base().read();
        if semantic {
            state.therepch.is_bound(&num)
        } else {
            state.thereports.is_bound(&num)
        }
    }
    /// `ReportEntity(num, semantic = false)`: null when none.
    fn report_entity(&self, num: i32, semantic: bool) -> Handle<InterfaceReportEntity> {
        let state = self.model_base().read();
        let map = if semantic {
            &state.therepch
        } else {
            &state.thereports
        };
        map.seek(&num).map(|r| r.cast()).unwrap_or_default()
    }
    /// `IsErrorEntity(num)`: its syntactic report has fails.
    fn is_error_entity(&self, num: i32) -> bool {
        self.report_entity(num, false)
            .get()
            .is_some_and(InterfaceReportEntity::is_error)
    }
    /// `IsRedefinedContent(num)`: its syntactic report has a new content.
    fn is_redefined_content(&self, num: i32) -> bool {
        self.report_entity(num, false)
            .get()
            .is_some_and(InterfaceReportEntity::has_new_content)
    }
    /// `ClearReportEntity(num)`: removes the syntactic report.
    fn clear_report_entity(&self, num: i32) -> bool {
        self.model_base().write().thereports.unbind(&num)
    }
    /// `SetReportEntity(num, rep)`: the syntactic report of entity `num` (or of `-num`,
    /// or of the concerned entity for 0); `Interface_InterfaceMismatch` when `rep` does
    /// not concern that entity.
    fn set_report_entity(
        &self,
        num: i32,
        rep: &Handle<InterfaceReportEntity>,
    ) -> Result<bool, Failure> {
        let concerned = rep
            .get()
            .map(InterfaceReportEntity::concerned)
            .unwrap_or_default();
        let mismatch = || {
            interface_mismatch(&format!(
                "InterfaceModel : SetReportEntity: the report does not concern entity {num}"
            ))
        };
        let nm = if num != 0 {
            let nm = num.abs();
            if self.value(nm)? != concerned {
                return Err(mismatch());
            }
            nm
        } else {
            let nm = self.number(&concerned);
            if nm == 0 {
                return Err(mismatch());
            }
            nm
        };
        let mut state = self.model_base().write();
        if !state.thereports.is_bound(&nm) {
            grow_reports(&mut state.thereports);
        }
        Ok(nm > 0 && state.thereports.bind(nm, rep.cast()))
    }
    /// `AddReportEntity(rep, semantic = false)`. Native quirk kept: a semantic report goes
    /// to the syntactic map and the reverse.
    fn add_report_entity(&self, rep: &Handle<InterfaceReportEntity>, semantic: bool) -> bool {
        let Some(live) = rep.get() else { return false };
        let ent = live.concerned();
        if ent.is_null() {
            return false;
        }
        let num = self.number(&ent);
        if num == 0 {
            return false;
        }
        let mut state = self.model_base().write();
        let map = if semantic {
            &mut state.thereports
        } else {
            &mut state.therepch
        };
        map.bind(num, rep.cast())
    }
    /// `IsUnknownEntity(num)`.
    fn is_unknown_entity(&self, num: i32) -> bool {
        self.report_entity(num, false)
            .get()
            .is_some_and(InterfaceReportEntity::is_unknown)
    }
    /// `FillSemanticChecks(checks, clear = true)`: the checks of another model are
    /// ignored; the check of number 0 goes to the global semantic check.
    fn fill_semantic_checks(
        &self,
        checks: &InterfaceCheckIterator,
        clear: bool,
    ) -> Result<(), Failure> {
        let model = checks.model();
        if model.is_some() && model != self.model_base().this() {
            return Ok(());
        }
        if clear {
            let semantic = {
                let mut state = self.model_base().write();
                state.therepch.clear(false);
                state.thechecksem.clone()
            };
            semantic.clear();
        }
        let items = checks.items();
        {
            let mut state = self.model_base().write();
            let size = state.therepch.len() + items.len() + 2;
            state.therepch.resize(size);
        }
        for (num, ach) in items {
            self.fill_one_semantic_check(num, &ach)?;
        }
        self.model_base().write().haschecksem = true;
        Ok(())
    }
    /// One check of `FillSemanticChecks`.
    fn fill_one_semantic_check(
        &self,
        num: i32,
        ach: &Handle<InterfaceCheck>,
    ) -> Result<(), Failure> {
        if num == 0 {
            let semantic = self.model_base().read().thechecksem.clone();
            if let Some(check) = ach.get() {
                semantic.get_messages(check);
            }
            return Ok(());
        }
        let ent = self.value(num)?;
        let rep = Handle::new(InterfaceReportEntity::with_check(ach, &ent));
        self.model_base().write().therepch.bind(num, rep.cast());
        Ok(())
    }
    /// `HasSemanticChecks`.
    fn has_semantic_checks(&self) -> bool {
        self.model_base().read().haschecksem
    }
    /// `Check(num, syntactic)`: the global check for 0, else the check of the report
    /// (a shared empty check without report).
    fn check(&self, num: i32, syntactic: bool) -> Handle<InterfaceCheck> {
        if num == 0 {
            return self.global_check(syntactic);
        }
        let rep = self.report_entity(num, !syntactic);
        rep.get()
            .map_or_else(null_check, InterfaceReportEntity::check)
    }
    /// `Reports(semantic = false)`: the reports, in the native map order.
    fn reports(&self, semantic: bool) -> InterfaceEntityIterator {
        let reports: Vec<Handle<StandardTransient>> = {
            let state = self.model_base().read();
            let map = if semantic {
                &state.therepch
            } else {
                &state.thereports
            };
            map.values().cloned().collect()
        };
        let mut iter = InterfaceEntityIterator::new();
        reports.iter().for_each(|rep| iter.add_item(rep));
        iter
    }
    /// `Redefineds`: the syntactic reports with a new content.
    fn redefineds(&self) -> InterfaceEntityIterator {
        let mut iter = InterfaceEntityIterator::new();
        for rep in self.reports(false).content() {
            if rep
                .cast::<InterfaceReportEntity>()
                .get()
                .is_some_and(InterfaceReportEntity::has_new_content)
            {
                iter.add_item(&rep);
            }
        }
        iter
    }
    /// `GlobalCheck(syntactic = true)`.
    fn global_check(&self, syntactic: bool) -> Handle<InterfaceCheck> {
        let state = self.model_base().read();
        if syntactic {
            state.thecheckstx.clone()
        } else {
            state.thechecksem.clone()
        }
    }
    /// `SetGlobalCheck(ach)`: the syntactic global check.
    fn set_global_check(&self, ach: &Handle<InterfaceCheck>) {
        self.model_base().write().thecheckstx = ach.clone();
    }
}

impl<T: InterfaceInterfaceModel + ?Sized> InterfaceModelReports for T {}
