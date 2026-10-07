// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the transformer and selection commands.

//! The commands running transformers and modifiers, and the commands creating and
//! editing selections and counters.
use super::{IFSelectFunctions, Status, first_byte, session_of};
use crate::if_select::{
    IFSelectIntParam, IFSelectModifReorder, IFSelectModifier, IFSelectReturnStatus,
    IFSelectSelectDiff, IFSelectSelectModelEntities, IFSelectSelectModelRoots,
    IFSelectSelectPointed, IFSelectSelectRange, IFSelectSelectRoots, IFSelectSelectShared,
    IFSelectSelection, IFSelectSessionPilot, IFSelectTransformer,
};
use rustycad_tkernel::{
    message::{Message, MessageStream},
    standard::Handle,
};
use std::fmt::Write as _;

use IFSelectReturnStatus::{RetDone, RetError, RetFail, RetVoid};

/// The message of the effect of a transformer run; `what` names the item for 0.
fn effect_message(sout: &mut MessageStream<'_>, effect: i32, missing: bool, what: &str) {
    let text = match effect {
        -4 => "Edition sur place, nouveau Protocole, erreur recalcul graphe",
        -3 => "Erreur, Transformation ignoree",
        -2 => "Error on in-place editing, risk of corruption (check)",
        -1 => "Error on local editing, risk of corruption (check)",
        0 if missing => {
            let _ = writeln!(sout, "{what}");
            return;
        }
        0 => "Execution non faite",
        1 => "Transformation locale (graphe non touche)",
        2 => "Edition sur place (graphe recalcule)",
        3 => "Modele reconstruit",
        4 => "Edition sur place, nouveau Protocole",
        5 => "Nouveau Modele avec nouveau Protocole",
        _ => return,
    };
    let _ = writeln!(sout, "{text}");
}

/// `fun61` (runtranformer): a transformer run on the model.
pub(super) fn fun61(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Give Transformer Name");
        return Ok(RetError);
    }
    let tsf: Handle<dyn IFSelectTransformer> = ws.named_item(&arg1).cast();
    let effect = ws.run_transformer(&tsf)?;
    effect_message(
        &mut sout,
        effect,
        tsf.is_null(),
        &format!("Erreur, pas un Transformer: {arg1}"),
    );
    Ok(if effect > 0 { RetDone } else { RetFail })
}

/// `fun62` (copy): a new standard transformer (copy) recorded.
pub(super) fn fun62(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    Ok(pilot.record_item(&ws.new_transform_standard(true, "")?.cast()))
}

/// `fun63` (onthespot): a new standard transformer (on the spot) recorded.
pub(super) fn fun63(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    Ok(pilot.record_item(&ws.new_transform_standard(false, "")?.cast()))
}

/// `fun6465` (runcopy, runonthespot): a model modifier (an item, else the one the rest
/// of the line creates) run by a standard transformer, on a list or the model.
pub(super) fn fun6465(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let arg1 = pilot.arg(1)?;
    let runcopy = pilot.arg(0)?.as_bytes().get(3) == Some(&b'c');
    let modif: Handle<dyn IFSelectModifier> = if ws.name_ident(&arg1) > 0 {
        ws.named_item(&arg1).cast()
    } else {
        pilot.remove_word(0)?; // it was the run command
        pilot.perform()?;
        pilot.recorded_item().cast()
    };
    let mut sout = Message::send_info();
    if modif.is_null() {
        let _ = writeln!(sout, "Not a Modifier name : {arg1}");
        return Ok(RetError);
    }
    let mut sp: Handle<dyn IFSelectSelection> = Handle::default();
    if argc > 2 {
        let part = String::from_utf8_lossy(pilot.command_part(2).as_bytes()).into_owned();
        let list = IFSelectFunctions::give_list(&ws, &part, "")?;
        let pointed = IFSelectSelectPointed::new();
        pointed.set_list(list.as_deref());
        sp = Handle::new(pointed).cast();
    }
    let effect = ws.run_modifier_selected(&modif, &sp, runcopy)?;
    effect_message(
        &mut sout,
        effect,
        modif.is_null(),
        &format!("Erreur, pas un Modifier: {arg1}"),
    );
    Ok(if effect > 0 { RetDone } else { RetFail })
}

/// `fun66` (reorder): a new reordering modifier (roots first or last) recorded.
pub(super) fn fun66(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let opt = if pilot.nb_words() >= 2 {
        first_byte(&pilot.arg(1)?)
    } else {
        b' '
    };
    if opt != b'f' && opt != b'l' {
        let _ = writeln!(
            Message::send_info(),
            "Donner option : f -> root-first  l -> root-last"
        );
        return Ok(RetError);
    }
    Ok(pilot.record_item(&Handle::new(IFSelectModifReorder::new(opt == b'l')).cast()))
}

/// `fun70` (toggle): the direction of an extraction toggled.
pub(super) fn fun70(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Give Selection Name");
        return Ok(RetError);
    }
    let sel: Handle<dyn IFSelectSelection> = ws.named_item(&arg1).cast();
    if !ws.toggle_select_extract(&sel) {
        let _ = writeln!(sout, "Pas une SelectExtract : {arg1}");
        return Ok(RetFail);
    }
    let state = if ws.is_reversed_select_extract(&sel) {
        "Reversed"
    } else {
        "Directe"
    };
    let _ = writeln!(sout, "{arg1} a present {state}");
    Ok(RetDone)
}

