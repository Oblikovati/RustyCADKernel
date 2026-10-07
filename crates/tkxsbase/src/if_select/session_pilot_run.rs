// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SessionPilot (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the execution.

//! The execution of [`IFSelectSessionPilot`]: scripts, commands, counters, and the
//! basic commands it is the activator of. The messages go to the standard output, as
//! natively.
use super::{
    IFSelectActivator, IFSelectActivatorBase, IFSelectPrintCount, IFSelectReturnStatus,
    IFSelectSessionPilot, IFSelectSignCounter, IFSelectSignCounterOps,
};
use rustycad_tkernel::{
    message::Message,
    standard::{Failure, Handle, exceptions},
    tcollection::AsciiString,
};
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Write};

/// The commands by line of the help, and the width of a command.
const MAXCOMPERLINE: usize = 5;
const LENGTHFORCOM: usize = 15;
/// The size of the native line buffer of a script (`fgets` reads one less).
const LINE_BUFFER: usize = 100;

/// One line of a script as the native `fgets(ligne, 100, fic)`: at most 99 bytes, up to
/// and with the end of line. `None` at the end, or (native quirk kept, `feof` tested
/// after `fgets`) for a last line without end of line.
fn read_script_line(reader: &mut dyn BufRead) -> Option<Vec<u8>> {
    let mut line = Vec::new();
    while line.len() < LINE_BUFFER - 1 {
        let mut byte = [0u8; 1];
        match reader.read(&mut byte) {
            Ok(1) => {
                line.push(byte[0]);
                if byte[0] == b'\n' {
                    return Some(line);
                }
            }
            _ => return None,
        }
    }
    Some(line)
}

/// The text of an `AsciiString`.
fn text(s: &AsciiString) -> String {
    String::from_utf8_lossy(s.as_bytes()).into_owned()
}

/// The help list of `Do(0)`: the commands, five by line.
fn print_command_list(list: &[AsciiString]) {
    let mut out = String::from(" -- List of Available Commands --\n");
    let mut nbcom = 0;
    for uncom in list {
        nbcom += 1;
        if nbcom > MAXCOMPERLINE {
            out.push('\n');
            nbcom = 1;
        }
        let _ = write!(out, " {}", text(uncom));
        if nbcom == MAXCOMPERLINE {
            continue;
        }
        out.push_str(&" ".repeat(LENGTHFORCOM.saturating_sub(uncom.len())));
    }
    if nbcom > 0 {
        out.push('\n');
    }
    out.push_str("\nhelp *  lists all commands with help on each\nhelp <com> lists the command(s) starting with <com> with help on each\n");
    print!("{out}");
}

