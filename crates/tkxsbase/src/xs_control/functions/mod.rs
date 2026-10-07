// Rust port of DataExchange/TKXSBase/XSControl/XSControl_Functions (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSControl_Functions`: the commands of the norm and of the transfers ("DE:
//! General"), registered as acts on the session pilot by `Init`. The commands are
//! split by subject: `norms` (norm, model, transfer processes), `transfers` (the
//! transfer reader and writer).
use super::{XSControl, XSControlTransferReader, XSControlWorkSession};
use crate::if_select::{IFSelectAct, IFSelectActFunc, IFSelectReturnStatus, IFSelectSessionPilot};
use rustycad_tkernel::{
    message::MessageStream,
    standard::{Failure, Handle, exceptions},
};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};

mod norms;
mod transfers;

use norms::*;
use transfers::*;

/// The result of a command.
type Status = Result<IFSelectReturnStatus, Failure>;

/// `XSControl_Functions`: the static members.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectActivatorBase;
/// use rustycad_tkxsbase::xs_control::XSControlFunctions;
/// XSControlFunctions::init().unwrap();
/// assert!(IFSelectActivatorBase::select("tpstat").is_some());
/// ```
pub struct XSControlFunctions;

/// The native static `THE_XSControl_Functions_initactor`.
static INIT_ACTOR: AtomicBool = AtomicBool::new(false);

/// A command: its name, its help, its function, and true for a command creating an
/// item (`AddFSet`).
type Command = (&'static str, &'static str, IFSelectActFunc, bool);

/// The commands of `Init`, in the native order.
const COMMANDS: &[Command] = &[
    (
        "xinit",
        "[norm:string to change norme] reinitialises according to the norm",
        xinit,
        false,
    ),
    (
        "xnorm",
        "displays current norm   +norm : changes it",
        xnorm,
        false,
    ),
    (
        "newmodel",
        "produces a new empty model, for the session",
        newmodel,
        false,
    ),
    ("tpclear", "Clears  TransferProcess (READ)", tpclear, false),
    ("twclear", "Clears  TransferProcess (WRITE)", tpclear, false),
    (
        "tpstat",
        "Statistics on TransferProcess (READ)",
        tpstat,
        false,
    ),
    (
        "tpent",
        "[num:integer] Statistics on an entity of the model (READ)",
        tpent,
        false,
    ),
    (
        "tpitem",
        "[num:integer] Statistics on ITEM of transfer (READ)",
        tpitem,
        false,
    ),
    (
        "tproot",
        "[num:integer] Statistics on a ROOT of transfert (READ)",
        tpitem,
        false,
    ),
    (
        "twitem",
        "[num:integer] Statistics on an ITEM of transfer (WRITE)",
        tpitem,
        false,
    ),
    (
        "twroot",
        "[num:integer] Statistics on a ROOT of transfer (WRITE)",
        tpitem,
        false,
    ),
    (
        "trecord",
        "record : all root results; or num : for entity n0.num",
        trecord,
        false,
    ),
    (
        "trstat",
        "general statistics;  or num : stats on entity n0 num",
        trstat,
        false,
    ),
    ("trbegin", "begin-transfer-reader [init]", trbegin, false),
    (
        "tread",
        "transfers all roots, or num|sel|sel num : entity list, by transfer-reader",
        tread,
        false,
    ),
    (
        "trtp",
        "feeds commands tp... with results from tr...",
        trtp,
        false,
    ),
    (
        "tptr",
        "feeds tr... from tp... (may be incomplete)",
        tptr,
        false,
    ),
    (
        "twmode",
        "displays mode transfer write, + num  changes it",
        twmode,
        false,
    ),
    (
        "twstat",
        "Statistics on TransferProcess (WRITE)",
        twstat,
        false,
    ),
    (
        "selecttransfer",
        "selection (recognize from transfer actor)",
        settransfert,
        true,
    ),
];

impl XSControlFunctions {
    /// `Init`: the commands registered (once per process) in the group "DE: General".
    pub fn init() -> Result<(), Failure> {
        if INIT_ACTOR.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        IFSelectAct::set_group("DE: General", "");
        for (name, help, func, set) in COMMANDS {
            if *set {
                IFSelectAct::add_f_set(name, help, *func)?;
            } else {
                IFSelectAct::add_func(name, help, *func)?;
            }
        }
        Ok(())
    }
}

/// `XSControl::Session(pilot)`, `Standard_NullObject` when the pilot has no XSControl
/// session (a native null dereference).
fn xs_session(
    pilot: &Handle<IFSelectSessionPilot>,
) -> Result<Handle<XSControlWorkSession>, Failure> {
    let ws = XSControl::session(pilot)?;
    if ws.is_null() {
        return Err(exceptions::null_object(
            "XSControl_Functions: the pilot has no XSControl work session",
        ));
    }
    Ok(ws)
}

/// The transfer reader of the session of the pilot (null when none).
fn reader_of(
    pilot: &Handle<IFSelectSessionPilot>,
) -> Result<Handle<XSControlTransferReader>, Failure> {
    Ok(xs_session(pilot)?.transfer_reader())
}

/// `Word(num).Value(index)`: `Standard_OutOfRange` beyond the word.
fn word_byte(pilot: &IFSelectSessionPilot, num: i32, index: usize) -> Result<u8, Failure> {
    let word = pilot.word(num)?;
    word.as_bytes().get(index - 1).copied().ok_or_else(|| {
        exceptions::out_of_range(&format!(
            "XSControl_Functions: character {index} of the word '{}'; expected at most {}",
            String::from_utf8_lossy(word.as_bytes()),
            word.length()
        ))
    })
}

/// Bytes printed to a native stream, sent to the message stream.
fn emit(sout: &mut MessageStream<'_>, bytes: &[u8]) {
    let _ = sout.write_str(&String::from_utf8_lossy(bytes));
}

/// The integer of an argument (`atoi`).
fn int_arg(pilot: &IFSelectSessionPilot, num: i32) -> Result<i32, Failure> {
    Ok(crate::c_text::atoi(pilot.arg(num)?.as_bytes()))
}
