//! Unit tests of the basic commands (IFSelect_Functions) run by the shared session
//! pilot on the fake chain (a, b, c, d: d shares c, c shares a and b).
use super::pilot_tests::shared_pilot;
use super::*;
use crate::interface::{InterfaceParamType, InterfaceTypedValue};
use rustycad_tkernel::{
    standard::{Handle, StandardTransient},
    tcollection::AsciiString,
};

use IFSelectReturnStatus::{RetDone, RetError, RetFail, RetVoid};

fn run(pilot: &Handle<IFSelectSessionPilot>, command: &str) -> IFSelectReturnStatus {
    IFSelectFunctions::init().unwrap();
    pilot.execute(&AsciiString::new(command)).unwrap()
}

fn check(pilot: &Handle<IFSelectSessionPilot>, commands: &[(&str, IFSelectReturnStatus)]) {
    for (command, expected) in commands {
        assert_eq!(run(pilot, command), *expected, "{command}");
    }
}

#[test]
fn functions_general_commands() {
    let (_guard, pilot) = shared_pilot();
    let ws = pilot.session();
    check(
        &pilot,
        &[
            ("xstatus", RetVoid),
            ("handler", RetDone),
            ("handler", RetDone),
            ("elabel", RetError),
            ("elabel 2", RetVoid),
            ("elabel 9", RetError),
            ("enum", RetError),
            ("enum c", RetVoid),
            ("listtypes", RetVoid),
            ("count", RetError),
            ("count zz", RetError),
            ("estatus", RetVoid),
            ("estatus 3", RetVoid),
            ("estatus 9", RetError),
            ("data ?", RetVoid),
            ("data g", RetVoid),
            ("data e", RetVoid),
            ("data q", RetError),
            ("entity 1", RetError),
            ("entity 1 0", RetVoid),
            ("entity 0", RetError),
            ("queryparent 4 1", RetVoid),
            ("queryparent", RetError),
            ("dumpshare", RetVoid),
            ("listitems", RetVoid),
            ("writeall", RetError),
            ("writeall all.stp", RetDone),
            ("writesel sel.stp (1,2)", RetDone),
            ("writeent ent.stp 1 2", RetDone),
            ("writeent ent.stp zz", RetError),
            ("xload", RetError),
        ],
    );
    assert!(ws.error_handle(), "toggled twice");
    assert_eq!(run(&pilot, "xnew n integer 5"), RetDone);
    assert_eq!(ws.named_item("n").cast::<IFSelectIntParam>().value(), 5);
    check(
        &pilot,
        &[
            ("setint n 7", RetDone),
            ("setint", RetError),
            ("setint t 1", RetFail),
        ],
    );
    assert_eq!(ws.named_item("n").cast::<IFSelectIntParam>().value(), 7);
    check(
        &pilot,
        &[
            ("xnew t text hello", RetDone),
            ("settext t world", RetDone),
            ("settext n x", RetFail),
        ],
    );
    assert_eq!(
        ws.text_value(&ws.named_item("t").cast()).as_bytes(),
        b"world"
    );
    check(
        &pilot,
        &[
            ("xload new.stp", RetDone),
            ("sentfiles", RetVoid),
            ("xload fail", RetError),
        ],
    );
}

#[test]
fn functions_signatures_and_counters() {
    let (_guard, pilot) = shared_pilot();
    let ws = pilot.session();
    let sign: Handle<StandardTransient> = Handle::new(IFSelectSignType::new(false)).cast();
    ws.add_named_item("type", &sign, true).unwrap();
    check(
        &pilot,
        &[
            ("signtype", RetVoid),
            ("signtype type", RetDone),
            ("signtype", RetVoid),
            ("signtype zz", RetError),
            ("signtype .", RetDone),
            ("signcase type", RetVoid),
            ("signcase zz", RetVoid),
            ("signature type 1", RetVoid),
            ("signature zz 1", RetError),
            ("count type", RetVoid),
            ("listcount type", RetVoid),
            ("sumcount type (1,2)", RetVoid),
            ("xnew cnt signcounter type", RetDone),
            ("count cnt", RetVoid),
            ("signcounter zz", RetError),
            ("xnew sel typexact Fake_Entity", RetDone),
            ("xnew sel2 typecontain Entity", RetDone),
            ("xnew ss signsel type Fake_Entity c", RetDone),
            ("signsel zz x", RetError),
            ("givecount sel", RetVoid),
            ("givecount", RetError),
        ],
    );
    assert!(ws.sign_type().is_null());
    assert_eq!(
        ws.selection_result(&ws.named_item("sel").cast())
            .unwrap()
            .map(|l| l.len()),
        Some(0),
        "a signature selection created alone has no input"
    );
}

