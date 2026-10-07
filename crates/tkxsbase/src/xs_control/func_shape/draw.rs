// Rust port of DataExchange/TKXSBase/XSControl/XSControl_FuncShape (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: `tpdraw`.

//! `tpdraw`: the results of model entities, transfer items or transfer roots given to
//! shape (or geometry) variables.
use super::{Status, reader_process, xs_session};
use crate::if_select::IFSelectReturnStatus::{RetDone, RetError};
use crate::if_select::{IFSelectFunctions, IFSelectSessionPilot};
use crate::interface::{InterfaceInterfaceModel, InterfaceModelEntities};
use crate::transfer::{TransferSimpleBinderOfTransient, TransferTransientProcess};
use crate::transfer_brep::{TransferBRep, TransferBRepShapeListBinder};
use crate::xs_control::{XSControl, XSControlVarsOps};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    message::{Message, MessageStream},
    standard::{Failure, Handle, StandardTransient, exceptions},
};
use rustycad_tkg3d::geom::GeomGeometry;
use std::fmt::Write as _;

/// The help of `tpdraw`.
const TPDRAW_HELP: &str = "Donner [mode facultatif : item ou root] , NUMERO , nom DRAW facultatif\n  mode si present : item ou root, sinon n0 d entite modele\n  INTEGER NUMBER : of entity, of transfer item or of transfer root\n    or * to say all\n";

/// The request of a `tpdraw` command: the mode (0 model entity, 1 transfer item, 2
/// transfer root), all items or one, the words, the variables.
struct DrawRequest {
    mode: i32,
    tout: bool,
    argc: i32,
    arg2: String,
    arg3: String,
    vars: Handle<dyn XSControlVarsOps>,
}

/// The state carried along the items: the count of variables (-1 for a single item:
/// its first simple shape takes no number) and the last shape (natively shared).
struct DrawState {
    nbvar: i32,
    sh: Shape,
}

impl DrawRequest {
    /// The root of the variable names given by the command (`None` for "tp_<i>").
    fn base(&self) -> Option<&str> {
        if self.argc > 3 && self.mode > 0 {
            return Some(&self.arg3);
        }
        (self.argc > 2 && self.mode == 0).then_some(self.arg2.as_str())
    }
    /// The name of a single shape or geometry.
    fn single_name(&self, i: i32, nbvar: i32) -> String {
        match self.base() {
            Some(b) if nbvar == 0 => b.to_owned(),
            Some(b) => format!("{b}_{nbvar}"),
            None => format!("tp_{i}"),
        }
    }
    /// The name of a shape of a list.
    fn list_name(&self, i: i32, nbvar: i32) -> String {
        match self.base() {
            Some(b) => format!("{b}_{nbvar}"),
            None => format!("tp_{i}_{nbvar}"),
        }
    }
    /// The prefix of an item: its rank (all items), its entity, its transfer item.
    fn item_prefix(&self, i: i32, num: i32, index: i32, sout: &mut MessageStream<'_>) {
        if self.tout {
            let _ = write!(sout, "[ {i} ]:");
        }
        let _ = match num {
            0 => sout.write_str(" pas dans le modele"),
            n => write!(sout, " ent.n0 {n}"),
        };
        let _ = write!(sout, ", item transfert n0 {index}");
    }
}

/// `tpdraw`: see the module documentation.
pub(super) fn tpdraw(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let ws = xs_session(pilot)?;
    let tp = reader_process(&ws)?;
    let mut sout = Message::send_info();
    let Some(process) = tp.get() else {
        let _ = writeln!(sout, "No Transfer Read");
        return Ok(RetError);
    };
    if argc < 2 {
        let _ = sout.write_str(TPDRAW_HELP);
        return Ok(RetError);
    }
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let mode = match arg1.bytes().next() {
        Some(b'i') => 1,
        Some(b'r') => 2,
        _ => 0,
    };
    let which = if mode == 0 { &arg1 } else { &arg2 };
    let tout = which.starts_with('*');
    let num = if tout {
        0
    } else {
        IFSelectFunctions::give_entity_number(&ws.clone().cast(), which)?
    };
    let req = DrawRequest {
        mode,
        tout,
        argc,
        arg2,
        arg3: pilot.arg(3)?,
        vars: XSControl::vars(pilot)?,
    };
    let model = process.model();
    if model.is_null() && mode == 0 {
        let _ = writeln!(sout, "No model, specify n0 of transfer item");
        return Ok(RetError);
    }
    let Some((n1, n2, nbvar)) = draw_range(&req, process, &model, num, &mut sout) else {
        return Ok(RetError);
    };
    let mut state = DrawState {
        nbvar,
        sh: Shape::default(),
    };
    for i in n1..=n2 {
        draw_item(&req, &tp, &model, i, &mut state, &mut sout)?;
    }
    if state.sh.is_null() {
        let _ = writeln!(sout, " (No Shape)");
    }
    Ok(RetDone)
}

/// The items to draw (`n1`, `n2`, the first `nbvar`), with the header; `None` (with a
/// message) for a number out of the items.
fn draw_range(
    req: &DrawRequest,
    process: &TransferTransientProcess,
    model: &Handle<dyn InterfaceInterfaceModel>,
    num: i32,
    sout: &mut MessageStream<'_>,
) -> Option<(i32, i32, i32)> {
    let (title, max) = match req.mode {
        0 => ("Model entity", model.get().map_or(0, |m| m.nb_entities())),
        1 => ("Transfer item", process.nb_mapped()),
        _ => ("Transfer root", process.nb_roots()),
    };
    let _ = sout.write_str(title);
    if req.tout {
        let _ = writeln!(sout, ", listing from 1 to {max}");
        return Some((1, max, 0));
    }
    if num <= 0 || num > max {
        let _ = writeln!(sout, " - Num={num} hors limite (de 1 a {max})");
        return None;
    }
    let _ = writeln!(sout, ", n0 {num}");
    Some((num, num, -1))
}

