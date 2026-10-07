// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Functions (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the edit form commands.

//! The commands on the edit forms: list, value edition, clear, apply, load.
use super::{Status, emit, first_byte, session_of};
use crate::c_text::atoi;
use crate::if_select::{
    IFSelectEditForm, IFSelectEditor, IFSelectReturnStatus, IFSelectSessionPilot,
};
use rustycad_tkernel::{
    message::{Message, MessageStream},
    standard::{Failure, Handle},
    tcollection::HAsciiString,
};
use std::fmt::Write as _;

use IFSelectReturnStatus::{RetDone, RetError, RetFail, RetVoid};

/// The text of a value, `(NULL)` for a null one.
fn value_text(val: &Handle<HAsciiString>) -> String {
    val.get().map_or_else(
        || String::from("(NULL)"),
        |t| String::from_utf8_lossy(t.string().as_bytes()).into_owned(),
    )
}

/// The edit form item of the first argument (with a message when not one).
fn named_form(
    pilot: &Handle<IFSelectSessionPilot>,
    sout: &mut MessageStream<'_>,
) -> Result<Handle<IFSelectEditForm>, Failure> {
    let arg1 = pilot.arg(1)?;
    let edf: Handle<IFSelectEditForm> = session_of(pilot)?.named_item(&arg1).cast();
    if edf.is_null() {
        let _ = writeln!(sout, "Not an EditForm : {arg1}");
    }
    Ok(edf)
}

/// The number of the value named by the second argument in a form; 0 (with a
/// message) when unknown or not extracted.
fn value_number(
    form: &IFSelectEditForm,
    arg2: &str,
    sout: &mut MessageStream<'_>,
) -> Result<i32, Failure> {
    let num = form.name_number(arg2)?;
    if num == 0 {
        let _ = writeln!(sout, "Unknown Value Name : {arg2}");
    }
    if num < 0 {
        let _ = writeln!(sout, "Not Extracted Value Name : {arg2}");
    }
    Ok(num.max(0))
}

/// The definitions of an editor and of its form in `fun_editlist`.
fn print_definitions(
    sout: &mut MessageStream<'_>,
    edt: &dyn IFSelectEditor,
    edf: &IFSelectEditForm,
) -> Result<(), Failure> {
    let _ = writeln!(
        sout,
        "Editor, Label : {}",
        String::from_utf8_lossy(edt.label().as_bytes())
    );
    let mut out = b"\n --  Names (short - complete) + Labels of Values\n".to_vec();
    let _ = edt.print_names(&mut out);
    out.extend_from_slice(b"\n --  Definitions  --\n");
    let _ = edt.print_defs(&mut out, false);
    let _ = edf.print_defs(&mut out);
    out.extend_from_slice(
        b"\nTo display values, add an option : o original  f final  m modified\n",
    );
    emit(sout, &out);
    Ok(())
}

/// `fun_editlist` (editlist): the definitions of an edit form, or its values (o
/// original, f final, else modified). Native quirk kept: an editor which is not an edit
/// form gives nothing.
pub(super) fn fun_editlist(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    if argc < 2 {
        let _ = writeln!(sout, "Give the name of an EditForm or an Editor");
        return Ok(RetError);
    }
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let ws = session_of(pilot)?;
    let edf: Handle<IFSelectEditForm> = ws.named_item(&arg1).cast();
    let Some(form) = edf.get() else {
        return Ok(RetVoid);
    };
    let _ = writeln!(sout, "Print EditForm {arg1}");
    if argc >= 3 {
        let what = match first_byte(&arg2) {
            b'o' => -1,
            b'f' => 1,
            _ => 0,
        };
        let mut out = Vec::new();
        form.print_values(&mut out, what, false, false)?;
        emit(&mut sout, &out);
        return Ok(RetVoid);
    }
    let mut edt = form.editor();
    if edt.is_null() {
        edt = ws.named_item(&arg1).cast();
    }
    let Some(editor) = edt.get() else {
        return Ok(RetVoid);
    };
    print_definitions(&mut sout, editor, form)?;
    Ok(RetVoid)
}

