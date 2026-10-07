// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the general commands.

//! The general commands: status, error handler, reading, writing, entities, counters,
//! signatures, data dumps and parameters.
use super::{IFSelectFunctions, Status, emit, session_of};
use crate::if_select::{
    IFSelectPrintCount, IFSelectReturnStatus, IFSelectSelectPointed, IFSelectSessionPilot,
    IFSelectSignCounter, IFSelectSignType,
};
use crate::interface::InterfaceModelEntities;
use rustycad_tkernel::{
    message::Message,
    standard::{Handle, version_info::occt_version_string},
};
use std::fmt::Write as _;

use IFSelectReturnStatus::{RetDone, RetError, RetFail, RetVoid};

/// `funstatus`: the versions (`XSTEP_PROCESSOR_VERSION` and the others of
/// Interface_Version.hxx).
pub(super) fn funstatus(_pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let v = occt_version_string();
    let mut sout = Message::send_info();
    let _ = write!(
        sout,
        "Processor Version : Open CASCADE %s processor {v}\nOL Version        : Open CASCADE {v}\nConfiguration     : {v}\nUL Names          : {v}\n"
    );
    Ok(RetVoid)
}

/// `fun1` (handler): toggles the error handling of the session.
pub(super) fn fun1(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let hand = !ws.error_handle();
    let mut sout = Message::send_info();
    let _ = writeln!(
        sout,
        " --  Mode Catch Error now {}",
        if hand { "Active" } else { "Inactive" }
    );
    ws.set_error_handle(hand);
    Ok(RetDone)
}

/// The message of the status of a read.
fn read_message(arg1: &str, status: IFSelectReturnStatus) -> String {
    match status {
        RetVoid => format!("file:{arg1} gives empty result"),
        RetError => format!("file:{arg1} could not be opened"),
        RetDone => format!("file:{arg1} read"),
        RetFail => format!("file:{arg1} : error while reading"),
        IFSelectReturnStatus::RetStop => format!("file:{arg1} : EXCEPTION while reading"),
    }
}

/// `fun3` (xload, xread): reads a file into the session.
pub(super) fn fun3(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Read/Load : give file name !");
        return Ok(RetError);
    }
    if ws.protocol().is_null() {
        let _ = writeln!(sout, "Protocol not defined");
        return Ok(RetError);
    }
    if ws.work_library().is_null() {
        let _ = writeln!(sout, "WorkLibrary not defined");
        return Ok(RetError);
    }
    let status = ws.read_file(&arg1)?;
    let _ = writeln!(sout, "{}", read_message(&arg1, status));
    if status == RetDone {
        ws.begin_sent_files(true);
    }
    Ok(status)
}

/// `fun4` (writeall): writes the whole model.
pub(super) fn fun4(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    if pilot.nb_words() < 2 {
        let _ = writeln!(Message::send_info(), "Write All : give file name !");
        return Ok(RetError);
    }
    ws.send_all(&pilot.arg(1)?, false)
}

/// `fun5` (writesel): writes the entities a list gives.
pub(super) fn fun5(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(sout, "Write Selected : give file name + givelist !");
        return Ok(RetError);
    }
    let part = String::from_utf8_lossy(pilot.command_part(2).as_bytes()).into_owned();
    let Some(result) = IFSelectFunctions::give_list(&ws, &part, "")? else {
        let _ = writeln!(sout, "No entity selected");
        return Ok(RetError);
    };
    let _ = writeln!(sout, "Nb Entities selected : {}", result.len());
    let sp = IFSelectSelectPointed::new();
    sp.set_list(Some(&result));
    ws.send_selected(&pilot.arg(1)?, &Handle::new(sp).cast(), false)
}

/// `fun6` (writeent, writent): writes the entities of given numbers.
pub(super) fn fun6(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    if argc < 3 {
        let _ = writeln!(sout, "Write Entitie(s) : give file name + n0s entitie(s)!");
        return Ok(RetError);
    }
    let mut ko = 0;
    let sp = IFSelectSelectPointed::new();
    for ia in 2..argc {
        let arg = pilot.arg(ia)?;
        let id = pilot.number(&arg)?;
        if id <= 0 {
            let _ = writeln!(sout, "Not an entity number:{arg}");
            ko += 1;
        } else if sp.add(&ws.starting_entity(id)) {
            let _ = writeln!(sout, "Added:no.{id}");
        } else {
            let _ = writeln!(sout, " Fail Add n0.{id}");
            ko += 1;
        }
    }
    if ko > 0 {
        let _ = writeln!(sout, "{ko} bad arguments, abandon");
        return Ok(RetError);
    }
    ws.send_selected(&pilot.arg(1)?, &Handle::new(sp).cast(), false)
}

/// `fun7` (elabel): the label of an entity.
pub(super) fn fun7(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Give entity number");
        return Ok(RetError);
    }
    if !ws.has_model() {
        let _ = writeln!(sout, "No loaded model, abandon");
        return Ok(RetError);
    }
    let nument = ws.number_from_label(&arg1, 0);
    if nument <= 0 || nument > ws.nb_starting_entities() {
        let _ = writeln!(sout, "Not a suitable number: {arg1}");
        return Ok(RetError);
    }
    let mut label = Vec::new();
    let _ = ws
        .model()
        .print_label(&ws.starting_entity(nument), &mut label);
    let _ = write!(sout, "N0.{nument} ->Label in Model : ");
    emit(&mut sout, &label);
    let _ = writeln!(sout);
    Ok(RetVoid)
}

/// `fun8` (enum): the entities whose label ends with the argument.
pub(super) fn fun8(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Give label to search");
        return Ok(RetError);
    }
    if !ws.has_model() {
        let _ = writeln!(sout, "No loaded model, abandon");
        return Ok(RetError);
    }
    let model = ws.model();
    let _ = writeln!(sout, " **  Search Entity Number for Label : {arg1}");
    let mut cnt = 0;
    let mut i = model.next_number_for_label(&arg1, 0, false);
    while i != 0 {
        cnt += 1;
        let mut found = Vec::new();
        let _ = model.print(&model.value(i)?, &mut found, 0);
        let _ = write!(sout, " **  Found n0/id:");
        emit(&mut sout, &found);
        let _ = writeln!(sout);
        i = model.next_number_for_label(&arg1, i, false);
    }
    let _ = match cnt {
        0 => writeln!(sout, " **  No Match"),
        1 => writeln!(sout, " **  1 Match"),
        _ => writeln!(sout, "{cnt} Matches"),
    };
    Ok(RetVoid)
}

/// `fun9` (listtypes): the count of entities by type.
pub(super) fn fun9(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut signtype = ws.sign_type();
    if signtype.is_null() {
        signtype = Handle::new(IFSelectSignType::new(false)).cast();
    }
    let counter = Handle::new(IFSelectSignCounter::with_signature(
        &signtype, false, false,
    )?);
    pilot.execute_counter(&counter, 1, IFSelectPrintCount::CountByItem)
}