/// The entity, its number in the model and its transfer item index for the rank `i`.
fn locate(
    mode: i32,
    process: &TransferTransientProcess,
    model: &Handle<dyn InterfaceInterfaceModel>,
    i: i32,
) -> Result<(i32, i32), Failure> {
    let number = |ent: &Handle<StandardTransient>| model.get().map_or(0, |m| m.number(ent));
    match mode {
        0 => {
            let m = model
                .get()
                .ok_or_else(|| exceptions::null_object("tpdraw: no model"))?;
            Ok((i, process.map_index(&m.value(i)?)))
        }
        1 => Ok((number(&process.mapped(i)?), i)),
        _ => {
            let ent = process.root(i)?;
            Ok((number(&ent), process.map_index(&ent)))
        }
    }
}

/// One item drawn: its shape, its list of shapes or its geometry given to variables.
fn draw_item(
    req: &DrawRequest,
    tp: &Handle<TransferTransientProcess>,
    model: &Handle<dyn InterfaceInterfaceModel>,
    i: i32,
    state: &mut DrawState,
    sout: &mut MessageStream<'_>,
) -> Result<(), Failure> {
    let process = tp
        .get()
        .ok_or_else(|| exceptions::null_object("tpdraw: no process"))?;
    let (num, index) = locate(req.mode, process, model, i)?;
    let binder = if index > 0 {
        process.map_item(index)?
    } else {
        Handle::default()
    };
    let Some(live) = binder.get().filter(|_| index > 0) else {
        if !req.tout {
            let _ = writeln!(sout, "Entite n0 {num} : non repertoriee");
        }
        return Ok(());
    };
    if !live.has_result() {
        if !req.tout {
            let _ = writeln!(sout, "Entity n0 {num} : no result");
        }
        return Ok(());
    }
    state.sh = TransferBRep::shape_result(&binder);
    if !state.sh.is_null() {
        state.nbvar += 1;
        req.item_prefix(i, num, index, sout);
        let name = req.single_name(i, state.nbvar);
        let _ = writeln!(sout, " -> 1 DRAW Shape: {name}");
        return set_shape(req, &name, &state.sh);
    }
    if let Some(slb) = binder.cast::<TransferBRepShapeListBinder>().get() {
        req.item_prefix(i, num, index, sout);
        return draw_shape_list(req, slb, i, state, sout);
    }
    if let Some(trb) = binder.cast::<TransferSimpleBinderOfTransient>().get() {
        return draw_geom(req, trb, i, num, state, sout);
    }
    if !req.tout {
        let _ = writeln!(
            sout,
            "Entite n0 {num} : resultat pas une Shape mais {}",
            live.result_type_name()
        );
    }
    Ok(())
}

/// The variables of the session, `Standard_NullObject` when none (a native null
/// dereference).
fn live_vars(req: &DrawRequest) -> Result<&dyn XSControlVarsOps, Failure> {
    req.vars
        .get()
        .ok_or_else(|| exceptions::null_object("tpdraw: the session has no variables"))
}

/// A shape given to a variable.
fn set_shape(req: &DrawRequest, name: &str, shape: &Shape) -> Result<(), Failure> {
    live_vars(req)?.set_shape(name, shape);
    Ok(())
}

/// The shapes of a list binder given to numbered variables (each one the last shape).
fn draw_shape_list(
    req: &DrawRequest,
    slb: &TransferBRepShapeListBinder,
    i: i32,
    state: &mut DrawState,
    sout: &mut MessageStream<'_>,
) -> Result<(), Failure> {
    let nbs = slb.nb_shapes()?;
    let _ = write!(sout, " -> {nbs} DRAW Shapes :");
    for j in 1..=nbs {
        state.sh = slb.shape(j)?;
        state.nbvar = state.nbvar.max(0) + 1;
        if state.sh.is_null() {
            let _ = writeln!(sout, " (no Shape recorded)");
            continue;
        }
        let name = req.list_name(i, state.nbvar);
        let _ = write!(sout, " {name}");
        set_shape(req, &name, &state.sh)?;
    }
    let _ = writeln!(sout);
    Ok(())
}

/// The transient result of a simple binder given to a variable when it is a geometry.
fn draw_geom(
    req: &DrawRequest,
    trb: &TransferSimpleBinderOfTransient,
    i: i32,
    num: i32,
    state: &mut DrawState,
    sout: &mut MessageStream<'_>,
) -> Result<(), Failure> {
    let resu = trb.result();
    let Some(kind) = resu.dynamic_type() else {
        let _ = writeln!(sout, "Entity n0 {num} : no result");
        return Ok(());
    };
    let _ = write!(sout, "Entite n0 {num} : resultat {}", kind.name());
    let geom: Handle<dyn GeomGeometry> = resu.cast();
    if geom.is_null() {
        let _ = writeln!(sout);
        return Ok(());
    }
    state.nbvar += 1;
    let name = req.single_name(i, state.nbvar);
    live_vars(req)?.set(&name, &geom.cast());
    let _ = writeln!(sout, " -> DRAW Geom : {name}");
    Ok(())
}
