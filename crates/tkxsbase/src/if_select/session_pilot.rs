// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SessionPilot (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_SessionPilot`: the command interpreter of a work session: it splits a
//! command line into words, finds the activator of the first word and executes it,
//! records the commands and reads scripts. It is itself the activator of the basic
//! commands (exit, help, command control, scripts, xstep, xsnew).
//!
//! The execution members are in `session_pilot_run`.
use super::{IFSelectActivator, IFSelectActivatorBase, IFSelectWorkLibrary, IFSelectWorkSession};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::AsciiString,
};
use std::sync::atomic::{AtomicBool, Ordering};

/// The maximum count of words of a command line.
pub(super) const MAXWORDS: usize = 200;
/// The maximum length of a word (natively one more character fits).
pub(super) const MAXCARS: usize = 1000;

/// The native static `THE_IFSelect_SessionPilot_initactor`: the basic commands are
/// registered by the first pilot.
static INIT_ACTOR: AtomicBool = AtomicBool::new(false);

/// The fields of `IFSelect_SessionPilot`.
pub(super) struct PilotState {
    pub(super) thesession: Handle<IFSelectWorkSession>,
    pub(super) theprompt: AsciiString,
    pub(super) thecommand: Vec<u8>,
    pub(super) thenbwords: i32,
    pub(super) thewords: Vec<Vec<u8>>,
    /// The 1-based start of each word in the command line (0 before any line).
    pub(super) thewordeb: Vec<i32>,
    pub(super) therecord: bool,
    pub(super) thenumrec: i32,
    pub(super) theobjrec: Handle<StandardTransient>,
    pub(super) thecomlist: Vec<AsciiString>,
}

/// `IFSelect_SessionPilot`.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectSessionPilot;
/// let pilot = IFSelectSessionPilot::new("").unwrap();
/// assert_eq!(pilot.nb_words(), 0);
/// ```
pub struct IFSelectSessionPilot {
    transient: StandardTransient,
    pub(super) base: IFSelectActivatorBase,
    pub(super) state: SharedState<PilotState>,
}
impl_transient!(IFSelectSessionPilot, "IFSelect_SessionPilot", dyn IFSelectActivator, transient, interfaces: [dyn IFSelectActivator]);

/// A separator of words: a native `char` not above a blank (the bytes from 0x80 are
/// negative `char`s natively, so separators too).
pub(super) fn is_separator(byte: u8) -> bool {
    (byte as i8) <= b' ' as i8
}

/// `Standard_OutOfRange` for a word index beyond the native array.
fn word_out_of_range(what: &str, index: i32) -> Failure {
    exceptions::out_of_range(&format!(
        "IFSelect_SessionPilot::{what}: word {index}; expected [0, {}]",
        MAXWORDS - 1
    ))
}

/// The word slot `index` of the native array.
fn word_slot(index: i32, what: &str) -> Result<usize, Failure> {
    usize::try_from(index)
        .ok()
        .filter(|i| *i < MAXWORDS)
        .ok_or_else(|| word_out_of_range(what, index))
}

