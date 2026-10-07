// Rust port of DataExchange/TKXSBase/XSControl/XSControl_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the transfer reader and writer.

//! The commands of the transfer reader (`trecord`, `trstat`, `trbegin`, `tread`,
//! `trtp`, `tptr`, `selecttransfer`) and of the transfer writer (`twmode`, `twstat`).
use super::{Status, emit, int_arg, reader_of, xs_session};
use crate::if_select::IFSelectReturnStatus::{RetDone, RetError, RetVoid};
use crate::if_select::{IFSelectFunctions, IFSelectPrintCount, IFSelectSessionPilot};
use crate::interface::{InterfaceCheckStatus, InterfaceInterfaceModel, InterfaceModelEntities};
use crate::xs_control::{XSControlSelectForTransfer, XSControlTransferReader};
use rustycad_tkernel::{
    message::{Message, MessageStream, ProgressRange},
    standard::{Failure, Handle, StandardTransient, exceptions},
};
use std::fmt::Write as _;

/// `trecord`: the results of all the roots recorded, or of the entity `num`.
pub(super) fn trecord(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = xs_session(pilot)?;
    let (mdl, tr) = (ws.model(), ws.transfer_reader());
    let tp = tr
        .get()
        .map(XSControlTransferReader::transient_process)
        .unwrap_or_default();
    let mut sout = Message::send_info();
    let (Some(m), Some(reader), Some(process)) = (mdl.get(), tr.get(), tp.get()) else {
        let _ = writeln!(sout, " init not done");
        return Ok(RetError);
    };
    if pilot.nb_words() == 1 {
        let nb = process.nb_roots();
        let _ = writeln!(sout, " Recording {nb} Roots");
        for i in 1..=nb {
            let recorded = reader.record_result(&process.root(i)?)?;
            let _ = writeln!(
                sout,
                " Root n0.{i}{}",
                if recorded { "" } else { " not recorded" }
            );
        }
        return Ok(RetDone);
    }
    let num = int_arg(pilot, 1)?;
    if num < 1 || num > m.nb_entities() {
        let _ = writeln!(sout, "incorrect number:{num}");
    } else {
        let recorded = reader.record_result(&m.value(num)?)?;
        let _ = writeln!(
            sout,
            " Entity n0.{num}{}",
            if recorded { "" } else { " not recorded" }
        );
    }
    Ok(RetDone)
}

/// `trstat`: the statistics of the transfer reader, or the result of the entity `num`.
pub(super) fn trstat(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let tr = reader_of(pilot)?;
    let mut sout = Message::send_info();
    let Some(reader) = tr.get() else {
        let _ = writeln!(sout, " init not done");
        return Ok(RetError);
    };
    let mdl = reader.model();
    let Some(m) = mdl.get() else {
        let _ = writeln!(sout, " No model");
        return Ok(RetError);
    };
    let _ = writeln!(
        sout,
        " Statistics : FileName : {}",
        String::from_utf8_lossy(reader.file_name().as_bytes())
    );
    if pilot.nb_words() == 1 {
        let mut out = Vec::new();
        let result = reader.print_stats(&mut out, 10, 0);
        emit(&mut sout, &out);
        result?;
        return Ok(RetVoid);
    }
    let arg1 = pilot.arg(1)?;
    let num = int_arg(pilot, 1)?;
    if num < 1 || num > m.nb_entities() {
        let _ = writeln!(sout, " incorrect number:{arg1}");
        return Ok(RetError);
    }
    let ent = m.value(num)?;
    if !reader.is_recorded(&ent)? {
        let _ = writeln!(sout, " Entity {num} not recorded");
        return Ok(RetError);
    }
    entity_result(pilot, reader, m, &ent, &mut sout)
}

/// The entities implied by the result of `ent` and its checks (`trstat num`).
fn entity_result(
    pilot: &IFSelectSessionPilot,
    reader: &XSControlTransferReader,
    m: &dyn InterfaceInterfaceModel,
    ent: &Handle<StandardTransient>,
    sout: &mut MessageStream<'_>,
) -> Status {
    let rm = reader.final_result(ent);
    let list = reader.checked_list(ent, InterfaceCheckStatus::CheckAny, true)?;
    if !list.is_empty() {
        let _ = write!(
            sout,
            " Entities implied by Check/Result :{} i.e.:",
            list.len()
        );
    }
    let mut out = Vec::new();
    for item in &list {
        out.extend_from_slice(b"  ");
        let _ = m.print(item, &mut out, 0);
    }
    out.push(b'\n');
    emit(sout, &out);
    let Some(result) = rm.get() else {
        let _ = writeln!(sout, " no other info");
        return Ok(RetVoid);
    };
    let chl = result.check_list(false, 2)?;
    let mut checks = Vec::new();
    let printed = pilot.session().get().map(|ws| {
        ws.print_check_list(&mut checks, &chl, false, IFSelectPrintCount::EntitiesByItem)
    });
    emit(sout, &checks);
    printed.unwrap_or(Ok(()))?;
    Ok(RetVoid)
}

