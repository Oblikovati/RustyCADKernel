# Native window lifecycle baseline diagnosis

Baseline `c814d12b3`. These probes diagnose a reproducible lifecycle defect;
they do not establish the cause of the earlier intermittent GUI `BadWindow`.
No renderer implementation, baseline image, or production error handler changed.

The independent requirement is that a delivered destruction notification retires
the matching target and subsequent public operations report a closed target.
They must neither terminate the process nor return a previously successful image.
Baseline evidence goes red on the process-survival part of this requirement.

The minimal public-window subprocess creates an owned `XwWindow`, wraps its
native ID, releases the owner, and calls the wrapper's public `size()`. Three
attempts exited 1 with `BadWindow`, request `X_GetWindowAttributes`. Retaining an
owned handle and retaining the borrowed native owner both passed. The command is
`DISPLAY=:1 target/debug/examples/native_window_lifecycle_probe wrapped-dead`.
`strace -k -e trace=exit_group` captures `_XDefaultError → _XReply →
XGetWindowAttributes → XlibApi::window_attributes`: Xlib exits before the ignored
return status can be checked.

The public-view diagnostic establishes a more directly supported boundary:

| Case | Baseline result |
| --- | --- |
| Caller drops owned window; view retains it | Success |
| Rebind to a fresh owned window | Success |
| Wrapped native owner retained | Success |
| Wrapped native owner destroyed, then public resize | Fatal `BadWindow` |
| Wrapped native owner destroyed, then public redraw | Fatal `BadWindow` |
| Owned window destroyed; actual `DestroyNotify` delivered; public resize | Fatal `BadWindow` |

All six cases repeated three times with RADV selected and three times with
lavapipe selected: 18 successes and 18 exact `BadWindow` failures. These are
lifecycle observations, not complete rendering or application acceptance.
Validation was enabled. No native OCCT or timing runs were made.

For delivered destruction, `XDestroyWindow` plus `XSync` establishes server
completion without a sleep. The actual pending `DestroyNotify` is read and passed
to `XwWindow::process_message`, which returns `false`; the native ID remains
nonzero. Public resize then calls `V3dView::must_be_resized → set_ratio →
XwWindow::size → live_attributes → XGetWindowAttributes`. Redraw reaches the
same query through `Graphic3dRenderView::try_redraw → capture_extent`. Temporary
boundary stack logging confirms these Rust callers and is retained only in the
diagnostic archive as an explicit patch.

The separate shared-display-reset probe exited 1 without instrumentation and
aborted with `corrupted double-linked list` when instrumented. Its logs and the
runner's failed expectation are retained. This is a separate unsupported lifetime
case, excluded from the delivered-destruction correction; no cause or fix is
claimed here. Unreported external destruction remains separate too.

`baseline-diagnostics.tar.gz` preserves the exact probe sources, temporary trace
patch, build logs, all outcomes, failed runner expectation, and stack traces.
Sources are under `window-lifecycle-diagnostic/sources/` at their original
repository-relative locations. For revision-bound replay, copy the two example
sources into a checkout of the baseline, build with `CARGO_BUILD_JOBS=2 cargo
+1.99.0 build -p rustycad-tkservice --example native_window_lifecycle_probe
--locked` and the analogous Vulkan example `native_view_lifecycle_probe`.
The optional trace patch and `run_trace_cases.py` reproduce the traced cases.
The committed production source contains no diagnostic stack instrumentation.
