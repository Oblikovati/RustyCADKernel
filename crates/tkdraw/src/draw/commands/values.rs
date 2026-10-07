//! The variable commands of `Draw_VariableCommands.cxx` that read and change values:
//! `protect`, `autodisplay`, `dtyp`, `dval`, `dname`, `dump`, `copy`, `renamevar`,
//! `dset`, `dsetenv`, `dgetenv`, `isdraw`, `isprot` and the numeric functions.
use crate::draw::{DrawInterpretor, DrawOStream, DrawSession};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::osd::{Environment, OsdErrorAccess};
use rustycad_tkernel::standard::Failure;
use std::fmt::Write;

/// `protect names` / `unprotect names`.
pub(super) fn protect(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() <= 1 {
        return Ok(1);
    }
    let protected = !a[0].starts_with('u');
    for name in &a[1..] {
        if let Some(drawable) = session.get(name) {
            drawable.set_protected(protected);
            di.append(*name)?.append(" ")?;
        }
    }
    Ok(0)
}

/// `autodisplay [0|1]`: toggles or sets the automatic display; prints the new state.
pub(super) fn autodisplay(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let enabled = match a.get(1) {
        None => !session.autodisplay.get(),
        Some(flag) => !flag.eq_ignore_ascii_case("0"),
    };
    session.autodisplay.set(enabled);
    di.append(if enabled { "1" } else { "0" })?;
    Ok(0)
}

/// `dtyp names`: the kind of each drawable.
pub(super) fn whatis(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() <= 1 {
        return Ok(1);
    }
    for name in &a[1..] {
        if let Some(drawable) = session.get(name) {
            drawable.whatis(di)?;
        }
    }
    Ok(0)
}

/// `dval expression`: the value of a numeric expression.
pub(super) fn value(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() != 2 {
        return Ok(1);
    }
    di.append(session.atof(a[1]))?;
    Ok(0)
}

/// `dname names`: the variable names of the drawables (useful with `.`).
pub(super) fn dname(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() <= 1 {
        return Ok(1);
    }
    for name in &a[1..] {
        if let Some(drawable) = session.get(name) {
            di.append(drawable.name().unwrap_or_default())?;
        }
    }
    Ok(0)
}

/// `dump names`: a banner and the dump of each drawable at precision 15.
pub(super) fn dump(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 2 {
        return Ok(1);
    }
    for name in &a[1..] {
        let Some(drawable) = session.get(name) else {
            continue;
        };
        let mut stream = DrawOStream::new();
        stream.set_precision(15);
        stream
            .put_str("\n\n*********** Dump of ")
            .put_str(name)
            .put_str(" *************\n");
        drawable.dump(&mut stream);
        di.append(stream.take_text())?;
    }
    Ok(0)
}

/// `copy from to ...` (a copy of the drawable) and `renamevar from to ...` (the same
/// drawable, the old name cleared).
pub(super) fn copy(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 3 {
        return Ok(1);
    }
    let make_copy = a[0].eq_ignore_ascii_case("copy");
    for pair in a[1..].chunks(2) {
        let [from, to] = pair else {
            return Ok(0);
        };
        let Some(drawable) = session.get(from) else {
            continue;
        };
        let target = if make_copy {
            drawable.copy(&drawable)
        } else {
            session.set_drawable_auto(from, None)?;
            drawable
        };
        session.set_drawable_auto(to, Some(target))?;
    }
    Ok(0)
}

/// `dset name value ...`: sets numeric variables; the result is the last value.
pub(super) fn dset(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 2 {
        return Ok(1);
    }
    let mut value = 0.0;
    for pair in a[1..].chunks(2) {
        value = pair.get(1).map_or(0.0, |text| session.atof(text));
        session.set_number(pair[0], value);
    }
    di.append(value)?;
    Ok(0)
}

/// `dsetenv name [value]`: sets (or, without a value, removes) an environment variable.
pub(super) fn dsetenv(
    _session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 2 {
        let _ = write!(Message::send_fail(), "Use: {} {{varname}} [value]", a[0]);
        return Ok(1);
    }
    let mut environment = Environment::named(a[1])?;
    match a.get(2).filter(|value| !value.is_empty()) {
        Some(value) => {
            environment.set_value(value)?;
            environment.build();
        }
        None => environment.remove(),
    }
    Ok(environment.failed() as i32)
}

/// `dgetenv name`: the value of an environment variable (empty when unset).
pub(super) fn dgetenv(
    _session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 2 {
        let _ = write!(Message::send_fail(), "Use: {} {{varname}}", a[0]);
        return Ok(1);
    }
    let value = std::env::var_os(a[1])
        .map(|v| v.to_string_lossy().into_owned())
        .unwrap_or_default();
    di.append(value)?;
    Ok(0)
}

/// `isdraw name`: 1 when the variable holds a drawable.
pub(super) fn isdraw(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() != 2 {
        return Ok(1);
    }
    di.append(if session.get(a[1]).is_some() {
        "1"
    } else {
        "0"
    })?;
    Ok(0)
}

/// `isprot name`: 1 when the variable holds a protected drawable.
pub(super) fn isprot(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() != 2 {
        return Ok(1);
    }
    let protected = session
        .get(a[1])
        .is_some_and(|drawable| drawable.protected());
    di.append(if protected { "1" } else { "0" })?;
    Ok(0)
}

/// `cos`, `sin`, `tan`, `sqrt`, `acos`, `asin`, `atan2`: the C library function of the
/// argument(s). The C++ reads `a[1]` (and `a[2]` for `atan2`) without checking the count;
/// a missing argument reads as the empty expression (0) here.
pub(super) fn trigo(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let x = session.atof(a.get(1).copied().unwrap_or(""));
    let value = match a[0].to_ascii_lowercase().as_str() {
        "cos" => x.cos(),
        "sin" => x.sin(),
        "tan" => x.tan(),
        "sqrt" => x.sqrt(),
        "acos" => x.acos(),
        "asin" => x.asin(),
        "atan2" => x.atan2(session.atof(a.get(2).copied().unwrap_or(""))),
        _ => return Ok(0),
    };
    di.append(value)?;
    Ok(0)
}
