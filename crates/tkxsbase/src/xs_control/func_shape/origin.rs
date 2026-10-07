// Rust port of DataExchange/TKXSBase/XSControl/XSControl_FuncShape (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: `fromshape` and `trconnexent`.

//! `fromshape` (the entity a shape variable was imported from or exported to) and
//! `trconnexent` (the entities whose shapes are connected to a shape variable).
use super::{Status, text, xs_session, xs_vars};
use crate::if_select::IFSelectReturnStatus::{RetDone, RetError, RetVoid};
use crate::if_select::IFSelectSessionPilot;
use crate::interface::{InterfaceInterfaceModel, InterfaceModelEntities};
use crate::transfer::{TransferFinder, TransferTransientListBinder, TransferTransientProcess};
use crate::transfer_brep::TransferBRep;
use crate::xs_control::{XSControlConnectedShapes, XSControlTransferReader, XSControlWorkSession};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::{
    message::{Message, MessageStream},
    standard::{Failure, Handle, StandardTransient, exceptions},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;
use std::fmt::Write as _;

/// `XSControl_IsEqualSubShape`: `sh` is the same as `shape`, or (down to `level`
/// levels) one of its sub-shapes is.
fn is_equal_sub_shape(shape: &Shape, sh: &Shape, level: i32) -> Result<bool, Failure> {
    if sh.is_same(shape) {
        return Ok(true);
    }
    if level <= 0 {
        return Ok(false);
    }
    let mut it = ShapeIterator::new(sh, true, true);
    while it.more() {
        if is_equal_sub_shape(shape, it.value()?, level - 1)? {
            return Ok(true);
        }
        it.next();
    }
    Ok(false)
}

/// The request of `fromshape`: the variable name, the shape, the level, silent mode.
struct FromShape {
    arg1: String,
    shape: Shape,
    level: i32,
    silent: bool,
}

/// An entity printed by the model of the session (`Model()->Print(ent, S)`);
/// `Standard_NullObject` without model (a native null dereference).
fn print_entity(
    ws: &XSControlWorkSession,
    ent: &Handle<StandardTransient>,
    sout: &mut MessageStream<'_>,
) -> Result<(), Failure> {
    let model = ws.model();
    let m = model
        .get()
        .ok_or_else(|| exceptions::null_object("fromshape: the session has no model"))?;
    let mut out = Vec::new();
    let _ = m.print(ent, &mut out, 0);
    let _ = sout.write_str(&String::from_utf8_lossy(&out));
    Ok(())
}

/// `fromshape`: see the module documentation.
pub(super) fn fromshape(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    if argc < 2 {
        let _ = writeln!(sout, "Give name of a DRAW Shape");
        return Ok(RetError);
    }
    let arg1 = pilot.arg(1)?;
    let shape = xs_vars(pilot)?.get_shape(&mut arg1.clone());
    if shape.is_null() {
        let _ = writeln!(sout, "Not a DRAW Shape:{arg1}");
        return Ok(RetError);
    }
    let level = if argc >= 3 {
        crate::c_text::atoi(pilot.arg(2)?.as_bytes())
    } else {
        1
    };
    let req = FromShape {
        arg1,
        shape,
        level: level.abs(),
        silent: level < 0,
    };
    let ws = xs_session(pilot)?;
    let mut yena = false;
    if let Some(reader) = ws.transfer_reader().get() {
        yena = true;
        imported_from(&ws, reader, &req, &mut sout)?;
    }
    let fp = ws
        .transfer_writer()
        .get()
        .map(|w| w.finder_process())
        .unwrap_or_default();
    if fp.is_some() {
        yena = true;
        exported_to(&ws, &fp, &req, &mut sout)?;
    }
    if !yena {
        let _ = writeln!(sout, "No transfer (either import or export) recorded");
    }
    Ok(RetVoid)
}

/// The entity of a shape result: as recorded (1), else as last transferred (-1).
fn entity_of_result(
    reader: &XSControlTransferReader,
    shape: &Shape,
) -> Result<(Handle<StandardTransient>, i32), Failure> {
    let ent = reader.entity_from_shape_result(shape, 1)?;
    if ent.is_some() {
        return Ok((ent, 1));
    }
    Ok((reader.entity_from_shape_result(shape, -1)?, -1))
}

/// The mapped entity whose shape result holds the shape (down to the level), else
/// holds it moved to the origin (`modrec` -2); `None` when none.
fn search_map(
    process: &TransferTransientProcess,
    tp: &Handle<TransferTransientProcess>,
    req: &FromShape,
) -> Result<Option<(Handle<StandardTransient>, i32)>, Failure> {
    let s0 = req.shape.located(&Location::default(), false)?;
    for i in 1..=process.nb_mapped() {
        let ent = process.mapped(i)?;
        let sh = TransferBRep::shape_result_of(tp, &ent)?;
        if sh.is_null() {
            continue;
        }
        if is_equal_sub_shape(&req.shape, &sh, req.level)? {
            return Ok(Some((ent, 2)));
        }
        if is_equal_sub_shape(&s0, &sh.located(&Location::default(), false)?, req.level)? {
            return Ok(Some((ent, -2)));
        }
    }
    Ok(None)
}

/// The import part of `fromshape`: the entity the shape (or its sub-shapes, for a
/// compound) was imported from.
fn imported_from(
    ws: &XSControlWorkSession,
    reader: &XSControlTransferReader,
    req: &FromShape,
    sout: &mut MessageStream<'_>,
) -> Result<(), Failure> {
    let arg1 = &req.arg1;
    if !req.silent {
        let _ = write!(sout, "Shape {arg1} : ");
    }
    let (mut ent, mut modrec) = entity_of_result(reader, &req.shape)?;
    if ent.is_null() {
        let tp = reader.transient_process();
        match tp.get() {
            None => {
                let _ = writeln!(
                    sout,
                    "{}no map",
                    if req.silent {
                        format!("Shape {arg1} : ")
                    } else {
                        String::new()
                    }
                );
            }
            Some(process) => {
                if !req.silent {
                    let _ = write!(sout, "searching in map among {} ...", process.nb_mapped());
                }
                (ent, modrec) = search_map(process, &tp, req)?.unwrap_or_default();
            }
        }
    }
    if ent.is_some() {
        let _ = write!(
            sout,
            "{}{}",
            if req.silent {
                format!("Shape {arg1}: ")
            } else {
                String::new()
            },
            if modrec < 0 {
                "(moved from origin) "
            } else {
                ""
            }
        );
        let _ = sout.write_str("imported from entity ");
        print_entity(ws, &ent, sout)?;
        if req.silent {
            let _ = writeln!(sout, " in file {}", text(&ws.loaded_file()));
        }
    } else {
        unknown_import(ws, reader, req, sout)?;
    }
    if !req.silent {
        let _ = writeln!(sout);
    }
    Ok(())
}

/// A shape unknown as imported: for a compound, the entities its sub-shapes were
/// imported from.
fn unknown_import(
    ws: &XSControlWorkSession,
    reader: &XSControlTransferReader,
    req: &FromShape,
    sout: &mut MessageStream<'_>,
) -> Result<(), Failure> {
    if !req.silent {
        let _ = sout.write_str(" unknown as imported");
    }
    if req.shape.shape_type()? != ShapeEnum::Compound {
        return Ok(());
    }
    let _ = write!(sout, "\nSubshapes imported from entities:");
    let mut it = ShapeIterator::new(&req.shape, true, true);
    while it.more() {
        let (subent, _) = entity_of_result(reader, it.value()?)?;
        if subent.is_some() {
            let model = ws.model();
            let m = model
                .get()
                .ok_or_else(|| exceptions::null_object("fromshape: the session has no model"))?;
            let _ = write!(sout, "  {}", m.number(&subent));
        }
        it.next();
    }
    Ok(())
}

/// The export part of `fromshape`: the entity (or the entities of a transient list
/// binder) the shape was exported to.
fn exported_to(
    ws: &XSControlWorkSession,
    fp: &Handle<crate::transfer::TransferFinderProcess>,
    req: &FromShape,
    sout: &mut MessageStream<'_>,
) -> Result<(), Failure> {
    let process = fp
        .get()
        .ok_or_else(|| exceptions::null_object("fromshape: no finder process"))?;
    let fnd: Handle<dyn TransferFinder> = TransferBRep::shape_mapper(fp, &req.shape)?.cast();
    let in_file = |sout: &mut MessageStream<'_>| {
        if req.silent {
            let _ = write!(sout, " in file {}", text(&ws.loaded_file()));
        }
    };
    let ent = process.find_transient(&fnd);
    if ent.is_some() {
        let _ = write!(sout, "Shape {}: exported to entity ", req.arg1);
        print_entity(ws, &ent, sout)?;
        in_file(sout);
        let _ = writeln!(sout);
        return Ok(());
    }
    let bnd = process.find(&fnd).cast::<TransferTransientListBinder>();
    let Some(list) = bnd.get() else {
        return Ok(());
    };
    let nb = list.nb_transients();
    if nb > 0 {
        let _ = write!(sout, "Shape {}: exported to entities ", req.arg1);
    }
    for i in 1..=nb {
        print_entity(ws, &list.transient(i)?, sout)?;
        if i < nb {
            let _ = sout.write_str(", ");
        }
    }
    if nb > 0 {
        in_file(sout);
        let _ = writeln!(sout);
    }
    Ok(())
}