#[test]
fn functions_lists_and_selections() {
    let (_guard, pilot) = shared_pilot();
    let ws = pilot.session();
    check(
        &pilot,
        &[
            ("xnew roots modelroots", RetDone),
            ("xnew all modelall", RetDone),
            ("xnew sh shared", RetDone),
            ("xnew shg sharing", RetDone),
            ("xnew lr roots", RetDone),
            ("input sh roots", RetDone),
            ("input sh zz", RetError),
            ("input roots all", RetFail),
            ("evalsel roots", RetVoid),
            ("givelist (1,2)", RetVoid),
            ("giveshort all", RetVoid),
            ("givepointed all", RetVoid),
            ("makelist lst (1,2)", RetVoid),
            ("makelist all (1)", RetFail),
            ("setlist lst", RetDone),
            ("setlist lst +3 -1 /2 x", RetDone),
            ("setlist zz", RetError),
            ("xnew pt pointed (1)", RetDone),
            ("xnew su selsuite sh", RetDone),
            ("dumpsel roots", RetVoid),
            ("dumpsel", RetError),
            ("xnew u union", RetDone),
            ("seladd u roots", RetDone),
            ("selrem u roots", RetDone),
            ("seladd roots u", RetFail),
            ("xnew it intersect", RetDone),
            ("xnew df diff all roots", RetDone),
            ("selmain df all", RetDone),
            ("selsecond df roots", RetDone),
            ("selmain roots all", RetFail),
            ("xnew rg range ?", RetVoid),
            ("integer 1", RetDone),
            ("xnew one integer 2", RetDone),
            ("xnew rg range one", RetDone),
            ("xnew rg2 range from one", RetDone),
            ("range until", RetError),
            ("toggle rg", RetDone),
            ("toggle roots", RetFail),
            ("xnew en number one", RetDone),
            ("xnew err errors", RetDone),
            ("xnew unk unknown", RetDone),
            ("xnew inc incorrect", RetDone),
            ("xnew nbs nbselected sh", RetDone),
            ("nbselected roots", RetError),
            ("itemlabel Selection", RetVoid),
            ("xremove pt", RetDone),
            ("xremove pt", RetFail),
            ("clearitems", RetDone),
        ],
    );
    assert_eq!(ws.max_ident(), 0, "cleared");
    check(
        &pilot,
        &[
            ("cleardata", RetVoid),
            ("cleardata ?", RetError),
            ("cleardata c", RetDone),
        ],
    );
}

#[test]
fn functions_dispatches_and_files() {
    let (_guard, pilot) = shared_pilot();
    let ws = pilot.session();
    ws.set_mode_stat(true);
    ws.compute_graph(true).unwrap();
    check(
        &pilot,
        &[
            ("xnew all modelall", RetDone),
            ("xnew dg dispglob", RetDone),
            ("dispsel dg all", RetDone),
            ("dispsel dg zz", RetError),
            ("dumpdisp dg", RetVoid),
            ("dumpdisp zz", RetError),
            ("fileprefix", RetVoid),
            ("fileprefix out_", RetDone),
            ("fileext .stp", RetDone),
            ("fileext", RetVoid),
            ("fileroot dg", RetVoid),
            ("fileroot dg part", RetDone),
            ("filedef", RetVoid),
            ("filedef def", RetDone),
            ("evalfile", RetDone),
            ("clearfile", RetDone),
            ("evaldisp 3 dg", RetVoid),
            ("evaldisp 0 zz", RetError),
            ("evalcomplete", RetVoid),
            ("evalcomplete 2", RetVoid),
            ("xsplit dg", RetDone),
            ("xsplit zz", RetError),
            ("remaining", RetDone),
            ("remaining l", RetDone),
            ("remaining f", RetDone),
            ("remaining q", RetError),
            ("runcheck", RetVoid),
            ("xnew c one integer 2", RetError),
            ("xnew two integer 2", RetDone),
            ("xnew dc dispcount two", RetDone),
            ("xnew df dispfile two", RetDone),
            ("dispcount zz", RetError),
            ("xnew d1 dispone", RetDone),
            ("evaladisp 1 dc", RetError),
            ("evaladisp 12 dc", RetError),
            ("evaladisp 0 dc(3) (1,2)", RetVoid),
            ("writedisp out/w.stp dc (1,2)", RetDone),
            ("writedisp noext dc", RetError),
            ("setcontent all k", RetDone),
            ("setcontent all q", RetError),
            ("remaining u", RetDone),
        ],
    );
    let dc = ws.named_item("dc").cast::<IFSelectDispPerCount>();
    assert_eq!(dc.count().value(), 3, "evaladisp dc(3) sets the count");
}

