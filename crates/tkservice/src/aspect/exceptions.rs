//! The exception classes of the Aspect package, as [`Failure`] values.
//!
//! Each `DEFINE_STANDARD_EXCEPTION(Aspect_*Error, Standard_OutOfRange)` becomes one
//! constructor; the failure keeps the most derived class name, which
//! `Failure::exception_type` reports.
use rustycad_tkernel::standard::Failure;

/// `Aspect_AspectFillAreaDefinitionError`; `aspect_fill_area_definition_error("bad style")`.
pub fn aspect_fill_area_definition_error(message: &str) -> Failure {
    Failure::with_type("Aspect_AspectFillAreaDefinitionError", message, "")
}
/// `Aspect_AspectLineDefinitionError`; `aspect_line_definition_error("bad width")`.
pub fn aspect_line_definition_error(message: &str) -> Failure {
    Failure::with_type("Aspect_AspectLineDefinitionError", message, "")
}
/// `Aspect_AspectMarkerDefinitionError`; `aspect_marker_definition_error("bad scale")`.
pub fn aspect_marker_definition_error(message: &str) -> Failure {
    Failure::with_type("Aspect_AspectMarkerDefinitionError", message, "")
}
/// `Aspect_DisplayConnectionDefinitionError`; `display_connection_definition_error("no display")`.
pub fn display_connection_definition_error(message: &str) -> Failure {
    Failure::with_type("Aspect_DisplayConnectionDefinitionError", message, "")
}
/// `Aspect_GraphicDeviceDefinitionError`; `graphic_device_definition_error("no device")`.
pub fn graphic_device_definition_error(message: &str) -> Failure {
    Failure::with_type("Aspect_GraphicDeviceDefinitionError", message, "")
}
/// `Aspect_IdentDefinitionError`; `ident_definition_error("GenId Create Error: wrong interval")`.
pub fn ident_definition_error(message: &str) -> Failure {
    Failure::with_type("Aspect_IdentDefinitionError", message, "")
}
/// `Aspect_WindowDefinitionError`; `window_definition_error("bad window")`.
pub fn window_definition_error(message: &str) -> Failure {
    Failure::with_type("Aspect_WindowDefinitionError", message, "")
}
/// `Aspect_WindowError`; `window_error("window is not mapped")`.
pub fn window_error(message: &str) -> Failure {
    Failure::with_type("Aspect_WindowError", message, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One exception constructor and the class name it must report.
    type ExceptionCase = (fn(&str) -> Failure, &'static str);

    #[test]
    fn every_constructor_keeps_its_class_name_and_message() {
        let cases: [ExceptionCase; 8] = [
            (
                aspect_fill_area_definition_error,
                "Aspect_AspectFillAreaDefinitionError",
            ),
            (
                aspect_line_definition_error,
                "Aspect_AspectLineDefinitionError",
            ),
            (
                aspect_marker_definition_error,
                "Aspect_AspectMarkerDefinitionError",
            ),
            (
                display_connection_definition_error,
                "Aspect_DisplayConnectionDefinitionError",
            ),
            (
                graphic_device_definition_error,
                "Aspect_GraphicDeviceDefinitionError",
            ),
            (ident_definition_error, "Aspect_IdentDefinitionError"),
            (window_definition_error, "Aspect_WindowDefinitionError"),
            (window_error, "Aspect_WindowError"),
        ];
        for (constructor, name) in cases {
            let failure = constructor("why");
            assert_eq!(failure.exception_type(), name);
            assert_eq!(failure.message(), "why");
        }
    }
}
