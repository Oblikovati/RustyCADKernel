// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the dispatch commands.

//! The commands creating, evaluating and writing by dispatches.
use super::{IFSelectFunctions, Status, emit, h_text, session_of};
use crate::c_text::atoi;
use crate::if_select::{
    IFSelectCheckCounter, IFSelectDispGlobal, IFSelectDispPerCount, IFSelectDispPerFiles,
    IFSelectDispPerOne, IFSelectDispPerSignature, IFSelectDispatch, IFSelectIntParam,
    IFSelectReturnStatus, IFSelectSelectPointed, IFSelectSelection, IFSelectSessionPilot,
    IFSelectSignCounter, IFSelectSignature, IFSelectSignatureListOps,
};
use rustycad_tkernel::{
    message::{Message, MessageStream},
    standard::{Failure, Handle, exceptions},
};
use std::fmt::Write as _;

use IFSelectReturnStatus::{RetDone, RetError, RetFail, RetVoid};

/// `fun51` (dispsel): the final selection of a dispatch.
pub(super) fn fun51(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(sout, "Donner Noms Dispatch et Selection Finale");
        return Ok(RetError);
    }
    let disp = ws.named_item(&arg1);
    if disp.cast::<dyn IFSelectDispatch>().is_null() {
        let _ = writeln!(sout, "Not a Dispatch name : {arg1}");
        return Ok(RetError);
    }
    let sel: Handle<dyn IFSelectSelection> = ws.named_item(&arg2).cast();
    if sel.is_null() {
        let _ = writeln!(sout, "Not a Selection name : {arg2}");
        return Ok(RetError);
    }
    Ok(if ws.set_item_selection(&disp, &sel) {
        RetDone
    } else {
        RetFail
    })
}

/// `fun_dispone` (dispone): a new dispatch per entity, recorded.
pub(super) fn fun_dispone(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?;
    Ok(pilot.record_item(&Handle::new(IFSelectDispPerOne::new()).cast()))
}

/// `fun_dispglob` (dispglob): a new global dispatch, recorded.
pub(super) fn fun_dispglob(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?;
    Ok(pilot.record_item(&Handle::new(IFSelectDispGlobal::new()).cast()))
}

/// The integer parameter item of the first argument (with a message when missing).
fn int_param_arg(
    pilot: &Handle<IFSelectSessionPilot>,
    missing: &str,
) -> Result<Option<Handle<IFSelectIntParam>>, Failure> {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "{missing}");
        return Ok(None);
    }
    let par: Handle<IFSelectIntParam> = ws.named_item(&arg1).cast();
    if par.is_null() {
        let _ = writeln!(sout, "Not an IntParam name : {arg1}");
        return Ok(None);
    }
    Ok(Some(par))
}

/// `fun_dispcount` (dispcount): a new dispatch per count of an integer parameter.
pub(super) fn fun_dispcount(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let Some(par) = int_param_arg(pilot, "Give IntParam Name for Count")? else {
        return Ok(RetError);
    };
    let disp = IFSelectDispPerCount::new();
    disp.set_count(&par);
    Ok(pilot.record_item(&Handle::new(disp).cast()))
}

/// `fun_dispfiles` (dispfile): a new dispatch per count of files.
pub(super) fn fun_dispfiles(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let Some(par) = int_param_arg(pilot, "Give IntParam Name for NbFiles")? else {
        return Ok(RetError);
    };
    let disp = IFSelectDispPerFiles::new();
    disp.set_count(&par);
    Ok(pilot.record_item(&Handle::new(disp).cast()))
}

/// `fun_dispsign` (dispsign): a new dispatch per signature.
pub(super) fn fun_dispsign(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Donner Nom Signature");
        return Ok(RetError);
    }
    let sig: Handle<dyn IFSelectSignature> = ws.named_item(&arg1).cast();
    if sig.is_null() {
        let _ = writeln!(sout, "Not a Signature name : {arg1}");
        return Ok(RetError);
    }
    let disp = IFSelectDispPerSignature::new();
    disp.set_sign_counter(
        &Handle::new(IFSelectSignCounter::with_signature(&sig, true, false)?).cast(),
    )?;
    Ok(pilot.record_item(&Handle::new(disp).cast()))
}

