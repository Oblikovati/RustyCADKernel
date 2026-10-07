// Rust port of DataExchange/TKXSBase/Interface/Interface_FileReaderTool.cxx (SetEntities,
// LoadModel, LoadedEntity) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The loading of a model by an `Interface_FileReaderTool`. The native `try`/`catch`
//! blocks are the `Err` results of the reading members; the messages are the XSTEP
//! catalog messages of the native code.
use super::{
    InterfaceCheck, InterfaceFileReaderDataParams, InterfaceFileReaderTool,
    InterfaceFileReaderToolOps, InterfaceInterfaceModel, InterfaceModelEntities,
    InterfaceModelReports, InterfaceReportEntity,
};
use rustycad_tkernel::{
    message::{Gravity, Messenger, Msg},
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};
use std::fmt::Write as _;
use std::sync::Arc;

/// Sends the catalog message `key`, with its arguments, as information.
fn send_info(
    messenger: &Option<Arc<Messenger>>,
    key: &str,
    args: impl FnOnce(&mut Msg) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let Some(target) = messenger else {
        return Ok(());
    };
    let mut msg = Msg::from_key(&AsciiString::new(key));
    args(&mut msg)?;
    target.send_extended(msg.value(), Gravity::Info);
    Ok(())
}

/// The string label of an entity in a model, for the messages.
fn label_of(
    model: &Handle<dyn InterfaceInterfaceModel>,
    ent: &Handle<StandardTransient>,
) -> Handle<rustycad_tkernel::tcollection::HAsciiString> {
    model.get().map(|m| m.string_label(ent)).unwrap_or_default()
}

/// `Msg.Arg(label)`: a null label (dereferenced natively) is an empty text.
fn arg_label(
    msg: &mut Msg,
    label: &Handle<rustycad_tkernel::tcollection::HAsciiString>,
) -> Result<(), Failure> {
    match label.get() {
        Some(text) => msg.arg_h_ascii(text).map(|_| ()),
        None => msg.arg_text(b"").map(|_| ()),
    }
}

/// Whether a failure is an `OSD_Exception` or an `OSD_Signal` (a fatal read error).
fn is_osd_failure(failure: &Failure) -> bool {
    failure.exception_type().starts_with("OSD_")
}

