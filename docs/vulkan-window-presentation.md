# Martensite native window presentation

The Vulkan 1.3 driver presents existing Graphic3d/V3d/AIS scenes to native X11
windows. Virtual windows and `ToPixMap` retain their offscreen paths. Presentation
uses a GPU blit from the rendered sRGB image; it does not upload CPU capture pixels.

`SetWindow` binds a new drawable, `MustBeResized` updates view geometry, and the
next redraw recreates a changed swapchain. Vertical sync selects FIFO; disabling
it prefers immediate, then mailbox, then FIFO when those are unavailable. Unmapped
windows render offscreen. View removal releases the presentation resources and
unregisters the outer renderer view from its structure manager.

This slice requires `VK_KHR_surface`, `VK_KHR_xlib_surface`,
`VK_KHR_get_surface_capabilities2`, `VK_EXT_surface_maintenance1`,
`VK_KHR_swapchain` and the `VK_EXT_swapchain_maintenance1` feature. These are
optional for offscreen rendering. RGBA8/BGRA8 sRGB nonlinear surfaces must support
transfer destinations and blits. Unsupported configurations fail explicitly.
Other window systems, external GL contexts and device/surface-loss recovery remain
open; the renderer capability family remains partial.

## Lifetime and regression coverage

Acquisition fences protect acquired images. Presentation fences protect reuse of
presentation semaphores and destruction of old swapchains. A submission fence alone
does not establish presentation-resource completion; this follows the
[Vulkan semaphore-reuse guidance](https://docs.vulkan.org/guide/latest/swapchain_semaphore_reuse.html).
Displays remain owned until Vulkan instance teardown: Mesa WSI retains a
[connection-pointer cache](https://chromium.googlesource.com/external/gitlab.freedesktop.org/mesa/mesa/+/9103cd287315cf56122d5556f969fc1bd7511c87/src/vulkan/wsi/wsi_common_x11.c).
Closing and reopening connections while reusing the device reproduced an XFixes
BadRequest; retaining those connections resolves the lifecycle regression. Reusing
an application's display connection avoids accumulating separate connections.

The X11 fixtures bypass desktop window-manager placement and minimum sizing so
resize assertions have deterministic geometry. This resolved captures made at
65×65 while the requested drawable was transitioning to 80×48. The lifecycle
regression exercises 24 alternating resizes, vertical-sync changes, window
replacement, explicit unmap/remap, independent simultaneous views and removal.
Offscreen dumps must leave the displayed pixels unchanged.

The six window modes run through six camera/extent configurations on lavapipe and
the native AMD host. Each run reads actual drawable pixels using XGetImage and
requires byte equality with its own completed GPU framebuffer. The harness then
compares Martensite and pinned OCCT color, depth and actual window pixels. Raw
`.window.rgba`, screen PNGs and difference PNGs remain inspectable alongside the
existing capture artifacts. The two known AMD infinite-line pixel-row cases remain
nonblocking special cases; this work does not broaden their tolerances.

## Reproduce

```sh
DISPLAY=:1 cargo run -p martensite-vulkan --example window_acceptance --locked
DISPLAY=:1 python3 tools/verify_vulkan.py --source ../OCCT \
  --build target/occt-all-full --output target/vulkan-window.json \
  --captures target/vulkan-window-captures \
  --performance-baseline porting/runtime-evidence/vulkan-window-baseline-2026-10-10.json
```

The default matrix requires lavapipe and the native host. Two additional release
workloads measure steady 256×256 presentation and alternating 128/256 resize plus
presentation, with 20 warmup and 100 measured samples. They include scene
extraction, allocation, acquisition, rendering, blit, GPU readback and presentation
queue submission. They do not measure scanout latency. Timing validation remains
disabled; the functional checks require synchronization validation.

## Integrated evidence

Main `90fce3665` passes 105 Vulkan, 231 TKService and 373 TKV3d Rust tests,
30 Python harness tests and Clippy with warnings denied. Each device passes all
36 new window comparisons and 17 runtime checks with zero validation errors.
The full worktree matrix retains 574/948 strict matches on lavapipe and 494/948
on AMD; all 912 earlier statuses are unchanged. The focused main run independently
repeats the new cases, all runtime checks and all performance workloads.

| Workload | Lavapipe median / p95 | AMD median / p95 |
| --- | --- | --- |
| Present, 256×256 | 0.565 / 0.731 ms | 0.363 / 0.776 ms |
| Resize and present | 1.794 / 16.044 ms | 0.969 / 1.360 ms |

The six established timing budgets and both new window budgets pass against the
[combined baseline](../porting/runtime-evidence/vulkan-window-baseline-2026-10-10.json).
Eight other workloads still have unmatched baselines; full strict renderer
acceptance remains incomplete. Lavapipe resize samples include presentation
retirement waits: the worktree median was 7.817 ms with p95 16.041 ms. The raw
samples preserve this observed variation; these measurements are end-to-end latency.

The [integration receipt](../porting/integration/2026-10-10/vulkan-window.json),
[full worktree matrix](../porting/runtime-evidence/vulkan-window-worktree-2026-10-10.json),
[focused main checks](../porting/runtime-evidence/vulkan-window-main-2026-10-10.json)
and [timing comparison](../porting/runtime-evidence/vulkan-window-performance-2026-10-10.json)
record the separate scopes. All 7,728 raw buffers match retained main artifacts:
7,296 earlier captures and 432 newly captured window buffers. The
[buffer audit](../porting/runtime-evidence/window-main-buffer-audit-2026-10-10.json)
accounts for the captures before deleting the completed worktree and branch;
[cleanup evidence](../porting/integration/2026-10-10/vulkan-window-cleanup.json)
records deletion without archiving.

The refreshed 31-crate inventory remains at 51,636 done, 974 partial and 23,431
open APIs out of 76,041, with zero unresolved ledger rows. Renderer replacement
work does not inflate mapped OCCT API counts.
