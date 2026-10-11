# Martensite fallible window-listener integration

`AisViewController` implements the complete `AspectWindowInputListener` interface.
Window, keyboard, mouse, touch and SpaceMouse callbacks delegate to its native
controller implementations. Existing native empty window hooks remain empty;
camera/render application and XR remain separate open work.

Mouse buttons/position and touch callbacks now return `Result<_, Failure>`.
The press/release helpers preserve the same result. Native C++ exceptions from
animation callbacks become typed Rust failures without conversion to false or
ignored errors. X11 dispatch propagates failures before calling `process_input`.
Scroll remains an infallible boolean because its native controller path cannot
invoke the fallible animation callback. Existing listener implementations must
wrap successful mouse/touch results in `Ok` and propagate failures with `?`.

The controller uses the blanket `MartensiteSpaceMouseInput` adapter rather than
a duplicate keyboard implementation. Inherited listener accessors and settings
share its existing base state. This integration completes no additional upstream
API declaration: it makes previously verified implementations usable through the
full window callback interface. The class and constructor remain partial.

Run `cargo test -p rustycad-tkservice -p rustycad-tkv3d --all-targets --locked`.
The 1,220 mouse, 254 touch and 288 SpaceMouse native snapshots exercise trait
routing. Dynamic listener regressions cover window notifications, keyboard/focus,
mouse helpers, touch lifecycle, HID routing and animation failures. Local X11
regressions verify button/release/motion failures stop before input processing;
these X-server tests skip when no display is available. Windows/macOS execution
is deferred; native OCCT remains a local oracle only.