/// The current value (or list) of `num` in `fun_editvalue`.
fn print_current(
    sout: &mut MessageStream<'_>,
    form: &IFSelectEditForm,
    num: i32,
    islist: bool,
) -> Result<(), Failure> {
    if !islist {
        let _ = writeln!(sout, "{}", value_text(&form.edited_value(num)?));
        return Ok(());
    }
    match form.edited_list(num)? {
        None => {
            let _ = writeln!(sout, "(NULL LIST)");
        }
        Some(list) => {
            let _ = writeln!(sout, "(List : {} Items)", list.len());
            for (i, item) in list.iter().enumerate() {
                let _ = writeln!(sout, "  [{}]\t{}", i + 1, value_text(item));
            }
        }
    }
    Ok(())
}

/// The list edition of `fun_editvalue`: ". " clear, "+[nn] text" add or insert,
/// "-nn" remove, "nn text" insert before nn. Native quirk kept: "-nn" removes nothing
/// (the number is read negative).
fn edit_list(
    pilot: &Handle<IFSelectSessionPilot>,
    sout: &mut MessageStream<'_>,
    form: &IFSelectEditForm,
    num: i32,
    argval: &str,
) -> Status {
    if first_byte(argval) == b'?' {
        let _ = writeln!(
            sout,
            "To Edit, options\n + val : add value at end (blanks allowed)\n +nn text : insert val before item nn\n nn text : replace item nn with a new value\n -nn : remove item nn\n . : clear the list"
        );
        return Ok(RetVoid);
    }
    let led = form.list_editor(num)?;
    let Some(listed) = led.get() else {
        return Ok(RetError);
    };
    let text = || Handle::new(HAsciiString::new(pilot.command_part(4).as_bytes()));
    let mut stated = match first_byte(argval) {
        b'.' => listed.load_edited(None),
        b'+' => listed.add_value(
            &text(),
            if argval.len() > 1 {
                atoi(argval.as_bytes())
            } else {
                0
            },
        )?,
        b'-' => listed.remove(atoi(argval.as_bytes()), 1)?,
        _ => {
            let numset = atoi(argval.as_bytes());
            numset > 0 && listed.add_value(&text(), numset)?
        }
    };
    if stated {
        stated = form.modify_list(num, &led, true)?;
    }
    if stated {
        let _ = writeln!(sout, "List Edition done");
    } else {
        let _ = writeln!(sout, "List Edition not done, option{argval}");
    }
    Ok(RetDone)
}

/// `fun_editvalue` (editvalue): a value of an edit form displayed, then edited (a
/// text, "." for null, list options for a list).
pub(super) fn fun_editvalue(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    if argc < 3 {
        let _ = writeln!(
            sout,
            "Give the name of an EditForm + name of Value [+ newvalue or . to nullify]"
        );
        return Ok(RetError);
    }
    let edf = named_form(pilot, &mut sout)?;
    let Some(form) = edf.get() else {
        return Ok(RetError);
    };
    let num = value_number(form, &pilot.arg(2)?, &mut sout)?;
    if num <= 0 {
        return Ok(RetError);
    }
    let editor = form.editor();
    let islist = editor.is_list(num);
    let name = String::from_utf8_lossy(editor.name(num, true)?.as_bytes()).into_owned(); // real name
    let _ = write!(
        sout,
        "Value Name : {name}{}",
        if form.is_modified(num)? {
            "(already edited) : "
        } else {
            " : "
        }
    );
    print_current(&mut sout, form, num, islist)?;
    if islist && argc < 4 {
        let _ = writeln!(sout, "To Edit, options by editval edit-form value-name ?");
    }
    if argc < 4 {
        return Ok(RetVoid);
    }
    let argval = pilot.arg(3)?;
    if islist {
        return edit_list(pilot, &mut sout, form, num, &argval);
    }
    let str_val = if argval == "." {
        Handle::default()
    } else {
        Handle::new(HAsciiString::new(pilot.command_part(3).as_bytes()))
    };
    if form.modify(num, &str_val, true)? {
        let _ = writeln!(sout, "Now set to {}", value_text(&str_val));
        return Ok(RetDone);
    }
    let _ = writeln!(sout, "Modify not done");
    Ok(RetFail)
}

