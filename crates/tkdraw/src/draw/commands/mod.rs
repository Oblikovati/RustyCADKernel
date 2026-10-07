//! The command groups of the `Draw` package (`Draw::Commands` and its parts).
mod basic;
mod chrono;
mod display;
mod files;
mod graphic;
mod graphic_list;
mod graphic_view;
mod message;
mod pload;
mod postscript;
mod system;
mod units;
mod values;

use super::color::{DrawColor, DrawColorKind};
use super::drawable::DrawableHandle;
use super::drawables::{DrawAxis2D, DrawAxis3D, DrawGrid};
use super::number::DrawNumber;
use super::session::{DrawSession, SessionCommand};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use std::rc::Rc;

/// The source file recorded in `Draw_Files` for the variable commands.
const VARIABLE_FILE: &str = file!();

pub use graphic_list::graphic_commands;

/// `Draw::Commands`: every command group of the `Draw` package.
pub fn commands(session: &DrawSession) -> Result<(), Failure> {
    basic_commands(session)?;
    message_commands(session)?;
    variable_commands(session)?;
    graphic_commands(session)?;
    pload_commands(session)?;
    unit_commands(session)
}

/// The `chrono`/`dchrono` help text.
const CHRONO_HELP: &str = "chrono Name [-start] [-stop] [-reset] [-restart] [-counter Text]\
\n\t\t:             [-show] [-elapsed] [-userCPU] [-sysCPU]\
\n\t\t:             [-thread|-process {0|1}]\
\n\t\t: Operates named timer:\
\n\t\t:   -start   starts (resumes) timer\
\n\t\t:   -stop    stops  (pauses)  timer\
\n\t\t:   -reset   resets timer progress\
\n\t\t:   -restart resets and starts timer\
\n\t\t:   -show    prints timer progress\
\n\t\t:           ('dchrono' puts into Tcl, 'chrono' puts into std::cout)\
\n\t\t:   -elapsed prints elapsed    time in seconds\
\n\t\t:   -userCPU prints user   CPU time in seconds\
\n\t\t:   -sysCPU  prints system CPU time in seconds\
\n\t\t:   -counter prints 'COUNTER <Text>'\
\n\t\t:   -thread  stops timer and sets measuring of CPU time for this thread only (FALSE by default)\
\n\t\t:   -process stops timer and sets measuring of CPU time for all  threads (TRUE by default)\
\n\t\t: Without arguments enables / disables global timer for all DRAW commands.\
\n\t\t:   chrono {0|1}\
\n\t\t: Typical usage:\
\n\t\t:   chrono t -restart\
\n\t\t:   <algorithm>\
\n\t\t:   chrono t -stop -show";

/// `Draw::BasicCommands`: the `DRAW General Commands` group.
pub fn basic_commands(session: &DrawSession) -> Result<(), Failure> {
    if !session.first_load("Draw::BasicCommands") {
        return Ok(());
    }
    let file = file!();
    register(
        session,
        "DRAW General Commands",
        file,
        &[
            ("batch", "returns 1 in batch mode", basic::ifbatch),
            (
                "spy",
                "spy [file], Save commands in file. no file close",
                basic::spy,
            ),
            ("wait", "wait [time(10)], wait time seconds", basic::wait),
            (
                "cpulimit",
                "cpulimit [nbseconds], no args remove limits",
                system::cpulimit,
            ),
            ("chrono", CHRONO_HELP, chrono::dchronom),
            ("dchrono", CHRONO_HELP, chrono::dchronom),
            (
                "meminfo",
                "meminfo [virt|v] [heap|h] [wset|w] [wsetpeak] [swap] [swappeak] [private] : memory counters for this process",
                system::dmeminfo,
            ),
            (
                "dperf",
                "dperf [reset] -- show performance counters, reset if argument is provided",
                system::dperf,
            ),
            (
                "dsetsignal",
                "dsetsignal [{asIs|set|unhandled|unset}=set] [{0|1|default=$CSF_FPE}]\n\t\t:            [-strackTraceLength Length]\n\t\t: Sets OSD signal handler, with FPE option if argument is given.\n\t\t:  -strackTraceLength specifies length of stack trace to put into exceptions redirected from signals.",
                system::dsetsignal,
            ),
            (
                "dparallel",
                "dparallel [-occt {0|1}] [-nbThreads Count] [-nbDefThreads Count]\n\t\t: Manages global parallelization parameters:\n\t\t:   -occt         use OCCT implementation or external library (if available)\n\t\t:   -nbThreads    specify the number of threads in default thread pool\n\t\t:   -nbDefThreads specify the upper limit of threads to be used for default thread pool\n\t\t:                 within single parallelization call (should be <= of overall number of threads),\n\t\t:                 so that nested algorithm can also use this pool",
                system::dparallel,
            ),
            // The names of `dlog` and `decho` are hard-coded in the interpreter, which never
            // logs nor echoes them; keep them in step.
            (
                "dlog",
                "manage logging of commands and output; run without args to get help",
                basic::dlog,
            ),
            (
                "decho",
                "switch on / off echo of commands to cout; run without args to get help",
                basic::decho,
            ),
            (
                "dtracelevel",
                "dtracelevel [trace|info|warn|alarm|fail]",
                basic::dtracelevel,
            ),
            (
                "ddebugtraces",
                "ddebugtraces nbTraces\n\t\t: Sets the number of lines for the stack trace within Standard_Failure constructor.\n\t\t: Intended for debug purposes.",
                basic::ddebugtraces,
            ),
            (
                "dbreak",
                "raises Tcl exception if user has pressed Control-Break key",
                basic::dbreak,
            ),
            (
                "dversion",
                "provides information on OCCT build configuration (version, compiler, OS, C library, etc.)",
                system::dversion,
            ),
            (
                "dlocale",
                "set and / or query locate of C subsystem (function setlocale())",
                system::dlocale,
            ),
            (
                "dputs",
                "dputs [-intense] [-black|-white|-red|-green|-blue|-yellow|-cyan|-magenta]\n\t\t:       [-nonewline] [stdcout|stdcerr] text\n\t\t: Puts text into console output",
                basic::dputs,
            ),
        ],
    )
}

