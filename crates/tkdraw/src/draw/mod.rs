//! The `Draw` package: the interpreter, variables, drawables and the base commands.
mod color;
mod console;
mod display;
mod drawable;
mod drawables;
mod interpretor;
mod istream;
mod number;
mod ostream;
mod stdio_capture;
mod view;
mod viewer;
mod window;

pub use color::{COLOR_NAMES, DrawColor, DrawColorKind, DrawMarkerShape, MAXCOLOR};
pub use console::{DrawConsole, ProcessConsole, RecordingConsole};
pub use display::{
    DRAWINFINITE, DisplayState, DisplaySurface, DrawBounds, DrawDisplay, DrawXSegment, DrawingMode,
    MAXSEGMENT, PickState, PostScriptState, ViewQuery, trim_segment,
};
pub use drawable::{Drawable3D, Drawable3DBase, DrawableHandle, downcast_handle};
pub use drawables::{
    DEFAULT_AXIS_SIZE, DrawAxis2D, DrawAxis3D, DrawBox, DrawGrid, DrawMarker2D, DrawMarker3D,
    DrawSegment2D, DrawSegment3D, DrawText2D, DrawText3D, MarkerSize,
};
pub use interpretor::{
    DrawAppend, DrawCommand, DrawInterpretor, IntegerParser, USER_COMMANDS, c_text,
};
pub use istream::DrawIStream;
pub use number::{DrawChronometer, DrawNumber};
pub use ostream::{DrawOStream, format_scientific};
pub use view::DrawView;
pub use viewer::{DrawSelection, DrawViewer, MAXVIEW};
pub use window::{BUTTON_PRESS, DrawWindow, DrawWindowEvent, DrawWindowSystem, MOTION_NOTIFY};
