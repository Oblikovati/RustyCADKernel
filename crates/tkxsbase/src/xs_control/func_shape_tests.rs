//! Unit tests of `XSControl_FuncShape`: the shape commands run by a pilot on a session
//! of the fake norm, and the helpers giving shapes and names.
use super::test_norm::vertex_compound;
use super::transfer_tests::reading_session;
use super::*;
use crate::if_select::IFSelectReturnStatus::{RetDone, RetError, RetVoid};
use crate::if_select::{IFSelectFunctions, IFSelectReturnStatus, IFSelectSessionPilot};
use crate::interface::InterfaceModelEntities;
use rustycad_tkernel::{message::ProgressRange, standard::Handle, tcollection::AsciiString};

/// A pilot on a session of the fake norm (four roots), its roots transferred when
/// `transferred`.
fn shape_pilot(transferred: bool) -> (Handle<IFSelectSessionPilot>, Handle<XSControlWorkSession>) {
    IFSelectFunctions::init().unwrap();
    XSControlFuncShape::init().unwrap();
    XSControlFunctions::init().unwrap();
    let pilot = IFSelectSessionPilot::new("xs>").unwrap();
    let (ws, _) = reading_session();
    if transferred {
        ws.transfer_read_roots(&ProgressRange::default()).unwrap();
    }
    pilot.set_session(&ws.clone().cast());
    (pilot, ws)
}

fn check(pilot: &Handle<IFSelectSessionPilot>, commands: &[(&str, IFSelectReturnStatus)]) {
    for (command, expected) in commands {
        let status = pilot.execute(&AsciiString::new(command)).unwrap();
        assert_eq!(status, *expected, "{command}");
    }
}

/// True when the variable `name` of the session holds a shape.
fn has_shape(ws: &XSControlWorkSession, name: &str) -> bool {
    !ws.vars().get_shape(&mut name.to_owned()).is_null()
}

#[test]
fn more_shapes_by_name() {
    let (_, ws) = shape_pilot(false);
    let mut list = Vec::new();
    assert_eq!(
        XSControlFuncShape::more_shapes(&ws, &mut list, "*").unwrap(),
        0
    );
    ws.transfer_read_roots(&ProgressRange::default()).unwrap();
    assert_eq!(
        XSControlFuncShape::more_shapes(&ws, &mut list, "*").unwrap(),
        2
    );
    assert_eq!(
        XSControlFuncShape::more_shapes(&ws, &mut list, "**").unwrap(),
        2
    );
    ws.vars().set_shape("v1", &vertex_compound());
    ws.vars().set_shape("v3", &vertex_compound());
    assert_eq!(
        XSControlFuncShape::more_shapes(&ws, &mut list, "v(1-3)").unwrap(),
        2
    );
    assert_eq!(
        XSControlFuncShape::more_shapes(&ws, &mut list, "v(-2-3)").unwrap(),
        2,
        "n1 = -2 + 3 = 1: v1 and v3"
    );
    assert_eq!(
        XSControlFuncShape::more_shapes(&ws, &mut list, "v1").unwrap(),
        1
    );
    assert_eq!(
        XSControlFuncShape::more_shapes(&ws, &mut list, "nope").unwrap(),
        0
    );
    assert_eq!(list.len(), 9);
}

#[test]
fn file_and_var_names() {
    let (_, ws) = shape_pilot(false);
    let names = |file: &str, var: &str| {
        let r = XSControlFuncShape::file_and_var(&ws, file, var, "DEF").unwrap();
        (
            r.from_file,
            String::from_utf8_lossy(r.file.as_bytes()).into_owned(),
            String::from_utf8_lossy(r.var.as_bytes()).into_owned(),
        )
    };
    assert_eq!(names(".", ""), (false, String::new(), String::from("DEF")));
    assert_eq!(
        names("/a/b/model.stp", "."),
        (true, String::from("/a/b/model.stp"), String::from("model"))
    );
    assert_eq!(
        names("dir\\m.igs", ""),
        (true, String::from("dir\\m.igs"), String::from("m"))
    );
    assert_eq!(
        names("plain", ""),
        (true, String::from("plain"), String::from("plain"))
    );
    assert_eq!(
        names("x.stp", "mine"),
        (true, String::from("x.stp"), String::from("mine"))
    );
}

