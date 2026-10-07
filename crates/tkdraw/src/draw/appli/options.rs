//! The command line of `Draw_Appli`.
use crate::draw::console::DrawConsole;
use rustycad_tkernel::standard::version_info::occt_version_string_extended;

/// What the command line asks for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppliOptions {
    /// `-h`/`--help`: print the usage and stop.
    pub help: bool,
    /// `-b`: batch mode (no GUI, no viewers).
    pub batch: bool,
    /// `-v`, or the default with `-f`/`-c`: off-screen viewer windows.
    pub virtual_windows: bool,
    /// `-i`: interactive even with `-f`/`-c`.
    pub interactive_forced: bool,
    /// `-f file`: the script to run.
    pub run_file: Option<String>,
    /// `-c command args...`: the command to run (the words joined by spaces).
    pub command: Option<String>,
}

impl AppliOptions {
    /// Parses `args` (`args[0]` is the program). Options are case-insensitive; `-f` and
    /// `-c` end the parsing; an unknown option is reported on `console` and skipped.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{AppliOptions, RecordingConsole};
    /// let args: Vec<String> = ["draw", "-B", "-c", "puts", "hi"].map(String::from).to_vec();
    /// let options = AppliOptions::parse(&args, &RecordingConsole::default());
    /// assert!(options.batch && options.virtual_windows);
    /// assert_eq!(options.command.as_deref(), Some("puts hi"));
    /// ```
    pub fn parse(args: &[String], console: &dyn DrawConsole) -> Self {
        let mut options = Self::default();
        let mut index = 1;
        while index < args.len() {
            let argument = args[index].to_ascii_lowercase();
            match argument.as_str() {
                "-h" | "--help" => {
                    options.help = true;
                    return options;
                }
                "-b" => options.batch = true,
                "-v" => options.virtual_windows = true,
                "-i" => {
                    options.virtual_windows = false;
                    options.interactive_forced = true;
                }
                "-f" => return options.with_file(args.get(index + 1)),
                "-c" => return options.with_command(&args[index + 1..]),
                _ => console.write_out(&format!("Error: unsupported option {}\n", args[index])),
            }
            index += 1;
        }
        options
    }

    /// `-f file`: virtual windows unless `-i` came first.
    fn with_file(mut self, file: Option<&String>) -> Self {
        self.virtual_windows = !self.interactive_forced;
        self.run_file = file.cloned();
        self
    }

    /// `-c words...`: virtual windows unless `-i` came first.
    fn with_command(mut self, words: &[String]) -> Self {
        self.virtual_windows = !self.interactive_forced;
        if !words.is_empty() {
            self.command = Some(words.join(" "));
        }
        self
    }
}

/// The text of `-h`.
pub(super) fn help_text() -> String {
    format!(
        "Open CASCADE {} DRAW Test Harness\n\n\
         Options:\n\
         \x20 -b: batch mode (no GUI, no viewers)\n\
         \x20 -v: no GUI, use virtual (off-screen) windows for viewers\n\
         \x20 -i: interactive mode\n\
         \x20 -f file: execute script from file\n\
         \x20 -c command args...: execute command (with optional arguments)\n\n\
         Options -b, -v, and -i are mutually exclusive.\n\
         If -c or -f are given, -v is default; otherwise default is -i.\n\
         Options -c and -f are alternatives and should be at the end \n\
         of the command line.\n\
         Option -c can accept set of commands separated by ';'.\n",
        occt_version_string_extended()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::RecordingConsole;

    fn parse(words: &[&str]) -> (AppliOptions, String) {
        let console = RecordingConsole::default();
        let args: Vec<String> = words.iter().map(|w| w.to_string()).collect();
        (AppliOptions::parse(&args, &console), console.output())
    }

    #[test]
    fn interactive_before_file_keeps_real_windows() {
        let (options, _) = parse(&["draw", "-i", "-f", "x.tcl", "ignored"]);
        assert!(options.interactive_forced && !options.virtual_windows);
        assert_eq!(options.run_file.as_deref(), Some("x.tcl"));
    }

    #[test]
    fn unknown_options_are_reported() {
        let (options, output) = parse(&["draw", "-z", "-h", "-b"]);
        assert!(options.help && !options.batch);
        assert_eq!(output, "Error: unsupported option -z\n");
    }

    #[test]
    fn the_help_names_the_version() {
        assert!(
            help_text().starts_with("Open CASCADE 8.1.0.dev1-3d097a0328 DRAW Test Harness\n\n")
        );
    }
}
