//! `Draw_Appli`, `Draw_Interprete` and the init-file handling of `Draw.cxx`: the DRAW
//! application start, its command line, and the evaluation of one console command.
mod init_files;
mod options;

pub use options::AppliOptions;

use super::console::DrawConsole;
use super::session::DrawSession;
use super::viewer::DrawViewer;
use rustycad_tkernel::message::{ConsoleColor, PrinterOStream};
use rustycad_tkernel::osd::{Osd, Timer};
use rustycad_tkernel::standard::Failure;
use std::io::{BufRead, Write};
use std::rc::Rc;

/// The application initializer (`FDraw_InitAppli`): registers the application's
/// commands, as `Draw_InitAppli` does in an executable.
pub type InitAppli = fn(&DrawSession) -> Result<(), Failure>;

/// `Draw_Main(argc, argv, Draw_InitAppli)`: `Tcl_FindExecutable`, then [`draw_appli`];
/// returns 0, as the C++ function does.
///
/// ```no_run
/// use rustycad_tkdraw::draw::{ProcessConsole, draw_main};
/// use std::rc::Rc;
/// let args: Vec<String> = std::env::args().collect();
/// draw_main(&args, |_| Ok(()), Rc::new(ProcessConsole));
/// ```
pub fn draw_main(args: &[String], init: InitAppli, console: Rc<dyn DrawConsole>) -> i32 {
    rustycad_tcl::initialize_library();
    if let Err(failure) = draw_appli(args, init, console.clone()) {
        console.write_err(&format!("DRAWEXE: {failure}\n"));
    }
    0
}

/// `MAXCMD`: the longest line the batch loop reads.
const MAX_COMMAND_LINE: usize = 2048;

/// `Draw_Appli(argc, argv, Draw_InitAppli)`: parses the options, creates the session
/// (batch, since the window layer of this build cannot open a display), registers the
/// base commands and `init`, reads the init file, then runs the `-f` file, the `-c`
/// command or the console loop. Returns the session (`None` after `-h`).
pub fn draw_appli(
    args: &[String],
    init: InitAppli,
    console: Rc<dyn DrawConsole>,
) -> Result<Option<Rc<DrawSession>>, Failure> {
    let options = AppliOptions::parse(args, console.as_ref());
    if options.help {
        console.write_out(&options::help_text());
        return Ok(None);
    }
    Osd::set_signal_default(false);
    let session = create_session(&options, console)?;
    register_base_commands(&session, init)?;
    init_files::read_default_init_file(&session);
    if let Some(file) = &options.run_file {
        if !options.interactive_forced {
            // No colors: they would spoil the log of a script run.
            session.interpretor().set_to_colorize(false);
        }
        init_files::read_init_file(&session, file);
        if !options.interactive_forced {
            return Ok(Some(session));
        }
    }
    if let Some(command) = &options.command {
        interprete(&session, command);
        if !options.interactive_forced {
            return Ok(Some(session));
        }
    }
    batch_loop(&session);
    Ok(Some(session))
}

/// Creates the session; without a window layer `Init_Appli` cannot succeed, so a
/// non-batch request falls back to batch mode with the C++ message of a display that
/// cannot be opened.
fn create_session(
    options: &AppliOptions,
    console: Rc<dyn DrawConsole>,
) -> Result<Rc<DrawSession>, Failure> {
    if options.batch {
        console.write_out("DRAW is running in batch mode\n");
    } else {
        console.write_out(
            "Cannot open display (this build has no DRAW window backend). Interpret commands in batch mode.\n",
        );
    }
    let session = DrawSession::new(console, DrawViewer::batch());
    session.set_virtual_windows(options.virtual_windows);
    session.interpretor().interp()?.init();
    Ok(session)
}

/// The standard command groups, then the application's own commands.
fn register_base_commands(session: &DrawSession, init: InitAppli) -> Result<(), Failure> {
    super::basic_commands(session)?;
    super::message_commands(session)?;
    super::variable_commands(session)?;
    super::unit_commands(session)?;
    if !session.is_batch() {
        super::graphic_commands(session)?;
    }
    init(session)
}

/// The batch console loop: `Draw[n]> ` prompts and one line per command until the end
/// of the input.
fn batch_loop(session: &DrawSession) {
    let stdin = std::io::stdin();
    let mut lines = stdin.lock();
    for number in 1.. {
        session.console().write_out(&format!("Draw[{number}]> "));
        session.console().flush();
        let mut line = String::new();
        if lines.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let line = line.trim_end_matches(['\n', '\r']);
        // `std::istream::getline` fails on a line longer than the buffer.
        if line.len() >= MAX_COMMAND_LINE {
            break;
        }
        interprete(session, line);
    }
}

/// `Draw_Interprete(command)`: evaluates one console command and prints its result
/// (in red when it failed and colors are on). Returns false when the command is
/// incomplete. Upstream converts the text with `Tcl_ExternalToUtfDString`, which
/// re-initializes the pending command each time, so an incomplete command is not
/// continued by the next call; kept.
pub fn interprete(session: &DrawSession, command: &str) -> bool {
    let mut pending = rustycad_tcl::DString::new();
    // A conversion error can only be a text longer than `INT_MAX` bytes.
    let _ = rustycad_tcl::external_to_utf(command.as_bytes(), &mut pending);
    let script = pending.to_text();
    if !super::DrawInterpretor::complete(&script) {
        return false;
    }
    let was_spying = session.spy_file.borrow().is_some();
    let had_chrono = session.chrono();
    let mut timer = Timer::new(false);
    if had_chrono {
        timer.start();
    }
    if session.command_hooks.get() {
        session.before_command();
    }
    let code = session
        .interpretor()
        .record_and_eval(&script, 0)
        .unwrap_or(1);
    // Deviation: a repaint failure is reported on the error console; the C++
    // exception would leave the command loop (it cannot occur in batch mode).
    if session.command_hooks.get()
        && let Err(failure) = session.repaint_now_if_necessary()
    {
        session.console().write_err(&format!("{failure}\n"));
    }
    if was_spying {
        spy(session, code, &script);
    }
    session.viewer().flush();
    print_result(session, code);
    if session.chrono() && had_chrono {
        timer.stop();
        session.console().write_out(&timer.show_to());
    }
    true
}

/// Writes the command to the spy file, commented out when it failed.
fn spy(session: &DrawSession, code: i32, script: &str) {
    let mut spy_file = session.spy_file.borrow_mut();
    let Some(file) = spy_file.as_mut() else {
        return;
    };
    let prefix = if code > 0 { "# " } else { "" };
    // `std::ostream` reports write failures through state bits nobody reads.
    let _ = writeln!(file, "{prefix}{script}");
}

/// Prints a non-empty result on its own line, in intense red after an error.
fn print_result(session: &DrawSession, code: i32) {
    let result = session.interpretor().result().unwrap_or_default();
    if result.is_empty() {
        return;
    }
    let colorize = code > 0 && session.interpretor().to_colorize();
    if colorize {
        session
            .console()
            .write_out(&color_code(ConsoleColor::Red, true));
    }
    session.console().write_out(&result);
    if colorize {
        session
            .console()
            .write_out(&color_code(ConsoleColor::Default, false));
    }
    session.console().write_out("\n");
}

/// The escape sequence `Message_PrinterOStream::SetConsoleTextColor` writes.
fn color_code(color: ConsoleColor, intense: bool) -> String {
    let mut bytes: Vec<u8> = Vec::new();
    PrinterOStream::set_console_text_color(Some(&mut bytes), color, intense);
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests;
