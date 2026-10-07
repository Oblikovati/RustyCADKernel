// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the counter, signature, entity and
// parameter commands.

//! The commands on counters, signatures, entity status and dumps, and parameters.
use super::{Status, emit, first_byte, session_of};
use crate::c_text::atoi;
use crate::if_select::{
    IFSelectGraphCounter, IFSelectIntParam, IFSelectPrintCount, IFSelectReturnStatus,
    IFSelectSelectDeduct, IFSelectSelectSuite, IFSelectSelection, IFSelectSessionPilot,
    IFSelectSignCounter, IFSelectSignature,
};
use crate::interface::InterfaceCategory;
use rustycad_tkernel::{message::Message, standard::Handle, tcollection::HAsciiString};
use std::fmt::Write as _;

use IFSelectReturnStatus::{RetDone, RetError, RetFail, RetVoid};

/// The counter of `funcount`: the counter item, else a counter of the signature item.
fn named_counter(
    ws: &crate::if_select::IFSelectWorkSession,
    arg1: &str,
    listmode: bool,
) -> Result<Handle<IFSelectSignCounter>, rustycad_tkernel::standard::Failure> {
    let item = ws.named_item(arg1);
    let counter: Handle<IFSelectSignCounter> = item.cast();
    if counter.is_some() {
        return Ok(counter);
    }
    let signa: Handle<dyn IFSelectSignature> = item.cast();
    if signa.is_null() {
        return Ok(Handle::default());
    }
    Ok(Handle::new(IFSelectSignCounter::with_signature(
        &signa, false, listmode,
    )?))
}

/// The help of `funcount`.
const COUNT_HELP: &str = "Designer signature ou compteur, + facultatif selection + facultatif entite\n signature/compteur seul -> tout le modele\n sign/compteur + selection -> cette selection, evaluation normale\n sign/compteur + sel + num -> cette selection evaluee sur entite n0 num\n";

/// `funcount` (count, listcount, sumcount): a counter or signature (or a deduction,
/// applied in chain up to "on") counted on the model or a list.
pub(super) fn funcount(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let (arg0, arg1) = (pilot.arg(0)?, pilot.arg(1)?);
    let listmode = first_byte(&arg0) == b'l';
    let mut sout = Message::send_info();
    if argc < 2 {
        let _ = write!(sout, "{COUNT_HELP}");
        return Ok(RetError);
    }
    let mut counter = named_counter(&ws, &arg1, listmode)?;
    let mut onflag = 0;
    for i in 2..argc {
        if pilot.arg(i)? == "on" {
            onflag = i;
            break;
        }
    }
    let mut seld: Handle<dyn IFSelectSelectDeduct> = ws.give_selection(&arg1).cast();
    if seld.is_some() {
        if onflag > 2 {
            let suite = IFSelectSelectSuite::new();
            for i in 1..onflag {
                let sel: Handle<dyn IFSelectSelection> = ws.give_selection(&pilot.arg(i)?);
                if !suite.add_input(&sel) {
                    let _ = writeln!(sout, "Incorrect definition for applied selection");
                    return Ok(RetError);
                }
            }
            seld = Handle::new(suite).cast();
        }
        let gc = IFSelectGraphCounter::new(false, listmode);
        gc.set_applied(&seld);
        counter = Handle::new(gc).cast();
    }
    if counter.is_null() {
        let _ = writeln!(sout, "Neither Counter nor Signature : {arg1}");
        return Ok(RetError);
    }
    let pcm = match first_byte(&arg0) {
        b'c' => IFSelectPrintCount::CountByItem,
        b's' => IFSelectPrintCount::CountSummary,
        _ => IFSelectPrintCount::ListByItem,
    };
    pilot.execute_counter(&counter, onflag.max(1) + 1, pcm)
}

/// `funsigntype` (signtype): displays or sets the signature type of the session.
pub(super) fn funsigntype(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut signtype = ws.sign_type();
    let mut sout = Message::send_info();
    match signtype.get() {
        None => {
            let _ = writeln!(sout, "signtype actually undefined");
        }
        Some(sign) => {
            let (name, id) = (ws.name(&signtype.cast()), ws.item_ident(&signtype.cast()));
            let _ = writeln!(sout, "{}", String::from_utf8_lossy(sign.label().as_bytes()));
            if let Some(n) = name.get() {
                let _ = writeln!(
                    sout,
                    "signtype : also named as {}",
                    String::from_utf8_lossy(n.string().as_bytes())
                );
            } else if id > 0 {
                let _ = writeln!(sout, "signtype : item n0 {id}");
            }
        }
    }
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "signtype newitem  to change, signtype . to clear");
        return Ok(RetVoid);
    }
    if arg1 == "." {
        signtype = Handle::default();
        let _ = writeln!(sout, "signtype now cleared");
    } else {
        signtype = ws.named_item(&arg1).cast();
        if signtype.is_null() {
            let _ = writeln!(sout, "Not a Signature : {arg1}");
            return Ok(RetError);
        }
        let _ = writeln!(sout, "signtype now set to {arg1}");
    }
    ws.set_sign_type(&signtype);
    Ok(RetDone)
}

