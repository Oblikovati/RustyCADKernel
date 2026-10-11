# Martensite viewer input transfer

`AisViewController::flush_buffers` ports the complete protected OCCT
`AIS_ViewController::flushBuffers` operation at upstream commit
`3d097a0328e71b826377d4814ab05ec3c3d23871`. Context and view handles retain
the native signature; this operation does not dereference either handle.

Transfer replaces the render zoom queue, gesture transition and FitAll flag.
Other pending render requests persist until consumed by rendering. Selection
points copy into independent render storage while preserving its allocator;
picking clears UI points immediately, while other tools retain them until apply.
Abort takes priority over stop, which takes priority over drag start/confirm/move;
lower-priority UI requests wait for subsequent transfers. Drag end coordinates
always transfer, including abort and stop frames.

The local native generator produces 3,072 snapshots covering every pairing of
five UI drag flags and five existing render drag flags over three consecutive
frames. Each snapshot compares all 117 columns of both queues and the next-frame
request, including retained coordinates, selection points and zoom actions.
Independent request patterns exercise orientation, highlighting, panning and
all three rotations. Two native runs are byte-identical; committed provenance
records upstream, source and fixture hashes. A separate Rust regression checks
distinct UI/render allocator identities and independent selection ownership.

Run `cargo test -p rustycad-tkv3d --all-targets --locked`. Rust replay requires
no OCCT installation, viewer or GPU. Validation runs locally on Linux;
Windows/macOS execution remains deferred.

Gesture synthesis, public `FlushViewEvents`, camera application, redraw and XR
remain open. Queue transfer alone does not complete the controller class or
constructor, and does not claim the complete public event-dispatch path.
