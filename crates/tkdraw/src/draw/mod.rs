//! The `Draw` package: the interpreter, variables, drawables and the base commands.
mod appli;
mod arguments;
mod color;
mod commands;
mod console;
mod display;
mod drawable;
mod drawables;
mod expression;
#[cfg(test)]
pub(crate) mod fake_window;
mod interpretor;
mod interpretor_link;
mod istream;
mod number;
mod os_limits;
mod ostream;
mod plugins;
mod printer;
mod progress;
mod session;
mod shared_buffer;
mod stdio_capture;
mod variables;
mod view;
mod viewer;
mod window;

pub use appli::{AppliOptions, InitAppli, draw_appli, draw_main, interprete};
pub use arguments::{parse_on_off, parse_on_off_iterator, parse_on_off_no_iterator};
pub use color::{COLOR_NAMES, DrawColor, DrawColorKind, DrawMarkerShape, MAXCOLOR};
pub use commands::{
    basic_commands, commands, graphic_commands, message_commands, pload_commands, unit_commands,
    variable_commands,
};
pub use console::{DrawConsole, ProcessConsole, RecordingConsole};
pub use display::{
    DRAWINFINITE, DisplayState, DisplaySurface, DrawBounds, DrawDisplay, DrawXSegment, DrawingMode,
    MAXSEGMENT, PickState, PostScriptState, ViewQuery, trim_segment,
};
pub use drawable::{Drawable3D, Drawable3DBase, DrawableHandle, downcast_handle, kind_of};
pub use drawables::{
    DEFAULT_AXIS_SIZE, DrawAxis2D, DrawAxis3D, DrawBox, DrawGrid, DrawMarker2D, DrawMarker3D,
    DrawSegment2D, DrawSegment3D, DrawText2D, DrawText3D, MarkerSize,
};
pub use expression::{ExpressionContext, evaluate};
pub use interpretor::{
    DrawAppend, DrawCommand, DrawInterpretor, IntegerParser, USER_COMMANDS, c_text,
};
pub use interpretor_link::InterpretorLink;
pub use istream::DrawIStream;
pub use number::{DrawChronometer, DrawNumber};
pub use ostream::{DrawOStream, format_scientific};
pub use plugins::{DrawPluginFactory, DrawPluginLibraries, draw_failure};
pub use printer::{DrawPrinter, DrawPrinterType};
pub use progress::{
    DrawProgressIndicator, default_console_mode, default_graph_mode, default_tcl_mode,
    set_default_console_mode, set_default_graph_mode, set_default_tcl_mode, set_stop_indicator,
    stop_indicator,
};
pub use session::{DrawSession, DrawableFactory, LastPick, SessionCommand};
pub use view::DrawView;
pub use viewer::{DrawSelection, DrawViewer, MAXVIEW};
pub use window::{BUTTON_PRESS, DrawWindow, DrawWindowEvent, DrawWindowSystem, MOTION_NOTIFY};