impl IFSelectSessionPilot {
    /// `IFSelect_SessionPilot(prompt = "")`: the prompt (default "Test-XSTEP>"); the
    /// first pilot of the process registers the basic commands (x, exit, ?, xhelp,
    /// xcommand, xsource, xstep, xnew) as their activator.
    pub fn new(prompt: &str) -> Result<Handle<Self>, Failure> {
        let state = PilotState {
            thesession: Handle::default(),
            theprompt: AsciiString::new(if prompt.is_empty() {
                "Test-XSTEP>"
            } else {
                prompt
            }),
            thecommand: Vec::new(),
            thenbwords: 0,
            thewords: vec![Vec::new(); MAXWORDS],
            thewordeb: vec![0; MAXWORDS],
            therecord: false,
            thenumrec: 0,
            theobjrec: Handle::default(),
            thecomlist: Vec::new(),
        };
        let pilot = Handle::new(Self {
            transient: StandardTransient::default(),
            base: IFSelectActivatorBase::new(),
            state: SharedState::new(state),
        });
        pilot.base.bind_self(&pilot.clone().cast());
        if INIT_ACTOR.swap(true, Ordering::SeqCst) {
            return Ok(pilot);
        }
        for (number, command) in [
            (1, "x"),
            (1, "exit"),
            (2, "?"),
            (2, "xhelp"),
            (3, "xcommand"),
            (4, "xsource"),
            (5, "xstep"),
            (6, "xnew"),
        ] {
            pilot.add(number, command)?;
        }
        Ok(pilot)
    }
    /// The handle of this pilot (natively `this`).
    pub(super) fn this_pilot(&self) -> Result<Handle<Self>, Failure> {
        Ok(self.base.this()?.cast())
    }
    /// `Session`.
    pub fn session(&self) -> Handle<IFSelectWorkSession> {
        self.state.read().thesession.clone()
    }
    /// The session, `Standard_NullObject` when none (a native null dereference).
    pub(super) fn live_session(&self) -> Result<Handle<IFSelectWorkSession>, Failure> {
        let session = self.session();
        if session.is_null() {
            return Err(exceptions::null_object(
                "IFSelect_SessionPilot: no work session",
            ));
        }
        Ok(session)
    }
    /// `Library`: the library of the session; `Standard_NullObject` without session.
    pub fn library(&self) -> Result<Handle<dyn IFSelectWorkLibrary>, Failure> {
        Ok(self.live_session()?.work_library())
    }
    /// `RecordMode`.
    pub fn record_mode(&self) -> bool {
        self.state.read().therecord
    }
    /// `SetSession(WS)`.
    pub fn set_session(&self, ws: &Handle<IFSelectWorkSession>) {
        self.state.write().thesession = ws.clone();
    }
    /// `SetLibrary(WL)`: the library of the session (nothing without session).
    pub fn set_library(&self, wl: &Handle<dyn IFSelectWorkLibrary>) {
        if let Some(session) = self.session().get() {
            session.set_library(wl);
        }
    }
    /// `SetRecordMode(mode)`.
    pub fn set_record_mode(&self, mode: bool) {
        self.state.write().therecord = mode;
    }
    /// `SetCommandLine(command)`: the line split into words (separated by the
    /// characters not above a blank); a last separator is removed from the line. The
    /// words are traced on the standard output, as natively. `Standard_OutOfRange` for
    /// an empty line or more than 200 words.
    pub fn set_command_line(&self, command: &AsciiString) -> Result<(), Failure> {
        let bytes = command.as_bytes();
        let mut lc = bytes.len();
        if lc > 200 {
            println!(
                " VERY LONG Command : {lc} characters :\n{}",
                String::from_utf8_lossy(bytes)
            );
        }
        let mut st = self.state.write();
        st.thecommand = bytes.to_vec();
        let last = lc.checked_sub(1).ok_or_else(|| {
            exceptions::out_of_range(
                "IFSelect_SessionPilot::SetCommandLine: empty command; expected a character",
            )
        })?;
        if is_separator(bytes[last]) {
            st.thecommand.pop();
            lc -= 1;
        }
        st.thenbwords = 0;
        let mut unarg: Vec<u8> = Vec::new();
        for (i, val) in bytes.iter().take(lc).copied().enumerate() {
            split_byte(&mut st, &mut unarg, i as i32 + 1, val)?;
        }
        if !unarg.is_empty() {
            let n = st.thenbwords;
            st.thewords[word_slot(n, "SetCommandLine")?] = unarg.clone();
            println!(
                "thewords({n})={}\n .. End with thenbwords={}",
                String::from_utf8_lossy(&unarg),
                n + 1
            );
            st.thenbwords += 1;
        }
        st.thenumrec = 0;
        st.theobjrec = Handle::default();
        Ok(())
    }
    /// `CommandLine`.
    pub fn command_line(&self) -> AsciiString {
        AsciiString::new(&String::from_utf8_lossy(&self.state.read().thecommand))
    }
    /// `CommandPart(numarg = 0)`: the line from the word `numarg` (the whole line for 0,
    /// empty beyond the words).
    pub fn command_part(&self, numarg: i32) -> AsciiString {
        let st = self.state.read();
        if numarg <= 0 {
            return AsciiString::new(&String::from_utf8_lossy(&st.thecommand));
        }
        if numarg >= st.thenbwords {
            return AsciiString::default();
        }
        let start = (st.thewordeb[numarg as usize] - 1).max(0) as usize;
        AsciiString::new(&String::from_utf8_lossy(
            st.thecommand.get(start..).unwrap_or_default(),
        ))
    }
    /// `NbWords`.
    pub fn nb_words(&self) -> i32 {
        self.state.read().thenbwords
    }
    /// `Word(num)`: empty beyond the words; `Standard_OutOfRange` for a negative number.
    pub fn word(&self, num: i32) -> Result<AsciiString, Failure> {
        let st = self.state.read();
        if num >= st.thenbwords {
            return Ok(AsciiString::default());
        }
        Ok(AsciiString::new(&String::from_utf8_lossy(
            &st.thewords[word_slot(num, "Word")?],
        )))
    }
    /// `Arg(num)`: `Word(num)` as text.
    pub fn arg(&self, num: i32) -> Result<String, Failure> {
        Ok(String::from_utf8_lossy(self.word(num)?.as_bytes()).into_owned())
    }
    /// `RemoveWord(num)`: the word removed; the line becomes the words joined by blanks.
    /// Native defects kept: removing the first word removes as many characters from the
    /// start of the line as the (stale) start position of the second word, and the
    /// start positions are not updated.
    pub fn remove_word(&self, num: i32) -> Result<bool, Failure> {
        let mut st = self.state.write();
        let nb = st.thenbwords;
        if num < 0 || num > nb {
            return Ok(false);
        }
        for i in num..nb {
            let next = st.thewords[word_slot(i + 1, "RemoveWord")?].clone();
            st.thewords[i as usize] = next;
        }
        st.thewords[word_slot(nb, "RemoveWord")?].clear();
        st.thenbwords -= 1;
        if num == 0 {
            let howmany = st.thewordeb[1].max(0) as usize;
            if howmany > st.thecommand.len() {
                return Err(exceptions::out_of_range(&format!(
                    "IFSelect_SessionPilot::RemoveWord: {howmany} characters from a line of {}",
                    st.thecommand.len()
                )));
            }
            st.thecommand.drain(..howmany);
        } else {
            let words: Vec<Vec<u8>> = st.thewords[..st.thenbwords.max(0) as usize].to_vec();
            st.thecommand = words.join(&b' ');
        }
        Ok(true)
    }
    /// `NbCommands`: the recorded commands.
    pub fn nb_commands(&self) -> i32 {
        self.state.read().thecomlist.len() as i32
    }
    /// `Command(num)`: `Standard_OutOfRange` out of 1..NbCommands.
    pub fn command(&self, num: i32) -> Result<AsciiString, Failure> {
        let st = self.state.read();
        usize::try_from(num - 1)
            .ok()
            .and_then(|i| st.thecomlist.get(i))
            .cloned()
            .ok_or_else(|| {
                exceptions::out_of_range(&format!(
                    "IFSelect_SessionPilot::Command: {num}; expected [1, {}]",
                    st.thecomlist.len()
                ))
            })
    }
    /// `RecordItem(item)`: the item the command creates; fail for null.
    pub fn record_item(&self, item: &Handle<StandardTransient>) -> super::IFSelectReturnStatus {
        self.state.write().theobjrec = item.clone();
        if item.is_null() {
            super::IFSelectReturnStatus::RetFail
        } else {
            super::IFSelectReturnStatus::RetDone
        }
    }
    /// `RecordedItem`.
    pub fn recorded_item(&self) -> Handle<StandardTransient> {
        self.state.read().theobjrec.clone()
    }
    /// `Clear`: the recorded commands forgotten.
    pub fn clear(&self) {
        self.state.write().thecomlist.clear();
    }
    /// `Number(val)`: the entity number of a label; negative (with a message) when
    /// several entities match.
    pub fn number(&self, val: &str) -> Result<i32, Failure> {
        let num = self.live_session()?.number_from_label(val, 0);
        if num < 0 {
            println!(" Label:{val} ->{} ent.s, refus", -num);
        }
        Ok(num)
    }
}

/// One byte of `SetCommandLine` at the 1-based position `i`: a separator ends the
/// current word (beyond 200 words it is kept in the last one), another byte extends it
/// (beyond the maximum length it is dropped with a message).
fn split_byte(st: &mut PilotState, unarg: &mut Vec<u8>, i: i32, val: u8) -> Result<(), Failure> {
    if is_separator(val) {
        if unarg.is_empty() {
            return Ok(());
        }
        if st.thenbwords as usize >= MAXWORDS {
            unarg.push(val);
            return Ok(());
        }
        let n = st.thenbwords;
        st.thewords[n as usize] = std::mem::take(unarg);
        println!(
            "thewords({n}) ={}",
            String::from_utf8_lossy(&st.thewords[n as usize])
        );
        st.thenbwords += 1;
        return Ok(());
    }
    if unarg.is_empty() {
        let n = st.thenbwords;
        st.thewordeb[word_slot(n, "SetCommandLine")?] = i;
    }
    if unarg.len() > MAXCARS {
        println!("Arg.{} > {MAXCARS} car.s, tronque", st.thenbwords);
        return Ok(());
    }
    unarg.push(val);
    Ok(())
}