/// Two selections (the target, then the input).
pub(super) type SelectionPair = (Handle<dyn IFSelectSelection>, Handle<dyn IFSelectSelection>);

/// The two selection items of the first two arguments.
pub(super) fn two_selections(
    pilot: &Handle<IFSelectSessionPilot>,
) -> Result<SelectionPair, rustycad_tkernel::standard::Failure> {
    let ws = session_of(pilot)?;
    Ok((
        ws.named_item(&pilot.arg(1)?).cast(),
        ws.named_item(&pilot.arg(2)?).cast(),
    ))
}

/// `fun71` (input): the input of an extraction or deduction.
pub(super) fn fun71(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(sout, "Donner Noms Selections cible et input");
        return Ok(RetError);
    }
    let (sel, sou) = two_selections(pilot)?;
    if sel.is_null() || sou.is_null() {
        let _ = writeln!(sout, "Incorrect : {arg1},{arg2}");
        return Ok(RetError);
    }
    if !ws.set_input_selection(&sel, &sou) {
        let _ = writeln!(
            sout,
            "Nom incorrect ou Selection {arg1} ni Extract ni Deduct"
        );
        return Ok(RetFail);
    }
    Ok(RetDone)
}

/// Records a new selection.
pub(super) fn record_selection(
    pilot: &Handle<IFSelectSessionPilot>,
    sel: Handle<dyn IFSelectSelection>,
) -> Status {
    Ok(pilot.record_item(&sel.cast()))
}

/// `fun72` (modelroots).
pub(super) fn fun72(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?;
    record_selection(pilot, Handle::new(IFSelectSelectModelRoots::new()).cast())
}

/// The help of `fun73`.
const RANGE_HELP: &str = "Donner la description du SelectRange    Accepted forms :\n <n1> <n2>  : Range from <n1> to <n2>\n <n1> tout seul : Range n0 <n1>\n  from <n1>  : Range From <n1>\n  until <n2> : Range Until <n2>\n";

/// `fun73` (range): a new range of integer parameters ("from i", "until i", "i",
/// "i j") recorded.
pub(super) fn fun73(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut argc = pilot.nb_words();
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let mut sout = Message::send_info();
    if argc >= 2 && first_byte(&arg1) == b'?' {
        argc = 1;
    }
    if argc < 2 {
        let _ = write!(sout, "{RANGE_HELP}");
        return Ok(RetVoid);
    }
    let param = |name: &str| -> Handle<IFSelectIntParam> { ws.named_item(name).cast() };
    let sel = IFSelectSelectRange::new();
    if arg1 == "from" || arg1 == "until" {
        if argc < 3 {
            let _ = writeln!(sout, "Forme admise : {arg1} <i>");
            return Ok(RetError);
        }
        if arg1 == "from" {
            sel.set_from(&param(&arg2))
        } else {
            sel.set_until(&param(&arg2))
        }
    } else if argc < 3 {
        sel.set_one(&param(&arg1));
    } else {
        sel.set_range(&param(&arg1), &param(&arg2));
    }
    record_selection(pilot, Handle::new(sel).cast())
}

/// `fun74` (roots).
pub(super) fn fun74(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?;
    record_selection(pilot, Handle::new(IFSelectSelectRoots::new()).cast())
}

/// `fun75` (shared).
pub(super) fn fun75(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?;
    record_selection(pilot, Handle::new(IFSelectSelectShared::new()).cast())
}

/// `fun76` (diff): a new difference of the selections given (if any) recorded.
pub(super) fn fun76(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let sel: Handle<dyn IFSelectSelection> = Handle::new(IFSelectSelectDiff::new()).cast();
    let mut sout = Message::send_info();
    if argc < 3 {
        let _ = writeln!(
            sout,
            "Diff without input : don't forget to define them (ctlmain, ctlsec)!"
        );
    }
    let (selmain, selsec) = two_selections(pilot)?;
    if argc >= 2 && !ws.set_control(&sel, &selmain, true) {
        let _ = writeln!(sout, "Echec ControlMain:{arg1} , a refaire (ctlmain)");
    }
    if argc >= 3 && !ws.set_control(&sel, &selsec, false) {
        let _ = writeln!(sout, "Echec ControlSecond:{arg2} , a refaire (ctlsec)");
    }
    record_selection(pilot, sel)
}

/// The main (`formain`) or second input of a control.
fn set_control_input(pilot: &Handle<IFSelectSessionPilot>, formain: bool, missing: &str) -> Status {
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(sout, "{missing}");
        return Ok(RetError);
    }
    let (sel, input) = two_selections(pilot)?;
    if ws.set_control(&sel, &input, formain) {
        return Ok(RetDone);
    }
    let _ = writeln!(
        sout,
        "Incorrect name or Selection {} not of Control type",
        pilot.arg(1)?
    );
    Ok(RetFail)
}

/// `fun77` (selmain).
pub(super) fn fun77(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    set_control_input(pilot, true, "Give Control and MainInput Names")
}

/// `fun78` (selsecond).
pub(super) fn fun78(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    set_control_input(pilot, false, "Give Control and SecondInput Names")
}

/// `fun79` (modelall).
pub(super) fn fun79(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?;
    record_selection(
        pilot,
        Handle::new(IFSelectSelectModelEntities::new()).cast(),
    )
}
