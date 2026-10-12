//! Native Aspect exception C strings retain byte encoding and declared catch ancestry.
use rustycad_tkernel::standard::Failure;
macro_rules! define_aspect_byte_exceptions {
    ($($constructor:ident => $name:literal;)*) => {
        $(
            #[doc = concat!("Constructs `", $name, "` without decoding C-string bytes; `", stringify!($constructor), "(b\"why\")`.")]
            pub fn $constructor(message: &[u8]) -> Failure {
                Failure::with_message_bytes($name, message)
                    .with_exception_parent("Standard_OutOfRange")
            }
        )*
        /// Native two-C-string constructors preserve explicit stack bytes without capture.
        pub mod stacks {
            use super::Failure;
            $(
                #[doc = concat!("Constructs `", $name, "` with explicit message/stack bytes; `", stringify!($constructor), "(b\"why\", b\"trace\")`.")]
                pub fn $constructor(message: &[u8], stack: &[u8]) -> Failure {
                    Failure::with_type_bytes($name, message, stack)
                        .with_exception_parent("Standard_OutOfRange")
                }
            )*
        }
    };
}
define_aspect_byte_exceptions! {
    aspect_fill_area_definition_error => "Aspect_AspectFillAreaDefinitionError";
    aspect_line_definition_error => "Aspect_AspectLineDefinitionError";
    aspect_marker_definition_error => "Aspect_AspectMarkerDefinitionError";
    display_connection_definition_error => "Aspect_DisplayConnectionDefinitionError";
    graphic_device_definition_error => "Aspect_GraphicDeviceDefinitionError";
    ident_definition_error => "Aspect_IdentDefinitionError";
    window_definition_error => "Aspect_WindowDefinitionError";
    window_error => "Aspect_WindowError";
}
