// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the selection creation commands.

//! The commands creating combinations, typed, pointed and signature selections, and
//! counters.
use super::selections::{record_selection, two_selections};
use super::{IFSelectFunctions, Status, emit, first_byte, session_of};
use crate::if_select::{
    IFSelectGraphCounter, IFSelectIntParam, IFSelectReturnStatus, IFSelectSelectDeduct,
    IFSelectSelectEntityNumber, IFSelectSelectErrorEntities, IFSelectSelectIncorrectEntities,
    IFSelectSelectIntersection, IFSelectSelectPointed, IFSelectSelectSharing,
    IFSelectSelectSignature, IFSelectSelectUnion, IFSelectSelectUnknownEntities,
    IFSelectSessionPilot, IFSelectSignCounter, IFSelectSignCounterOps, IFSelectSignType,
    IFSelectSignature, IFSelectSignatureList,
};
use crate::interface::{InterfaceInterfaceModel, InterfaceModelEntities};
use rustycad_tkernel::{
    message::{Message, MessageStream},
    standard::{Handle, StandardTransient},
};
use std::fmt::Write as _;

use IFSelectReturnStatus::{RetDone, RetError, RetFail};

/// `fun80` (seladd): an input added to a combination.
pub(super) fn fun80(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(sout, "Donner n0 Combine et une Input");
        return Ok(RetError);
    }
    let (sel, seladd) = two_selections(pilot)?;
    if ws.combine_add(&sel, &seladd, 0) != 0 {
        return Ok(RetDone);
    }
    let _ = writeln!(
        sout,
        "Nom incorrect ou Selection {} pas Combine",
        pilot.arg(1)?
    );
    Ok(RetFail)
}

/// `fun81` (selrem): an input removed from a combination.
pub(super) fn fun81(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 3 {
        let _ = writeln!(sout, "Donner n0 Combine et RANG a supprimer");
        return Ok(RetError);
    }
    let (sel, inp) = two_selections(pilot)?;
    if ws.combine_remove(&sel, &inp)? {
        return Ok(RetDone);
    }
    let _ = writeln!(
        sout,
        "Nom incorrect ou Selection {} ni Union ni Intersection",
        pilot.arg(1)?
    );
    Ok(RetFail)
}

/// `fun82` (number): a new selection of the entity of an integer parameter.
pub(super) fn fun82(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    if pilot.nb_words() < 2 {
        let _ = writeln!(Message::send_info(), "Give IntParam Name for Entity n0");
        return Ok(RetError);
    }
    let par: Handle<IFSelectIntParam> = ws.named_item(&pilot.arg(1)?).cast();
    let sel = IFSelectSelectEntityNumber::new();
    sel.set_number(&par);
    record_selection(pilot, Handle::new(sel).cast())
}

/// `fun83` (union).
pub(super) fn fun83(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?;
    record_selection(pilot, Handle::new(IFSelectSelectUnion::new()).cast())
}

/// `fun84` (intersect).
pub(super) fn fun84(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?;
    record_selection(pilot, Handle::new(IFSelectSelectIntersection::new()).cast())
}

/// A new selection of the entities whose type is (`exact`) or contains the argument.
fn type_selection(pilot: &Handle<IFSelectSessionPilot>, exact: bool) -> Status {
    if pilot.nb_words() < 2 {
        let _ = writeln!(Message::send_info(), "Donner le TYPE a selectionner");
        return Ok(RetError);
    }
    let sign: Handle<dyn IFSelectSignature> = Handle::new(IFSelectSignType::new(false)).cast();
    record_selection(
        pilot,
        Handle::new(IFSelectSelectSignature::with_signature(
            &sign,
            &pilot.arg(1)?,
            exact,
        ))
        .cast(),
    )
}

/// `fun85` (typexact).
pub(super) fn fun85(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    type_selection(pilot, true)
}

/// `fun86` (errors).
pub(super) fn fun86(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    record_selection(
        pilot,
        Handle::new(IFSelectSelectErrorEntities::new()).cast(),
    )
}

/// `fun87` (unknown).
pub(super) fn fun87(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    record_selection(
        pilot,
        Handle::new(IFSelectSelectUnknownEntities::new()).cast(),
    )
}

/// `fun88` (sharing).
pub(super) fn fun88(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    record_selection(pilot, Handle::new(IFSelectSelectSharing::new()).cast())
}

/// `fun89` (typecontain).
pub(super) fn fun89(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    type_selection(pilot, false)
}

/// `fun90` (pointed): a new pointed selection of the list given (if any) recorded.
pub(super) fn fun90(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let sp = IFSelectSelectPointed::new();
    if pilot.nb_words() > 1 {
        let part = String::from_utf8_lossy(pilot.command_part(1).as_bytes()).into_owned();
        let Some(list) = IFSelectFunctions::give_list(&pilot.session(), &part, "")? else {
            return Ok(RetFail);
        };
        let _ = writeln!(
            Message::send_info(),
            "SelectPointed : {} entities",
            list.len()
        );
        sp.add_list(Some(&list));
    }
    record_selection(pilot, Handle::new(sp).cast())
}

/// The listing of a pointed selection in `fun91`.
fn list_pointed(
    sout: &mut MessageStream<'_>,
    ws: &crate::if_select::IFSelectWorkSession,
    sp: &IFSelectSelectPointed,
    arg1: &str,
    model: &Handle<dyn InterfaceInterfaceModel>,
) {
    let nb = sp.nb_items();
    let _ = writeln!(sout, " SelectPointed : {arg1} : {nb} Items :");
    for i in 1..=nb {
        let pointed = sp.item(i);
        if ws.starting_number(&pointed) == 0 {
            let _ = write!(sout, " (inconnu)");
        } else {
            let mut out = b"  ".to_vec();
            let _ = model.print(&pointed, &mut out, 0);
            emit(sout, &out);
        }
    }
    if nb > 0 {
        let _ = writeln!(sout);
    }
}

