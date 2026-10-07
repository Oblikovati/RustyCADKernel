// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the split, remaining, modifier and
// dispatch commands.

//! The commands of the split (xsplit, remaining, setcontent), of the final modifiers
//! and of the dispatches (creation, evaluation, writing).
use super::{Status, first_byte, h_text, session_of};
use crate::c_text::atoi;
use crate::if_select::{
    IFSelectDispatch, IFSelectGeneralModifier, IFSelectModifier, IFSelectRemainMode,
    IFSelectReturnStatus, IFSelectSelection, IFSelectSessionPilot, IFSelectWorkSession,
};
use rustycad_tkernel::{
    message::{Message, MessageStream},
    standard::Handle,
};
use std::fmt::Write as _;

use IFSelectReturnStatus::{RetDone, RetError, RetFail, RetVoid};

/// The text of a label.
fn label(l: rustycad_tkernel::tcollection::AsciiString) -> String {
    String::from_utf8_lossy(l.as_bytes()).into_owned()
}

/// `fun36` (xsplit): the split of the share-out (on the dispatches given, else the
/// last ones), the sent files recorded.
pub(super) fn fun36(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    let mut stat = RetVoid;
    if argc < 2 {
        let _ = writeln!(sout, "Split : last dispatch list defined");
    } else {
        ws.clear_share_out(true)?;
        for i in 1..argc {
            let arg = pilot.arg(i)?;
            let disp = ws.named_item(&arg);
            if disp.cast::<dyn IFSelectDispatch>().is_null() {
                let _ = writeln!(sout, "Pas un dispatch:{arg}, Splitt abandonne");
                stat = RetError;
            } else {
                ws.set_active(&disp, true)?;
            }
        }
    }
    if stat == RetError {
        return Ok(stat);
    }
    ws.begin_sent_files(true);
    Ok(if ws.send_split()? { RetDone } else { RetFail })
}

/// `fun37` (remaining): the remaining data listed, computed, undone or forgotten.
pub(super) fn fun37(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let mode = if argc >= 2 {
        first_byte(&pilot.arg(1)?)
    } else {
        b'?'
    };
    let numod = match mode {
        b'u' => IFSelectRemainMode::RemainUndo,
        b'l' => IFSelectRemainMode::RemainDisplay,
        b'c' => IFSelectRemainMode::RemainCompute,
        b'f' => IFSelectRemainMode::RemainForget,
        _ => {
            let mut sout = Message::send_info();
            if argc < 2 {
                let _ = write!(sout, "Donner un Mode - ");
            }
            let _ = writeln!(
                sout,
                "Modes possibles : l  list, c compute, u undo, f forget"
            );
            return Ok(if mode == b'?' { RetDone } else { RetError });
        }
    };
    Ok(if ws.set_remaining(numod)? {
        RetDone
    } else {
        RetVoid
    })
}

/// `fun38` (setcontent): the model reduced to (k) or without (r) a selection.
pub(super) fn fun38(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(sout, "Donner nom selection et mode (k=keep,r=remove)");
        return Ok(RetError);
    }
    let sel: Handle<dyn IFSelectSelection> = ws.named_item(&arg1).cast();
    if sel.is_null() {
        let _ = writeln!(sout, "No Selection with Name : {arg1}");
        return Ok(RetError);
    }
    let keepmode = match first_byte(&arg2) {
        b'k' => true,
        b'r' => false,
        _ => {
            let _ = writeln!(sout, "Donner nom selection et mode (k=keep,r=remove)");
            return Ok(RetError);
        }
    };
    let _ = write!(
        sout,
        " -- SetContent {} ...",
        if keepmode { "keep" } else { "remove" }
    );
    let done = ws.set_model_content(&sel, keepmode)?;
    let _ = writeln!(
        sout,
        "{}",
        if done {
            " Done"
        } else {
            " Result empty, ignored"
        }
    );
    Ok(RetDone)
}

/// `fun40` (listmodif): the model then file modifiers.
pub(super) fn fun40(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    ws.list_final_modifiers(true)?;
    ws.list_final_modifiers(false)?;
    Ok(RetVoid)
}

/// The modifier named by the first argument, with a message when not one.
fn named_modifier(
    ws: &IFSelectWorkSession,
    arg1: &str,
    sout: &mut MessageStream<'_>,
    text: &str,
) -> Handle<dyn IFSelectGeneralModifier> {
    let modif: Handle<dyn IFSelectGeneralModifier> = ws.named_item(arg1).cast();
    if modif.is_null() {
        let _ = writeln!(sout, "{text}{arg1}");
    }
    modif
}

