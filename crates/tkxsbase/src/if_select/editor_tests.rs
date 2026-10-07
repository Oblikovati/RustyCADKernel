//! Unit tests of the editors, the list editor and the edit forms, over a parameter
//! editor of three values: an integer "count" (short "n"), a text "title" and a
//! constant text "kind".
use super::*;
use crate::interface::test_norm::fake_chain;
use crate::interface::{InterfaceParamType, InterfaceStatic, InterfaceTypedValue};
use rustycad_tkernel::{standard::Handle, tcollection::HAsciiString};

fn text(s: &str) -> Handle<HAsciiString> {
    Handle::new(HAsciiString::new(s))
}

fn texts(items: &[&str]) -> Vec<Handle<HAsciiString>> {
    items.iter().map(|s| text(s)).collect()
}

fn bytes(h: &Handle<HAsciiString>) -> Vec<u8> {
    h.get()
        .map(|t| t.string().as_bytes().to_vec())
        .unwrap_or_default()
}

fn typed(name: &str, kind: InterfaceParamType, init: &str) -> Handle<InterfaceTypedValue> {
    Handle::new(InterfaceTypedValue::new(name, kind, init).unwrap())
}

fn editor() -> Handle<IFSelectParamEditor> {
    let editor = IFSelectParamEditor::new(3, "Params").unwrap();
    editor
        .add_value(&typed("count", InterfaceParamType::ParamInteger, "3"), "n")
        .unwrap();
    editor
        .add_value(&typed("title", InterfaceParamType::ParamText, "t"), "")
        .unwrap();
    editor.add_constant_text("const", "kind", "").unwrap();
    editor
}

#[test]
fn editor_values_and_names() {
    let editor = editor();
    assert!(
        editor
            .add_value(&typed("x", InterfaceParamType::ParamText, ""), "")
            .is_err(),
        "no room"
    );
    assert_eq!(editor.nb_values(), 3);
    assert_eq!(editor.name(1, true).unwrap().as_bytes(), b"n");
    assert_eq!(editor.name(1, false).unwrap().as_bytes(), b"count");
    assert!(editor.name(4, false).unwrap().is_empty());
    assert_eq!(
        (
            editor.name_number("n"),
            editor.name_number("title"),
            editor.name_number("2")
        ),
        (1, 2, 2)
    );
    assert_eq!(
        (editor.name_number("9"), editor.name_number("none")),
        (0, 0)
    );
    assert_eq!(editor.edit_mode(3), IFSelectEditValue::EditRead);
    assert_eq!(editor.edit_mode(0), IFSelectEditValue::EditDynamic);
    assert_eq!(
        (
            editor.max_name_length(-1),
            editor.max_name_length(0),
            editor.max_name_length(2)
        ),
        (4, 5, 0)
    );
    assert!(
        !editor.is_list(2) && editor.max_list(2) == -1 && editor.list_editor(2).unwrap().is_null()
    );
    editor.set_list(2, 4);
    assert!(
        editor.is_list(2) && editor.max_list(2) == 4 && editor.list_editor(2).unwrap().is_some()
    );
    assert!(editor.typed_value(4).is_err() && editor.set_nb_values(4).is_err());
    editor
        .set_value(9, &Handle::default(), "", IFSelectEditValue::Optional)
        .unwrap();
    assert!(
        editor
            .set_value(1, &Handle::default(), "", IFSelectEditValue::Optional)
            .is_err()
    );
}

#[test]
fn editor_prints() {
    let editor = editor();
    editor.set_list(2, 0);
    let mut out = Vec::new();
    editor.print_names(&mut out).unwrap();
    let names = String::from_utf8(out).unwrap();
    assert!(names.starts_with("****    Editor : Params\n****    Nb Values = 3    ****    Names / Labels\n Num Short Complete  Label\n"));
    assert!(names.contains("  1 n    count  "), "{names}");
    let mut out = Vec::new();
    editor.print_defs(&mut out, false).unwrap();
    let defs = String::from_utf8(out).unwrap();
    assert!(
        defs.contains("Names  /  Definitions") && defs.contains(" (List) Editable"),
        "{defs}"
    );
    assert!(defs.contains("ReadOnly "), "{defs}");
    let mut out = Vec::new();
    editor.print_defs(&mut out, true).unwrap();
    assert!(
        String::from_utf8(out)
            .unwrap()
            .contains("Labels  /  Definitions")
    );
}

