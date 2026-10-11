# Martensite text culling local evidence

Pinned native policy: 384 calls and Vulkan all targets: 260 tests pass. Clippy with warnings denied, global formatting, canonical discovery and discovery: 13 Python tests pass. Every new Rust function is 4–20 formatted lines; files remain below 500 lines.

Real nativeGL and both Vulkan device profiles match coverage for 19 functional/capture cases. Host RGBA is byte-identical. Lavapipe differs in at most 18 channel bytes by one, with identical coverage; depth delta is at most 1.2e-7. This narrow real-font blending difference is recorded explicitly, without changing the shared strict comparison gate.

Both optimized-device timing processes exited 0 with 19 cases, 20 warmups and 100 samples each. `performance-environment.json` records the exclusive team idle slot, binary/source identity, process names and host load. Raw sample distributions are compressed in `performance.json.gz`; no previous-source baseline or performance improvement is claimed.

`lifecycle.json` preserves the displayed-empty-before-text diagnostic: native and Martensite both omit the draw; fresh native display-afterward renders 102 pixels. No bounds behavior change was made.

Source and pinned native library provenance is in `crates/vulkan/tests/fixtures/text_culling/provenance.json`. Native generators and ordinary Rust replay tests are committed. GPU captures were produced before Shadow integration; the generic all-pass policy is covered with two draws and root checks real Shadow integration after merging. Linux only; Windows/macOS are deferred. No complete OpenGl_Text/API ledger credit is claimed.
