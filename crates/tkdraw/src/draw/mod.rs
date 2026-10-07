//! The `Draw` package: the interpreter, variables, drawables and the base commands.
mod color;
mod console;
mod display;
mod interpretor;
mod ostream;
mod stdio_capture;

pub use color::{COLOR_NAMES, DrawColor, DrawColorKind, DrawMarkerShape, MAXCOLOR};
pub use console::{DrawConsole, ProcessConsole, RecordingConsole};
pub use display::{
    DRAWINFINITE, DisplayState, DisplaySurface, DrawBounds, DrawDisplay, DrawXSegment, DrawingMode,
    MAXSEGMENT, PickState, PostScriptState, trim_segment,
};
pub use interpretor::{
    DrawAppend, DrawCommand, DrawInterpretor, IntegerParser, USER_COMMANDS, c_text,
};
pub use ostream::{DrawOStream, format_scientific};
