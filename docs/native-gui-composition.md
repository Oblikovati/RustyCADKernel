# Native GUI composition on the existing Martensite view

This extension consumes the owned `RenderLayer` produced by the application's
`GuiFrameCompiler::layer` (see application
`architecture/rust-port/implementation-imgui-packets.md`). It adds no application
renderer, ImGui dependency, geometry structure, or production image-copy path.
The existing kernel renderer owns device resources, presentation and capture.

```rust,ignore
let render = view.view().cast::<martensite_vulkan::Graphic3dRenderView>();
let layer = martensite_vulkan::RenderLayer::new(draws, false, false, false);
render.attach_gui_layer([width, height], layer)?;
render.try_redraw()?;
let pixels = render.capture()?;
render.clear_gui_layer()?;
render.try_redraw()?;
```

Attachment also replaces an existing packet. It validates a defined live view,
ordered draws, disabled depth test/write/clear, and expected target dimensions
before changing the accepted packet. Rejected replacement leaves that packet and
its matching capture intact. Successful attachment or clear invalidates capture
until redraw succeeds. Rendering errors never return an earlier successful capture.
A packet changed during rendering makes publication fail and requires another
redraw. GUI mutation releases retired packet ownership outside its slot guard.

Notify resizing through the normal view resize operation, then submit a fresh
packet for the current dimensions. Even resizing back to the previous dimensions
requires a fresh packet. A changed target extent also rejects capture before the
resize callback. The current framebuffer target participates in extent validation;
a different-size image dump needs a packet for that target, or an explicit clear.
Native presentation also rejects a differing actual swapchain extent instead of
scaling GUI pixels. After updating the application window extent and notifying the
view, attach a fresh matching packet and redraw.

The complete model frame is composed first, including deferred transparency/OIT.
UI then appends an ordered span without clearing or writing model depth. Float
colors, UVs and intersected clipping chains retain their existing immutable draw
representation. UI receives no model camera, culling, clipping or lighting rewrite.
It never enters model structures, Fit bounds or AIS selection. GUI draws receive
complete local compatibility-alpha state, and only the model-prefix ending state
persists into the next model frame. A retained model mask therefore neither
discards translucent widgets nor changes after clearing them. Retained immutable
atlas images survive producer destruction and packet snapshots; GPU resources
remain in the existing fence-completed frame lifecycle. Remove releases the UI
packet and presenter, without changing ordinary views that have no GUI packet.

The ordinary toolkit GPU cases are display independent. The same assertions run
against actual mapped X11 pixels through the explicit native harness:

```sh
python3 tools/with_vulkan_validation.py --cache /path/to/pinned/cache -- \
  cargo test -p martensite-vulkan --lib --test toolkit gui_ --locked
DISPLAY=:1 python3 tools/with_vulkan_validation.py --cache /path/to/pinned/cache -- \
  cargo run -p martensite-vulkan --example native_gui_acceptance --locked -- target/gui
```

The native harness preserves RGBA, depth and actual-window RGBA buffers when given
a prefix. It verifies replacement, removal, resize, font retirement, all four clip
edges, asymmetric font UVs, straight alpha order, all three model transparency
modes, selected-owner identity and selection bounds. Analytic sRGB expectations
allow one quantization byte; actual native pixels must equal GPU capture exactly.
No old oracle assets or numerical tolerances change. Software and native device
runs, source fingerprints, validation provenance and separately reserved performance
evidence are required before integration. This kernel extension does not close
application R1V/R4, the full renderer port, or the deferred controller family.

The native performance example uses the same existing view and accepted packet
shapes, with model-only, static, replace, resize, clip, atlas retirement and ordered
alpha workloads. Build its optimized executable before reserving a quiet host:

```sh
cargo build -p martensite-vulkan --example native_gui_performance --release --locked
DISPLAY=:1 /usr/bin/time -v target/release/examples/native_gui_performance 20 100
```

Run the frozen executable separately on lavapipe and the host ICD. It records raw
per-workload samples and actual device identity; validation is disabled for timing.
The functional harness with pinned validation must pass before and after these
measurements. This does not authorize a timing run while other host jobs are active.