/// `trbegin`: the transfer begun (the reader initialised first without one, or for
/// an argument starting with 'i').
pub(super) fn trbegin(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = xs_session(pilot)?;
    let mut tr = ws.transfer_reader();
    let init = tr.is_null() || (pilot.nb_words() > 1 && pilot.arg(1)?.starts_with('i'));
    if init {
        ws.init_transfer_reader(0)?;
        tr = ws.transfer_reader();
    }
    let Some(reader) = tr.get() else {
        let _ = writeln!(Message::send_info(), " init not done or failed");
        return Ok(RetError);
    };
    reader.begin_transfer();
    Ok(RetDone)
}

/// A list of entities, `Standard_NullObject` for none (a native null list
/// dereference).
fn live_list(
    list: Option<Vec<Handle<StandardTransient>>>,
) -> Result<Vec<Handle<StandardTransient>>, Failure> {
    list.ok_or_else(|| exceptions::null_object("XSControl_Functions::tread: no list of entities"))
}

/// `tread`: all the roots ("xst-model-roots"), or the list the command part 1 gives,
/// transferred by the transfer reader.
pub(super) fn tread(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let tr = reader_of(pilot)?;
    let mut sout = Message::send_info();
    let Some(reader) = tr.get() else {
        let _ = writeln!(sout, " init not done");
        return Ok(RetError);
    };
    if reader.model().is_null() {
        let _ = writeln!(sout, " No model");
        return Ok(RetError);
    }
    let session = pilot.session();
    let ws = session
        .get()
        .ok_or_else(|| exceptions::null_object("XSControl_Functions::tread: no session"))?;
    let none = ProgressRange::default();
    if pilot.nb_words() < 2 {
        let sel = ws.named_item("xst-model-roots");
        if sel.is_null() {
            let _ = writeln!(sout, "Select Roots absent");
            return Ok(RetError);
        }
        let list = live_list(ws.give_list(&sel)?)?;
        let _ = writeln!(
            sout,
            " Transferring all roots i.e. : {}",
            reader.transfer_list(&list, true, &none)?
        );
        return Ok(RetDone);
    }
    let part = String::from_utf8_lossy(pilot.command_part(1).as_bytes()).into_owned();
    let list = live_list(IFSelectFunctions::give_list(&session, &part, "")?)?;
    let _ = writeln!(sout, " Transfer of {} entities", list.len());
    let nb = reader.transfer_list(&list, true, &none)?;
    let _ = writeln!(sout, " Gives {nb} results");
    Ok(RetDone)
}

/// `trtp`: a message when the transfer reader or its process is missing.
pub(super) fn trtp(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let tr = reader_of(pilot)?;
    let mut sout = Message::send_info();
    match tr.get() {
        None => {
            let _ = writeln!(sout, " No TransferReader");
        }
        Some(reader) if reader.transient_process().is_null() => {
            let _ = writeln!(sout, " Transfer Reader without Process");
        }
        Some(_) => {}
    }
    Ok(RetVoid)
}

/// `tptr`: the transfer reader initialised in mode 3 (the roots recorded).
pub(super) fn tptr(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    xs_session(pilot)?.init_transfer_reader(3)?;
    Ok(RetDone)
}

/// `twmode`: the modes of the writing and the actual one; with an argument, the mode
/// changed (with a warning when the norm does not support it). `Standard_NullObject`
/// without norm or writer.
pub(super) fn twmode(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = xs_session(pilot)?;
    let (tw, control) = (ws.transfer_writer(), ws.norm_adaptor());
    let (Some(writer), Some(ctl)) = (tw.get(), control.get()) else {
        return Err(exceptions::null_object(
            "XSControl_Functions::twmode: no norm or no transfer writer",
        ));
    };
    let mut sout = Message::send_info();
    if let Some((modemin, modemax)) = ctl.mode_write_bounds(true) {
        let _ = writeln!(sout, "Write Mode : allowed values  {modemin} to {modemax}");
        for modd in modemin..=modemax {
            let _ = writeln!(
                sout,
                "{modd}\t: {}",
                String::from_utf8_lossy(ctl.mode_write_help(modd, true).as_bytes())
            );
        }
    }
    let _ = writeln!(sout, "Write Mode : actual = {}", writer.transfer_mode());
    if pilot.nb_words() <= 1 {
        return Ok(RetVoid);
    }
    let _ = writeln!(sout, "New value -> {}", pilot.arg(1)?);
    let mode = int_arg(pilot, 1)?;
    writer.set_transfer_mode(mode);
    if !ctl.is_mode_write(mode, true) {
        let _ = writeln!(sout, "Warning : this new value is not supported");
    }
    Ok(RetDone)
}

/// `twstat`: the statistics of the transfer process of the writing.
pub(super) fn twstat(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let fp = xs_session(pilot)?
        .transfer_writer()
        .get()
        .map(|w| w.finder_process())
        .unwrap_or_default();
    let mut sout = Message::send_info();
    let Some(process) = fp.get() else {
        let _ = writeln!(sout, "TransferWrite: not defined");
        return Ok(RetVoid);
    };
    let _ = sout.write_str("TransferWrite:");
    let mut out = Vec::new();
    let _ = process.print_stats(1, &mut out);
    emit(&mut sout, &out);
    Ok(RetVoid)
}

/// `selecttransfer`: a selection of the entities the transfer reader recognizes,
/// recorded as an item.
pub(super) fn settransfert(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let tr = xs_session(pilot)?.transfer_reader();
    let sel = Handle::new(XSControlSelectForTransfer::with_reader(&tr));
    Ok(pilot.record_item(&sel.cast()))
}
