# DRAW (TKDraw) design

The Rust port of the OCCT DRAW test harness: `crates/tcl` (`rustycad-tcl`, the Tcl/Tk
binding) and `crates/tkdraw` (`rustycad-tkdraw`, packages Draw, DBRep, DrawTrSurf, and
the `drawexe` binary). This document says how the C++ structure maps to Rust, how the
window layer works, how the port is checked against native DRAWEXE, and what still
waits for other toolkits.

## 1. Tcl/Tk binding (`rustycad-tcl`)

No crates.io Tcl crate builds here (the `tcl` crate needs libclang for bindgen;
`tcl-sys` asks pkg-config for a module named `tcl`, which Debian/Ubuntu ship as
`tcl8.6`). The project owns a thin binding instead (user decision, 2026-10):

- `sys`: hand-written `extern "C"` declarations of exactly the Tcl/Tk 8.6 functions that
  OCCT `src/Draw` calls, plus three the safe wrapper needs (`Tcl_Alloc`,
  `Tcl_CreateCloseHandler`, `Tcl_DeleteCloseHandler`). The declarations are written
  through macros that also produce a table of their Rust signatures.
- `build.rs` asks the `pkg-config` program for `tcl8.6` and `tk8.6` (no pkg-config
  crate). A one-line `tcl.pc` forwarder in `OUT_DIR` lets the unmodified `tk8.6.pc`
  resolve its `Requires: tcl`. Stub libraries are not linked (they serve extensions).
- `tests/header_check.rs` turns the tables into C and compiles it with `cc
  -fsyntax-only -Werror` against the installed `tcl.h`/`tk.h`: each function is assigned
  to a pointer of the Rust-declared type, each callback typedef receives a function
  declared with the Rust argument list, every constant and the `Tcl_DString` /
  `Tk_FakeWin` layouts are `_Static_assert`ed. Negative tests prove that a wrong return
  type, a lost `const`, a lost `...`, a wrong constant and a wrong offset are rejected.
  Building the crate therefore needs the Tcl/Tk 8.6 development packages and `cc`.
- The safe wrapper: `Interp` (owning) and `InterpHandle` (borrowed, given to callbacks);
  Rust commands and variable traces whose client data is an `Rc` record (the trampoline
  takes its own reference for each call, so a command or trace may delete itself);
  panics are caught at the C boundary; `DString`; `Channel` with close detection (Tcl
  owns channels and a script may `close stdout`; counting a reference instead would make
  Tcl close a standard channel when the count drops back); `NewChannel` for a channel
  made from a descriptor; channel, file and exit handlers; static packages; and the Tk
  calls of the window layer behind a `TkApp` token (Tk reaches Tcl through a stubs table
  that only `Tk_Init` fills; `Tk_GetUid` before it crashes). No wrapper type is `Send`.

## 2. Session instead of globals

The C++ keeps the DRAW state in globals: `theCommands`, `dout`, `Draw_Batch`,
`Draw_Spying`, `Draw_Chrono`, `Draw_ParseFailed`, the drawable map, the last pick,
`global_Progress`, the restore factories, `Draw_BlackBackGround`, the font of
`hardcopy`, the units and perf-meter registries, and one `static bool Done` per command
group. They form one `DrawSession`, created with an injected console (`ProcessConsole`
for the executable, `RecordingConsole` in tests) and a viewer. Commands are
`fn(&DrawSession, &DrawInterpretor, &[&str]) -> Result<i32, Failure>`; a `Failure`
takes the C++ `Standard_Failure` path of `CommandCmd` ("An exception was caught ..."),
a panic the `std::exception` path. Commands hold a weak reference to the session (the
interpreter owns them). The `Done` guards are per session, so several interpreters can
coexist (one per test).

`Draw::Set`/`Get` keep the C++ mechanism: a drawable variable is a Tcl variable whose
value is its name, with a write/unset trace whose client data identifies the drawable
(an id standing for the C++ object address).

Process-wide statics that are process-wide in C++ stay process-wide: the default modes
and stop indicator of `Draw_ProgressIndicator`, the `cpulimit` limit and watchdog.