/// `trconnexent`: the entities whose face results share a vertex with the shape
/// variable `arg1`.
pub(super) fn trconnexentities(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = xs_session(pilot)?;
    let tp = ws
        .transfer_reader()
        .get()
        .map(XSControlTransferReader::transient_process)
        .unwrap_or_default();
    let mut sout = Message::send_info();
    if tp.is_null() {
        let _ = writeln!(sout, "no transfer map");
        return Ok(RetVoid);
    }
    if pilot.nb_words() < 2 {
        let _ = writeln!(
            sout,
            "Give name of a DRAW Shape + optional shape type v-e-w-f(D)-s"
        );
        return Ok(RetError);
    }
    let arg1 = pilot.arg(1)?;
    let shape = xs_vars(pilot)?.get_shape(&mut arg1.clone());
    if shape.is_null() {
        let _ = writeln!(sout, "Not a DRAW Shape:{arg1}");
        return Ok(RetError);
    }
    let _ = write!(sout, "Shape {arg1} : ");
    let list = XSControlConnectedShapes::adjacent_entities(&shape, &tp, ShapeEnum::Face)?;
    let _ = writeln!(sout, "{} Entities produced Connected Shapes :", list.len());
    let model = ws.model();
    let mut numbers = Vec::new();
    for ent in &list {
        let m: &dyn InterfaceInterfaceModel = model
            .get()
            .ok_or_else(|| exceptions::null_object("trconnexent: the session has no model"))?;
        numbers.push(m.number(ent).to_string());
    }
    let _ = writeln!(sout, "({})", numbers.join(","));
    Ok(RetDone)
}