impl IFSelectSessionPilot {
    /// `ReadScript(file = "")`: the commands of a file (else of the standard input, with
    /// the prompt) executed up to a stop, or for a file up to an error. Void at the end
    /// of a file, stop at the end of the standard input.
    pub fn read_script(&self, file: &str) -> Result<IFSelectReturnStatus, Failure> {
        if file.is_empty() {
            let stdin = std::io::stdin();
            self.run_script(&mut stdin.lock(), None)?;
            return Ok(IFSelectReturnStatus::RetStop);
        }
        let Ok(fic) = std::fs::File::open(file) else {
            println!(" ...   Script File {file} not found");
            return Ok(IFSelectReturnStatus::RetFail);
        };
        println!(" ...   Reading Script File {file}");
        let stat = self.run_script(&mut BufReader::new(fic), Some(file))?;
        println!("End of Reading Script File {file}");
        if matches!(
            stat,
            IFSelectReturnStatus::RetError | IFSelectReturnStatus::RetFail
        ) {
            return Ok(stat);
        }
        Ok(IFSelectReturnStatus::RetVoid) // end file : unstack
    }
    /// The loop of `ReadScript` on `reader` (`file` named, else interactive).
    pub(super) fn run_script(
        &self,
        reader: &mut dyn BufRead,
        file: Option<&str>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let mut stat = IFSelectReturnStatus::RetVoid;
        loop {
            if file.is_none() {
                print!("{}", text(&self.state.read().theprompt));
                let _ = std::io::stdout().flush();
            }
            let Some(ligne) = read_script_line(reader) else {
                break;
            };
            let command = AsciiString::new(&String::from_utf8_lossy(&ligne));
            if let Some(name) = file {
                print!("{name}:{}", text(&command)); // the return is in the line ... !
            }
            stat = self.execute(&command)?;
            if stat == IFSelectReturnStatus::RetStop {
                break;
            }
            if matches!(
                stat,
                IFSelectReturnStatus::RetError | IFSelectReturnStatus::RetFail
            ) && file.is_some()
            {
                println!(" ...   Error in Script File, abandon");
                break;
            }
        }
        Ok(stat)
    }
    /// The item a command recorded, put in the session (under `name` when given);
    /// false when the session refuses it.
    fn take_recorded(&self, name: Option<&str>) -> Result<Option<bool>, Failure> {
        let objrec = self.state.read().theobjrec.clone();
        if objrec.is_null() {
            return Ok(None);
        }
        let session = self.live_session()?;
        session.remove_item(&objrec)?; //// troubleshooting ?
        let addws = match name {
            Some(n) => session.add_named_item(n, &objrec, true)?,
            None => session.add_item(&objrec, true)?,
        };
        Ok(Some(addws != 0))
    }
    /// `Perform`: the command of the current line executed by its activator; the item it
    /// records goes in the session; the line is recorded in record mode.
    pub fn perform(&self) -> Result<IFSelectReturnStatus, Failure> {
        let first = {
            let st = self.state.read();
            if st.thenbwords == 0 || st.thewords[0].first() == Some(&b'#') {
                return Ok(IFSelectReturnStatus::RetVoid); // comment
            }
            String::from_utf8_lossy(&st.thewords[0]).into_owned()
        };
        self.state.write().theobjrec = Handle::default();
        let Some((num, actor)) = IFSelectActivatorBase::select(&first) else {
            println!(" Command : {first} unknown");
            return Ok(IFSelectReturnStatus::RetError); // not recognized therefore incorrect
        };
        let live = actor.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_SessionPilot::Perform: null activator")
        })?;
        let stat = live.do_command(num, &self.this_pilot()?)?;
        if self.take_recorded(None)? == Some(false) {
            println!("Could not add item to session, sorry");
            return Ok(IFSelectReturnStatus::RetFail);
        }
        let command = text(&self.command_line());
        match stat {
            IFSelectReturnStatus::RetVoid | IFSelectReturnStatus::RetDone => {
                let mut st = self.state.write();
                if st.therecord {
                    st.thecomlist.push(AsciiString::new(&command));
                }
            }
            IFSelectReturnStatus::RetError => println!("Error in Command : {command}"),
            IFSelectReturnStatus::RetFail => println!("Execution Failure for : {command}"),
            IFSelectReturnStatus::RetStop => {}
        }
        Ok(stat)
    }
    /// `ExecuteAlias(aliasname)`: `Perform` with the first word replaced (when given).
    pub fn execute_alias(&self, aliasname: &AsciiString) -> Result<IFSelectReturnStatus, Failure> {
        if !aliasname.is_empty() {
            self.state.write().thewords[0] = aliasname.as_bytes().to_vec();
        }
        self.perform()
    }
    /// `Execute(command)`: `SetCommandLine` then `Perform`.
    pub fn execute(&self, command: &AsciiString) -> Result<IFSelectReturnStatus, Failure> {
        self.set_command_line(command)?;
        self.perform()
    }
    /// `ExecuteCounter(counter, numword, mode = IFSelect_CountByItem)`: the counter on the
    /// model (without words from `numword`) or on the list the words give, printed.
    pub fn execute_counter(
        &self,
        counter: &Handle<IFSelectSignCounter>,
        numword: i32,
        mode: IFSelectPrintCount,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let ops = counter.cast::<dyn IFSelectSignCounterOps>();
        let Some(live) = ops.get() else {
            return Ok(IFSelectReturnStatus::RetError);
        };
        live.clear();
        let session = self.live_session()?;
        if self.nb_words() <= numword {
            live.add_model(&session.model())?;
        } else {
            let part = text(&self.command_part(numword));
            let Some(list) = session.give_list_texts(&part, "")? else {
                println!("Nothing selected from : {part}");
                return Ok(IFSelectReturnStatus::RetError);
            };
            live.add_with_graph(Some(&list), &session.graph()?.graph())?;
        }
        let mut out = Vec::new();
        live.print_list(&mut out, &session.model(), mode)?;
        let _ = write!(Message::send_info(), "{}", String::from_utf8_lossy(&out));
        Ok(IFSelectReturnStatus::RetVoid)
    }
    /// The help of `Do(0)` and `Do(-1)`: the list of the commands (of the mode
    /// `modhelp`), or the help of each command starting with the first argument.
    fn command_help(&self, modhelp: i32) -> Result<IFSelectReturnStatus, Failure> {
        if self.nb_words() <= 1 {
            print_command_list(&IFSelectActivatorBase::commands(modhelp, ""));
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        let arg1 = self.arg(1)?;
        let list = if arg1 == "*" {
            IFSelectActivatorBase::commands(modhelp, "")
        } else {
            IFSelectActivatorBase::commands(modhelp, &arg1)
        };
        for name in &list {
            let name = text(name);
            let Some((num, actor)) = IFSelectActivatorBase::select(&name) else {
                continue;
            };
            let prefix = if IFSelectActivatorBase::mode(&name) == 1 {
                "[xsnew name] "
            } else {
                ""
            };
            let help = actor.get().map(|a| text(&a.help(num))).unwrap_or_default();
            println!("{prefix}{name}\t: {help}");
        }
        if list.is_empty() {
            println!(" Command {arg1} unknown.  help (without command) lists all the commands");
        }
        Ok(IFSelectReturnStatus::RetVoid)
    }
    /// `Do(3)`: the command control (analyse, clear, save in a file, list, record).
    fn command_control(
        &self,
        session: &Handle<IFSelectSessionPilot>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let argc = self.nb_words();
        if argc < 2 {
            println!(
                "Give an option :\na : analyze a line  r : toggle record mode\nl : list recorded  c : clear  f name : save in file of name"
            );
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        match self.arg(1)?.bytes().next() {
            Some(b'a') => {
                println!(
                    "Command n0 3 : {}\nNb Words : {} :",
                    text(&session.command_line()),
                    argc - 2
                );
                for i in 2..argc {
                    println!(" Word.{} : {}", i - 1, text(&session.word(i)?));
                }
            }
            Some(b'c') => session.clear(),
            Some(b'f') => return self.save_commands(session, argc),
            Some(b'l') => {
                println!(
                    "  -- Record Mode {}",
                    if session.record_mode() {
                        "Active"
                    } else {
                        "Inactive"
                    }
                );
                let nb = session.nb_commands();
                println!("Nb Recorded Commands : {nb} :");
                for i in 1..=nb {
                    println!("  {i}\t{}", text(&session.command(i)?));
                }
            }
            Some(b'r') => {
                let mode = session.record_mode();
                println!(
                    " -- Record Mode now {}",
                    if mode { "Inactive" } else { "Active" }
                );
                session.set_record_mode(!mode);
            }
            _ => println!("Command control option not understood"),
        }
        Ok(IFSelectReturnStatus::RetVoid)
    }
    /// `Do(3)` option f: the recorded commands saved in the file of the second argument.
    fn save_commands(
        &self,
        session: &Handle<IFSelectSessionPilot>,
        argc: i32,
    ) -> Result<IFSelectReturnStatus, Failure> {
        if argc < 3 {
            println!("Give file name");
            return Ok(IFSelectReturnStatus::RetError);
        }
        let nb = session.nb_commands();
        if nb == 0 {
            println!("No command recorded");
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        println!("Nb Recorded Commands : {nb}");
        let mut lines = Vec::new();
        for i in 1..=nb {
            lines.extend_from_slice(session.command(i)?.as_bytes());
            lines.push(b'\n');
        }
        let _ = std::fs::write(self.arg(2)?, lines); // a stream that fails to open writes nothing
        Ok(IFSelectReturnStatus::RetVoid)
    }
    /// `Do(6)`: `xsnew name command args`, the item the command records added under
    /// `name`.
    fn new_named_item(&self) -> Result<IFSelectReturnStatus, Failure> {
        if self.nb_words() < 3 {
            println!(
                "xsnew varname command [args]   creates an item\n  varname : name of item (must be a new name) in the session"
            );
            return self.do_command(-1, &self.this_pilot()?);
        }
        self.state.write().theobjrec = Handle::default();
        let name = self.arg(1)?;
        if self.session().is_null() {
            println!("Command with a Name and no Session defined !");
            return Ok(IFSelectReturnStatus::RetFail);
        }
        self.remove_word(0)?;
        self.remove_word(0)?;
        let first = self.arg(0)?;
        let Some((num, actor)) = IFSelectActivatorBase::select(&first) else {
            println!(" Command : {first} unknown");
            return Ok(IFSelectReturnStatus::RetError); // not recognized therefore incorrect
        };
        self.state.write().theobjrec = Handle::default();
        let live = actor
            .get()
            .ok_or_else(|| exceptions::null_object("IFSelect_SessionPilot::Do: null activator"))?;
        let stat = live.do_command(num, &self.this_pilot()?)?;
        match self.take_recorded(Some(&name))? {
            None => println!("Remark : xsnew with name:{name} and no result"),
            Some(added) => {
                self.state.write().theobjrec = Handle::default();
                if !added {
                    println!("Could not add named item:{name}, sorry");
                    return Ok(IFSelectReturnStatus::RetFail);
                }
            }
        }
        Ok(stat)
    }
}

impl IFSelectActivator for IFSelectSessionPilot {
    fn activator_base(&self) -> &IFSelectActivatorBase {
        &self.base
    }
    /// `Do(number, session)`: the basic commands (-1/0 help, 1 exit, 2 help, 3 command
    /// control, 4 script, 5 xstep, 6 xsnew) on the words of this pilot (the activator,
    /// the first pilot of the process); `session` is the pilot that executes. Unknown
    /// numbers are an error.
    fn do_command(
        &self,
        number: i32,
        session: &Handle<IFSelectSessionPilot>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        match number {
            -1 => {
                println!("  --  Commands candidate for  xsnew  --");
                self.command_help(1)
            }
            0 => self.command_help(-1),
            1 => Ok(IFSelectReturnStatus::RetStop), //        ****     End of session
            2 => self.do_command(0, &self.this_pilot()?),
            3 => self.command_control(session),
            4 => {
                if self.nb_words() < 2 {
                    println!("Give file name");
                    return Ok(IFSelectReturnStatus::RetError);
                }
                session.read_script(&session.arg(1)?)
            }
            5 => {
                if self.nb_words() < 2 {
                    println!(
                        "xstep : neutral prefix for any xstep-draw command\nxstep command args  equivalent to  command args"
                    );
                    return self.do_command(2, &self.this_pilot()?);
                }
                self.remove_word(0)?;
                self.perform()
            }
            6 => self.new_named_item(),
            _ => Ok(IFSelectReturnStatus::RetError),
        }
    }
    /// `Help(number)`.
    fn help(&self, number: i32) -> AsciiString {
        AsciiString::new(match number {
            1 => "exit or x : End of session",
            2 => "Lists the commands. ? <title> : commands starting with <title>",
            3 => "command control. command alone for complete help",
            4 => "reads commands from a file",
            5 => "neutral prefix for xstep-draw",
            6 => "item creation : give item_name then command args",
            _ => "",
        })
    }
}
