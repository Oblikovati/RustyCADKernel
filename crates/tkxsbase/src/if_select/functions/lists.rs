// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the list, item, data, parameter and
// file naming commands.

//! The commands on lists (givelist and its variants), items, data, static parameters
//! and file naming.
use super::{IFSelectFunctions, Status, emit, first_byte, h_text, session_of};
use crate::c_text::atoi;
use crate::if_select::{
    IFSelectDispatch, IFSelectReturnStatus, IFSelectSelectPointed, IFSelectSelectSuite,
    IFSelectSessionPilot,
};
use crate::interface::{InterfaceEntityIterator, InterfaceModelEntities, InterfaceStatic};
use rustycad_tkernel::{message::Message, standard::Handle};
use std::fmt::Write as _;

use IFSelectReturnStatus::{RetDone, RetError, RetFail, RetVoid};

/// The text of the command line from the word `num`.
fn part(pilot: &Handle<IFSelectSessionPilot>, num: i32) -> String {
    String::from_utf8_lossy(pilot.command_part(num).as_bytes()).into_owned()
}

/// The pointed selection `makelist` fills: the existing one cleared, or a new item;
/// `None` (command refused, with a message) for another item.
fn list_target(
    ws: &crate::if_select::IFSelectWorkSession,
    arg1: &str,
    sout: &mut rustycad_tkernel::message::MessageStream<'_>,
) -> Result<Option<Handle<IFSelectSelectPointed>>, rustycad_tkernel::standard::Failure> {
    let item = ws.named_item(arg1);
    let pnt: Handle<IFSelectSelectPointed> = item.cast();
    if let Some(p) = pnt.get() {
        let _ = writeln!(
            sout,
            "{arg1}:Already existing Selection for List, cleared then filled"
        );
        p.clear();
        return Ok(Some(pnt));
    }
    if item.is_some() {
        let _ = writeln!(
            sout,
            "{arg1}:Already existing Item not for a List, command ignored"
        );
        return Ok(None);
    }
    let pnt = Handle::new(IFSelectSelectPointed::new());
    ws.add_named_item(arg1, &pnt.clone().cast(), true)?;
    Ok(Some(pnt))
}

/// `fun20` (evalsel, givelist, giveshort, givepointed, makelist): the list given by
/// the arguments, listed by mode (makelist fills a pointed selection with it).
pub(super) fn fun20(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg0 = pilot.arg(0)?;
    let mut mode = first_byte(&arg0); // givelist/makelist
    if mode == b'g' {
        mode = arg0.as_bytes().get(4).copied().unwrap_or(0); // l list  s short  p pointed
    }
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(
            sout,
            "Give Entity ID, or Selection Name [+ optional other selection or entity]"
        );
        return Ok(RetError);
    }
    let mut pnt = Handle::default();
    if mode == b'm' {
        match list_target(&ws, &pilot.arg(1)?, &mut sout)? {
            Some(p) => pnt = p,
            None => return Ok(RetFail),
        }
    }
    let first = if mode == b'm' { 2 } else { 1 };
    let Some(result) = IFSelectFunctions::give_list(&ws, &part(pilot, first), "")? else {
        return Ok(RetError);
    };
    let mut iter = InterfaceEntityIterator::new();
    iter.add_list(&result);
    let _ = write!(sout, "{} : ", part(pilot, first));
    let mut out = Vec::new();
    match mode {
        b'l' => ws.list_entities(&iter, 0, &mut out)?,
        b's' | b'm' => ws.list_entities(&iter, 2, &mut out)?,
        b'p' => {
            let _ = write!(sout, "{} Entities : ", iter.nb_entities());
            for ent in &result {
                let _ = write!(sout, " +{}", ws.starting_number(ent));
            }
            let _ = writeln!(sout);
        }
        _ => {}
    }
    emit(&mut sout, &out);
    if let Some(p) = pnt.get() {
        p.set_list(Some(&result));
        let _ = write!(
            sout,
            "List set to a SelectPointed : {}\nLater editable by command setlist\n",
            pilot.arg(1)?
        );
    }
    Ok(RetVoid)
}

/// `fun20c` (givecount): the count of the list given by the arguments.
pub(super) fn fun20c(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(
            sout,
            "Give Entity ID, or Selection Name [+ optional other selection or entity]"
        );
        return Ok(RetError);
    }
    let Some(result) = IFSelectFunctions::give_list(&ws, &part(pilot, 1), "")? else {
        return Ok(RetError);
    };
    let _ = writeln!(
        sout,
        "{} : List of {} Entities",
        part(pilot, 1),
        result.len()
    );
    Ok(RetVoid)
}

