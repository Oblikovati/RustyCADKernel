# Native GUI capture-oracle diagnosis

The reported AMD failure has a demonstrated capture-oracle fault. On this
rootless Xwayland host, client `XGetImage` can retain old pixels while the
compositor displays the current Vulkan frame. This does not establish a renderer
defect or clear the complete application acceptance gate.

The unchanged, hash-verified v0.2.7 minimal consumer failed both empty-scene runs:
the GPU capture includes the GUI, while the client image retains green across
exactly 33,000 pixels. Diagnostic instrumentation then retains the failed frame
without another redraw. Client reads immediately before and after an independent
COSMIC compositor capture both return the identical stale green GUI rectangle.
Every pixel of the compositor GUI rectangle matches the GPU capture exactly.
The four compressed RGBA buffers retain that bracketed comparison.

`receipt.json` records the immutable binary, diagnostic source/rlibs, commands,
rectangle and hashes. `capture_repro.rs` and `read_client_gui.py` are diagnostic
sources, not production paths or replacement regression tests. The optional
45-second hold exists only to inspect an already-failed frame; it is not a fix
or a widened presentation deadline.

Only the unobscured 300-by-110 GUI rectangle is proven by this comparison. A
neighboring window occluded part of the requested full client rectangle, so the
larger crop is not full-frame acceptance evidence. Uncropped desktop captures
and larger crops were deleted; the retained PNG contains only the test GUI.

The initial follow-up explored a compositor oracle for the full managed window.
User decision on 2026-10-14 supersedes that approach: use completed framebuffer
images on both drivers, as recorded in
[ADR 0016](../../adr/0016-framebuffer-images-for-rendering-acceptance.md).
Preserve the rapid three-packet replacement sequence and independent pixel
assertions, then rerun the application command gate through that capture path.
No additional redraw, arbitrary wait, renderer fallback or API completion credit
follows from this diagnosis.
