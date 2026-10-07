//! Command registration: `Draw_Interpretor::Add`/`add`/`Remove` and the callback types
//! (`CommandFunction`, `CallBackData`, `CallBackDataFunc`, `CallBackDataMethod`).
use super::command_call::run_command;
use super::{DrawInterpretor, c_text};
use rustycad_tcl::{Completion, InterpHandle, sys};
use rustycad_tkernel::osd::CadPath;
use rustycad_tkernel::standard::Failure;
use std::rc::Rc;

/// A DRAW command body (`CallBackData::Invoke`): receives the interpreter and the words
/// (`args[0]` is the command name) and returns 0 for success, any other value for a Tcl
/// error. A `Failure` is the C++ exception the command would throw; the interpreter
/// reports it as "An exception was caught ...". A free function is
/// `CallBackDataFunc`; a closure capturing an object is `CallBackDataMethod`.
pub trait DrawCommand {
    /// Runs the command.
    fn invoke(&self, di: &DrawInterpretor, args: &[&str]) -> Result<i32, Failure>;
}

impl<F: Fn(&DrawInterpretor, &[&str]) -> Result<i32, Failure>> DrawCommand for F {
    fn invoke(&self, di: &DrawInterpretor, args: &[&str]) -> Result<i32, Failure> {
        self(di, args)
    }
}

/// The group of commands added without one (the C++ default argument).
pub const USER_COMMANDS: &str = "User Commands";

impl DrawInterpretor {
    /// `Add(name, help, function, group)`: defines a command without a source file.
    ///
    /// ```
    /// use rustycad_tkdraw::draw::{DrawInterpretor, RecordingConsole};
    /// use std::rc::Rc;
    /// let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
    /// di.init();
    /// let hello = |di: &DrawInterpretor, _: &[&str]| { di.append("hi")?; Ok(0) };
    /// di.add("hello", "says hi", Box::new(hello), "Demo").unwrap();
    /// assert_eq!(di.eval("hello").unwrap(), 0);
    /// assert_eq!(di.result().unwrap(), "hi");
    /// ```
    pub fn add(
        &self,
        name: &str,
        help: &str,
        command: Box<dyn DrawCommand>,
        group: &str,
    ) -> Result<(), Failure> {
        self.add_with_file(name, help, "", command, group)
    }

    /// `Add(name, help, fileName, function, group)` (the protected `add` behind every
    /// overload): creates the Tcl command, stores `help` in `Draw_Helps(name)`, appends
    /// `name` to `Draw_Groups(group)` and records the last parts of `file_name` in
    /// `Draw_Files(name)`.
    pub fn add_with_file(
        &self,
        name: &str,
        help: &str,
        file_name: &str,
        command: Box<dyn DrawCommand>,
        group: &str,
    ) -> Result<(), Failure> {
        let interp = self.interp().map_err(|_| null_interpreter())?;
        let (name, help, group) = (c_text(name), c_text(help), c_text(group));
        create_tcl_command(self, &interp, name, Rc::from(command))?;
        set_global_element(&interp, "Draw_Helps", name, help, 0)?;
        let list_append = sys::TCL_APPEND_VALUE | sys::TCL_LIST_ELEMENT;
        set_global_element(&interp, "Draw_Groups", group, name, list_append)?;
        let file_name = c_text(file_name);
        if file_name.is_empty() {
            return Ok(());
        }
        set_global_element(&interp, "Draw_Files", name, &source_path(file_name), 0)
    }

    /// `Remove`: deletes the command; true when it existed.
    pub fn remove(&self, name: &str) -> Result<bool, Failure> {
        let interp = self.interp()?;
        interp
            .delete_command(c_text(name))
            .map_err(super::tcl_failure)
    }
}

/// `Standard_ASSERT_RAISE(myInterp != nullptr, ...)` raises `Standard_ProgramError`.
fn null_interpreter() -> Failure {
    Failure::with_message(
        "Standard_ProgramError",
        "Attempt to add command to Null interpreter",
    )
}

/// `Tcl_CreateCommand` with the `CommandCmd` wrapper around `command`.
fn create_tcl_command(
    di: &DrawInterpretor,
    interp: &InterpHandle,
    name: &str,
    command: Rc<dyn DrawCommand>,
) -> Result<(), Failure> {
    let core = di.downgrade();
    let body = move |handle: &InterpHandle, args: &[&str]| -> Completion {
        run_command(&core, &*command, handle, args)
    };
    interp
        .create_command(name, Box::new(body))
        .map_err(super::tcl_failure)
}

/// `Tcl_SetVar2(interp, array, element, value, TCL_GLOBAL_ONLY | extra)`.
fn set_global_element(
    interp: &InterpHandle,
    array: &str,
    element: &str,
    value: &str,
    extra: i32,
) -> Result<(), Failure> {
    let flags = sys::TCL_GLOBAL_ONLY | extra;
    interp
        .set_var2(array, Some(element), value, flags)
        .map_err(super::tcl_failure)?;
    Ok(())
}

/// The source path kept for `Draw_Files`: at most the last four path components, without
/// disk or node, and without a leading '/'.
fn source_path(file_name: &str) -> String {
    let mut path = CadPath::new(file_name);
    let trek_count = path.trek_length();
    for _ in 4..trek_count {
        // The first trek exists while more than four remain, so removal cannot fail.
        let _ = path.remove_a_trek(1);
    }
    path.set_disk("");
    path.set_node("");
    let text = String::from_utf8_lossy(path.system_name().as_bytes()).into_owned();
    text.strip_prefix('/').map(str::to_string).unwrap_or(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_paths_keep_the_last_components() {
        assert_eq!(source_path("/a/b/c/d/e/f/file.cxx"), "c/d/e/f/file.cxx");
        assert_eq!(source_path("src/x.rs"), "src/x.rs");
    }
}