/// One option of `fun91`: "+nn" add, "-nn" remove, "/nn" toggle (natively a removal
/// too), "0" clear.
fn edit_pointed(
    sout: &mut MessageStream<'_>,
    pilot: &Handle<IFSelectSessionPilot>,
    sp: &IFSelectSelectPointed,
    argi: &str,
    model: &Handle<dyn InterfaceInterfaceModel>,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let ws = session_of(pilot)?;
    let id = pilot.number(argi.get(1..).unwrap_or(""))?;
    if id == 0 {
        if argi == "0" {
            let _ = writeln!(sout, "Clear SelectPointed");
            sp.clear();
        } else {
            let _ = writeln!(sout, "Incorrect,ignore:{argi}");
        }
        return Ok(());
    }
    let item = ws.starting_entity(id);
    let (done, failed) = match first_byte(argi) {
        b'-' => (sp.remove(&item), ("Removed:no.", " Echec Remove ")),
        b'/' => (sp.remove(&item), ("Toggled:n0.", " Echec Toggle ")),
        b'+' => (sp.add(&item), ("Added:no.", " Echec Add ")),
        _ => {
            let _ = writeln!(sout, "Ignore:{argi} , give n0 PRECEDED by + or - or /");
            return Ok(());
        }
    };
    let _ = writeln!(sout, "{}{id}: ", if done { failed.0 } else { failed.1 });
    let mut out = Vec::new();
    let _ = model.print(&item, &mut out, 0);
    emit(sout, &out);
    Ok(())
}

/// `fun91` (setpointed, setlist): a pointed selection listed or edited.
pub(super) fn fun91(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = session_of(pilot)?;
    let argc = pilot.nb_words();
    let arg1 = pilot.arg(1)?;
    let mut sout = Message::send_info();
    if argc < 2 {
        let _ = writeln!(
            sout,
            "Donner NOM SelectPointed + Option(s) :\n aucune : liste des entites pointees\n 0: Clear  +nn ajout entite nn  -nn enleve nn  /nn toggle nn"
        );
        return Ok(RetError);
    }
    let sp: Handle<IFSelectSelectPointed> = ws.named_item(&arg1).cast();
    let Some(pointed) = sp.get() else {
        let _ = writeln!(sout, "Pas une SelectPointed:{arg1}");
        return Ok(RetError);
    };
    let model = ws.model(); // for Print
    if argc == 2 {
        list_pointed(&mut sout, &ws, pointed, &arg1, &model);
        return Ok(RetDone);
    }
    for ia in 2..argc {
        edit_pointed(&mut sout, pilot, pointed, &pilot.arg(ia)?, &model)?;
    }
    Ok(RetDone)
}

/// `fun92` (incorrect): the check computed, a new selection of the incorrect entities.
pub(super) fn fun92(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    session_of(pilot)?.compute_check(false)?;
    record_selection(
        pilot,
        Handle::new(IFSelectSelectIncorrectEntities::new()).cast(),
    )
}

/// `fun93` (signsel): a new selection by the value of a signature or counter.
pub(super) fn fun93(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if argc < 3 {
        let _ = writeln!(
            sout,
            "Give name of Signature or Counter, text + option exact(D) else contains"
        );
        return Ok(RetError);
    }
    let exact = !(argc > 3 && first_byte(&pilot.arg(3)?) == b'c');
    let item = ws.named_item(&arg1);
    let sign: Handle<dyn IFSelectSignature> = item.cast();
    let cnt = item
        .cast::<IFSelectSignatureList>()
        .cast::<dyn IFSelectSignCounterOps>();
    let sel = if sign.is_some() {
        IFSelectSelectSignature::with_signature(&sign, &arg2, exact)
    } else if cnt.is_some() {
        IFSelectSelectSignature::with_counter(&cnt, &arg2, exact)
    } else {
        let _ = writeln!(sout, "{arg1}:neither Signature nor Counter");
        return Ok(RetError);
    };
    record_selection(pilot, Handle::new(sel).cast())
}

/// `fun94` (signcounter): a new counter of a signature recorded.
pub(super) fn fun94(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let arg1 = pilot.arg(1)?;
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Donner nom signature");
        return Ok(RetError);
    }
    let sign: Handle<dyn IFSelectSignature> = ws.named_item(&arg1).cast();
    if sign.is_null() {
        let _ = writeln!(sout, "{arg1}:pas une signature");
        return Ok(RetError);
    }
    let cnt = IFSelectSignCounter::with_signature(&sign, true, true)?;
    Ok(pilot.record_item(&Handle::new(cnt).cast::<StandardTransient>()))
}

/// `funbselected` (nbselected): a new graph counter of a deduction recorded.
pub(super) fn funbselected(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let arg1 = pilot.arg(1)?;
    let ws = session_of(pilot)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 2 {
        let _ = writeln!(sout, "Donner nom selection (deduction) a appliquer");
        return Ok(RetError);
    }
    let applied: Handle<dyn IFSelectSelectDeduct> = ws.give_selection(&arg1).cast();
    if applied.is_null() {
        let _ = writeln!(sout, "{arg1}:pas une SelectDeduct");
        return Ok(RetError);
    }
    let cnt = IFSelectGraphCounter::new(true, true);
    cnt.set_applied(&applied);
    Ok(pilot.record_item(&Handle::new(cnt).cast()))
}
