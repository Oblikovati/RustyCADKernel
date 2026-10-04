//! Named alert fixtures shared by the report callback and protected-hook tests.
use rustycad_tkernel::message::{AlertExtended, AlertHandle, BaseAttribute};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Arc;

/// Builds an attributed fixture; for example, `extended_alert("parent")`.
pub fn extended_alert(name: &str) -> Arc<AlertExtended> {
    let alert = Arc::new(AlertExtended::new());
    alert.set_attribute(Some(Arc::new(BaseAttribute::new(&AsciiString::new(name)))));
    alert
}

/// Erases the fixture type for list insertion; for example, `named_alert("child")`.
pub fn named_alert(name: &str) -> AlertHandle {
    extended_alert(name)
}