/// `funsigncase` (signcase): the integer case and the predefined cases of a signature.
pub(super) fn funsigncase(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let signcase: Handle<dyn IFSelectSignature> = ws.named_item(&arg1).cast();
    let mut sout = Message::send_info();
    let Some(sign) = signcase.get() else {
        let _ = writeln!(sout, "Not a Signature : {arg1}");
        return Ok(RetVoid);
    };
    if let Some((hasmin, valmin, hasmax, valmax)) = sign.is_int_case() {
        let _ = write!(sout, "Signature {arg1} : Integer Case");
        if hasmin {
            let _ = write!(sout, " - Mini:{valmin}");
        }
        if hasmax {
            let _ = write!(sout, " - Maxi:{valmax}");
        }
        let _ = writeln!(sout);
    }
    match sign.case_list() {
        None => {
            let _ = writeln!(
                sout,
                "Signature {arg1} : no predefined case, see command  count {arg1}"
            );
        }
        Some(cases) => {
            let _ = writeln!(sout, "Signature {arg1} : {} basic cases :", cases.len());
            for case in &cases {
                let _ = write!(sout, "  {}", String::from_utf8_lossy(case.as_bytes()));
            }
            let _ = writeln!(sout);
        }
    }
    Ok(RetVoid)
}

/// `fun10` (estatus): the categories, or the status of an entity.
pub(super) fn fun10(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let nb = InterfaceCategory::nb_categories();
        let _ = writeln!(sout, " Categories defined :{nb} i.e. :");
        for i in 0..=nb {
            let _ = writeln!(sout, "Cat.{i}  : {}", InterfaceCategory::name(i));
        }
        let _ = writeln!(sout, " On a given entity : give its number");
        return Ok(RetVoid);
    }
    let num = pilot.number(&arg1)?;
    if num <= 0 || num > ws.nb_starting_entities() {
        let _ = writeln!(sout, "Not a suitable entity number : {arg1}");
        return Ok(RetError);
    }
    let mut out = Vec::new();
    ws.print_entity_status(&ws.starting_entity(num), &mut out)?;
    emit(&mut sout, &out);
    Ok(RetVoid)
}

/// The help of `fun11`.
const DATA_HELP: &str = "? for this help, else give a listing mode (first letter suffices) :\n general    General Statistics\n roots    Roots\n entities   All Entities\n listfails  CheckList (fails)    per entity\n messages   CheckList (complete) per entity\n fails      CheckList (fails)    per message (counting)\n check      CheckList (complete) per message (counting)\n totalcheck CheckList (complete) per message (listing n0 ents)\n FAILS      CheckList (fails)    per message (listing complete)\n TOTALCHECK CheckList (complete) per message (listing complete)\n";

/// `fun11` (data): the dump of the model by mode.
pub(super) fn fun11(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let arg1 = pilot.arg(1)?;
    let niv = match first_byte(&arg1) {
        b'?' => {
            let _ = write!(Message::send_info(), "{DATA_HELP}");
            return Ok(RetVoid);
        }
        b'g' => 0,
        b'r' => 1,
        b'e' => 2,
        b'l' => 3,
        b'm' => 4,
        b'c' => 5,
        b't' => 6,
        b'T' => 7,
        b'f' => 8,
        b'F' => 10,
        _ => {
            let _ = writeln!(Message::send_info(), "Unknown Mode .  data simply for help");
            return Ok(RetError);
        }
    };
    ws.trace_dump_model(niv)?;
    Ok(RetVoid)
}

/// The help of `fundumpent`: the dump levels of the library.
fn dump_levels_help(
    sout: &mut rustycad_tkernel::message::MessageStream<'_>,
    wl: &dyn crate::if_select::IFSelectWorkLibrary,
    levdef: i32,
    levmax: i32,
) {
    let _ = write!(sout, "Give n0 or id of entity");
    if levmax < 0 {
        let _ = writeln!(sout, "  and dump level");
    } else {
        let _ = writeln!(
            sout,
            "  + optional, dump level in [0 - {levmax}] , default = {levdef}"
        );
    }
    for level in 0..=levmax {
        let help = wl.dump_help(level);
        if !help.is_empty() {
            let _ = writeln!(
                sout,
                "{level} : {}",
                String::from_utf8_lossy(help.as_bytes())
            );
        }
    }
}

