// Rust port of DataExchange/TKXSBase/XSControl/XSControl_FuncShape (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSControl_FuncShape`: the commands between the transfers and the shape variables
//! of the session ("DE: General"), registered by `Init`, and the helpers giving shapes
//! by name and the file and variable names of an import. The commands are split by
//! subject: `draw` (`tpdraw`), `access` (`tpcompound`, `trdraw`, `trsave`, `trcomp`,
//! `trscomp`), `origin` (`fromshape`, `trconnexent`), `import` (`trimport`,
//! `trimpcomp`, `twrite`).
use super::{XSControl, XSControlVarsOps, XSControlWorkSession};
use crate::if_select::{IFSelectAct, IFSelectActFunc, IFSelectReturnStatus, IFSelectSessionPilot};
use crate::transfer_brep::TransferBRep;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    message::Message,
    standard::{Failure, Handle, exceptions},
    tcollection::AsciiString,
};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};

mod access;
mod draw;
mod import;
mod origin;

/// The result of a command.
type Status = Result<IFSelectReturnStatus, Failure>;

/// `XSControl_FuncShape`: the static members.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectActivatorBase;
/// use rustycad_tkxsbase::xs_control::XSControlFuncShape;
/// XSControlFuncShape::init().unwrap();
/// assert!(IFSelectActivatorBase::select("tpdraw").is_some());
/// ```
pub struct XSControlFuncShape;

/// The names `FileAndVar` gives: the file to read (or the loaded one) and the root of
/// the variable names.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct XSControlFileAndVar {
    /// True when a file is to be read (natively the result of `FileAndVar`).
    pub from_file: bool,
    /// The file (`resfile`).
    pub file: AsciiString,
    /// The root of the variable names (`resvar`).
    pub var: AsciiString,
}

/// The native static `THE_XSControl_FuncShape_initactor`.
static INIT_ACTOR: AtomicBool = AtomicBool::new(false);

/// The commands of `Init`, in the native order: name, help, function.
const COMMANDS: &[(&str, &str, IFSelectActFunc)] = &[
    (
        "tpdraw",
        "[mode:item or root]  num|*  [nomvar] Passes an ITEM to Shape Draw (Start or Result)",
        draw::tpdraw,
    ),
    (
        "tpcompound",
        "name:cstring [givelist] : -> compound with Shapes Root or from givelist",
        access::tpcompound,
    ),
    (
        "trdraw",
        "results ->DRAW : all;  or num [name] : from ent.num -> DRAW [name/tread_num]",
        access::traccess,
    ),
    (
        "trsave",
        "results ->files : all;  or num [name] : from ent.num -> DRAW [name/tread_num]",
        access::traccess,
    ),
    (
        "trcomp",
        "results -> 1 compound -> DRAW + name optional",
        access::traccess,
    ),
    (
        "trscomp",
        "results -> 1 compound -> file + name optional",
        access::traccess,
    ),
    (
        "fromshape",
        "shape [level=1]: imported/exported entity (when known)",
        origin::fromshape,
    ),
    (
        "trconnexent",
        "name of draw shape : entities -> connected shapes (when known)",
        origin::trconnexentities,
    ),
    (
        "trimport",
        "filename or .  varname  givelist  -> 1 shape per entity",
        import::trimport,
    ),
    (
        "trimpcomp",
        "filename or .  varname  givelist -> one xcompound",
        import::trimport,
    ),
    (
        "twrite",
        "shape : transfer write for this shape, AFTER newmodel !",
        import::twrite,
    ),
];

impl XSControlFuncShape {
    /// `Init`: the commands registered (once per process) in the group "DE: General".
    pub fn init() -> Result<(), Failure> {
        if INIT_ACTOR.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        IFSelectAct::set_group("DE: General", "");
        for (name, help, func) in COMMANDS {
            IFSelectAct::add_func(name, help, *func)?;
        }
        Ok(())
    }
    /// `MoreShapes(session, list, name)`: the shapes `name` gives added to `list`: "*"
    /// the root results of the last transfer, "**" all its results, "nom(n1-n2)" the
    /// variables nom<n1> to nom<n2>, else the variable `name`; their count.
    pub fn more_shapes(
        session: &Handle<XSControlWorkSession>,
        list: &mut Vec<Shape>,
        name: &str,
    ) -> Result<i32, Failure> {
        let ws = live_ws(session)?;
        let mut sout = Message::send_info();
        if matches!(name, "*" | "**") {
            let tp = reader_process(ws)?;
            if tp.is_null() {
                let _ = writeln!(sout, "last transfer : unknown");
                return Ok(0);
            }
            let Some(li) = TransferBRep::shapes(&tp, name == "*")? else {
                return Ok(0);
            };
            list.extend_from_slice(&li);
            return Ok(li.len() as i32);
        }
        if let Some((nom, n1, n2)) = numbered_range(name) {
            return Ok(numbered_shapes(ws, list, &nom, n1, n2, &mut sout));
        }
        let shape = ws
            .vars()
            .get()
            .map(|v| v.get_shape(&mut name.to_owned()))
            .unwrap_or_default();
        if shape.is_null() {
            let _ = writeln!(sout, "not a shape draw:{name}");
            return Ok(0);
        }
        list.push(shape);
        Ok(1)
    }
    /// `FileAndVar(session, file, var, def, resfile, resvar)`: the file to read (the
    /// loaded file for "" or ".") and the root of the variables: `var` itself (unless ""
    /// or "."), `def` without file, else the base name of the file.
    pub fn file_and_var(
        session: &Handle<XSControlWorkSession>,
        file: &str,
        var: &str,
        def: &str,
    ) -> Result<XSControlFileAndVar, Failure> {
        let ws = live_ws(session)?;
        let from_file = !(file.is_empty() || file == ".");
        let resfile = if from_file {
            AsciiString::new(file)
        } else {
            ws.loaded_file()
        };
        let resvar = if !var.is_empty() && var != "." {
            AsciiString::new(var)
        } else if resfile.is_empty() {
            AsciiString::new(def)
        } else {
            base_name(&resfile)?
        };
        Ok(XSControlFileAndVar {
            from_file,
            file: resfile,
            var: resvar,
        })
    }
}

