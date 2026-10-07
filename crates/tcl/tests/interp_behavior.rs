//! Behavior of the safe wrapper against the real Tcl library.
use rustycad_tcl::{
    Completion, DString, Interp, InterpHandle, StaticPackage, TkWindow, TraceEvent, TraceTag,
    command_complete, external_to_utf, sys, tk_init, utf_to_unichars,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn evaluation_sets_result_and_completion() {
    let interp = Interp::new();
    assert_eq!(interp.eval("expr {2*21}").unwrap(), Completion::Ok);
    assert_eq!(interp.result(), "42");
    assert_eq!(interp.eval("error boom").unwrap(), Completion::Error);
    assert_eq!(interp.result(), "boom");
    // Tcl_Eval at level 0 turns a stray `break` into an error, as tclsh does.
    assert_eq!(interp.eval("break").unwrap(), Completion::Error);
    assert_eq!(interp.result(), "invoked \"break\" outside of a loop");
    assert!(interp.eval("a\0b").is_err());
}

#[test]
fn result_building_matches_the_c_calls() {
    let interp = Interp::new();
    interp.set_result("a").unwrap();
    interp.append_result("b c").unwrap();
    interp.append_element("d e").unwrap();
    assert_eq!(interp.result(), "ab c {d e}");
    interp.reset_result();
    assert_eq!(interp.result(), "");
}

#[test]
fn variables_and_array_list_append() {
    let interp = Interp::new();
    assert_eq!(interp.get_var("missing", 0).unwrap(), None);
    interp.set_var("v", "1", sys::TCL_GLOBAL_ONLY).unwrap();
    let flags = sys::TCL_GLOBAL_ONLY | sys::TCL_APPEND_VALUE | sys::TCL_LIST_ELEMENT;
    interp.set_var2("G", Some("grp"), "c1", flags).unwrap();
    interp.set_var2("G", Some("grp"), "c 2", flags).unwrap();
    assert_eq!(
        interp.get_var("G(grp)", 0).unwrap().as_deref(),
        Some("c1 {c 2}")
    );
    assert!(interp.unset_var("v", 0).unwrap());
    assert!(!interp.unset_var("v", 0).unwrap());
}

#[test]
fn commands_receive_words_and_can_reenter() {
    let interp = Interp::new();
    let body = |i: &InterpHandle, args: &[&str]| -> Completion {
        if args.len() > 2 {
            return i.eval("words inner").unwrap();
        }
        i.set_result(&args.join("|")).unwrap();
        Completion::Ok
    };
    interp.create_command("words", Box::new(body)).unwrap();
    interp.eval("words {a b} c").unwrap();
    assert_eq!(interp.result(), "words|inner");
    interp.eval("words x").unwrap();
    assert_eq!(interp.result(), "words|x");
}

#[test]
fn deleting_a_command_from_itself_is_safe_and_frees_the_body() {
    let interp = Interp::new();
    let witness = Rc::new(());
    let held = Rc::clone(&witness);
    let body = move |i: &InterpHandle, _: &[&str]| {
        let _keep = &held;
        assert!(i.delete_command("once").unwrap());
        i.set_result("still alive").unwrap();
        Completion::Ok
    };
    interp.create_command("once", Box::new(body)).unwrap();
    interp.eval("once").unwrap();
    assert_eq!(interp.result(), "still alive");
    assert_eq!(Rc::strong_count(&witness), 1);
    assert_eq!(interp.eval("once").unwrap(), Completion::Error);
}

#[test]
fn a_panicking_command_becomes_a_tcl_error() {
    let interp = Interp::new();
    interp
        .create_command(
            "bad",
            Box::new(|_: &InterpHandle, _: &[&str]| -> Completion { panic!("kaput") }),
        )
        .unwrap();
    assert_eq!(interp.eval("bad").unwrap(), Completion::Error);
    assert_eq!(interp.result(), "Rust panic in command \"bad\": kaput");
}

#[test]
fn a_write_trace_can_refuse_and_is_found_by_class() {
    let interp = Interp::new();
    let tag = TraceTag {
        class: "test",
        key: 99,
    };
    let refuse =
        |_: &InterpHandle, event: &TraceEvent| Some(format!("{} is protected", event.part1));
    interp.set_var("p", "p", 0).unwrap();
    assert!(
        interp
            .trace_var("p", sys::TCL_TRACE_WRITES, tag, Box::new(refuse))
            .unwrap()
    );
    assert_eq!(interp.var_trace_key("p", 0, "test").unwrap(), Some(99));
    assert_eq!(interp.var_trace_key("p", 0, "other").unwrap(), None);
    assert_eq!(interp.eval("set p 5").unwrap(), Completion::Error);
    assert_eq!(interp.result(), "can't set \"p\": p is protected");
    assert!(interp.untrace_var("p", 0, tag).unwrap());
    assert_eq!(interp.eval("set p 5").unwrap(), Completion::Ok);
}

#[test]
fn an_unset_trace_may_remove_itself_and_is_freed_once() {
    let interp = Interp::new();
    let witness = Rc::new(Cell::new(0));
    let seen = Rc::clone(&witness);
    let tag = TraceTag {
        class: "self",
        key: 1,
    };
    let body = move |i: &InterpHandle, event: &TraceEvent| {
        seen.set(seen.get() + 1);
        i.untrace_var(&event.part1, 0, tag).unwrap();
        None
    };
    interp.set_var("u", "1", 0).unwrap();
    interp
        .trace_var(
            "u",
            sys::TCL_TRACE_UNSETS | sys::TCL_TRACE_WRITES,
            tag,
            Box::new(body),
        )
        .unwrap();
    interp.eval("unset u").unwrap();
    assert_eq!(witness.get(), 1);
    assert_eq!(Rc::strong_count(&witness), 1);
}

#[test]
fn deleting_the_interpreter_releases_traces_and_commands() {
    let witness = Rc::new(());
    {
        let interp = Interp::new();
        let (for_trace, for_command) = (Rc::clone(&witness), Rc::clone(&witness));
        let trace = move |_: &InterpHandle, _: &TraceEvent| {
            let _keep = &for_trace;
            None
        };
        let tag = TraceTag {
            class: "life",
            key: 2,
        };
        interp
            .trace_var("t", sys::TCL_TRACE_WRITES, tag, Box::new(trace))
            .unwrap();
        interp
            .create_command(
                "c",
                Box::new(move |_: &InterpHandle, _: &[&str]| {
                    let _keep = &for_command;
                    Completion::Ok
                }),
            )
            .unwrap();
        assert_eq!(Rc::strong_count(&witness), 3);
    }
    assert_eq!(Rc::strong_count(&witness), 1);
}

#[test]
fn unrequested_unset_events_do_not_reach_the_body() {
    let interp = Interp::new();
    let calls = Rc::new(Cell::new(0));
    let counter = Rc::clone(&calls);
    let body = move |_: &InterpHandle, _: &TraceEvent| {
        counter.set(counter.get() + 1);
        None
    };
    interp
        .trace_var(
            "w",
            sys::TCL_TRACE_WRITES,
            TraceTag { class: "w", key: 0 },
            Box::new(body),
        )
        .unwrap();
    interp.eval("set w 1; unset w").unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(Rc::strong_count(&calls), 1);
}

#[test]
fn dynamic_strings_and_encodings() {
    let mut text = DString::new();
    external_to_utf(b"abc", &mut text).unwrap();
    text.append(b"def").unwrap();
    assert_eq!(text.to_text(), "abcdef");
    text.clear();
    assert_eq!(text.value(), b"");
    assert_eq!(utf_to_unichars("A\u{20AC}").unwrap(), vec![0x41, 0x20AC]);
}

#[test]
fn completeness_of_scripts() {
    assert!(command_complete("set a {b}"));
    assert!(!command_complete("set a {b"));
    assert!(!command_complete("set a \"b"));
    assert!(command_complete("set a b\0{"));
}

struct Probe;
impl StaticPackage for Probe {
    fn init(interp: &InterpHandle) -> Completion {
        interp
            .set_var("probe_loaded", "yes", sys::TCL_GLOBAL_ONLY)
            .unwrap();
        Completion::Ok
    }
}

#[test]
fn static_packages_load_through_load() {
    let interp = Interp::new();
    interp.static_package::<Probe>("Probe").unwrap();
    // Tcl marks the package as loaded in `interp` itself without calling the
    // initializer; loading it into a child interpreter calls it.
    assert_eq!(interp.get_var("probe_loaded", 0).unwrap(), None);
    let script = "interp create child; load {} Probe child; child eval {set probe_loaded}";
    assert_eq!(
        interp.eval(script).unwrap(),
        Completion::Ok,
        "{}",
        interp.result()
    );
    assert_eq!(interp.result(), "yes");
}

#[test]
fn tk_main_window_is_absent_before_tk_init() {
    let interp = Interp::new();
    assert!(TkWindow::main(&interp).is_none());
}

/// Needs an X display; without one, Tk's refusal is checked instead.
#[test]
fn tk_init_opens_the_main_window_or_names_the_missing_display() {
    let interp = Interp::new();
    // Tk_Init runs tk.tcl, which needs Tcl's own init.tcl (tcl_findLibrary), as in DRAW.
    assert_eq!(interp.init(), Completion::Ok, "{}", interp.result());
    let app = match tk_init(&interp) {
        Ok(app) => app,
        Err(reason) => {
            assert!(
                std::env::var_os("DISPLAY").is_none(),
                "Tk_Init failed with a display: {reason}"
            );
            assert!(reason.contains("display"), "{reason}");
            return;
        }
    };
    interp.eval("wm withdraw .").unwrap();
    let window = app.main_window().expect("Tk_Init creates the main window");
    let name = window.set_app_name("RustyCadTclTest").unwrap();
    assert!(name.starts_with("RustyCadTclTest"), "{name}");
    window.set_name_uid(app.get_uid(&name).unwrap());
    assert_eq!(window.name(), name);
    assert!(std::ptr::eq(
        app.get_uid("x").unwrap(),
        app.get_uid("x").unwrap()
    ));
    assert!(app.main_window_count() >= 1);
    assert!(!window.display().is_null());
    window.geometry_request(200, 200);
    interp.eval("destroy .").unwrap();
}
