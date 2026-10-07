// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_Functions`: the basic commands of the work session ("DE: General"),
//! registered as acts on the session pilot by `Init`, and the helpers they share
//! (entities, lists and dispatches given by text).
//!
//! The commands are split by subject: `general` (status, files, entities, counters,
//! parameters), `lists` (lists, items, data, file naming), `modifiers` (split,
//! remaining, modifiers, dispatches), `selections` (transformers, selections), `edit`
//! (edit forms), `register` (`Init`).
use super::{
    IFSelectDispPerCount, IFSelectDispPerFiles, IFSelectDispPerSignature, IFSelectDispatch,
    IFSelectIntParam, IFSelectReturnStatus, IFSelectSessionPilot, IFSelectSignature,
    IFSelectTransientList, IFSelectWorkSession,
};
use rustycad_tkernel::{
    message::MessageStream,
    standard::{Failure, Handle, StandardTransient, exceptions},
};
use std::fmt::Write as _;

mod command_table;
mod dispatches;
mod edit;
mod entities;
mod general;
mod lists;
mod modifiers;
mod register;
mod selection_items;
mod selections;

/// The result of a command.
pub(super) type Status = Result<IFSelectReturnStatus, Failure>;

/// `IFSelect_Functions`: the static members.
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectActivatorBase, IFSelectFunctions};
/// IFSelectFunctions::init().unwrap();
/// assert!(IFSelectActivatorBase::select("xload").is_some());
/// ```
pub struct IFSelectFunctions;

/// The session of the pilot, `Standard_NullObject` without one (a native null
/// dereference).
pub(super) fn session_of(
    pilot: &Handle<IFSelectSessionPilot>,
) -> Result<Handle<IFSelectWorkSession>, Failure> {
    let ws = pilot.session();
    if ws.is_null() {
        return Err(exceptions::null_object(
            "IFSelect_Functions: the pilot has no work session",
        ));
    }
    Ok(ws)
}

/// Bytes printed to a native stream, sent to the message stream.
pub(super) fn emit(sout: &mut MessageStream<'_>, bytes: &[u8]) {
    let _ = sout.write_str(&String::from_utf8_lossy(bytes));
}

/// The text of a handle text (empty for null).
pub(super) fn h_text(h: &Handle<rustycad_tkernel::tcollection::HAsciiString>) -> String {
    h.get()
        .map(|t| String::from_utf8_lossy(t.string().as_bytes()).into_owned())
        .unwrap_or_default()
}

/// The first byte of an argument (0 for an empty one).
pub(super) fn first_byte(arg: &str) -> u8 {
    arg.bytes().next().unwrap_or(0)
}

/// A whitespace-delimited word read from the standard input as `std::cin >> ligne` with
/// a width of 80 (at most 79 bytes); empty at the end of the input.
fn read_word_from_stdin() -> String {
    use std::io::Read;
    let mut word = Vec::new();
    for byte in std::io::stdin().lock().bytes() {
        let Ok(b) = byte else { break };
        if b.is_ascii_whitespace() {
            if word.is_empty() {
                continue;
            }
            break;
        }
        word.push(b);
        if word.len() == 79 {
            break;
        }
    }
    String::from_utf8_lossy(&word).into_owned()
}

/// `TCollection_AsciiString::SetValue(where, '\0')` on the text: `Standard_OutOfRange`
/// out of 1..Length.
fn cut_at(text: &mut [u8], position: usize) -> Result<(), Failure> {
    let len = text.len();
    let slot = position
        .checked_sub(1)
        .filter(|p| *p < len)
        .ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "IFSelect_Functions::GiveDispatch: position {position}; expected [1, {len}]"
            ))
        })?;
    text[slot] = 0;
    Ok(())
}

/// The C string of a text from `start` (up to its first NUL).
fn c_string_from(text: &[u8], start: usize) -> String {
    let part = text.get(start..).unwrap_or_default();
    let end = part.iter().position(|b| *b == 0).unwrap_or(part.len());
    String::from_utf8_lossy(&part[..end]).into_owned()
}

