// Rust port of DataExchange/TKXSBase/XSControl/XSControl_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the norm, the model and the
// transfer processes.

//! The commands of the norm (`xinit`, `xnorm`, `newmodel`) and of the transfer
//! processes (`tpclear`, `tpstat`, `tpent`, `tpitem`).
use super::{Status, emit, int_arg, reader_of, word_byte, xs_session};
use crate::if_select::IFSelectReturnStatus::{RetDone, RetError, RetFail, RetVoid};
use crate::if_select::{IFSelectFunctions, IFSelectSessionPilot};
use crate::interface::InterfaceModelEntities;
use crate::xs_control::{XSControlControllerBase, XSControlTransferReader};
use rustycad_tkernel::{message::Message, standard::Handle};
use std::fmt::Write as _;

/// The text of an `AsciiString`.
fn text(s: &rustycad_tkernel::tcollection::AsciiString) -> String {
    String::from_utf8_lossy(s.as_bytes()).into_owned()
}

/// `xinit`: the norm selected (done or fail), else the selected norm shown.
pub(super) fn xinit(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = xs_session(pilot)?;
    if pilot.nb_words() > 1 {
        return Ok(if ws.select_norm(&pilot.arg(1)?)? {
            RetDone
        } else {
            RetFail
        });
    }
    let _ = writeln!(
        Message::send_info(),
        "Selected Norm:{}",
        text(&ws.selected_norm(false))
    );
    Ok(RetVoid)
}

/// `xnorm`: the current norm shown; with an argument, the norm recorded under it set.
pub(super) fn xnorm(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let ws = xs_session(pilot)?;
    let mut sout = Message::send_info();
    let _ = writeln!(
        sout,
        "{}",
        if argc == 1 {
            "Current Norm. xnorm newnorm to change"
        } else {
            "Current Norm :"
        }
    );
    match ws.norm_adaptor().get() {
        None => {
            let _ = writeln!(sout, "no norm currently defined");
        }
        Some(c) => {
            let _ = write!(
                sout,
                "  Long  Name (complete) : {}\n  Short name (resource) : {}\n",
                text(&c.name(false)),
                text(&c.name(true))
            );
        }
    }
    if argc == 1 {
        return Ok(RetVoid);
    }
    let arg1 = pilot.arg(1)?;
    let control = XSControlControllerBase::recorded(&arg1);
    let Some(live) = control.get() else {
        let _ = writeln!(sout, " No norm named : {arg1}");
        return Ok(RetError);
    };
    ws.set_controller(&control)?;
    let _ = writeln!(sout, "new norm : {}", text(&live.name(false)));
    Ok(RetDone)
}

/// `newmodel`: a new model for the session (a message when none).
pub(super) fn newmodel(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    if xs_session(pilot)?.new_model()?.is_null() {
        let _ = writeln!(Message::send_info(), "No new Model produced");
    }
    Ok(RetDone)
}

/// `tpclear`, `twclear`: the transfer process of the reading (or of the writing for a
/// 'w' second character) cleared.
pub(super) fn tpclear(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let modew = word_byte(pilot, 0, 2)? == b'w';
    let ws = xs_session(pilot)?;
    let fp = ws
        .transfer_writer()
        .get()
        .map(|w| w.finder_process())
        .unwrap_or_default();
    let tp = ws
        .transfer_reader()
        .get()
        .map(XSControlTransferReader::transient_process)
        .unwrap_or_default();
    let mut sout = Message::send_info();
    let cleared = if modew {
        fp.get().map(|p| p.clear())
    } else {
        tp.get().map(|p| p.clear())
    };
    if cleared.is_none() {
        let _ = writeln!(
            sout,
            "{}",
            if modew {
                "No Transfer Write"
            } else {
                "No Transfer Read"
            }
        );
    }
    Ok(RetDone)
}

/// The modes of `tpstat` given by its argument: (`mod1`, `mod2`); `mod1` is -1 for the
/// help, -2 for an unknown mode.
fn tpstat_modes(arg1: &[u8], argc: i32) -> (i32, i32) {
    let first = arg1.first().copied().unwrap_or(0);
    let mut a2 = match arg1.get(1).copied().unwrap_or(0) {
        0 => b'!',
        c => c,
    };
    let (mut mod1, mut mod2) = match first {
        b'g' => (0, 0),
        b'c' => (4, 4),
        b'C' => (4, 2),
        b'f' => (5, 4),
        b'F' => (5, 2),
        b'*' => (2, 0),
        b'?' => (3, 0),
        _ => {
            a2 = first;
            (if argc > 2 { 2 } else { 1 }, 0)
        }
    };
    if !(1..=3).contains(&mod1) {
        a2 = b'!';
    }
    match a2 {
        b'n' => mod2 = 0,
        b's' => mod2 = 1,
        b'b' => mod2 = 2,
        b't' => mod2 = 3,
        b'r' => mod2 = 4,
        b'l' => mod2 = 5,
        b'L' => mod2 = 6,
        b'!' => {}
        b'?' => mod1 = -1,
        _ => mod1 = -2,
    }
    (mod1, mod2)
}