#[test]
fn functions_modifiers_and_transformers() {
    let (_guard, pilot) = shared_pilot();
    let ws = pilot.session();
    check(
        &pilot,
        &[
            ("xnew reo reorder f", RetDone),
            ("reorder q", RetError),
            ("xnew all modelall", RetDone),
            ("modifsel reo all", RetDone),
            ("modifsel reo", RetDone),
            ("modifsel zz", RetError),
            ("setapplied reo", RetDone),
            ("dumpmodif reo", RetVoid),
            ("dumpmodif zz", RetVoid),
            ("listmodif", RetVoid),
            ("modifmove M 1 1", RetDone),
            ("modifmove Q 1 1", RetError),
            ("resetapplied reo", RetDone),
            ("resetapplied zz", RetError),
            ("xnew cp copy", RetDone),
            ("xnew os onthespot", RetDone),
            ("setapplied reo os", RetDone),
            ("runtranformer os", RetDone),
            ("runtranformer zz", RetFail),
            ("runcopy reo", RetDone),
            ("runonthespot reo (1,2)", RetDone),
            ("runcopy zz", RetError),
        ],
    );
    assert!(ws.has_model());
}

#[test]
fn functions_edit_forms() {
    let (_guard, pilot) = shared_pilot();
    let ws = pilot.session();
    let count: Handle<StandardTransient> = Handle::new(
        InterfaceTypedValue::new("count", InterfaceParamType::ParamInteger, "1").unwrap(),
    )
    .cast();
    ws.set_params(&[count], &[1]).unwrap();
    check(
        &pilot,
        &[
            ("editlist xst-params-all", RetVoid),
            ("editlist xst-params-all o", RetVoid),
            ("editlist", RetError),
            ("editload xst-params-all", RetDone),
            ("editload xst-params-all 2", RetDone),
            ("editload zz", RetError),
            ("editvalue xst-params-all count", RetVoid),
            ("editvalue xst-params-all count 4", RetDone),
            ("editvalue xst-params-all zz 4", RetError),
            ("editvalue xst-params-all count x", RetFail),
            ("editclear xst-params-all count", RetDone),
            ("editclear xst-params-all count", RetVoid),
            ("editvalue xst-params-all count 6", RetDone),
            ("editapply xst-params-all keep", RetDone),
            ("editapply xst-params-all", RetDone),
            ("editclear xst-params-all", RetDone),
            ("editapply zz", RetError),
            ("param", RetVoid),
            ("param -p test", RetVoid),
            ("param 4", RetVoid),
            ("param zz.none", RetError),
        ],
    );
    let edit = ws
        .named_item("xst-params-edit")
        .cast::<IFSelectParamEditor>();
    assert_eq!(edit.typed_value(1).unwrap().integer_value(), 6);
}

#[test]
fn functions_helpers() {
    let (_guard, pilot) = shared_pilot();
    let ws = pilot.session();
    assert_eq!(IFSelectFunctions::give_entity_number(&ws, "3").unwrap(), 3);
    assert!(IFSelectFunctions::give_entity(&ws, "2").unwrap() == ws.starting_entity(2));
    assert!(IFSelectFunctions::give_entity(&ws, "zz").unwrap().is_null());
    assert_eq!(
        IFSelectFunctions::give_list(&ws, "(1,2)", "")
            .unwrap()
            .map(|l| l.len()),
        Some(2)
    );
    assert!(
        IFSelectFunctions::give_dispatch(&ws, "zz", true).is_err(),
        "native defect: no parenthesis"
    );
    assert!(
        IFSelectFunctions::give_dispatch(&ws, "zz(2)", true)
            .unwrap()
            .is_null()
    );
    let dc: Handle<StandardTransient> = Handle::new(IFSelectDispPerCount::new()).cast();
    ws.add_named_item("dc", &dc, false).unwrap();
    assert!(
        IFSelectFunctions::give_dispatch(&ws, "dc(0)", true)
            .unwrap()
            .is_null(),
        "count not positive"
    );
    assert!(
        IFSelectFunctions::give_dispatch(&ws, "dc(4)", false)
            .unwrap()
            .is_some()
    );
    assert!(IFSelectFunctions::give_entity_number(&Handle::default(), "1").is_err());
}
