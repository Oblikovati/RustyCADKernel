// Rust port of DataExchange/TKXSBase/XSControl/XSControl_FuncShape (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: `trimport` and `twrite`.

//! `trimport`, `trimpcomp` (a file, or the loaded model, transferred to shape
//! variables or to one compound) and `twrite` (shape variables written to the model).
use super::{Status, XSControlFuncShape, text, xs_session, xs_vars};
use crate::if_select::IFSelectReturnStatus::{RetDone, RetError};
use crate::if_select::IFSelectSessionPilot;
use crate::xs_control::{XSControlTransferReader, XSControlWorkSession};
use rustycad_tkbrep::topods::{Builder, Compound, Shape};
use rustycad_tkernel::{
    message::{Message, MessageStream, ProgressRange},
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};
use std::fmt::Write as _;

/// The help of `trimport`.
const TRIMPORT_HELP: &str = "Give : filename or . for current model;  varname or . to take fileroot\n  GiveList, * for all transferable roots\n";

/// `trimport`, `trimpcomp`: the file `arg1` loaded (or the loaded model for "."), the
/// list `arg3...` gives (or all the transferable roots for "*") transferred, the shapes
/// given to the variables `<var>_<n>` (or one compound for `trimpcomp`).
pub(super) fn trimport(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let ws = xs_session(pilot)?;
    let mut sout = Message::send_info();
    if pilot.nb_words() < 4 {
        let _ = sout.write_str(TRIMPORT_HELP);
        return Ok(RetError);
    }
    let (arg1, arg3) = (pilot.arg(1)?, pilot.arg(3)?);
    let names = XSControlFuncShape::file_and_var(&ws, &arg1, &pilot.arg(2)?, "IMPORT")?;
    let title = if names.from_file {
        " File to read : "
    } else {
        " Model taken from the session : "
    };
    let _ = writeln!(sout, "{title}{}", text(&names.file));
    let rnom = text(&names.var);
    let _ = writeln!(
        sout,
        " -- Names of variables BREP-DRAW prefixed by : {rnom}"
    );
    let compart = text(&pilot.command_part(3));
    if names.from_file {
        let status = pilot.execute(&AsciiString::new(&format!("xload {arg1}")))?;
        if status != RetDone {
            let _ = writeln!(sout, "Abandon import");
            return Ok(status);
        }
    } else {
        let _ = writeln!(sout, "Currently Loaded Model");
    }
    let list = if arg3 == "*" {
        let _ = sout.write_str("All Transferable Roots : ");
        ws.give_list_texts("xst-transferrable-roots", "")?
    } else {
        let _ = write!(sout, "List given by {compart} : ");
        ws.give_list_texts(&compart, "")?
    };
    let Some(list) = list else {
        let _ = writeln!(sout, "No list defined. Abandon");
        return Ok(RetError);
    };
    let iscomp = pilot.arg(0)?.as_bytes().get(5) == Some(&b'c');
    import_list(pilot, &ws, &list, iscomp, &rnom, &mut sout)
}

/// The list transferred by a new transfer reader and its shapes given to variables.
fn import_list(
    pilot: &Handle<IFSelectSessionPilot>,
    ws: &XSControlWorkSession,
    list: &[Handle<StandardTransient>],
    iscomp: bool,
    rnom: &str,
    sout: &mut MessageStream<'_>,
) -> Status {
    let nbl = list.len();
    let _ = writeln!(sout, "Nb entities selected : {nbl}");
    ws.init_transfer_reader(0)?;
    let tr = ws.transfer_reader();
    let Some(reader) = tr.get() else {
        let _ = writeln!(sout, " init not done or failed");
        return Ok(RetError);
    };
    reader.begin_transfer();
    let nbt = reader.transfer_list(list, true, &ProgressRange::default())?;
    let _ = writeln!(
        sout,
        "Nb Entities Selected : {nbl} have given {nbt} results"
    );
    if reader.model().is_null() {
        let _ = writeln!(sout, " modele absent");
        return Ok(RetError);
    }
    let (nbs, last, compound) = collect_shapes(pilot, reader, list, iscomp, rnom)?;
    let vars = xs_vars(pilot)?;
    match nbs {
        0 => {
            let _ = writeln!(sout, "No Shape produced");
        }
        1 => {
            let _ = writeln!(sout, "One Shape produced, named {rnom}");
            vars.set_shape(rnom, &last);
        }
        _ if iscomp => {
            let _ = writeln!(sout, "One compound made of {nbs} Shapes, named {rnom}");
            vars.set_shape(rnom, &compound);
        }
        _ => {
            let _ = writeln!(sout, "{nbs} Shapes, named {rnom}_1 to {rnom}_{nbs}");
        }
    }
    Ok(RetDone)
}

/// The shape results of the list: each given to `<rnom>_<n>`, or added to a compound;
/// their count, the last result read (native quirk kept: possibly null, then given as
/// the single shape) and the compound.
fn collect_shapes(
    pilot: &Handle<IFSelectSessionPilot>,
    reader: &XSControlTransferReader,
    list: &[Handle<StandardTransient>],
    iscomp: bool,
    rnom: &str,
) -> Result<(i32, Shape, Shape), Failure> {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let mut compound = compound.into_shape();
    let (mut nbs, mut last) = (0, Shape::default());
    for ent in list {
        last = reader.shape_result(ent)?;
        if last.is_null() {
            continue;
        }
        nbs += 1;
        if iscomp {
            Builder.add(&mut compound, &last)?;
        } else {
            xs_vars(pilot)?.set_shape(&format!("{rnom}_{nbs}"), &last);
        }
    }
    Ok((nbs, last, compound))
}

/// `twrite`: each shape variable written by the transfer writer to the model of the
/// session, then the graph computed. Native quirk kept: an unknown name is reported as
/// the first one.
pub(super) fn twrite(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    let ws = xs_session(pilot)?;
    let tw = ws.transfer_writer();
    if argc < 2 {
        let _ = writeln!(sout, " give draw shape name");
        return Ok(RetError);
    }
    let _ = writeln!(sout, "Attention, on alimente le modele courant ...");
    let arg1 = pilot.arg(1)?;
    for i in 1..argc {
        let ai = pilot.arg(i)?;
        let shape = xs_vars(pilot)?.get_shape(&mut ai.clone());
        if shape.is_null() {
            let _ = writeln!(sout, "not a draw shape name:{arg1}");
            continue;
        }
        let _ = write!(sout, "Pour Shape : {ai}");
        let writer = tw.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "twrite: the session has no transfer writer",
            )
        })?;
        let stat = writer.transfer_write_shape(&ws.model(), &shape, &ProgressRange::default())?;
        let _ = writeln!(sout, " Transfer Write Status = {}", stat as i32);
    }
    if let Some(session) = pilot.session().get() {
        session.compute_graph(false)?;
    }
    Ok(RetDone)
}
