//! The exception classes of the Aspect package, as [`Failure`] values.
//!
//! Each `DEFINE_STANDARD_EXCEPTION(Aspect_*Error, Standard_OutOfRange)` becomes one
//! constructor; the failure keeps the most derived class name, which
//! `Failure::exception_type` reports.
use rustycad_tkernel::standard::Failure;
/// Native C-string constructors retain arbitrary bytes and explicit stack traces.
pub mod exception_bytes;

/// `Aspect_AspectFillAreaDefinitionError`; `aspect_fill_area_definition_error("bad style")`.
pub fn aspect_fill_area_definition_error(message: &str) -> Failure {
    exception_bytes::aspect_fill_area_definition_error(message.as_bytes())
}
/// `Aspect_AspectLineDefinitionError`; `aspect_line_definition_error("bad width")`.
pub fn aspect_line_definition_error(message: &str) -> Failure {
    exception_bytes::aspect_line_definition_error(message.as_bytes())
}
/// `Aspect_AspectMarkerDefinitionError`; `aspect_marker_definition_error("bad scale")`.
pub fn aspect_marker_definition_error(message: &str) -> Failure {
    exception_bytes::aspect_marker_definition_error(message.as_bytes())
}
/// `Aspect_DisplayConnectionDefinitionError`; `display_connection_definition_error("no display")`.
pub fn display_connection_definition_error(message: &str) -> Failure {
    exception_bytes::display_connection_definition_error(message.as_bytes())
}
/// `Aspect_GraphicDeviceDefinitionError`; `graphic_device_definition_error("no device")`.
pub fn graphic_device_definition_error(message: &str) -> Failure {
    exception_bytes::graphic_device_definition_error(message.as_bytes())
}
/// `Aspect_IdentDefinitionError`; `ident_definition_error("GenId Create Error: wrong interval")`.
pub fn ident_definition_error(message: &str) -> Failure {
    exception_bytes::ident_definition_error(message.as_bytes())
}
/// `Aspect_WindowDefinitionError`; `window_definition_error("bad window")`.
pub fn window_definition_error(message: &str) -> Failure {
    exception_bytes::window_definition_error(message.as_bytes())
}
/// `Aspect_WindowError`; `window_error("window is not mapped")`.
pub fn window_error(message: &str) -> Failure {
    exception_bytes::window_error(message.as_bytes())
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

#[cfg(test)]
mod ancestry_tests {
    use super::*;
    const ASPECT_CONSTRUCTORS: [fn(&str) -> Failure; 8] = [
        aspect_fill_area_definition_error,
        aspect_line_definition_error,
        aspect_marker_definition_error,
        display_connection_definition_error,
        graphic_device_definition_error,
        ident_definition_error,
        window_definition_error,
        window_error,
    ];
    const NATIVE_BASES: [&str; 4] = [
        "Standard_OutOfRange",
        "Standard_RangeError",
        "Standard_DomainError",
        "Standard_Failure",
    ];
    const BYTE_CONSTRUCTORS: [fn(&[u8]) -> Failure; 8] = [
        exception_bytes::aspect_fill_area_definition_error,
        exception_bytes::aspect_line_definition_error,
        exception_bytes::aspect_marker_definition_error,
        exception_bytes::display_connection_definition_error,
        exception_bytes::graphic_device_definition_error,
        exception_bytes::ident_definition_error,
        exception_bytes::window_definition_error,
        exception_bytes::window_error,
    ];
    type StackConstructor = fn(&[u8], &[u8]) -> Failure;
    const STACK_CONSTRUCTORS: [StackConstructor; 8] = [
        exception_bytes::stacks::aspect_fill_area_definition_error,
        exception_bytes::stacks::aspect_line_definition_error,
        exception_bytes::stacks::aspect_marker_definition_error,
        exception_bytes::stacks::display_connection_definition_error,
        exception_bytes::stacks::graphic_device_definition_error,
        exception_bytes::stacks::ident_definition_error,
        exception_bytes::stacks::window_definition_error,
        exception_bytes::stacks::window_error,
    ];
    #[test]
    fn aspect_byte_constructors_preserve_encoding_nuls_and_ancestry() {
        for (index, constructor) in BYTE_CONSTRUCTORS.iter().enumerate() {
            let failure = constructor(&[255, 0, 65]);
            assert_eq!(failure.what(), &[255]);
            assert!(failure.is_kind("Standard_OutOfRange"));
            let stack = STACK_CONSTRUCTORS[index](&[255, 0, 65], &[254, 0, 66]);
            assert_eq!(stack.what(), &[255]);
            assert_eq!(stack.stack_bytes(), &[254]);
            assert!(stack.is_kind("Standard_DomainError"));
        }
    }
    #[test]
    fn aspect_native_constructor_and_catch_rows_replay_without_occt() {
        let mut rows: Vec<String> = Vec::new();
        for (index, constructor) in ASPECT_CONSTRUCTORS.into_iter().enumerate() {
            for (mode, failure) in aspect_failure_modes(constructor, STACK_CONSTRUCTORS[index]) {
                rows.push(aspect_native_failure_row(mode, &failure));
            }
            rows.push(aspect_native_byte_row(STACK_CONSTRUCTORS[index]));
        }
        let actual = rows.join("\n") + "\n";
        assert_eq!(
            actual,
            include_str!("../../tests/fixtures/aspect-exception-ancestry-native.txt")
        );
    }
    fn aspect_catch_bits(failure: &Failure) -> String {
        NATIVE_BASES
            .iter()
            .map(|base| if failure.is_kind(base) { '1' } else { '0' })
            .collect()
    }
    fn aspect_native_byte_row(constructor: StackConstructor) -> String {
        let failure = constructor(&[255, 0, 65], &[254, 0, 66]);
        let bases = aspect_catch_bits(&failure);
        format!(
            "{}|bytes|{bases}{}|{}|{}|{}|{}",
            failure.exception_type(),
            u8::from(failure.is_kind("Standard_ProgramError")),
            failure.what()[0],
            failure.stack_bytes()[0],
            failure.what().len(),
            failure.stack_bytes().len()
        )
    }
    fn aspect_failure_modes(
        constructor: fn(&str) -> Failure,
        stacked: StackConstructor,
    ) -> [(&'static str, Failure); 3] {
        let default = constructor("");
        let message = constructor("why");
        let stack = stacked(b"why", b"trace");
        [
            ("default", default),
            ("message", message),
            ("explicit-stack", stack),
        ]
    }
    fn aspect_native_failure_row(mode: &str, failure: &Failure) -> String {
        let bases = aspect_catch_bits(failure);
        format!(
            "{}|{mode}|{bases}{}|{}|{}",
            failure.exception_type(),
            u8::from(failure.is_kind("Standard_ProgramError")),
            failure.message(),
            failure.stack_string()
        )
    }
    #[test]
    fn aspect_errors_are_caught_by_every_native_base() {
        for constructor in ASPECT_CONSTRUCTORS {
            let failure = constructor("invalid drawable -1; expected a mapped window");
            for ancestor in NATIVE_BASES {
                assert!(
                    failure.is_kind(ancestor),
                    "{} should derive from {ancestor}",
                    failure.exception_type()
                );
            }
            assert!(!failure.is_kind("Standard_ProgramError"));
        }
    }
}