#[test]
fn list_editor_edition() {
    let led =
        IFSelectListEditor::with_definition(&typed("v", InterfaceParamType::ParamText, ""), 3);
    assert!(!led.add_value(&text("x"), 0).unwrap(), "nothing loaded");
    led.load_values(Some(&texts(&["a", "b"])));
    assert!(!led.is_touched() && led.nb_values(false) == 2);
    assert!(led.set_value(1, &text("A")) && !led.set_value(3, &text("c")));
    assert!(led.add_value(&text("c"), 0).unwrap() && !led.add_value(&text("d"), 0).unwrap());
    assert!(led.is_modified(1) && led.is_added(3) && led.is_changed(3) && !led.is_changed(2));
    assert_eq!(bytes(&led.value(1, true)), b"A");
    assert_eq!(bytes(&led.value(1, false)), b"a");
    assert!(led.value(4, true).is_null() && led.is_touched());
    assert!(
        led.remove(1, 1).unwrap() && led.nb_values(true) == 2,
        "the range 1..1"
    );
    assert!(!led.remove(2, 1).unwrap(), "2 + 1 beyond the count");
    assert!(
        led.remove(0, 2).is_err(),
        "native defect: endless recursion"
    );
    led.clear_edit();
    assert_eq!(led.edited_values().map(|l| l.len()), Some(2));
    assert!(led.add_value(&text("z"), 3).unwrap() && !led.add_value(&text("y"), 9).unwrap());
}

#[test]
fn list_editor_load_edited() {
    let led = IFSelectListEditor::new();
    assert!(
        !led.load_edited(None) && !led.load_edited(Some(&texts(&["a"]))),
        "native quirk: max 0"
    );
    let bounded = IFSelectListEditor::with_definition(&Handle::default(), 2);
    assert!(bounded.load_edited(Some(&texts(&["a", "b"]))) && bounded.is_modified(2));
    assert!(bounded.original_values().is_none() && bounded.remove(-1, 1).is_ok());
    let (model, _) = fake_chain();
    let ident =
        IFSelectListEditor::with_definition(&typed("e", InterfaceParamType::ParamIdent, ""), 2);
    ident.load_model(&model);
    assert!(!ident.load_edited(Some(&texts(&["zz"]))) && ident.load_edited(Some(&texts(&["c"]))));
    ident.set_touched();
    assert!(ident.is_touched());
}

#[test]
fn edit_form_load_modify_apply() {
    let editor = editor();
    let form = editor.form(false, true).unwrap();
    assert!(form.is_complete() && form.label().as_bytes() == b"Params" && !form.is_loaded());
    assert!(form.load_data_default().unwrap() && form.is_loaded());
    assert_eq!(bytes(&form.original_value(1).unwrap()), b"3");
    assert!(form.modify(1, &text("7"), false).unwrap() && form.is_modified(1).unwrap());
    assert!(
        !form.modify(1, &text("x"), false).unwrap(),
        "not an integer"
    );
    assert!(
        form.modify(3, &text("y"), false).unwrap(),
        "native: Modify does not refuse a read only value"
    );
    assert_eq!(bytes(&form.edited_value(1).unwrap()), b"7");
    assert!(form.touch(2, &text("new")).unwrap() && form.is_touched(2).unwrap());
    assert!(form.edited_value(9).is_err(), "native: out of the arrays");
    let mut out = Vec::new();
    form.print_values(&mut out, 0, true, false).unwrap();
    let mods = String::from_utf8(out).unwrap();
    assert!(
        mods.contains("ORIG:3")
            && mods.contains(" MOD :7")
            && mods.contains("On 3 Values, 3 Modified"),
        "{mods}"
    );
    assert!(form.apply().unwrap() && !form.is_modified(1).unwrap());
    assert_eq!(editor.typed_value(1).unwrap().integer_value(), 7);
    form.modify(1, &text("9"), false).unwrap();
    assert!(form.undo().unwrap());
    assert_eq!(
        editor.typed_value(1).unwrap().integer_value(),
        3,
        "back to the original"
    );
    assert!(form.recognize().unwrap());
}