## 3. Window layer

`Draw_Window` talks to X11 (or Win32/Cocoa) directly. The port splits it:

- `DrawWindow` (one window: position, size, title, map state, clear, color, raster
  mode, `DrawSegments`, `DrawString`, redraw, save, pointer-motion selection, identity)
  and `DrawWindowSystem` (create a window, `DefineColor`, `Flush`, next pointer event)
  are traits owned by `rustycad-tkdraw`. Nothing else in the crate knows about a
  platform.
- `Draw_View` owns a `DrawWindow` and keeps the projection; it implements
  `DisplaySurface`, the interface `Draw_Display` draws on. `Draw_Display` and its
  file-scope state (`DisplayState`), clipping, picking and PostScript output are pure
  Rust and fully ported.
- `Draw_Viewer` with no window system is the batch viewer: every method returns at once,
  as each C++ method starts with `if (Draw_Batch) return`.

The native backend is a later step and is not in this crate yet. Decision (2026-10-15,
following the user's direction that the DRAW window draws through the Vulkan driver
instead of X11): the backend implements the two traits on the Vulkan 1.3 driver of
`crates/vulkan` (`docs/vulkan-renderer.md`, `docs/vulkan-window-presentation.md`), not with
Xlib drawing. `Aspect_DisplayConnection` still gives the native window and its events,
which the display connection delivers as a Tcl file handler, and the Tk main window
comes from `rustycad-tcl` (`TkApp`); `DrawSegments` and `DrawString` become line and
text draws of the driver in the palette color and raster mode of the window, and the
`-v` virtual windows render to the driver's offscreen target instead of a pixmap.
Until the backend exists, `Init_Appli` cannot open a display: `drawexe` without `-b`
reports "Cannot open display (this build has no DRAW window backend). Interpret commands
in batch mode." and continues in batch mode, which is the C++ behavior when the display
connection fails.

The `Draw_Window` members that only the backend has stay open in the API ledger until it
lands: `Init_Appli`, `Run_Appli` and `Destroy_Appli` (the display and event loop),
`Destroy`, `SetUseBuffer`/`GetUseBuffer`/`InitBuffer` (the off-screen buffer of a window),
the before-terminate callbacks (`AddCallbackBeforeTerminate`,
`RemoveCallbackBeforeTerminate`, `FCallbackBeforeTerminate`, run when the event loop ends)
and the protected `init`. The members the viewer already calls are partial rows on
`DrawWindow`/`DrawWindowSystem` with the batch behavior; the pure virtual `WExpose`
hook is `DrawViewer::expose_view`, which the backend will call on an expose event.

## 4. Plugins

`Draw::Load` resolves a key through the `DrawPlugin` resource file to a library name,
then calls the library's `PLUGINFACTORY`. Rust code cannot be loaded that way, so the
session holds `DrawPluginLibraries`: plugin libraries registered by file name with their
factory, the model of OCCT's static build (`OCCT_NO_PLUGINS`). The resource lookup, the
cache by key and the `Draw_Failure` texts (with glibc `dlerror()` wording) are kept.

## 5. Drawables, DrawTrSurf and DBRep

- `Draw_Drawable3D` is the trait `Drawable3D`; a drawable is shared as
  `Rc<dyn Drawable3D>` and keeps its common state in a `Drawable3DBase` with interior
  mutability. `DrawOn`, `Copy`, `Dump` and `Save` return `Result`: the C++ methods may
  throw while they evaluate geometry, and the failure travels through the viewer
  (display, repaint, pick, PostScript, fit) and `Draw::Set` to the command.
- A C++ subclass is composition: `DrawTrSurf_BSplineCurve` embeds a `DrawTrSurfCurve`,
  which embeds a `DrawTrSurfDrawable`. Each drawable answers `Drawable3D::ancestor` for
  its embedded part, so `downcast_kind::<T>()` finds `T` in a subclass as
  `occ::down_cast` does. Members a C++ subclass inherits live on the shared part
  (`CurveStyle`, `PolesStyle`, `KnotsStyle`, the surface iso settings); the virtual
  `ShowIsos`/`ClearIsos` of the BSpline surface keep their override as a flag of the
  surface part.
