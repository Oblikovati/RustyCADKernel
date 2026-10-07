//! `DRAWEXE`: the DRAW test harness executable (`Draw_Main` with `Draw_InitAppli`).
//!
//! Registers the `Draw`, `DBRep` and `DrawTrSurf` commands (as `Draw_InitAppli` of the
//! C++ DRAWEXE does) and runs `Draw_Appli`. The process
//! ends with `Tcl_Exit(0)`, as the C++ executable does when its global interpreter is
//! destroyed: the exit status is 0 whatever the scripts did.
use rustycad_tkdraw::draw::{DrawConsole, DrawSession, ProcessConsole, commands, draw_appli};
use rustycad_tkernel::standard::Failure;
use std::rc::Rc;

/// `Draw_InitAppli` of DRAWEXE: the commands of this toolkit.
fn init_appli(session: &DrawSession) -> Result<(), Failure> {
    commands(session)?;
    rustycad_tkdraw::dbrep::basic_commands(session)?;
    rustycad_tkdraw::draw_tr_surf::basic_commands(session)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let console: Rc<dyn DrawConsole> = Rc::new(ProcessConsole);
    if let Err(failure) = draw_appli(&args, init_appli, console.clone()) {
        console.write_err(&format!("DRAWEXE: {failure}\n"));
    }
    console.flush();
    rustycad_tcl::exit(0);
}
