// Rust port of DataExchange/TKXSBase/XSControl/XSControl_FuncShape (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: `tpcompound` and the access to the
// results of the transfer reader.

//! `tpcompound` (a compound of the transferred shapes) and `trdraw`, `trsave`,
//! `trcomp`, `trscomp` (the results of the transfer reader given to variables or saved
//! as BRep files, one by one or as one compound).
use super::{Status, reader_process, text, xs_session, xs_vars};
use crate::if_select::IFSelectReturnStatus::{RetDone, RetError};
use crate::if_select::{IFSelectFunctions, IFSelectSessionPilot};
use crate::interface::InterfaceModelEntities;
use crate::transfer_brep::TransferBRep;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{Builder, Compound, Shape};
use rustycad_tkernel::{
    message::{Message, MessageStream, ProgressRange},
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use std::fmt::Write as _;

/// A new empty compound.
fn new_compound() -> Shape {
    let mut c = Compound::default();
    Builder.make_compound(&mut c);
    c.into_shape()
}

/// `tpcompound`: a compound of the root results of the transfer (or of the results of
/// the list the command part 2 gives) given to the variable `arg1`.
pub(super) fn tpcompound(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = xs_session(pilot)?;
    let tp = reader_process(&ws)?;
    let mut sout = Message::send_info();
    if tp.is_null() {
        let _ = writeln!(sout, "No Transfer Read");
        return Ok(RetError);
    }
    if pilot.nb_words() < 2 {
        let _ = writeln!(
            sout,
            "Give a NAME for the Compound  + optional givelist, else roots are taken"
        );
        return Ok(RetError);
    }
    let list = if pilot.nb_words() == 2 {
        TransferBRep::shapes(&tp, true)?
    } else {
        let part = text(&pilot.command_part(2));
        let Some(lise) = IFSelectFunctions::give_list(&pilot.session(), &part, "")? else {
            let _ = writeln!(sout, "Not a valid entity list : {part}");
            return Ok(RetError);
        };
        let _ = write!(sout, "{} Entities, ", lise.len());
        TransferBRep::shapes_of(&tp, Some(&lise))?
    };
    let Some(list) = list else {
        let _ = writeln!(sout, "No Shape listed");
        return Ok(RetError);
    };
    let _ = writeln!(sout, "{} Shape(s) listed", list.len());
    let mut compound = new_compound();
    for shape in &list {
        Builder.add(&mut compound, shape)?;
    }
    xs_vars(pilot)?.set_shape(&pilot.arg(1)?, &compound);
    Ok(RetDone)
}

/// The kind of a `traccess` command: one compound (an 'o' in the first five
/// characters), saved to files (an 's').
#[derive(Clone, Copy)]
struct AccessKind {
    cascomp: bool,
    cassave: bool,
}

/// A shape given to the variable `name` or saved to the BRep file `name`.
fn deliver(
    pilot: &Handle<IFSelectSessionPilot>,
    save: bool,
    name: &str,
    shape: &Shape,
) -> Result<(), Failure> {
    if save {
        brep_tools::write_file(shape, &AsciiString::new(name), &ProgressRange::default())?;
        return Ok(());
    }
    xs_vars(pilot)?.set_shape(name, shape);
    Ok(())
}

/// The progress mark of the result `i` (every 10, 100, 1000 results).
fn progress_mark(i: usize, sout: &mut MessageStream<'_>) {
    let _ = match i {
        _ if i.is_multiple_of(1000) => writeln!(sout, "({i})"),
        _ if i.is_multiple_of(100) => sout.write_str("*"),
        _ if i.is_multiple_of(10) => sout.write_str("0"),
        _ => sout.write_str("."),
    };
}

/// `trdraw`, `trsave`, `trcomp`, `trscomp`: the results of the transfer reader (all, or
/// of the entity `arg1`) given to variables or saved, one by one or as one compound.
pub(super) fn traccess(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let word = pilot.word(0)?;
    let kind = AccessKind {
        cascomp: word.location_char(1, b'o', 1, 5)? > 0,
        cassave: word.location_char(1, b's', 1, 5)? > 0,
    };
    let ws = xs_session(pilot)?;
    let tr = ws.transfer_reader();
    let mut sout = Message::send_info();
    let Some(reader) = tr.get() else {
        let _ = writeln!(sout, " manque init");
        return Ok(RetError);
    };
    if reader.model().is_null() {
        let _ = writeln!(sout, " modele absent");
        return Ok(RetError);
    }
    let arg1 = pilot.arg(1)?;
    let num = if argc > 1 {
        IFSelectFunctions::give_entity_number(&ws.clone().cast(), &arg1)?
    } else {
        0
    };
    let nomsh = match (argc > 1, kind.cascomp) {
        (true, _) => arg1.clone(),
        (false, true) => String::from("TREAD_COMP"),
        (false, false) => String::from("TREAD_LIST"),
    };
    if kind.cassave {
        let _ = writeln!(sout, " save shapes -> current directory");
    }
    if num == 0 || kind.cascomp {
        access_all(pilot, reader, kind, &nomsh, &mut sout)?;
        return Ok(RetDone);
    }
    access_one(pilot, reader, kind, num, &mut sout)
}

/// All the results of the reader (`ShapeResultList`), each given or saved, or one
/// compound of them.
fn access_all(
    pilot: &Handle<IFSelectSessionPilot>,
    reader: &crate::xs_control::XSControlTransferReader,
    kind: AccessKind,
    nomsh: &str,
    sout: &mut MessageStream<'_>,
) -> Result<(), Failure> {
    let list = reader.shape_result_list(true)?;
    let _ = writeln!(
        sout,
        " TOUS RESULTATS par ShapeResultList, soit {}",
        list.len()
    );
    let mut compound = new_compound();
    for (i, shape) in (1..).zip(&list) {
        progress_mark(i, sout);
        if shape.is_null() {
            continue;
        }
        if kind.cascomp {
            Builder.add(&mut compound, shape)?;
        } else {
            deliver(pilot, kind.cassave, &format!("{nomsh}_{i}"), shape)?;
        }
    }
    let _ = writeln!(sout);
    if kind.cascomp {
        deliver(pilot, kind.cassave, nomsh, &compound)?;
    }
    Ok(())
}

/// The result of the entity `num`, given or saved under `arg2` (else "TREAD_<num>").
fn access_one(
    pilot: &Handle<IFSelectSessionPilot>,
    reader: &crate::xs_control::XSControlTransferReader,
    kind: AccessKind,
    num: i32,
    sout: &mut MessageStream<'_>,
) -> Status {
    let arg1 = pilot.arg(1)?;
    let mdl = reader.model();
    let Some(m) = mdl.get().filter(|m| (1..=m.nb_entities()).contains(&num)) else {
        let _ = writeln!(sout, " incorrect:{arg1}");
        return Ok(RetError);
    };
    let sh = reader.shape_result(&m.value(num)?)?;
    if sh.is_null() {
        let _ = writeln!(sout, " No result for {arg1}");
        return Ok(RetError);
    }
    let nomsh = if pilot.nb_words() > 2 {
        pilot.arg(2)?
    } else {
        format!("TREAD_{num}")
    };
    if kind.cascomp {
        let _ = writeln!(sout, "Option non comprise");
        return Ok(RetDone);
    }
    deliver(pilot, kind.cassave, &nomsh, &sh)?;
    Ok(RetDone)
}