#[test]
fn tpdraw_gives_results_to_variables() {
    let (pilot, ws) = shape_pilot(true);
    check(
        &pilot,
        &[
            ("tpdraw", RetError),
            ("tpdraw 1", RetDone),
            ("tpdraw 2 second", RetDone),
            ("tpdraw 3", RetDone),
            ("tpdraw 4", RetDone),
        ],
    );
    assert!(has_shape(&ws, "tp_1") && has_shape(&ws, "second"));
    check(
        &pilot,
        &[
            ("tpdraw item 1 it", RetDone),
            ("tpdraw root 2 rt", RetDone),
            ("tpdraw root 9", RetError),
            ("tpdraw *", RetDone),
            ("tpdraw item * all", RetDone),
        ],
    );
    assert!(has_shape(&ws, "it") && has_shape(&ws, "rt") && has_shape(&ws, "all_2"));
    let (empty, _) = shape_pilot(false);
    check(&empty, &[("tpdraw 1", RetError)]);
}

#[test]
fn compound_and_access_commands() {
    let (pilot, ws) = shape_pilot(true);
    check(
        &pilot,
        &[
            ("tpcompound", RetError),
            ("tpcompound roots", RetDone),
            ("tpcompound some xst-model-all", RetDone),
        ],
    );
    assert!(has_shape(&ws, "roots") && has_shape(&ws, "some"));
    check(
        &pilot,
        &[
            ("trdraw", RetDone),
            ("trcomp", RetDone),
            ("trdraw 1 one", RetDone),
            ("trdraw 3", RetError),
            ("trdraw 9", RetError),
        ],
    );
    assert!(
        has_shape(&ws, "TREAD_LIST_1") && has_shape(&ws, "TREAD_COMP") && has_shape(&ws, "one")
    );
    let path = std::env::temp_dir().join(format!("xsfuncshape-{}.brep", std::process::id()));
    let command = format!("trscomp {}", path.display());
    check(&pilot, &[(&command, RetDone)]);
    assert!(path.exists(), "the compound is saved");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn origin_of_shapes() {
    let (pilot, ws) = shape_pilot(true);
    check(
        &pilot,
        &[
            ("fromshape", RetError),
            ("fromshape nope", RetError),
            ("trdraw 1 one", RetDone),
            ("fromshape one", RetVoid),
            ("fromshape one -1", RetVoid),
        ],
    );
    ws.vars().set_shape("other", &vertex_compound());
    check(
        &pilot,
        &[
            ("fromshape other", RetVoid),
            ("trconnexent", RetError),
            ("trconnexent nope", RetError),
            ("trconnexent one", RetDone),
        ],
    );
    let (empty, _) = shape_pilot(false);
    check(&empty, &[("trconnexent one", RetVoid)]);
}

#[test]
fn import_and_write_commands() {
    let (pilot, ws) = shape_pilot(false);
    check(
        &pilot,
        &[
            ("trimport . imp", RetError),
            ("trimport . imp *", RetDone),
            ("trimpcomp . cmp *", RetDone),
        ],
    );
    assert!(has_shape(&ws, "imp_1") && has_shape(&ws, "imp_2") && has_shape(&ws, "cmp"));
    check(
        &pilot,
        &[
            ("twrite", RetError),
            ("newmodel", RetDone),
            ("twrite cmp nope", RetDone),
        ],
    );
    assert_eq!(ws.model().nb_entities(), 1, "the writer adds one entity");
    // The fake library reads an empty model: without graph the transfer reader has no
    // model, so the import stops (as natively, " modele absent").
    check(&pilot, &[("trimport model.fake v *", RetError)]);
    assert_eq!(ws.loaded_file().as_bytes(), b"model.fake");
    assert_ne!(
        pilot
            .execute(&AsciiString::new("trimport fail.fake v *"))
            .unwrap(),
        RetDone
    );
}