/// `fun_editclear` (editclear): the edition of all values, or of one, cleared.
pub(super) fn fun_editclear(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    if argc < 2 {
        let _ = writeln!(
            sout,
            "Give the name of an EditForm [+ name of Value  else all]"
        );
        return Ok(RetError);
    }
    let arg2 = pilot.arg(2)?;
    let edf = named_form(pilot, &mut sout)?;
    let Some(form) = edf.get() else {
        return Ok(RetError);
    };
    if argc < 3 {
        form.clear_edit(0);
        let _ = writeln!(sout, "All Modifications Cleared");
        return Ok(RetDone);
    }
    let num = value_number(form, &arg2, &mut sout)?;
    if num <= 0 {
        return Ok(RetError);
    }
    if !form.is_modified(num)? {
        let _ = writeln!(sout, "Value {arg2} was not modified");
        return Ok(RetVoid);
    }
    form.clear_edit(num);
    let _ = writeln!(sout, "Modification on Value {arg2} Cleared");
    Ok(RetDone)
}

/// `fun_editapply` (editapply): the edition applied to the loaded data; the edition is
/// then cleared unless "keep".
pub(super) fn fun_editapply(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    if argc < 2 {
        let _ = writeln!(
            sout,
            "Give the name of an EditForm [+ option keep to re-apply edited values]"
        );
        return Ok(RetError);
    }
    let arg2 = pilot.arg(2)?;
    let edf = named_form(pilot, &mut sout)?;
    let Some(form) = edf.get() else {
        return Ok(RetError);
    };
    let (ent, model) = (form.entity(), form.model());
    match model.get() {
        None => {
            let _ = writeln!(sout, "Applying modifications");
        }
        Some(_) if ent.is_null() => {
            let _ = writeln!(sout, "Applying modifications on loaded model");
        }
        Some(m) => {
            let mut out = b"Applying modifications on loaded entity : ".to_vec();
            let _ = m.print_label(&ent, &mut out);
            emit(&mut sout, &out);
        }
    }
    if !form.apply_data(&form.entity(), &form.model())? {
        let _ = writeln!(sout, "Modifications could not be applied");
        return Ok(RetFail);
    }
    let _ = writeln!(sout, "Modifications have been applied");
    if argc > 2 && first_byte(&arg2) == b'k' {
        let _ = writeln!(sout, "Edited values are kept for another loading/applying");
    } else {
        form.clear_edit(0);
        let _ = writeln!(sout, "Edited values are cleared");
    }
    Ok(RetDone)
}

/// `fun_editload` (editload): an edit form loaded from the model or an entity.
pub(super) fn fun_editload(pilot: &Handle<IFSelectSessionPilot>) -> Status {
    let argc = pilot.nb_words();
    let mut sout = Message::send_info();
    if argc < 2 {
        let _ = writeln!(sout, "Give the name of an EditForm [+ Entity-Ident]");
        return Ok(RetError);
    }
    let (arg1, arg2) = (pilot.arg(1)?, pilot.arg(2)?);
    let ws = session_of(pilot)?;
    let edf = named_form(pilot, &mut sout)?;
    let Some(form) = edf.get() else {
        return Ok(RetError);
    };
    let num = if argc < 3 { 0 } else { pilot.number(&arg2)? };
    let stat = if argc < 3 {
        let _ = writeln!(sout, "EditForm {arg1} : Loading Model");
        form.load_model(&ws.model())?
    } else if num <= 0 {
        let _ = writeln!(sout, "Not an entity ident : {arg2}");
        return Ok(RetError);
    } else {
        let _ = writeln!(sout, "EditForm {arg1} : Loading Entity {arg2}");
        form.load_data(&ws.starting_entity(num), &ws.model())?
    };
    if !stat {
        let _ = writeln!(sout, "Loading not done");
        return Ok(RetFail);
    }
    let _ = writeln!(sout, "Loading done");
    Ok(RetDone)
}