/// `fun56` (dumpdisp): the status of a dispatch.
pub(super) fn fun56(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Donner Nom du Dispatch");
        return Ok(RetError);
    }
    let disp: Handle<dyn IFSelectDispatch> = ws.named_item(&arg1).cast();
    let Some(d) = disp.get() else {
        let _ = writeln!(sout, "Pas un dispatch : {arg1}");
        return Ok(RetError);
    };
    let _ = write!(
        sout,
        "Dispatch with Name : {arg1} , in ShareOut, Number {} : ",
        ws.dispatch_rank(&disp)?
    );
    let sel = ws.item_selection(&disp.cast());
    let selname = ws.name(&sel.cast());
    if sel.is_null() {
        let _ = writeln!(sout, "No Final Selection");
    } else if selname.is_null() {
        let _ = writeln!(sout, "Selection Finale : #{}", ws.item_ident(&sel.cast()));
    } else {
        let _ = writeln!(sout, "Selection Finale : {}", h_text(&selname));
    }
    if d.has_root_name() {
        let _ = writeln!(sout, "-- Root file name : {}", h_text(&d.root_name()));
    }
    Ok(RetVoid)
}

/// `fun57` (xremove): a named item removed.
pub(super) fn fun57(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    if pilot.nb_words() < 2 {
        let _ = writeln!(Message::send_info(), "Give Name to Remove !");
        return Ok(RetError);
    }
    Ok(if ws.remove_named_item(&pilot.arg(1)?)? {
        RetDone
    } else {
        RetFail
    })
}

/// `fun58` (evaldisp): the evaluation of the dispatches given (they become the
/// share-out).
pub(super) fn fun58(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    if argc < 3 {
        let _ = writeln!(
            sout,
            "evaldisp mode disp [disp ...] :  Mode + Name(s) of Dispatch(es). Mode:\n  0 brief  1 +forgotten ents  2 +duplicata  3 1+2\nSee also : evaladisp  writedisp  xsplit"
        );
        return Ok(RetVoid);
    }
    let mode = atoi(pilot.arg(1)?.as_bytes());
    let _ = writeln!(sout, " Mode {mode}");
    let mut ok = true;
    for i in 2..argc {
        let arg = pilot.arg(i)?;
        if ws.named_item(&arg).cast::<dyn IFSelectDispatch>().is_null() {
            let _ = writeln!(sout, "Not a dispatch:{arg}");
            ok = false;
        }
    }
    if !ok {
        let _ = writeln!(sout, "Some of the parameters are not correct");
        return Ok(RetError);
    }
    ws.clear_share_out(true)?;
    for i in 2..argc {
        ws.set_active(&ws.named_item(&pilot.arg(i)?), true)?;
    }
    ws.evaluate_complete(mode)?;
    Ok(RetVoid)
}

/// The selection of `evaladisp` and `writedisp`: a pointed selection of the list the
/// words from 3 give, else the final selection of the dispatch; `None` (with a
/// message) when neither.
fn applied_selection(
    pilot: &Handle<IFSelectSessionPilot>,
    selsav: &Handle<dyn IFSelectSelection>,
    sout: &mut MessageStream<'_>,
) -> Result<Option<Handle<dyn IFSelectSelection>>, Failure> {
    let argc = pilot.nb_words();
    let mut sel: Handle<dyn IFSelectSelection> = Handle::default();
    if argc > 3 {
        let part = String::from_utf8_lossy(pilot.command_part(3).as_bytes()).into_owned();
        let list = IFSelectFunctions::give_list(&session_of(pilot)?, &part, "")?;
        if let Some(l) = list.filter(|l| !l.is_empty()) {
            let sp = IFSelectSelectPointed::new();
            sp.add_list(Some(&l));
            sel = Handle::new(sp).cast();
        }
    }
    if sel.is_some() {
        return Ok(Some(sel));
    }
    if selsav.is_null() {
        let _ = writeln!(sout, "No Selection nor GiveList defined");
        return Ok(None);
    }
    if argc > 3 {
        let _ = writeln!(
            sout,
            "GiveList is empty, hence computed from the Selection of the Dispatch"
        );
    }
    Ok(Some(selsav.clone()))
}

/// `fun_evaladisp` (evaladisp): the evaluation of a dispatch on a list.
pub(super) fn fun_evaladisp(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(
            sout,
            "evaladisp mode(=0-1-2-3) disp [givelist] :  Mode + Dispatch [+ GiveList]\n  If GiveList not given, computed from Selection of the Dispatch. Mode:\n  0 brief  1 +forgotten ents  2 +duplicata  3 1+2\nSee also : writedisp"
        );
        return Ok(RetVoid);
    }
    if arg1.len() != 1 {
        let _ = writeln!(
            sout,
            "first parameter : mode, must be a number between 0 and 3"
        );
        return Ok(RetError);
    }
    let mode = atoi(arg1.as_bytes());
    let _ = writeln!(sout, " Mode {mode}");
    let arg2 = pilot.arg(2)?;
    let disp = IFSelectFunctions::give_dispatch(&ws, &arg2, true)?;
    let Some(d) = disp.get() else {
        let _ = writeln!(sout, "Not a dispatch:{arg2}");
        return Ok(RetError);
    };
    let selsav = d.final_selection();
    let Some(sel) = applied_selection(pilot, &selsav, &mut sout)? else {
        return Ok(RetError);
    };
    d.set_final_selection(&sel);
    ws.evaluate_dispatch(&disp, mode)?;
    d.set_final_selection(&selsav);
    Ok(RetVoid)
}