/// `fundumpent` (entity): the dump of an entity by the library, with its check.
pub(super) fn fundumpent(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let wlh = ws.work_library();
    let wl = wlh.get().ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::null_object(
            "IFSelect_Functions::entity: no work library",
        )
    })?;
    let (levdef, levmax) = wl.dump_levels();
    let mut sout = Message::send_info();
    if argc < 2 || (argc == 2 && levmax < 0) {
        dump_levels_help(&mut sout, wl, levdef, levmax);
        return Ok(RetError);
    }
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let num = pilot.number(&arg1)?;
    if num == 0 {
        return Ok(RetError);
    }
    let level = if argc > 2 {
        atoi(arg2.as_bytes())
    } else {
        levdef
    };
    let ent = ws.starting_entity(num);
    if ent.is_null() {
        let _ = writeln!(
            sout,
            "No entity with given id {arg1} ({num}) is found in the current model"
        );
        return Ok(RetVoid);
    }
    let _ = writeln!(sout, "  --   DUMP  Entity n0 {num}  level {level}");
    let mut out = Vec::new();
    wl.dump_entity(&ws.model(), &ws.protocol(), &ent, &mut out, level)?;
    let chl = ws.check_one(&ent, true)?;
    if !chl.is_empty(false) {
        let _ = chl.print_with_model(&mut out, &ws.model(), false, 0);
    }
    emit(&mut sout, &out);
    Ok(RetVoid)
}

/// `funsign` (signature): the value of a signature for an entity.
pub(super) fn funsign(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(sout, " Give signature name + n0 or id of entity");
        return Ok(RetError);
    }
    let sign: Handle<dyn IFSelectSignature> = ws.named_item(&arg1).cast();
    if sign.is_null() {
        let _ = writeln!(sout, "Not a signature : {arg1}");
        return Ok(RetError);
    }
    let num = pilot.number(&arg2)?;
    let ent = ws.starting_entity(num);
    if num == 0 {
        return Ok(RetError);
    }
    let _ = writeln!(sout, "Entity n0 {num} : {}", ws.sign_value(&sign, &ent));
    Ok(RetVoid)
}

/// `funqp` (queryparent): the parenthood of two entities.
pub(super) fn funqp(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(sout, " Give 2 numeros or labels : dad son");
        return Ok(RetError);
    }
    let (n1, n2) = (
        ws.number_from_label(&arg1, 0),
        ws.number_from_label(&arg2, 0),
    );
    let _ = writeln!(sout, "QueryParent for dad:{arg1}:{n1} and son:{arg2}:{n2}");
    let qp = ws.query_parent(&ws.starting_entity(n1), &ws.starting_entity(n2))?;
    let _ = match qp {
        q if q < 0 => writeln!(sout, "{arg1} is not super-entity of {arg2}"),
        0 => writeln!(sout, "{arg1} is same as {arg2}"),
        _ => writeln!(
            sout,
            "{arg1} is super-entity of {arg2} , max level found={qp}"
        ),
    };
    Ok(RetVoid)
}

/// `fun12` (dumpshare).
pub(super) fn fun12(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?.dump_share()?;
    Ok(RetVoid)
}

/// `fun13` (listitems).
pub(super) fn fun13(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?.list_items(&pilot.arg(1)?);
    Ok(RetVoid)
}

/// `fun14` (integer): a new integer parameter recorded. Native quirk kept: the test on
/// the count of words is always false.
pub(super) fn fun14(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?;
    let intpar = IFSelectIntParam::new();
    intpar.set_value(atoi(pilot.arg(1)?.as_bytes()));
    Ok(pilot.record_item(&Handle::new(intpar).cast()))
}

/// `fun15` (setint): the value of an integer parameter item.
pub(super) fn fun15(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    if pilot.nb_words() < 3 {
        let _ = writeln!(
            Message::send_info(),
            "Donner 2 arguments : nom Parametre et Valeur"
        );
        return Ok(RetError);
    }
    let val = atoi(pilot.arg(2)?.as_bytes());
    let par: Handle<IFSelectIntParam> = ws.named_item(&pilot.arg(1)?).cast();
    Ok(if ws.set_int_value(&par, val) {
        RetDone
    } else {
        RetFail
    })
}

/// `fun16` (text): a new text parameter recorded. Native quirk kept: the test on the
/// count of words is always false.
pub(super) fn fun16(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?;
    let textpar = Handle::new(HAsciiString::new(pilot.arg(1)?));
    Ok(pilot.record_item(&textpar.cast()))
}

/// `fun17` (settext): the value of a text parameter item.
pub(super) fn fun17(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    if pilot.nb_words() < 3 {
        let _ = writeln!(
            Message::send_info(),
            "Donner 2 arguments : nom Parametre et Valeur"
        );
        return Ok(RetError);
    }
    let par: Handle<HAsciiString> = ws.named_item(&pilot.arg(1)?).cast();
    Ok(if ws.set_text_value(&par, &pilot.arg(2)?) {
        RetDone
    } else {
        RetFail
    })
}

/// `fun19` (dumpsel): the dump of a selection item.
pub(super) fn fun19(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    if pilot.nb_words() < 2 {
        let _ = writeln!(Message::send_info(), "Give 1 argument : Selection Name");
        return Ok(RetError);
    }
    ws.dump_selection(
        &ws.named_item(&pilot.arg(1)?)
            .cast::<dyn IFSelectSelection>(),
    )?;
    Ok(RetVoid)
}