/// The loading members of `Interface_FileReaderTool`.
pub trait InterfaceFileReaderToolLoad: InterfaceFileReaderTool {
    /// `SetEntities`: recognises every record and binds its empty entity; an
    /// unrecognised record gets the unknown entity, and a record with messages a report.
    fn set_entities(&self) -> Result<(), Failure> {
        let reader = self.data();
        {
            let mut state = self.reader_tool_base().state.write();
            state.thenbreps = 0;
            state.thenbrep0 = 0;
        }
        let mut num = reader.find_next_record(0);
        while num > 0 {
            let mut newent = Handle::default();
            let mut ach = Handle::new(InterfaceCheck::new());
            let recognized = self.recognize(num, &mut ach, &mut newent)?;
            if !recognized {
                newent = self.unknown_entity()?;
            }
            if !recognized || (ach.nb_fails() + ach.nb_warnings() > 0 && newent.is_some()) {
                let report = Handle::new(InterfaceReportEntity::with_check(&ach, &newent));
                self.store_report(num, &report.cast(), true)?;
            }
            reader.bind_entity(num, &newent)?;
            num = reader.find_next_record(num);
        }
        Ok(())
    }
    /// Records a report for record `num` (`thereports` is created with the records);
    /// `count_both` also counts it in the initial reports (`thenbrep0`).
    fn store_report(
        &self,
        num: i32,
        report: &Handle<StandardTransient>,
        count_both: bool,
    ) -> Result<(), Failure> {
        let records = self.data().nb_records();
        let mut state = self.reader_tool_base().state.write();
        let reports = state
            .thereports
            .get_or_insert_with(|| vec![Handle::default(); (records + 1).max(1) as usize]);
        let slot = usize::try_from(num)
            .ok()
            .filter(|i| *i >= 1)
            .and_then(|i| reports.get_mut(i));
        let slot = slot.ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::out_of_range(&format!(
                "Interface_FileReaderTool: record {num}; expected [1, {records}]"
            ))
        })?;
        *slot = report.clone();
        state.thenbreps += 1;
        if count_both {
            state.thenbrep0 += 1;
        }
        Ok(())
    }
    /// `LoadedEntity(num)`: analyses record `num` into its bound entity, adds it to the
    /// model and records its report when it has messages (a failed entity gets the
    /// unknown entity as content).
    fn loaded_entity(&self, num: i32) -> Result<Handle<StandardTransient>, Failure> {
        let reader = self.data();
        let anent = reader.bound_entity(num);
        let (mut ach, mut irep, mut rep) = self.initial_report(num, &anent);
        self.trace_unknown(&anent)?;
        self.analyse_record(num, &anent, &mut ach)?;
        let model = self.model();
        model.add_entity(&anent)?;
        let mut nbf = ach.nb_fails();
        if nbf + ach.nb_warnings() > 0 {
            rep = Handle::new(InterfaceReportEntity::with_check(&ach, &anent));
            if irep == 0 {
                irep = num;
                self.store_report(irep, &rep.cast(), false)?;
            } else {
                self.store_report_again(irep, &rep.cast())?;
            }
            self.trace_check(&ach)?;
        }
        if reader.is_error_load() {
            nbf = i32::from(reader.reset_error_load());
        }
        if nbf > 0 {
            let undef = self.unknown_entity()?;
            self.analyse_record(num, &undef, &mut ach)?;
            if let Some(report) = rep.get() {
                report.set_content(&undef);
            }
        }
        Ok(anent)
    }
    /// The check and report of record `num` before its analysis (from `SetEntities`).
    fn initial_report(
        &self,
        num: i32,
        anent: &Handle<StandardTransient>,
    ) -> (Handle<InterfaceCheck>, i32, Handle<InterfaceReportEntity>) {
        let state = self.reader_tool_base().state.read();
        let fresh = Handle::new(InterfaceCheck::with_entity(anent));
        if state.thenbrep0 <= 0 {
            return (fresh, 0, Handle::default());
        }
        let found = state
            .thereports
            .as_ref()
            .and_then(|r| r.get(num as usize).cloned())
            .unwrap_or_default();
        let rep = found.cast::<InterfaceReportEntity>();
        match rep.get() {
            Some(live) => (live.check(), num, rep.clone()),
            None => (fresh, 0, rep),
        }
    }
    /// Replaces the report of a record already counted.
    fn store_report_again(
        &self,
        irep: i32,
        report: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        let mut state = self.reader_tool_base().state.write();
        if let Some(slot) = state
            .thereports
            .as_mut()
            .and_then(|r| r.get_mut(irep as usize))
        {
            *slot = report.clone();
        }
        Ok(())
    }
    /// The "XSTEP_22" message of an unknown entity at trace level 2.
    fn trace_unknown(&self, anent: &Handle<StandardTransient>) -> Result<(), Failure> {
        if self.trace_level() < 2
            || !self
                .protocol()
                .get()
                .is_some_and(|p| p.is_unknown_entity(anent))
        {
            return Ok(());
        }
        let label = label_of(&self.model(), anent);
        send_info(&self.messenger(), "XSTEP_22", |msg| arg_label(msg, &label))
    }
    /// The print of the check at trace level 2.
    fn trace_check(&self, ach: &Handle<InterfaceCheck>) -> Result<(), Failure> {
        let Some(messenger) = self.messenger().filter(|_| self.trace_level() >= 2) else {
            return Ok(());
        };
        let mut text = Vec::new();
        ach.print(&mut text, 2, 1)
            .map_err(|e| Failure::new(&format!("Interface_FileReaderTool: print: {e}")))?;
        let _ = messenger
            .send_info()
            .write_str(&String::from_utf8_lossy(&text));
        Ok(())
    }
    /// `LoadModel(amodel)`: reads the header, loads every entity (an entity whose analysis
    /// fails gets a report with the unknown entity as content, an OSD failure ends the
    /// loading), records the reports in the model and ends the reading.
    fn load_model(&self, amodel: &Handle<dyn InterfaceInterfaceModel>) -> Result<(), Failure> {
        let messenger = self.messenger();
        self.set_model(amodel);
        self.guarded(self.begin_read(amodel), &messenger)?;
        let reader = self.data();
        amodel.reservate(reader.nb_entities());
        let mut num0 = reader.find_next_record(0);
        while num0 > 0 {
            match self.load_entities_from(num0, amodel, &messenger) {
                Ok(()) => num0 = 0,
                Err((num, failure)) => {
                    num0 = reader.find_next_record(num); //:g9 abv 28 May 98: no infinite cycle
                    if !self.recover_entity(num, &failure, amodel, &messenger)? {
                        return Ok(());
                    }
                }
            }
        }
        self.record_reports(amodel, &messenger)?;
        self.guarded(self.end_read(amodel), &messenger)
    }
    /// The `try` block of `LoadModel`: the entities from record `num0`; a failure carries
    /// the record that raised it.
    fn load_entities_from(
        &self,
        num0: i32,
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        messenger: &Option<Arc<Messenger>>,
    ) -> Result<(), (i32, Failure)> {
        let reader = self.data();
        let mut num = num0;
        while num > 0 {
            let anent = self.loaded_entity(num).map_err(|f| (num, f))?;
            if anent.is_null() {
                send_info(messenger, "XSTEP_21", |msg| {
                    msg.arg_integer(amodel.nb_entities()).map(|_| ())
                })
                .map_err(|f| (num, f))?;
            }
            num = reader.find_next_record(num);
        }
        Ok(())
    }
    /// The `catch` block of `LoadModel`; false when the loading must stop.
    fn recover_entity(
        &self,
        num: i32,
        failure: &Failure,
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        messenger: &Option<Arc<Messenger>>,
    ) -> Result<bool, Failure> {
        let ierr = if is_osd_failure(failure) { 2 } else { 0 };
        let anent = self.data().bound_entity(num);
        if anent.is_null() && self.trace_level() > 0 {
            send_info(messenger, "XSTEP_21", |msg| {
                msg.arg_integer(amodel.nb_entities() + 1).map(|_| ())
            })?;
            return Ok(true);
        }
        let mut ach = Handle::new(InterfaceCheck::with_entity(&anent));
        let label = label_of(amodel, &anent);
        let mut msg278 = Msg::from_key(&AsciiString::new("XSTEP_278"));
        arg_label(&mut msg278, &label)?;
        ach.send_fail(&msg278);
        if ierr == 2 {
            send_info(messenger, "XSTEP_22", |msg| arg_label(msg, &label))?;
            return Ok(false);
        }
        if self.trace_level() > 0 {
            send_info(messenger, "XSTEP_23", |msg| {
                msg.arg_integer(num).map(|_| ())
            })?;
        }
        self.reader_tool_base().state.write().thenbreps += 1;
        let rep = Handle::new(InterfaceReportEntity::with_check(&ach, &anent));
        let undef = self.unknown_entity()?;
        self.analyse_record(num, &undef, &mut ach)?;
        rep.set_content(&undef);
        self.store_report(num, &rep.cast(), false)?;
        amodel.add_entity(&anent)?; // not done by LoadedEntity ...
        Ok(true)
    }
    /// The reports recorded during the reading go to the model.
    fn record_reports(
        &self,
        amodel: &Handle<dyn InterfaceInterfaceModel>,
        messenger: &Option<Arc<Messenger>>,
    ) -> Result<(), Failure> {
        let (reports, nbreps) = {
            let state = self.reader_tool_base().state.read();
            (state.thereports.clone(), state.thenbreps)
        };
        let Some(reports) = reports else {
            return Ok(());
        };
        if self.trace_level() > 0 {
            send_info(messenger, "XSTEP_24", |msg| {
                msg.arg_integer(nbreps).map(|_| ())
            })?;
        }
        amodel.reservate(-nbreps - 10);
        self.reader_tool_base().state.write().thenbreps = reports.len() as i32 - 1;
        let reader = self.data();
        for (nr, report) in reports.iter().enumerate().skip(1) {
            if report.is_null() {
                continue;
            }
            let anent = reader.bound_entity(nr as i32);
            amodel.set_report_entity(-amodel.number(&anent), &report.cast())?;
        }
        Ok(())
    }
    /// The error handling of `BeginRead` and `EndRead`: with error handling, a failure is
    /// reported by the "XSTEP_11" message instead of ending the reading.
    fn guarded(
        &self,
        result: Result<(), Failure>,
        messenger: &Option<Arc<Messenger>>,
    ) -> Result<(), Failure> {
        match result {
            Err(_) if self.error_handle() => send_info(messenger, "XSTEP_11", |_| Ok(())),
            other => other,
        }
    }
}

impl<T: InterfaceFileReaderTool + ?Sized> InterfaceFileReaderToolLoad for T {}
