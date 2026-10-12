# Martensite derived ViewCube camera transport proof

The 198 committed native CPU records replay actual derived Start/Update,
StartTimer/UpdateTimer, Stop, inherited camera atoms and viewFitAll callbacks.
Every generator ran twice with byte-identical outputs. Native originals remain
strongly owned through replacement; null is observed/restored before dereference.
The fixtures record exact animation/view/camera/owner/context/widget intrusive
counts. Context instrumentation borrows native InteractiveContext(); using
GetContext() in the instrument would create its own unwanted count increment.

The ten scoped rows replace existing API IDs: seven formerly partial animation
transport rows and the existing virtual fitting, owner-click and SelectDetected
bindings. No private-helper or new class credit is included. The complete
preflight output is compressed in canonical-bindings.json.gz; decompress it for
full captured C++/Rust declarations and source hashes. Fresh compiler artifacts
remain in the paths/hashes recorded by rustdoc-artifacts.json until integration.

Both device profiles use the actual concrete widget and real derived camera
source: normal context display, 26 owners, pixel detection, consumed click,
continuing/terminal camera frames, cyan immediate geometry and live appearance
changes. Each dispatches StartTimer once, UpdateTimer three times and the real
camera atom four times. Capture pairs retain 320×320 RGBA and finite f32 depth,
with no validation errors and 4149 cyan pixels. Native assets are the unchanged
pinned baseline under ../view-cube-widget-appearance-2026-10-12/native; its
viewer remains strongly owned and does not use the generic policy-probe helper.

Strict identical-RGBA and maximum depth delta<=1e-6 checks remain unchanged.
All 18 strict comparison failures are retained; no pixel-boundary acceptance or
historical performance baseline is claimed. Whole widget/owner/sensitive classes
remain partial pending generated RTTI/base-alias obligations.

The final optimized executable ran eight completed-frame workloads on each
device, with 20 warmups and 100 samples per workload (1,600 raw samples).
The added workload dispatches the actual derived camera UpdateTimer and inherited
atom before completing every rendered frame; final counters are 1/123/124.
The coordinator held other workers and verified the executable, 69 focused
source files and all 11,984 crate/build-input files before GO. Each manifest and
the executable was rechecked before and after both profiles without changes.
Raw vectors, device identities, commands, process observations and the final
optimized relink log are retained. No historical performance baseline is accepted.
