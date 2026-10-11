# Martensite mouse dispatch

Martensite ports the controller mouse gesture map, button transitions and pointer
motion from OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The live primary map retains the nine native default bindings and supports edits.
The independent secondary map preserves native left-button dragging.

Button routing preserves priority, strict click-release thresholds, double-click
counter/timer transitions, selection completion using the previous binding, and
animation pause before selection. Animation failures propagate as `Failure` while
retaining changes made before the native callback would throw.

Motion supports all native gestures: rectangular/lasso selection, horizontal and
vertical zoom, zoom window, accumulated pan, orbit/view rotation and object drag.
Emulated motion uses native touch tolerances. Rotation consumes its start request
before movement exceeds tolerance; panning waits until it does. First-person idle
rotation can queue a start while returning false. Coalesced zoom can advance drag
and pointer state while returning false. Modifier changes route through button
transitions after the motion delta was computed.

## Local verification

The local native generator is `tools/oracle/martensite_view_controller_mouse.cxx`.
Its 1,220 snapshots were reproduced byte for byte on this Linux host. Committed
fixtures contain return values, controller state and selection, drag, highlight,
pan, rotation and zoom requests. Rust replays use an injected frozen clock and
require no OCCT installation. Additional regressions cover exact timer boundaries,
NaN interval comparisons, live map isolation, hover suppression and failed pause
ordering. Provenance records pinned source and fixture SHA-256 values.

Run `cargo test -p rustycad-tkv3d --all-targets --locked` for the toolkit tests.
Windows and macOS execution is deferred while porting is tested on this host.

The controller class and constructor remain partial. Camera/window application,
redraw dispatch and 3D mouse controller integration still require porting; these
input request methods do not claim completion of those downstream paths.