impl IFSelectFunctions {
    /// `GiveEntity(WS, name = "")`: the entity of a number or label (null when none).
    pub fn give_entity(
        ws: &Handle<IFSelectWorkSession>,
        name: &str,
    ) -> Result<Handle<StandardTransient>, Failure> {
        let num = Self::give_entity_number(ws, name)?;
        if num > 0 {
            return Ok(ws.starting_entity(num));
        }
        Ok(Handle::default())
    }
    /// `GiveEntityNumber(WS, name = "")`: the number of an entity given by number or
    /// label; an empty name is read from the standard input. `Standard_NullObject`
    /// without session.
    pub fn give_entity_number(
        ws: &Handle<IFSelectWorkSession>,
        name: &str,
    ) -> Result<i32, Failure> {
        let live = ws.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_Functions::GiveEntityNumber: null session")
        })?;
        if !name.is_empty() {
            return Ok(live.number_from_label(name, 0));
        }
        let ligne = read_word_from_stdin();
        if ligne.is_empty() {
            return Ok(0);
        }
        Ok(live.number_from_label(&ligne, 0))
    }
    /// `GiveList(WS, first = "", second = "")`: the list the texts give (see the work
    /// session).
    pub fn give_list(
        ws: &Handle<IFSelectWorkSession>,
        first: &str,
        second: &str,
    ) -> Result<IFSelectTransientList, Failure> {
        let live = ws
            .get()
            .ok_or_else(|| exceptions::null_object("IFSelect_Functions::GiveList: null session"))?;
        live.give_list_texts(first, second)
    }
    /// `GiveDispatch(WS, name, mode = true)`: the dispatch item `name`, or for
    /// "name(arg)" the dispatch item `name` with its parameter: a count for a dispatch
    /// per count or per files, a signature item for a dispatch per signature (set on the
    /// item when `mode`); null when not available, with a message. Native defect kept:
    /// a name which is not an item and lacks '(' or ')' raises `Standard_OutOfRange`.
    pub fn give_dispatch(
        ws: &Handle<IFSelectWorkSession>,
        name: &str,
        mode: bool,
    ) -> Result<Handle<dyn IFSelectDispatch>, Failure> {
        let live = ws.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_Functions::GiveDispatch: null session")
        })?;
        let disp: Handle<dyn IFSelectDispatch> = live.named_item(name).cast();
        if disp.is_some() {
            return Ok(disp); // OK as it is given
        }
        let mut nam = name.as_bytes().to_vec();
        let paro = nam.iter().position(|b| *b == b'(').map_or(0, |p| p + 1);
        let parf = nam.iter().position(|b| *b == b')').map_or(0, |p| p + 1);
        cut_at(&mut nam, paro)?;
        cut_at(&mut nam, parf)?;
        let disp: Handle<dyn IFSelectDispatch> = live.named_item(&c_string_from(&nam, 0)).cast();
        if disp.is_null() {
            return Ok(disp); // KO anyway
        }
        set_dispatch_parameter(live, name, &disp, &c_string_from(&nam, paro), mode)
    }
}

/// The parameter `arg` of a dispatch given as "name(arg)" (see `GiveDispatch`).
fn set_dispatch_parameter(
    ws: &IFSelectWorkSession,
    name: &str,
    disp: &Handle<dyn IFSelectDispatch>,
    arg: &str,
    mode: bool,
) -> Result<Handle<dyn IFSelectDispatch>, Failure> {
    let mut sout = rustycad_tkernel::message::Message::send_info();
    let count = || {
        let val = IFSelectIntParam::new();
        val.set_value(crate::c_text::atoi(arg.as_bytes()));
        Handle::new(val)
    };
    let nb = crate::c_text::atoi(arg.as_bytes());
    if let Some(dc) = disp.cast::<IFSelectDispPerCount>().get() {
        if nb <= 0 {
            let _ = writeln!(sout, " DispPerCount, count is not positive");
            return Ok(Handle::default());
        }
        if mode {
            dc.set_count(&count());
        }
        return Ok(disp.clone());
    }
    if let Some(dp) = disp.cast::<IFSelectDispPerFiles>().get() {
        if nb <= 0 {
            let _ = writeln!(sout, " DispPerFiles, count is not positive");
            return Ok(Handle::default());
        }
        if mode {
            dp.set_count(&count());
        }
        return Ok(disp.clone());
    }
    if let Some(ds) = disp.cast::<IFSelectDispPerSignature>().get() {
        let sg: Handle<dyn IFSelectSignature> = ws.named_item(arg).cast();
        if sg.is_null() {
            let _ = writeln!(
                sout,
                "DispPerSignature {} , Signature not valid : {arg}",
                name.split(['(', ')']).next().unwrap_or("")
            );
            return Ok(Handle::default());
        }
        if mode {
            ds.set_sign_counter(
                &Handle::new(super::IFSelectSignCounter::with_signature(
                    &sg, true, false,
                )?)
                .cast(),
            )?;
        }
        return Ok(disp.clone());
    }
    let _ = writeln!(sout, "Dispatch : {name} , Parameter : {arg}");
    Ok(disp.clone())
}
