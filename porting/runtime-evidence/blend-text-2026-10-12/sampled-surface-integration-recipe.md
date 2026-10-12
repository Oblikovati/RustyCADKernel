# Required combined-main sampled-surface-after-XOR regression

The texture frontend is absent from this branch base. Root serial integration must run this real-image regression after merging the checked texture and XOR features; a legacy atlas substitute does not establish sampled-surface parity.

Use `SurfaceTextureScene::with_mode(&driver, mode)` from `tests/support/surface_texture_scene.rs` with a named `RecordingDevice` backend. Redraw and clone `recorded.last_frame().draws()`. Keep the fixture/texture alive. Build actual input `[fullscreen_xor(), real lowered sampled draws...]` on blue and control input `identical real sampled draws` on yellow. The existing fixture declares ordinary polygon-offset0. Compare exact RGBA/depth through actual VulkanRenderer, with validation enabled, on lavapipe and AMD. Repeat nearest, phong, mask-blend and blend modes; keep any transparent draw/layer depth policies identical in actual/control. This checks SRGB sampler decoding, lighting/alpha, LOAD ordering and depth after UNORM→SRGB attachment transitions.

The XOR branch owns `blend_boundary_runtime.rs` and its reusable solid XOR fixture; the sampled-image worker owns the new real texture frontend. No combined-main success is claimed in this pre-integration receipt.