#[test]
fn edit_form_extraction() {
    let editor = editor();
    let dynamic: Handle<dyn IFSelectEditor> = editor.clone().cast();
    let form = IFSelectEditForm::with_numbers(&dynamic, &[2], false, false, "part").unwrap();
    assert!(!form.is_complete() && form.nb_values(true) == 1 && form.nb_values(false) == 3);
    assert_eq!(
        (
            form.number_from_rank(1),
            form.rank_from_number(2),
            form.rank_from_number(1)
        ),
        (2, 1, 0)
    );
    assert_eq!(
        (
            form.name_number("title").unwrap(),
            form.name_rank("title").unwrap()
        ),
        (2, 1)
    );
    assert!(
        form.name_number("count").is_err(),
        "native defect: the search runs to Length"
    );
    assert_eq!(form.name_rank("none").unwrap(), 0);
    assert!(form.load_model(&fake_chain().0).unwrap() && form.entity().is_null());
    assert_eq!(
        bytes(&form.original_value(2).unwrap()),
        b"t",
        "not undoable: from the editor"
    );
    assert!(
        form.modify(2, &text("u"), false).unwrap() && !form.modify(1, &text("4"), false).unwrap()
    );
    assert!(!form.undo().unwrap(), "not undoable");
    form.clear_edit(2);
    assert!(
        form.is_modified(2).unwrap(),
        "native quirk: the bound test reads the number 2 > 1"
    );
    form.clear_edit(0);
    assert!(!form.is_modified(2).unwrap());
    let mut out = Vec::new();
    form.print_defs(&mut out).unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "***** EditForm,  Label : part\nExtraction on 1 Values : (extracted<-editor)\n  1<-2\n*****\n"
    );
    form.set_edit_keep_status(true);
    assert!(form.edit_keep_status());
}

#[test]
fn edit_form_lists_and_data() {
    let editor = editor();
    editor.set_list(2, 2);
    let form = editor.form(false, true).unwrap();
    let (model, ents) = fake_chain();
    assert!(form.load_data(&ents[0], &model).unwrap() && form.entity() == ents[0]);
    let led = form.list_editor(2).unwrap();
    assert!(
        led.is_some() && led.original_values().is_none(),
        "the default list value is null"
    );
    assert!(
        form.modify_list_value(2, Some(&texts(&["p", "q"])), false)
            .unwrap()
    );
    assert_eq!(form.edited_list(2).unwrap().map(|l| l.len()), Some(2));
    assert!(form.original_list(2).unwrap().is_none());
    assert!(!form.modify_list(2, &Handle::default(), false).unwrap());
    assert!(form.touch_list(2, None).unwrap() && form.edited_list(2).unwrap().is_none());
    form.load_list(2, Some(&texts(&["o"])));
    assert_eq!(form.original_list(2).unwrap().map(|l| l.len()), Some(1));
    let mut out = Vec::new();
    form.print_values(&mut out, 1, false, true).unwrap();
    let values = String::from_utf8(out).unwrap();
    assert!(
        values.contains("*****  Loaded entity : #1") && values.contains("(NULL LIST)"),
        "{values}"
    );
    form.clear_data();
    assert!(!form.is_loaded() && form.model().is_null());
    assert!(form.load_entity(&ents[1]).unwrap() && form.entity() == ents[1]);
    form.load_default().unwrap();
    assert_eq!(bytes(&form.original_value(1).unwrap()), b"3");
}

#[test]
fn param_static_editor() {
    InterfaceStatic::init(
        "test",
        "test.ifselect.editor.count",
        InterfaceParamType::ParamInteger,
        "5",
    )
    .unwrap();
    assert!(
        IFSelectParamEditor::static_editor(None, "")
            .unwrap()
            .is_null()
    );
    let names = texts(&["test.ifselect.editor.count", "test.ifselect.editor.unknown"]);
    let editor = IFSelectParamEditor::static_editor(Some(&names), "Statics").unwrap();
    assert!(editor.nb_values() == 1 && editor.label().as_bytes() == b"Statics");
    let form = editor.form(false, true).unwrap();
    form.load_data_default().unwrap();
    assert!(form.modify(1, &text("8"), false).unwrap() && form.apply().unwrap());
    assert_eq!(InterfaceStatic::i_val("test.ifselect.editor.count"), 8);
}