/// `fun41` (dumpmodif): the status of a modifier.
pub(super) fn fun41(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Donner Nom du Modifier");
        return Ok(RetError);
    }
    let modif = named_modifier(&ws, &arg1, &mut sout, "No Modifier with Name : ");
    let Some(m) = modif.get() else {
        return Ok(RetVoid);
    };
    let disp = m.dispatch();
    let _ = writeln!(sout, "Modifier : {arg1} Label : {}", label(m.label()));
    let rank = ws.modifier_rank(&modif)?;
    let kind = if modif.cast::<dyn IFSelectModifier>().is_some() {
        "Model"
    } else {
        "File"
    };
    let _ = write!(sout, "{kind} Modifier n0.{rank}");
    match disp.get() {
        None => {
            let _ = writeln!(sout, "  Applique a tous les Dispatchs");
        }
        Some(d) => {
            let _ = write!(sout, "  Dispatch : {}", label(d.label()));
            if ws.has_name(&disp.cast()) {
                let _ = write!(sout, " - Nom:{}", h_text(&ws.name(&disp.cast())));
            }
            let _ = writeln!(sout);
        }
    }
    let sel = m.selection();
    if let Some(s) = sel.get() {
        let _ = write!(sout, "  Selection : {}", label(s.label()));
    }
    if ws.has_name(&sel.cast()) {
        let _ = write!(sout, " - Nom:{}", h_text(&ws.name(&sel.cast())));
    }
    let _ = writeln!(sout);
    Ok(RetVoid)
}

/// `fun42` (modifsel): the selection of a modifier set or cancelled.
pub(super) fn fun42(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(
            sout,
            "Donner Nom Modifier; + Nom Selection optionnel\nSelection to Set a Selection, otherwise Cancel"
        );
        return Ok(RetError);
    }
    let modif = named_modifier(&ws, &arg1, &mut sout, "Not a Modifier name : ");
    if modif.is_null() {
        return Ok(RetError);
    }
    let mut sel = Handle::default();
    if !arg2.is_empty() {
        sel = ws.named_item(&arg2).cast::<dyn IFSelectSelection>();
        if sel.is_null() {
            let _ = writeln!(sout, "Not a Selection name : {arg2}");
            return Ok(RetError);
        }
    }
    Ok(if ws.set_item_selection(&modif.cast(), &sel) {
        RetDone
    } else {
        RetFail
    })
}

/// The help of `fun43`.
const SETAPPLIED_HELP: &str = "Donner Nom Modifier; + Nom Dispatch ou Transformer optionnel :\n - rien : tous Dispatches\n - Dispatch : ce Dispatch seul\n - Transformer : pas un Dispatch mais un Transformer\n";

/// `fun43` (setapplied): a modifier applied to the share-out, a dispatch or a
/// transformer.
pub(super) fn fun43(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = write!(sout, "{SETAPPLIED_HELP}");
        return Ok(RetError);
    }
    let modif = named_modifier(&ws, &arg1, &mut sout, "Not a Modifier name : ");
    if modif.is_null() {
        return Ok(RetError);
    }
    let item = if arg2.is_empty() {
        ws.share_out().cast()
    } else {
        let item = ws.named_item(&arg2);
        if item.is_null() {
            let _ = writeln!(sout, "Pas un nom connu : {arg2}");
            return Ok(RetError);
        }
        item
    };
    Ok(if ws.set_applied_modifier(&modif, &item)? {
        RetDone
    } else {
        RetFail
    })
}

/// `fun44` (resetapplied): a modifier removed from the file output.
pub(super) fn fun44(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Designer un modifier");
        return Ok(RetError);
    }
    let modif = named_modifier(&ws, &arg1, &mut sout, "Not a Modifier name : ");
    if modif.is_null() {
        return Ok(RetError);
    }
    Ok(if ws.reset_applied_modifier(&modif)? {
        RetDone
    } else {
        RetFail
    })
}

/// `fun45` (modifmove): a model (M) or file (F) modifier moved to another rank.
pub(super) fn fun45(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 4 {
        let _ = writeln!(sout, "modifmove MF rang1 rang2, M for Model F for File");
        return Ok(RetError);
    }
    let formodel = match first_byte(&pilot.arg(1)?) {
        b'm' | b'M' => true,
        b'f' | b'F' => false,
        _ => {
            let _ = writeln!(sout, "specify M for Model, F for File");
            return Ok(RetError);
        }
    };
    let (before, after) = (
        atoi(pilot.arg(2)?.as_bytes()),
        atoi(pilot.arg(3)?.as_bytes()),
    );
    if before == 0 || after == 0 {
        let _ = writeln!(sout, "Donner 2 Entiers Positifs");
        return Ok(RetError);
    }
    Ok(if ws.change_modifier_rank(formodel, before, after)? {
        RetDone
    } else {
        RetFail
    })
}