- The function-local statics of the packages above Draw (`DrawTrSurf::Parameters()`,
  `DBRep::Parameters()`) are per-session package state (`DrawSession::package_state`).
  Restore factories receive the session and return `Result<Option<_>>` (a failing
  shape read leaves `brestore` with the error, as the C++ exception does).
- `Save` has no session parameter but reads `Draw::GetProgressBar()`: the indicator a
  session installs is also published to `draw::active_progress_bar` (per thread). The
  pick statics of `DBRep_DrawableShape` (`LastPick`) are per thread for the same
  reason (`DrawOn` only receives the display).
- `Standard_OStream`/`Standard_IStream` of the DRAW files are the text streams
  `DrawOStream`/`DrawIStream`; they hand their bytes to the TKernel `IoStream` (GeomTools,
  BRepTools) or to `std::io` (Poly) and continue at the position the reader stopped.
- `std::cout` reports of library code (iso builder, normals) go to a `DrawConsole`
  parameter (the process console in `drawexe`).
- The native persistence classes (GeomTools, BRepTools and BinTools shape sets, Poly)
  are the existing TKG2d/TKG3d/TKBRep/TKMath ports; DRAW uses them and does not extend
  them (user decision 2026-09).

## 6. Checking against native DRAWEXE

`tools/verify_tkdraw_oracle.py --native-build <build>` runs every
`crates/tkdraw/tests/oracle/*.tcl` script with `-b -f` on the native DRAWEXE of the build
and on the Rust `drawexe`, both with an empty `DRAWDEFAULT`, and compares the outputs
(stdout then stderr). `--record` stores the native output as `<script>.expected`; the
integration test `drawexe_oracle` compares the Rust executable with the recorded outputs
without a native build. The scripts cover the interpreter (results, errors, help and
group registration, log, echo), the expression language, variables and protection,
the basic, graphic (batch), unit and message commands, `bsave`/`brestore`, every
DrawTrSurf drawable and command, and the DBRep commands on a BRep file. Batch mode never
draws; the drawing paths are unit-tested on a recording window system.

## 7. Deviations (each is documented at its code)

- `~Draw_Interpretor` calls `Tcl_Exit(0)`, ending the process. A Rust value is dropped in
  tests, so `drawexe` calls `Tcl_Exit(0)` explicitly at the end instead.
- A `DrawInterpretor` without a Tcl interpreter returns the `Standard_DomainError` of
  `Interp()` where C++ dereferences a null pointer.
- Text from other threads to `Draw_Printer`/`Draw_ProgressIndicator` is queued and
  appended on the interpreter's thread (C++ writes to the Tcl result from any thread).
- The `SIGXCPU` handler of `cpulimit` uses `write`/`_exit` (async-signal-safe) where C++
  uses `std::cout` and `exit`.
- `wait` sleeps between clock checks where C++ spins.
- `dversion` reports the Rust build (compiler `rustc`, the enabled optional libraries).
- Undefined C++ behavior gets a defined result: a view index outside `0..30`, `pick` and
  `wzoom` in batch mode (zero coordinates), `Atof` past a missing `)`.
- `dperf` reports the session's `PerfRegistry` (the Rust meters have no process-wide
  registry).
- A drawable restored from an unreadable record keeps a null geometry; drawing or
  dumping it fails with `Standard_NullObject` where C++ dereferences the null handle.
- `Draw::Get(".")` reports a pick that fails while drawing on the error console and
  returns nothing (every caller of `Draw::Get` would otherwise need a failure path);
  `clpoles`/`shpoles`/`clknots`/`shknots` pick once where C++ calls `Draw::Get` once
  per spline kind it tries.
- `normals` takes the faces in exploration order; the C++ map of faces iterates in
  object-address order.

## 8. Waiting for other toolkits

- The native window backend (section 3) and `xwd` snapshots (`Image_AlienPixMap`) wait
  for TKService.
- The status of the packages is in `docs/tkdraw-progress.md`.