/// `Draw::MessageCommands`: the `DRAW Message Commands` group.
pub fn message_commands(session: &DrawSession) -> Result<(), Failure> {
    if !session.first_load("Draw::MessageCommands") {
        return Ok(());
    }
    register(
        session,
        "DRAW Message Commands",
        file!(),
        &[
            (
                "PrintMessenger",
                "PrintMessenger\n\t\t: Prints DumpJson information about messenger.",
                message::print_messenger,
            ),
            (
                "SetMessagePrinter",
                "SetMessagePrinter [-type ostream|systemlog|report|draw] [-state {on|off}=on]\n\t\t: Sets or removes the printer in messenger.\n\t\t: Option -type set type of printer. Printers are applied with And combination.\n\t\t: Option -state add or remove printer",
                message::set_message_printer,
            ),
            (
                "SendMessage",
                "SendMessage text [text ...]\n Sends the text into the messenger.\n",
                message::send_message,
            ),
            (
                "ClearReport",
                "Removes all alerts in default printer",
                message::clear_report,
            ),
            (
                "SetReportMetric",
                "SetReportMetric [metric ...] \n Activate report metrics, deactivate all if there are no parameters.\n\n\t\t: metric is a value of Message_MetricType, e.g. 1 is Message_MetricType_UserTimeCPU",
                message::set_report_metric,
            ),
            (
                "CollectMetricMessages",
                "CollectMetricMessages [-activate {0|1}]\n Start metric collection by 1, stop by 0. Result is placed in metric attributes of message report.\n",
                message::collect_metric_messages,
            ),
            (
                "PrintReport",
                "PrintReport [-messenger] [-dump] [-dumpJson]\n\t\t: Send report content to default messenger or stream\n\t\t: Output options:\n\t\t:   -messenger Prints the information about report into messenger.\n\t\t:   -dump      Prints Dump information about report.\n\t\t:   -dumpJson  Prints DumpJson information about report.",
                message::print_report,
            ),
        ],
    )
}

/// `Draw::UnitCommands`: the `DRAW Unit Commands` group.
pub fn unit_commands(session: &DrawSession) -> Result<(), Failure> {
    if !session.first_load("Draw::UnitCommands") {
        return Ok(());
    }
    register(
        session,
        "DRAW Unit Commands",
        file!(),
        &[
            ("unitparsing", "unitparsing string [nbiter]", units::parsing),
            ("unitsdico", "unitsdico", units::unitsdico),
            (
                "unitconvtoSI",
                "unitconvtoSI real string",
                units::convert_to_si,
            ),
            (
                "unitconvtoMDTV",
                "unitconvtoMDTV real string",
                units::convert_to_mdtv,
            ),
            ("unit", "unit value unitfrom unitto", units::unit),
        ],
    )
}

/// `Draw::PloadCommands`: the `Draw Plugin` group.
pub fn pload_commands(session: &DrawSession) -> Result<(), Failure> {
    if !session.first_load("Draw::PloadCommands") {
        return Ok(());
    }
    register(
        session,
        "Draw Plugin",
        file!(),
        &[
            (
                "pload",
                "pload [-PluginFilename] [[Key1] [Key2] ...]: Loads Draw plugins ",
                pload::pload,
            ),
            (
                "dtryload",
                "dtryload path : load and unload specified dynamic loaded library",
                pload::dtryload,
            ),
        ],
    )
}

