//! Message_Algorithm statuses, recorded data and status messages.
use super::fakes::RecordingPrinter;
use rustycad_tkernel::message::{Algorithm, ExecStatus, Gravity, Messenger, Msg, MsgFile, Status};
use rustycad_tkernel::ncollection::PackedMap;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString, HAsciiString, HExtendedString};
use std::sync::Arc;

fn recorded() -> (Algorithm, Arc<RecordingPrinter>) {
    let printer = Arc::new(RecordingPrinter::default());
    let mut algorithm = Algorithm::new();
    algorithm.set_messenger(Some(Arc::new(Messenger::with_printer(printer.clone()))));
    (algorithm, printer)
}

fn strings(algorithm: &Algorithm, status: Status) -> Vec<String> {
    let recorded = algorithm.get_message_strings(status).unwrap_or_default();
    recorded
        .iter()
        .map(|text| text.string().to_utf8())
        .collect()
}

#[test]
fn statuses_record_integers_and_strings() {
    let (mut algorithm, _) = recorded();
    algorithm.set_status_with_integer(Status::WARN1, 3);
    algorithm.set_status_with_integer(Status::WARN1, 1);
    algorithm.set_status_with_c_str(Status::FAIL2, b"edge", true);
    algorithm.set_status_with_ascii(Status::FAIL2, &AsciiString::new("edge"), true);
    algorithm.set_status_with_h_ascii(Status::FAIL2, &HAsciiString::new("face"), false);
    algorithm.set_status_with_extended(Status::FAIL2, &ExtendedString::from_utf8("face"), false);
    algorithm.set_status_with_h_extended(Status::FAIL2, &Handle::default(), true);
    algorithm.set_status_with_h_extended(
        Status::FAIL2,
        &Handle::new(HExtendedString::from_utf16(&[0x76])),
        true,
    );
    assert!(algorithm.get_status().is_warn() && algorithm.get_status().is_fail());
    assert_eq!(
        algorithm.get_message_numbers(Status::WARN1).unwrap().len(),
        2
    );
    assert_eq!(
        strings(&algorithm, Status::FAIL2),
        ["edge", "face", "face", "v"]
    );
    assert!(algorithm.get_message_numbers(Status::NONE).is_none());
    algorithm.change_status().clear_status(Status::WARN1);
    assert!(!algorithm.get_status().is_warn());
    algorithm.clear_status();
    assert!(algorithm.get_message_strings(Status::FAIL2).is_none());
}

#[test]
fn report_text_limits_its_items() {
    let mut numbers = PackedMap::<i32>::default();
    for value in [4, 1, 9] {
        numbers.add(value);
    }
    let all = Algorithm::prepare_report_numbers(&numbers, 20).to_utf8();
    assert_eq!(all.split(' ').count(), 3);
    assert!(
        Algorithm::prepare_report_numbers(&numbers, 1)
            .to_utf8()
            .ends_with(" ... (total 3)")
    );
    let texts: Vec<Handle<HExtendedString>> = ["a", "b", "c"]
        .iter()
        .map(|text| Handle::new(HExtendedString::from_c_string(text.as_bytes())))
        .collect();
    assert_eq!(
        Algorithm::prepare_report_strings(&texts, 5).to_utf8(),
        "'a', 'b', 'c'"
    );
    assert_eq!(
        Algorithm::prepare_report_strings(&texts, 2).to_utf8(),
        "'a', 'b' ... (total 3) "
    );
}

#[test]
fn status_messages_use_the_catalog_and_custom_messages() {
    let (mut algorithm, printer) = recorded();
    MsgFile::global().add_msg(
        &AsciiString::new("Message_Algorithm.Warn7"),
        &ExtendedString::from_utf8("Warn %s"),
    );
    algorithm.set_status_with_c_str(Status::WARN7, b"x", true);
    let mut custom = Msg::new();
    custom.set(&ExtendedString::from_utf8("custom %d"));
    algorithm.set_status_with_msg(Status::ALARM3, &custom);
    algorithm.set_status(Status::DONE1);
    algorithm.send_messages(Gravity::Info, 20).unwrap();
    assert_eq!(printer.texts(), ["Warn 'x'", "custom UNKNOWN"]);
    algorithm
        .send_status_messages(&ExecStatus::from_status(Status::DONE1), Gravity::Info, 20)
        .unwrap();
    assert!(printer.texts()[2].ends_with("Standard_Transient.Done1"));
}

#[test]
fn add_status_skips_the_last_string() {
    let (mut other, _) = recorded();
    other.set_status_with_integer(Status::FAIL1, 5);
    for text in [&b"one"[..], b"two", b"three"] {
        other.set_status_with_c_str(Status::FAIL1, text, true);
    }
    other.set_status(Status::DONE4);
    let (mut algorithm, _) = recorded();
    algorithm.add_status_filtered(&ExecStatus::from_status(Status::DONE4), &other);
    assert!(algorithm.get_status().is_done() && !algorithm.get_status().is_fail());
    algorithm.add_status(&other);
    assert!(
        algorithm
            .get_message_numbers(Status::FAIL1)
            .unwrap()
            .contains(5)
    );
    assert_eq!(strings(&algorithm, Status::FAIL1), ["one", "two"]);
    assert!(Arc::ptr_eq(
        &algorithm.get_messenger(),
        &algorithm.get_messenger()
    ));
    assert_eq!(algorithm.dynamic_type().name(), "Message_Algorithm");
}
