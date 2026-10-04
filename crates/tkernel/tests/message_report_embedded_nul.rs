//! OCCT 3d097a0 passes message keys to ostream as C strings, including nested alerts.
use rustycad_tkernel::message::{AlertExtended, AlertHandle, BaseAttribute, Gravity, Report};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Arc;

fn named_alert(bytes: &[u8]) -> Arc<AlertExtended> {
    let alert = Arc::new(AlertExtended::new());
    alert.set_attribute(Some(Arc::new(BaseAttribute::new(&AsciiString::from_raw(
        bytes,
    )))));
    alert
}

#[test]
fn report_dump_truncates_each_key_at_its_first_nul() {
    let report = Report::new();
    let parent = named_alert(b"parent\0hidden");
    parent.composite_alerts(true).unwrap().add_alert(
        Gravity::Info,
        &(named_alert(b"child\0hidden") as AlertHandle),
    );
    report.add_alert(Gravity::Info, &(parent as AlertHandle));
    report.add_alert(Gravity::Info, &(named_alert(b"\0hidden") as AlertHandle));
    report.add_alert(
        Gravity::Warning,
        &(named_alert(b"warning\0hidden") as AlertHandle),
    );
    let mut output = Vec::new();
    report.dump(&mut output).unwrap();
    assert_eq!(
        output,
        include_bytes!("../../../tools/oracle/tkernel_report_embedded_nul.expected")
    );
}

#[test]
fn gravity_dump_uses_the_same_c_string_key_boundary() {
    let report = Report::new();
    report.add_alert(
        Gravity::Warning,
        &(named_alert(b"warning\0hidden") as AlertHandle),
    );
    let mut output = Vec::new();
    report.dump_gravity(&mut output, Gravity::Warning).unwrap();
    assert_eq!(output, b"warning\n");
}