/// Registers commands of one group with their help texts.
pub(crate) fn register(
    session: &DrawSession,
    group: &str,
    file: &str,
    commands: &[(&str, &str, SessionCommand)],
) -> Result<(), Failure> {
    for (name, help, command) in commands {
        session.add_command(name, help, file, *command, group)?;
    }
    Ok(())
}

/// `Draw::VariableCommands`: variables (`axes`, `axes2d`, `pi`, `pinf`, `minf`, `grid`),
/// the repaint hooks, the `Draw_Number` restore factory, and the numeric-function and
/// variable-management commands. Registered once per session.
pub fn variable_commands(session: &DrawSession) -> Result<(), Failure> {
    if !session.first_load("Draw::VariableCommands") {
        return Ok(());
    }
    session.command_hooks.set(true);
    DrawNumber::register_factory(session);
    standard_variables(session)?;
    register(
        session,
        "DRAW Numeric functions",
        VARIABLE_FILE,
        &[
            ("cos", "cos(x)", values::trigo),
            ("sin", "sin(x)", values::trigo),
            ("tan", "tan(x)", values::trigo),
            ("acos", "acos(x)", values::trigo),
            ("asin", "asin(x)", values::trigo),
            ("atan2", "atan2(x,y)", values::trigo),
            ("sqrt", "sqrt(x)", values::trigo),
        ],
    )?;
    register(
        session,
        "DRAW Variables management",
        VARIABLE_FILE,
        &variable_management(),
    )
}

/// The protected standard variables.
fn standard_variables(session: &DrawSession) -> Result<(), Failure> {
    let blue = DrawColor::new(DrawColorKind::Bleu);
    let axes: DrawableHandle = Rc::new(DrawAxis3D::at_point(Point3::new(0.0, 0.0, 0.0), blue, 20));
    session.set_drawable_auto("axes", Some(axes.clone()))?;
    axes.set_protected(true);
    let axes_2d: DrawableHandle = Rc::new(DrawAxis2D::at_point(Point2::new(0.0, 0.0), blue, 20));
    session.set_drawable_auto("axes2d", Some(axes_2d.clone()))?;
    axes_2d.set_protected(true);
    for (name, value) in [
        ("pi", std::f64::consts::PI),
        ("pinf", f64::MAX),
        ("minf", f64::MIN),
    ] {
        session.set_number(name, value);
        if let Some(number) = session.get(name) {
            number.set_protected(true);
        }
    }
    let grid: DrawableHandle = Rc::new(DrawGrid::new());
    session.set_drawable_auto("grid", Some(grid.clone()))?;
    grid.set_protected(true);
    Ok(())
}

/// The `DRAW Variables management` group.
fn variable_management() -> Vec<(&'static str, &'static str, SessionCommand)> {
    vec![
        ("protect", "protect name ...", values::protect),
        ("unprotect", "unprotect name ...", values::protect),
        ("bsave", "bsave name filename", files::save),
        ("brestore", "brestore filename name", files::restore),
        (
            "isdraw",
            "isdraw var, return 1 if Draw value",
            values::isdraw,
        ),
        (
            "isprot",
            "isprot var, return 1 if Draw var is protected",
            values::isprot,
        ),
        (
            "autodisplay",
            "toggle autodisplay [0/1]",
            values::autodisplay,
        ),
        (
            "display",
            "display [name1 name2 ...], no names display all",
            display::display,
        ),
        (
            "donly",
            "donly [name1 name2 ...], erase and display",
            display::erase,
        ),
        (
            "erase",
            "erase [name1 name2 ...], no names erase all",
            display::erase,
        ),
        (
            "draw",
            "draw view mode [name1 name2 ...], draw on view with mode",
            display::draw,
        ),
        ("clear", "clear display", display::erase),
        ("2dclear", "clear display (2d objects)", display::erase),
        ("repaint", "repaint, force redraw", display::repaint),
        ("dtyp", "dtyp name1 name2", values::whatis),
        ("dval", "dval name, return value", values::value),
        ("dname", "dname name, print name", values::dname),
        ("dump", "dump name1 name2 ...", values::dump),
        ("copy", "copy name1 toname1 name2 toname2 ...", values::copy),
        // `renamevar`, not `rename`, which is a Tcl built-in (san, 02/08/2002).
        (
            "renamevar",
            "renamevar name1 toname1 name2 toname2 ...",
            values::copy,
        ),
        ("dset", "var1 value1 vr2 value2 ...", values::dset),
        // Access to the C environment; see Mantis issue #23197.
        (
            "dgetenv",
            "var : get value of environment variable in C subsystem",
            values::dgetenv,
        ),
        (
            "dsetenv",
            "var [value] : set (unset if value is empty) environment variable in C subsystem",
            values::dsetenv,
        ),
        ("pick", "pick id X Y Z b [nowait]", files::pick),
        (
            "lastrep",
            "lastrep id X Y [Z] b, return name",
            files::lastrep,
        ),
    ]
}
