//! `Draw_Interpretor`: the Tcl interpreter that DRAW commands are defined in.
//!
//! [`DrawInterpretor`] is a cheap handle (`Clone` shares the interpreter, as passing a
//! `Draw_Interpretor&` does in C++). The Tcl interpreter is created by
//! [`DrawInterpretor::init`] or attached with [`DrawInterpretor::set`]; until then every
//! operation that needs it fails with the `Standard_DomainError` that the C++
//! `Interp()` raises ("No call for  Draw_Interpretor::Init()", spacing as upstream).
//!
//! C strings stop at the first NUL byte; texts handed to the C++ API behave the same
//! here: everything after a NUL is ignored ([`c_text`]).
mod append;
mod command_call;
mod log;
mod registration;

pub use append::DrawAppend;
pub use registration::{DrawCommand, USER_COMMANDS};

use super::console::DrawConsole;
use super::stdio_capture::LogFile;
use rustycad_tcl::{Interp, command_complete};
use rustycad_tkernel::message::{Message, PrinterOStream};
use rustycad_tkernel::standard::Failure;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

/// The message of `Interp()` when no Tcl interpreter is attached.
const NO_INTERPRETER: &str = "No call for  Draw_Interpretor::Init()";

/// Parses a numeric Tcl value as `Draw::Atoi` does (variables and functions included);
/// the session installs it so that `Draw_ExitOnCatch` is read like upstream.
pub type IntegerParser = dyn Fn(&str) -> i32;

/// Shared state of one interpreter.
pub(crate) struct InterpretorCore {
    interp: RefCell<Option<Rc<Interp>>>,
    do_log: Cell<bool>,
    do_echo: Cell<bool>,
    to_colorize: Cell<bool>,
    log: RefCell<Option<LogFile>>,
    console: Rc<dyn DrawConsole>,
    integer_parser: RefCell<Option<Weak<IntegerParser>>>,
}

/// The DRAW command interpreter (`Draw_Interpretor`).
///
/// ```
/// use rustycad_tkdraw::draw::{DrawInterpretor, RecordingConsole};
/// use std::rc::Rc;
/// let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
/// di.init();
/// di.append("x = ").unwrap().append(1.5).unwrap();
/// assert_eq!(di.result().unwrap(), "x = 1.5");
/// ```
#[derive(Clone)]
pub struct DrawInterpretor {
    core: Rc<InterpretorCore>,
}

/// The part of `text` a C string carries: everything before the first NUL.
///
/// ```
/// assert_eq!(rustycad_tkdraw::draw::c_text("ab\0cd"), "ab");
/// ```
pub fn c_text(text: &str) -> &str {
    text.split('\0').next().unwrap_or_default()
}

impl DrawInterpretor {
    /// The empty constructor: no Tcl interpreter yet (it is created by [`Self::init`]),
    /// logging and echo off, colorized output on; text goes to `console`.
    pub fn new(console: Rc<dyn DrawConsole>) -> Self {
        let core = InterpretorCore {
            interp: RefCell::new(None),
            do_log: Cell::new(false),
            do_echo: Cell::new(false),
            to_colorize: Cell::new(true),
            log: RefCell::new(None),
            console,
            integer_parser: RefCell::new(None),
        };
        Self {
            core: Rc::new(core),
        }
    }

    /// `Draw_Interpretor(Tcl_Interp*)`: wraps an existing interpreter.
    pub fn with_interp(interp: Rc<Interp>, console: Rc<dyn DrawConsole>) -> Self {
        let di = Self::new(console);
        di.set(interp);
        di
    }

    /// Rebuilds a handle from the shared core (used by command callbacks).
    pub(crate) fn from_core(core: Rc<InterpretorCore>) -> Self {
        Self { core }
    }

    /// A weak reference to the shared core (held by command callbacks, so that the
    /// interpreter does not keep itself alive).
    pub(crate) fn downgrade(&self) -> Weak<InterpretorCore> {
        Rc::downgrade(&self.core)
    }

    /// `Init`: creates a new Tcl interpreter, replacing (and deleting) the previous one.
    pub fn init(&self) {
        *self.core.interp.borrow_mut() = Some(Rc::new(Interp::new()));
    }

    /// `Set(Tcl_Interp*)`: uses `interp`, releasing the previous interpreter.
    pub fn set(&self, interp: Rc<Interp>) {
        *self.core.interp.borrow_mut() = Some(interp);
    }

    /// `Interp()`: the Tcl interpreter, or the `Standard_DomainError` of a missing `Init`.
    pub fn interp(&self) -> Result<Rc<Interp>, Failure> {
        let interp = self.core.interp.borrow();
        interp
            .clone()
            .ok_or_else(|| Failure::with_message("Standard_DomainError", NO_INTERPRETER))
    }