/// The help of the modes of `tpstat`.
const TPSTAT_HELP: &str = "Modes available :\ng : general    c : checks (count)  C (list)\n               f : fails  (count)  F (list)\n  n : numbers of transferred entities (on TRANSFER ROOTS)\n  s : their status (type entity-result , presence checks)\n  b : detail of binders\n  t : count per entity type    r : per type/status result\n  l : count per couple  type entity/result\n  L : list  per couple  type entity/result\n  *n  *s  *b  *t  *r  *l  *L : idem on ALL recorded items\n  ?n  ?s  ?b  ?t ... : idem on abnormal items\n  n select : n applied on a selection   idem for  s b t r l\n";

/// `tpstat`: the statistics of the transfer process of the reading, by mode, on the
/// list the command part 2 gives.
pub(super) fn tpstat(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let tp = reader_of(pilot)?
        .get()
        .map(XSControlTransferReader::transient_process)
        .unwrap_or_default();
    let mut sout = Message::send_info();
    let Some(process) = tp.get() else {
        let _ = writeln!(sout, "No Transfer Read");
        return Ok(RetError);
    };
    let (mod1, mod2) = if argc > 1 {
        tpstat_modes(pilot.arg(1)?.as_bytes(), argc)
    } else {
        (-1, 0)
    };
    if mod1 < -1 {
        let _ = writeln!(sout, "Unknown Mode");
    }
    if mod1 < 0 {
        let _ = sout.write_str(TPSTAT_HELP);
        return Ok(if mod1 < -1 { RetError } else { RetVoid });
    }
    let _ = sout.write_str("TransferRead :");
    if process.model()
        != pilot
            .session()
            .get()
            .map(|ws| ws.model())
            .unwrap_or_default()
    {
        let _ = sout.write_str("Model differs from the session");
    }
    // sout stays alive: as natively, its text is flushed after the statistics.
    let list = IFSelectFunctions::give_list(&pilot.session(), &text(&pilot.command_part(2)), "")?;
    XSControlTransferReader::print_stats_on_list(&tp, list.as_deref(), mod1, mod2)?;
    Ok(RetVoid)
}

/// `tpent`: the transfer status of the entity `num` of the model of the reading
/// process.
pub(super) fn tpent(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = xs_session(pilot)?;
    let tp = ws
        .transfer_reader()
        .get()
        .map(XSControlTransferReader::transient_process)
        .unwrap_or_default();
    let mut sout = Message::send_info();
    let Some(process) = tp.get() else {
        let _ = writeln!(sout, "No Transfer Read");
        return Ok(RetError);
    };
    let model = process.model();
    let Some(m) = model.get() else {
        return Ok(RetFail);
    };
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Give ENTITY NUMBER (IN MODEL TransferProcess)");
        return Ok(RetError);
    }
    let num = int_arg(pilot, 1)?;
    if num <= 0 || num > m.nb_entities() {
        let _ = writeln!(sout, "Number not in [1 - {}]", m.nb_entities());
        return Ok(RetError);
    }
    let index = process.map_index(&m.value(num)?);
    if index == 0 {
        let _ = writeln!(sout, "Entity {num}  not recorded in transfer");
        return Ok(RetVoid);
    }
    let mut out = Vec::new();
    ws.print_transfer_status(index, false, &mut out)?;
    emit(&mut sout, &out);
    Ok(RetVoid)
}

/// `tpitem`, `tproot`, `twitem`, `twroot`: the transfer status of an item (a root for a
/// third character 'r') of the reading (of the writing for a second character 'w').
pub(super) fn tpitem(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Give ITEM NUMBER (in TransferProcess)");
        return Ok(RetError);
    }
    let mut num = int_arg(pilot, 1)?;
    if word_byte(pilot, 0, 3)? == b'r' {
        num = -num;
    }
    let modew = word_byte(pilot, 0, 2)? == b'w';
    let mut out = Vec::new();
    let found = xs_session(pilot)?.print_transfer_status(num, modew, &mut out)?;
    emit(&mut sout, &out);
    if !found {
        let _ = writeln!(sout, " - Num={num} incorrect");
    }
    Ok(RetVoid)
}
