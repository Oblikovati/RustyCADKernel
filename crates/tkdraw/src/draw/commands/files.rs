//! `bsave`/`brestore` (drawables in files) and `pick`/`lastrep` (interactive picking)
//! of `Draw_VariableCommands.cxx`.
use crate::draw::{DrawIStream, DrawInterpretor, DrawOStream, DrawProgressIndicator, DrawSession};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use std::io::Write;
use std::rc::Rc;

/// The type `brestore` falls back to when the file's first word has no factory.
const SHAPE_TYPE: &str = "DBRep_DrawableShape";

/// `bsave name file`: the type name, the drawable's own text, then `\n0\n\n`.
pub(super) fn save(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() != 3 {
        di.append("Syntax error: wrong number of arguments!\n")?;
        di.print_help(a[0])?;
        return Ok(1);
    }
    let Some(drawable) = session.get(a[1]) else {
        di.append(format!("Syntax error: '{}' is not a drawable", a[1]))?;
        return Ok(1);
    };
    let Ok(mut file) = std::fs::File::create(a[2]) else {
        di.append(format!("Error: cannot open file for writing {}", a[2]))?;
        return Ok(1);
    };
    let mut stream = DrawOStream::new();
    stream.set_precision(15);
    session.set_progress_bar(Some(Rc::new(DrawProgressIndicator::new(di, 1.0))));
    stream.put_str(drawable.type_name()).put_str("\n");
    if let Err(failure) = drawable.save(&mut stream) {
        if failure.exception_type() != "Standard_NotImplemented" {
            return Err(failure);
        }
        // The progress bar stays installed on this path, as upstream.
        di.append(format!("Error: no method for saving {}", a[1]))?;
        return Ok(1);
    }
    stream.put_str("\n").put_str("0\n\n");
    session.set_progress_bar(None);
    if file
        .write_all(stream.text().as_bytes())
        .and_then(|()| file.flush())
        .is_err()
    {
        di.append("Error: file has not been written")?;
        return Ok(1);
    }
    di.append(a[1])?;
    Ok(0)
}

/// `brestore file name`: reads the type name and restores with its factory, or as a
/// shape when the type is unknown; displays the result when autodisplay is on.
pub(super) fn restore(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() != 3 {
        return Ok(1);
    }
    let (file_name, variable) = (a[1], a[2]);
    let Ok(bytes) = std::fs::read(file_name) else {
        di.append(format!(
            "Error: cannot open file for reading: '{file_name}'"
        ))?;
        return Ok(1);
    };
    let mut stream = DrawIStream::new(bytes);
    let Some(type_name) = stream.read_word() else {
        di.append(format!("Error: cannot read file: '{file_name}'"))?;
        return Ok(1);
    };
    session.set_progress_bar(Some(Rc::new(DrawProgressIndicator::new(di, 1.0))));
    let mut restored = session.restore(&type_name, &mut stream)?;
    if restored.is_none() {
        stream.rewind();
        restored = session.restore(SHAPE_TYPE, &mut stream)?;
    }
    let Some(drawable) = restored else {
        di.append(format!("Error: cannot restore a {type_name}"))?;
        return Ok(1);
    };
    let display = drawable.is_displayable() && session.autodisplay.get();
    session.set_drawable(variable, Some(drawable), display)?;
    session.set_progress_bar(None);
    di.append(variable)?;
    Ok(0)
}

/// A pointer position in view pixels taken back to model coordinates
/// (`(X/zoom, Y/zoom, 0)` through the inverted view matrix).
fn to_model(session: &DrawSession, view: i32, x: i32, y: i32) -> Point3 {
    let viewer = session.viewer();
    let zoom = viewer.zoom(view);
    let mut transform = viewer.trsf(view).unwrap_or_default();
    // A view matrix is a rigid motion; the identity of batch mode is invertible too.
    let _ = transform.invert();
    Point3::new(x as f64 / zoom, y as f64 / zoom, 0.0).transformed(transform)
}

/// `pick id X Y Z b [nowait]`: waits for a click (or, with a 7th argument, reads the
/// pointer without waiting) and stores the view, the model point and the button. In
/// batch mode the C++ reads uninitialized values; here they are 0, so the coordinates
/// are NaN (0 divided by the batch zoom 0).
pub(super) fn pick(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 6 {
        return Ok(1);
    }
    let wait = a.len() == 6;
    let mut errors = String::new();
    let selection = session
        .viewer()
        .select(wait, &mut errors)
        .unwrap_or_default();
    session.console().write_err(&errors);
    let point = to_model(session, selection.view, selection.x, selection.y);
    session.set_number(a[1], selection.view as f64);
    session.set_number(a[2], point.x());
    session.set_number(a[3], point.y());
    session.set_number(a[4], point.z());
    session.set_number(a[5], selection.button as f64);
    Ok(0)
}

/// `lastrep id X Y [Z] b`: the last pick, in pixels (4 values) or model coordinates
/// (5 values); the result is the picked name.
pub(super) fn lastrep(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 5 {
        return Ok(1);
    }
    let last = session.last_pick();
    session.set_number(a[1], last.view as f64);
    match a.len() {
        5 => {
            session.set_number(a[2], last.x as f64);
            session.set_number(a[3], last.y as f64);
        }
        6 => {
            let point = to_model(session, last.view, last.x, last.y);
            session.set_number(a[2], point.x());
            session.set_number(a[3], point.y());
            session.set_number(a[4], point.z());
        }
        _ => {
            di.append("Too many args")?;
            return Ok(1);
        }
    }
    session.set_number(a[a.len() - 1], last.button as f64);
    di.append(last.name)?;
    Ok(0)
}