    /// The console this interpreter prints to (`std::cout`).
    pub fn console(&self) -> &Rc<dyn DrawConsole> {
        &self.core.console
    }

    /// Installs the `Draw::Atoi` parser used for `Draw_ExitOnCatch`.
    pub fn set_integer_parser(&self, parser: Weak<IntegerParser>) {
        *self.core.integer_parser.borrow_mut() = Some(parser);
    }

    /// Applies the installed `Draw::Atoi`, or the leading integer of `text` when no
    /// session installed one.
    pub(crate) fn parse_integer(&self, text: &str) -> i32 {
        let parser = self
            .core
            .integer_parser
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade);
        match parser {
            Some(parser) => parser(text),
            None => text.trim().parse::<f64>().map_or(0, |value| value as i32),
        }
    }

    /// `Result`: the interpreter result.
    pub fn result(&self) -> Result<String, Failure> {
        Ok(self.interp()?.result())
    }

    /// `Reset`: empties the result.
    pub fn reset(&self) -> Result<(), Failure> {
        self.interp()?.reset_result();
        Ok(())
    }

    /// `Append` / `operator<<`: appends `value` to the result, formatted as the C++
    /// overload for its type does (see [`DrawAppend`]).
    pub fn append(&self, value: impl DrawAppend) -> Result<&Self, Failure> {
        let interp = self.interp()?;
        let text = value.draw_text();
        interp.append_result(c_text(&text)).map_err(tcl_failure)?;
        Ok(self)
    }

    /// `AppendElement`: appends `element` as a list element.
    pub fn append_element(&self, element: &str) -> Result<(), Failure> {
        self.interp()?
            .append_element(c_text(element))
            .map_err(tcl_failure)
    }

    /// `Eval`: evaluates `script`; returns the Tcl code (0 = OK, 1 = ERROR, ...).
    pub fn eval(&self, script: &str) -> Result<i32, Failure> {
        let code = self.interp()?.eval(c_text(script)).map_err(tcl_failure)?;
        Ok(code.code())
    }

    /// `RecordAndEval`: records `script` in the history, then evaluates it.
    pub fn record_and_eval(&self, script: &str, flags: i32) -> Result<i32, Failure> {
        let interp = self.interp()?;
        let code = interp
            .record_and_eval(c_text(script), flags)
            .map_err(tcl_failure)?;
        Ok(code.code())
    }

    /// `EvalFile`: sources the file `file_name`.
    pub fn eval_file(&self, file_name: &str) -> Result<i32, Failure> {
        let code = self
            .interp()?
            .eval_file(c_text(file_name))
            .map_err(tcl_failure)?;
        Ok(code.code())
    }

    /// `PrintHelp`: evaluates `help <command>`.
    pub fn print_help(&self, command_name: &str) -> Result<i32, Failure> {
        self.eval(&format!("help {}", c_text(command_name)))
    }

    /// `Complete`: true when `script` has no pending open brace, quote or bracket.
    pub fn complete(script: &str) -> bool {
        command_complete(c_text(script))
    }

    /// `GetDoLog`.
    pub fn get_do_log(&self) -> bool {
        self.core.do_log.get()
    }

    /// `SetDoEcho`: echo each command and its result to the console.
    pub fn set_do_echo(&self, do_echo: bool) {
        self.core.do_echo.set(do_echo);
    }

    /// `GetDoEcho`.
    pub fn get_do_echo(&self) -> bool {
        self.core.do_echo.get()
    }

    /// `ToColorize`: whether console output is colorized (true by default).
    pub fn to_colorize(&self) -> bool {
        self.core.to_colorize.get()
    }

    /// `SetToColorize`: also switches every `Message_PrinterOStream` of the default
    /// messenger, as upstream does.
    pub fn set_to_colorize(&self, to_colorize: bool) {
        self.core.to_colorize.set(to_colorize);
        let printers = Message::default_messenger().printers();
        for printer in printers.iter() {
            let any: &dyn std::any::Any = printer.as_ref();
            if let Some(stream_printer) = any.downcast_ref::<PrinterOStream>() {
                stream_printer.set_to_colorize(to_colorize);
            }
        }
    }
}

/// A wrapper failure: after [`c_text`] no NUL reaches Tcl, so only an over-long text
/// (more than `INT_MAX` bytes) can fail; it is reported as `Standard_OutOfRange`.
fn tcl_failure(error: rustycad_tcl::TclError) -> Failure {
    Failure::with_message("Standard_OutOfRange", &error.to_string())
}

#[cfg(test)]
mod tests;