/// `TCollection_AsciiString::SubString(from, to)`: `Standard_OutOfRange` out of the text
/// or for `from > to`.
fn sub_string(text: &str, from: i32, to: i32) -> Result<String, Failure> {
    if to > text.len() as i32 || from <= 0 || from > to {
        return Err(exceptions::out_of_range(&format!(
            "SplitFileName: SubString({from}, {to}) of {text:?}; expected 1 <= from <= to <= {}",
            text.len()
        )));
    }
    Ok(text[from as usize - 1..to as usize].to_owned())
}

/// The last 1-based position of `what` in `text`, -1 when absent.
fn search_from_end(text: &str, what: char) -> i32 {
    text.rfind(what).map_or(-1, |p| p as i32 + 1)
}

/// `SplitFileName(filename, prefix, fileroot, suffix)`: the directory part (with its
/// separator), the root and the extension (with its dot) of a file name.
/// `Standard_OutOfRange` for an empty root (as ".ext" or "dir/.ext").
pub(super) fn split_file_name(filename: &str) -> Result<(String, String, String), Failure> {
    let nomlon = filename.len() as i32;
    let mut nomdeb = search_from_end(filename, '/');
    if nomdeb <= 0 {
        nomdeb = search_from_end(filename, '\\'); // for NT
    }
    nomdeb = nomdeb.max(0);
    let mut nomfin = search_from_end(filename, '.');
    if nomfin < nomdeb {
        nomfin = nomlon + 1;
    }
    let prefix = if nomdeb > 0 {
        sub_string(filename, 1, nomdeb)?
    } else {
        String::new()
    };
    let fileroot = sub_string(filename, nomdeb + 1, nomfin - 1)?;
    let suffix = if nomfin <= nomlon {
        sub_string(filename, nomfin, nomlon)?
    } else {
        String::new()
    };
    Ok((prefix, fileroot, suffix))
}

/// `fun_writedisp` (writedisp): the entities written by splitting with a dispatch,
/// the files named after a file pattern.
pub(super) fn fun_writedisp(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(
            sout,
            "writedisp filename disp [givelist] :  FileName + Dispatch [+ GiveList]\n  If GiveList not given, computed from Selection of the Dispatch.\nFileName : rootname.ext will gives rootname_1.ext etc...\n  path/rootname.ext gives  path/rootname_1.ext etc...\nSee also : evaladisp"
        );
        return Ok(RetVoid);
    }
    let (prefix, rootname, suffix) = split_file_name(&pilot.arg(1)?)?;
    if rootname.is_empty() || suffix.is_empty() {
        let _ = writeln!(sout, "Empty Root Name or Extension");
        return Ok(RetError);
    }
    let arg2 = pilot.arg(2)?;
    let disp = IFSelectFunctions::give_dispatch(&ws, &arg2, true)?;
    let Some(d) = disp.get() else {
        let _ = writeln!(sout, "Not a dispatch:{arg2}");
        return Ok(RetError);
    };
    let selsav = d.final_selection();
    let Some(sel) = applied_selection(pilot, &selsav, &mut sout)? else {
        return Ok(RetError);
    };
    ws.clear_share_out(true)?;
    d.set_final_selection(&sel);
    ws.set_active(&disp.cast(), true)?;
    ws.begin_sent_files(true);
    ws.set_file_prefix(&prefix)?;
    ws.set_file_extension(&suffix)?;
    ws.set_file_root(&disp, &rootname)?;
    let ok = ws.send_split()?;
    d.set_final_selection(&selsav);
    Ok(if ok { RetDone } else { RetFail })
}

/// `fun59` (evalcomplete): the complete evaluation of the share-out.
pub(super) fn fun59(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    let mode = if pilot.nb_words() < 2 {
        let _ = writeln!(sout, " -- mode par defaut 0");
        0
    } else {
        let mode = atoi(pilot.arg(1)?.as_bytes());
        let _ = writeln!(sout, " -- mode : {mode}");
        mode
    };
    ws.evaluate_complete(mode)?;
    Ok(RetVoid)
}

/// `fun60` (runcheck): the last run check list counted.
pub(super) fn fun60(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let chlist = ws.last_run_check_list();
    let counter = IFSelectCheckCounter::new(false);
    counter.analyse(&chlist, &ws.model(), false, false)?;
    let mut out = Vec::new();
    let _ = counter.print_count(&mut out);
    let mut sout = Message::send_info();
    emit(&mut sout, &out);
    Ok(RetVoid)
}