/// `funselsuite` (selsuite): a suite of the deductions given, recorded.
pub(super) fn funselsuite(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    if argc < 2 {
        let _ = writeln!(
            sout,
            "Give Entity ID, or Selection Name [+ optional other selection or entity]"
        );
        return Ok(RetError);
    }
    let selsuite = IFSelectSelectSuite::new();
    for i in 1..argc {
        let sel = ws.give_selection(&pilot.arg(i)?);
        if !selsuite.add_input(&sel) {
            let _ = writeln!(
                sout,
                "{} : not a SelectDeduct, no more can be added. Abandon",
                pilot.arg(i - 1)?
            );
            return Ok(RetError);
        }
    }
    selsuite.set_label(&part(pilot, 1));
    Ok(pilot.record_item(&Handle::new(selsuite).cast()))
}

/// `fun21` (clearitems): all the items, the final modifiers and the share-out cleared.
pub(super) fn fun21(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    ws.clear_items();
    ws.clear_final_modifiers()?;
    ws.clear_share_out(false)?;
    Ok(RetDone)
}

/// `fun22` (cleardata): the data cleared by mode (a all, g graph, c check, p pointed).
pub(super) fn fun22(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mode = if pilot.nb_words() >= 2 {
        match first_byte(&pilot.arg(1)?) {
            b'a' => 1,
            b'g' => 2,
            b'c' => 3,
            b'p' => 4,
            _ => -1,
        }
    } else {
        0
    };
    if mode <= 0 {
        let mut sout = Message::send_info();
        if mode < 0 {
            let _ = write!(sout, "Give a suitable mode");
        }
        let _ = writeln!(
            sout,
            "  Available Modes :\n a : all data    g : graph+check  c : check  p : selectpointed"
        );
        return Ok(if mode < 0 { RetError } else { RetVoid });
    }
    ws.clear_data_virtual(mode)?;
    Ok(RetDone)
}

/// `fun24` (itemlabel): the items whose label is, starts with or contains the words.
pub(super) fn fun24(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    if argc < 2 {
        let _ = writeln!(sout, " Give  label to search");
        return Ok(RetError);
    }
    let words: Result<Vec<String>, _> = (1..argc).map(|i| pilot.arg(i)).collect();
    let label = words?.join(" ");
    for (mode, title) in [(0, " exact"), (1, " same head"), (2, " search if present")] {
        let _ = writeln!(sout, "Searching label : {label}. in mode {title}");
        let mut nbitems = 0;
        let mut id = ws.next_ident_for_label(&label, 0, mode);
        while id != 0 {
            let _ = write!(sout, " {id}");
            nbitems += 1;
            id = ws.next_ident_for_label(&label, id, mode);
        }
        let _ = writeln!(sout, " -- giving {nbitems} found");
    }
    Ok(RetVoid)
}

/// The list of the static parameters of `fun27`, all or those starting with `pattern`.
fn list_statics(sout: &mut rustycad_tkernel::message::MessageStream<'_>, pattern: Option<&str>) {
    let li = InterfaceStatic::items(0, "");
    let names: Vec<String> = li.iter().map(h_text).collect();
    let kept: Vec<&String> = names
        .iter()
        .filter(|n| pattern.is_none_or(|p| n.as_bytes().starts_with(p.as_bytes())))
        .collect();
    let _ = writeln!(sout, " List of parameters : {} items : ", kept.len());
    for name in kept {
        let _ = writeln!(
            sout,
            "{name} : {}",
            String::from_utf8_lossy(InterfaceStatic::c_val(name).as_bytes())
        );
    }
}

/// `fun27` (param): the static parameters listed, traced by use, displayed or set.
pub(super) fn fun27(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut arg2 = pilot.arg(2)?;
    if arg2 == "\"\"" {
        arg2.clear();
    }
    let mut sout = Message::send_info();
    if argc < 2 || (argc == 3 && arg1 == "-p") {
        list_statics(&mut sout, (argc == 3).then_some(arg2.as_str()));
        return Ok(RetVoid);
    }
    if atoi(arg1.as_bytes()) > 0 {
        ws.trace_statics(atoi(arg1.as_bytes()), 0)?;
        return Ok(RetVoid);
    }
    let _ = writeln!(
        sout,
        "     {} STATUS of Static Parameter {arg1}",
        if argc > 2 { "FORMER" } else { "ACTUAL" }
    );
    if !InterfaceStatic::is_present(&arg1) {
        let _ = writeln!(sout, " Parameter {arg1} undefined");
        return Ok(RetError);
    }
    if !InterfaceStatic::is_set(&arg1, true) {
        let _ = writeln!(sout, " Parameter {arg1} not valued");
    } else if argc == 2 {
        let mut out = Vec::new();
        let _ = InterfaceStatic::static_named(&arg1).print(&mut out);
        emit(&mut sout, &out);
    } else {
        let _ = writeln!(
            sout,
            " Value : {}",
            String::from_utf8_lossy(InterfaceStatic::c_val(&arg1).as_bytes())
        );
    }
    if argc == 2 {
        let _ = writeln!(sout, "To modify, param name_param new_val");
        return Ok(RetVoid);
    }
    let _ = write!(
        sout,
        " New demanded value : {}",
        if arg2.is_empty() {
            "not valued"
        } else {
            arg2.as_str()
        }
    );
    if InterfaceStatic::set_c_val(&arg1, &arg2)? {
        let _ = writeln!(sout, "   OK");
        return Ok(RetDone);
    }
    let _ = writeln!(sout, " , refused");
    Ok(RetError)
}

