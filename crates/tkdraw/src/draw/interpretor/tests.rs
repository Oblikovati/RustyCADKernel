use super::*;
use crate::draw::RecordingConsole;

fn interpretor() -> (DrawInterpretor, Rc<RecordingConsole>) {
    let console = Rc::new(RecordingConsole::default());
    let di = DrawInterpretor::new(console.clone());
    di.init();
    (di, console)
}

#[test]
fn operations_need_an_interpreter() {
    let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
    let error = di.result().unwrap_err();
    assert_eq!(
        error.to_string(),
        "Standard_DomainError: No call for  Draw_Interpretor::Init()"
    );
    let add = di.add(
        "x",
        "",
        Box::new(|_: &DrawInterpretor, _: &[&str]| Ok(0)),
        "g",
    );
    assert_eq!(
        add.unwrap_err().to_string(),
        "Standard_ProgramError: Attempt to add command to Null interpreter"
    );
}

#[test]
fn appends_format_like_the_cpp_overloads() {
    let (di, _) = interpretor();
    di.append("a")
        .unwrap()
        .append(2)
        .unwrap()
        .append(0.5)
        .unwrap()
        .append(7usize)
        .unwrap();
    assert_eq!(di.result().unwrap(), "a20.57");
    di.reset().unwrap();
    di.append("cut\0here").unwrap();
    di.append_element("x y").unwrap();
    assert_eq!(di.result().unwrap(), "cut {x y}");
}

#[test]
fn add_records_help_group_and_file() {
    let (di, _) = interpretor();
    let ok = |_: &DrawInterpretor, _: &[&str]| Ok(0);
    di.add_with_file("c1", "help one", "/x/a/b/c/d/f.cxx", Box::new(ok), "G")
        .unwrap();
    di.add("c2", "help two", Box::new(ok), "G").unwrap();
    di.eval(
        "set r [list $Draw_Helps(c1) $Draw_Groups(G) $Draw_Files(c1) [info exists Draw_Files(c2)]]",
    )
    .unwrap();
    assert_eq!(di.result().unwrap(), "{help one} {c1 c2} a/b/c/d/f.cxx 0");
    assert!(di.remove("c1").unwrap());
    assert!(!di.remove("c1").unwrap());
}

#[test]
fn command_return_values_map_to_tcl_codes() {
    let (di, _) = interpretor();
    di.add(
        "ret",
        "",
        Box::new(|_: &DrawInterpretor, a: &[&str]| Ok(a[1].parse().unwrap())),
        "G",
    )
    .unwrap();
    assert_eq!(di.eval("ret 0").unwrap(), 0);
    assert_eq!(di.eval("ret 5").unwrap(), 1);
}

#[test]
fn exceptions_and_panics_become_the_result() {
    let (di, _) = interpretor();
    let failing = |_: &DrawInterpretor, _: &[&str]| -> Result<i32, Failure> {
        Err(Failure::with_message("Standard_DomainError", "bad input"))
    };
    di.add("fail", "", Box::new(failing), "G").unwrap();
    assert_eq!(di.eval("fail").unwrap(), 1);
    assert_eq!(
        di.result().unwrap(),
        "An exception was caught Standard_DomainError: bad input"
    );
    let panicking = |_: &DrawInterpretor, _: &[&str]| -> Result<i32, Failure> { panic!("boom") };
    di.add("crash", "", Box::new(panicking), "G").unwrap();
    assert_eq!(di.eval("crash").unwrap(), 1);
    assert_eq!(
        di.result().unwrap(),
        "An exception was caught boom [rust panic]"
    );
}

#[test]
fn echo_prints_the_words_and_the_result() {
    let (di, console) = interpretor();
    di.add(
        "say",
        "",
        Box::new(|di: &DrawInterpretor, a: &[&str]| {
            di.append(a[1])?;
            Ok(0)
        }),
        "G",
    )
    .unwrap();
    di.set_do_echo(true);
    di.eval("say hello").unwrap();
    assert_eq!(console.take_output(), "say hello \nhello\n");
    di.add(
        "decho",
        "",
        Box::new(|_: &DrawInterpretor, _: &[&str]| Ok(0)),
        "G",
    )
    .unwrap();
    di.eval("decho").unwrap();
    assert_eq!(console.take_output(), "");
}

#[test]
fn help_and_completeness() {
    let (di, _) = interpretor();
    di.eval("proc help {name} {return \"help for $name\"}")
        .unwrap();
    assert_eq!(di.print_help("dset").unwrap(), 0);
    assert_eq!(di.result().unwrap(), "help for dset");
    assert!(DrawInterpretor::complete("a {b}"));
    assert!(!DrawInterpretor::complete("a {b"));
}

#[test]
fn exit_on_catch_reports_and_evaluates_exit() {
    let (di, _) = interpretor();
    di.eval("set exited 0; rename exit real_exit; proc exit {} {set ::exited 1}")
        .unwrap();
    di.eval("set Draw_ExitOnCatch 1").unwrap();
    let failing =
        |_: &DrawInterpretor, _: &[&str]| -> Result<i32, Failure> { Err(Failure::new("x")) };
    di.add("fail", "", Box::new(failing), "G").unwrap();
    di.eval("fail").unwrap();
    di.eval("set exited").unwrap();
    assert_eq!(di.result().unwrap(), "1");
}
