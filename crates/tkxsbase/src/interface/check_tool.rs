// Rust port of DataExchange/TKXSBase/Interface/Interface_CheckTool (.hxx, .cxx: the tool,
// FillCheck, Print, Check, CheckSuccess, UnknownEntities)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_CheckTool`: the checks of the entities of a model (from their reports, the
//! semantic checks of the model or the general modules). The check lists are in
//! `check_tool_lists`.
use super::errors::check_failure;
use super::{
    InterfaceCheck, InterfaceCheckIterator, InterfaceEntityIterator, InterfaceGTool,
    InterfaceGraph, InterfaceHGraph, InterfaceInterfaceModel, InterfaceModelEntities,
    InterfaceModelReports, InterfaceProtocol, InterfaceReportEntity, InterfaceShareTool,
};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

/// The native static `errh`: whether `FillCheck` catches what a check raises. The check
/// lists clear it (they catch themselves), `Check` sets it; it is process-wide natively.
pub(super) static ERROR_HANDLING: AtomicBool = AtomicBool::new(true);

/// `raisecheck`: records a caught failure in `ach`; an `OSD_Signal` is raised again.
pub(super) fn raise_check(
    failure: Failure,
    ach: &mut Handle<InterfaceCheck>,
) -> Result<(), Failure> {
    let mess = format!(
        "** Exception Raised during Check : {} **",
        failure.exception_type()
    );
    ach.add_fail_text(&mess, "");
    if failure.exception_type().starts_with("OSD_Signal") || failure.is_kind("OSD_Signal") {
        return Err(failure);
    }
    Ok(())
}

/// `Interface_CheckTool`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceCheckTool;
/// fn unknowns(tool: &InterfaceCheckTool) -> i32 { tool.unknown_entities().map_or(0, |u| u.nb_entities()) }
/// ```
pub struct InterfaceCheckTool {
    pub(super) thegtool: Handle<InterfaceGTool>,
    pub(super) theshare: InterfaceShareTool,
    pub(super) thestat: i32,
}