/// `fun29` (sentfiles): the files sent since the last load.
pub(super) fn fun29(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    let Some(list) = ws.sent_files() else {
        let _ = writeln!(sout, "List of Sent Files not enabled");
        return Ok(RetVoid);
    };
    let _ = writeln!(sout, "  Sent Files : {} : ", list.len());
    for file in &list {
        let _ = writeln!(sout, "{}", h_text(file));
    }
    Ok(RetVoid)
}

/// `fun30` (fileprefix): displays or sets the file prefix.
pub(super) fn fun30(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    if pilot.nb_words() >= 2 {
        ws.set_file_prefix(&pilot.arg(1)?)?;
        return Ok(RetDone);
    }
    let prefix = ws.file_prefix()?;
    let mut sout = Message::send_info();
    if prefix.is_null() {
        let _ = writeln!(sout, "No prefix defined");
    } else {
        let _ = writeln!(sout, "Prefixe : {}", h_text(&prefix));
    }
    let _ = writeln!(sout, "Pour changer :  filepref newprefix");
    Ok(RetVoid)
}

/// `fun31` (fileext): displays or sets the file extension.
pub(super) fn fun31(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    if pilot.nb_words() >= 2 {
        ws.set_file_extension(&pilot.arg(1)?)?;
        return Ok(RetDone);
    }
    let ext = ws.file_extension()?;
    let mut sout = Message::send_info();
    if ext.is_null() {
        let _ = writeln!(sout, "Pas d extension definie");
    } else {
        let _ = writeln!(sout, "Extension : {}", h_text(&ext));
    }
    let _ = writeln!(sout, "Pour changer :  fileext newext");
    Ok(RetVoid)
}

/// `fun32` (fileroot): displays or sets the root name of a dispatch.
pub(super) fn fun32(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if argc < 2 {
        let _ = writeln!(sout, "Give Dispatch and Root name");
        return Ok(RetError);
    }
    let disp: Handle<dyn IFSelectDispatch> = ws.named_item(&arg1).cast();
    if argc >= 3 {
        return Ok(if ws.set_file_root(&disp, &pilot.arg(2)?)? {
            RetDone
        } else {
            RetFail
        });
    }
    let root = ws.file_root(&disp)?;
    if root.is_null() {
        let _ = writeln!(sout, "No root defined for {arg1}");
    } else {
        let _ = writeln!(sout, "Root for {arg1} : {}", h_text(&root));
    }
    let _ = writeln!(sout, "Pour changer :  fileroot nomdisp newroot");
    Ok(RetVoid)
}

/// `fun33` (filedef): displays or sets the default root name.
pub(super) fn fun33(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    if pilot.nb_words() >= 2 {
        ws.set_default_file_root(&pilot.arg(1)?)?;
        return Ok(RetDone);
    }
    let def = ws.default_file_root()?;
    let mut sout = Message::send_info();
    if def.is_null() {
        let _ = writeln!(sout, "No default root defined");
    } else {
        let _ = writeln!(sout, "Racine par defaut : {}", h_text(&def));
    }
    let _ = writeln!(sout, "Pour changer :  filedef newdef");
    Ok(RetVoid)
}

/// `fun34` (evalfile): the file evaluation, with the files listed.
pub(super) fn fun34(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if !ws.has_model() {
        let _ = writeln!(sout, "No Model loaded, abort");
        return Ok(RetFail);
    }
    let _ = writeln!(sout, "Evaluation avec Memorisation des resultats");
    ws.evaluate_file()?;
    for i in 1..=ws.nb_files() {
        let model = ws.file_model(i)?;
        let Some(m) = model.get() else {
            let _ = writeln!(sout, "Modele {i} Model non genere ...");
            continue;
        };
        let name = ws.file_name(i)?;
        let _ = writeln!(
            sout,
            "Fichier n0 {i} Nb Entites : {}  Nom: {}",
            m.nb_entities(),
            String::from_utf8_lossy(name.as_bytes())
        );
    }
    Ok(RetDone)
}

/// `fun35` (clearfile).
pub(super) fn fun35(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?.clear_file()?;
    Ok(RetDone)
}
