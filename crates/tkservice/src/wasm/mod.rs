//! Wasm: the Emscripten canvas window (OCCT package Wasm) as the Linux library builds it,
//! without Emscripten (visualization-design.md section 4): the window keeps its size and
//! mapping, and the event translators answer as their non-Emscripten bodies do.
mod window;

pub use window::{
    EmscriptenFocusEvent, EmscriptenKeyboardEvent, EmscriptenMouseEvent, EmscriptenTouchEvent,
    EmscriptenUiEvent, EmscriptenWheelEvent, WasmWindow, WasmWindowFields,
};