impl InterfaceCheckTool {
    /// `Interface_CheckTool(model, protocol)`: a new GTool of `protocol`.
    pub fn with_protocol(
        model: &Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<Self, Failure> {
        let nb = live_model(model)?.nb_entities();
        Ok(Self {
            thegtool: Handle::new(InterfaceGTool::with_protocol(protocol, nb)),
            theshare: InterfaceShareTool::with_protocol(model, protocol)?,
            thestat: 0,
        })
    }
    /// `Interface_CheckTool(model)`: the GTool of the model.
    pub fn new(model: &Handle<dyn InterfaceInterfaceModel>) -> Result<Self, Failure> {
        let live = live_model(model)?;
        let gtool = live.gtool();
        let tool = Self {
            thegtool: gtool.clone(),
            theshare: InterfaceShareTool::with_gtool(model, &gtool)?,
            thestat: 0,
        };
        live_gtool(&tool.thegtool)?.reservate(live.nb_entities(), false);
        Ok(tool)
    }
    /// `Interface_CheckTool(graph)`. Native quirk kept: the status is not initialised
    /// natively; it starts at 0 here.
    pub fn from_graph(graph: &InterfaceGraph) -> Result<Self, Failure> {
        let gtool = live_model(graph.model())?.gtool();
        Ok(Self {
            thegtool: gtool,
            theshare: InterfaceShareTool::from_graph(graph)?,
            thestat: 0,
        })
    }
    /// `Interface_CheckTool(hgraph)`.
    pub fn from_h_graph(hgraph: &Handle<InterfaceHGraph>) -> Result<Self, Failure> {
        let gtool = live_model(hgraph.graph().model())?.gtool();
        Ok(Self {
            thegtool: gtool,
            theshare: InterfaceShareTool::from_h_graph(hgraph),
            thestat: 0,
        })
    }
    /// `FillCheck(ent, sh, ach)`: the check of the entity's module (failures recorded in
    /// `ach` when error handling is on), or the check of a report entity; then the share
    /// error of the graph.
    pub fn fill_check(
        &self,
        ent: &Handle<StandardTransient>,
        sh: &InterfaceShareTool,
        ach: &mut Handle<InterfaceCheck>,
    ) -> Result<(), Failure> {
        match live_gtool(&self.thegtool)?.select(ent, false) {
            Some((module, cn)) => {
                let result = module.check_case(cn, ent, sh, ach);
                match result {
                    Err(failure) if ERROR_HANDLING.load(Ordering::SeqCst) => {
                        raise_check(failure, ach)?
                    }
                    other => other?,
                }
            }
            None => {
                let rep = ent.cast::<InterfaceReportEntity>();
                let Some(report) = rep.get() else {
                    return Ok(());
                };
                *ach = report.check();
            }
        }
        if self.theshare.graph().has_share_errors(ent)? {
            ach.add_fail_text("** Shared Items unknown from the containing Model", "");
        }
        Ok(())
    }
    /// `Print(ach, S)`: the fails, then the warnings, with their counts.
    pub fn print_check(&self, ach: &InterfaceCheck, s: &mut dyn Write) -> io::Result<()> {
        let nb = ach.nb_fails();
        if nb > 0 {
            writeln!(s, " Fail Messages : {nb} :")?;
        }
        for text in ach.fails(true) {
            s.write_all(text.string().as_bytes())?;
            s.write_all(b"\n")?;
        }
        let nb = ach.nb_warnings();
        if nb > 0 {
            writeln!(s, " Warning Messages : {nb} :")?;
        }
        for text in ach.warnings(true) {
            s.write_all(text.string().as_bytes())?;
            s.write_all(b"\n")?;
        }
        Ok(())
    }
    /// `Print(list, S)`: the list with the model of the tool, fails and warnings.
    pub fn print_list(&self, list: &InterfaceCheckIterator, s: &mut dyn Write) -> io::Result<()> {
        list.print_with_model(s, &self.theshare.model(), false, 0)
    }
    /// `Check(num)`: the check of entity `num` (error handling on).
    pub fn check(&self, num: i32) -> Result<Handle<InterfaceCheck>, Failure> {
        let model = self.theshare.model();
        let ent = live_model(&model)?.value(num)?;
        let mut ach = Handle::new(InterfaceCheck::with_entity(&ent));
        ERROR_HANDLING.store(true, Ordering::SeqCst);
        self.fill_check(&ent, &self.theshare, &mut ach)?;
        Ok(ach)
    }
    /// `CheckSuccess(reset = false)`: `Interface_CheckFailure` at the first fail of the
    /// global checks or of an entity (not caught: error handling off).
    pub fn check_success(&mut self, reset: bool) -> Result<(), Failure> {
        if reset {
            self.thestat = 0;
        }
        if self.thestat > 3 {
            return Err(check_failure("Interface Model : Global Check")); // already tested with error
        }
        let model = self.theshare.model();
        let live = live_model(&model)?;
        if live.global_check(true).nb_fails() > 0 {
            return Err(check_failure("Interface Model : Global Check"));
        }
        let mut modchk = Handle::new(InterfaceCheck::new());
        live.verify_check(&mut modchk);
        if let Some(protocol) = live.protocol().get() {
            protocol.global_check(&self.theshare.graph(), &mut modchk);
        }
        if modchk.has_failed() {
            return Err(check_failure("Interface Model : Verify Check"));
        }
        if self.thestat == 3 {
            return Ok(()); // everything tested and it passes
        }
        ERROR_HANDLING.store(false, Ordering::SeqCst); // No try/catch, because we precisely raise
        for i in 1..=live.nb_entities() {
            self.check_success_entity(live, i)?;
        }
        Ok(())
    }
    /// One entity of `CheckSuccess`.
    fn check_success_entity(
        &self,
        model: &dyn InterfaceInterfaceModel,
        i: i32,
    ) -> Result<(), Failure> {
        if model.is_error_entity(i) {
            return Err(check_failure(
                "Interface Model : an Entity is recorded as Erroneous",
            ));
        }
        let ent = model.value(i)?;
        // The two native skips test IsErrorEntity, already false here.
        if (self.thestat & 1 != 0) || (self.thestat & 2 != 0 && model.is_error_entity(i)) {
            return Ok(());
        }
        let mut ach = Handle::new(InterfaceCheck::with_entity(&ent));
        self.fill_check(&ent, &self.theshare, &mut ach)?;
        if ach.has_failed() {
            return Err(check_failure(
                "Interface Model : Check on an Entity has Failed",
            ));
        }
        Ok(())
    }
    /// `UnknownEntities`: the entities recorded as unknown.
    pub fn unknown_entities(&self) -> Result<InterfaceEntityIterator, Failure> {
        let model = self.theshare.model();
        let live = live_model(&model)?;
        let mut res = InterfaceEntityIterator::new();
        for i in 1..=live.nb_entities() {
            if live.is_unknown_entity(i) {
                res.get_one_item(&live.value(i)?);
            }
        }
        Ok(res)
    }
}

pub(super) fn live_model(
    model: &Handle<dyn InterfaceInterfaceModel>,
) -> Result<&dyn InterfaceInterfaceModel, Failure> {
    model
        .get()
        .ok_or_else(|| exceptions::null_object("Interface_CheckTool: null model; expected a model"))
}

fn live_gtool(gtool: &Handle<InterfaceGTool>) -> Result<&InterfaceGTool, Failure> {
    gtool.get().ok_or_else(|| {
        exceptions::null_object("Interface_CheckTool: the model has no GTool; expected one")
    })
}