/// The base name of a file: between its last '/' (else '\\') and its last '.'.
fn base_name(resfile: &AsciiString) -> Result<AsciiString, Failure> {
    let mut nomdeb = resfile.search_from_end("/");
    if nomdeb <= 0 {
        nomdeb = resfile.search_from_end("\\"); // for NT
    }
    nomdeb = nomdeb.max(0);
    let mut nomfin = resfile.search_from_end(".");
    if nomfin < nomdeb {
        nomfin = resfile.length() + 1;
    }
    resfile.sub_string(nomdeb + 1, nomfin - 1)
}

/// The name and the bounds of a range "nom(n1-n2)" (n1 negative: n1 + n2); `None`
/// unless 0 < n1 <= n2. Native quirk kept: the positions of '(', '-' and ')' are the
/// last ones, and a mark at position 0 does not count.
fn numbered_range(name: &str) -> Option<(String, i32, i32)> {
    let bytes = name.as_bytes();
    let last = |c: u8| bytes.iter().rposition(|b| *b == c).unwrap_or(0);
    let (paro, moins, parf) = (last(b'('), last(b'-'), last(b')'));
    if paro == 0 || moins == 0 || parf == 0 {
        return None;
    }
    let n2 = crate::c_text::atoi(&bytes[moins + 1..]);
    let mut n1 = crate::c_text::atoi(&bytes[paro + 1..]);
    if n1 < 0 {
        n1 += n2; // otherwise we have n1-n2
    }
    (n1 > 0 && n1 <= n2).then(|| (String::from_utf8_lossy(&bytes[..paro]).into_owned(), n1, n2))
}

/// The variables nom<n1> to nom<n2> that are shapes, added to `list`; their count.
fn numbered_shapes(
    ws: &XSControlWorkSession,
    list: &mut Vec<Shape>,
    nom: &str,
    n1: i32,
    n2: i32,
    sout: &mut rustycad_tkernel::message::MessageStream<'_>,
) -> i32 {
    let _ = write!(sout, "Shapes DRAW named : {nom}{n1} to {nom}{n2}");
    let vars = ws.vars();
    let mut nbsh = 0;
    for i in n1..=n2 {
        let shape = vars
            .get()
            .map(|v| v.get_shape(&mut format!("{nom}{i}")))
            .unwrap_or_default();
        if !shape.is_null() {
            list.push(shape);
            nbsh += 1;
        }
    }
    let _ = writeln!(sout, "  -> taken {nbsh} Shapes");
    nbsh
}

/// A live session, `Standard_NullObject` when null (a native null dereference).
fn live_ws(session: &Handle<XSControlWorkSession>) -> Result<&XSControlWorkSession, Failure> {
    session
        .get()
        .ok_or_else(|| exceptions::null_object("XSControl_FuncShape: null work session"))
}

/// The transient process of the transfer reader of a session (null when none);
/// `Standard_NullObject` without transfer reader (a native null dereference).
fn reader_process(
    ws: &XSControlWorkSession,
) -> Result<Handle<crate::transfer::TransferTransientProcess>, Failure> {
    let tr = ws.transfer_reader();
    let reader = tr.get().ok_or_else(|| {
        exceptions::null_object("XSControl_FuncShape: the session has no transfer reader")
    })?;
    Ok(reader.transient_process())
}

/// `XSControl::Session(pilot)`, `Standard_NullObject` when the pilot has no XSControl
/// session (a native null dereference).
fn xs_session(
    pilot: &Handle<IFSelectSessionPilot>,
) -> Result<Handle<XSControlWorkSession>, Failure> {
    let ws = XSControl::session(pilot)?;
    if ws.is_null() {
        return Err(exceptions::null_object(
            "XSControl_FuncShape: the pilot has no XSControl work session",
        ));
    }
    Ok(ws)
}

/// `XSControl::Vars(pilot)`, `Standard_NullObject` without variables (a native null
/// dereference).
fn xs_vars(pilot: &Handle<IFSelectSessionPilot>) -> Result<Handle<dyn XSControlVarsOps>, Failure> {
    let vars = XSControl::vars(pilot)?;
    if vars.is_null() {
        return Err(exceptions::null_object(
            "XSControl_FuncShape: the session of the pilot has no variables",
        ));
    }
    Ok(vars)
}

/// The text of an `AsciiString`.
fn text(s: &AsciiString) -> String {
    String::from_utf8_lossy(s.as_bytes()).into_owned()
}
