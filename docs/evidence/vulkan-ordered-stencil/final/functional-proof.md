# Martensite ordered stencil functional proof

The original 87-case catalog passes on both actual Mesa 25.2.8 profiles. Each
case/profile has two native and two Rust runs: 696 successful processes and 174
strict comparisons. RGBA and eight context fields match exactly; depth keeps the
existing finite absolute `1e-6` gate. Native draw/stencil traces and each backend's
buffers/state repeat byte-for-byte. `raw-functional.tar.gz` retains every capture,
command, log and trace; the compressed member manifest binds their bytes.

The committed `software-native.bin` and `host-native.bin` each contain 87 ordered
records of 16,384 RGBA bytes followed by 16,384 little-endian depth bytes.
`native-cases.txt` supplies mode, method, seed and eight final state fields in the
same order. The independent Rust-only replay constructs actual retained objects,
checks buffers and state before/after reuse, and never loads OCCT. The original
line, point and text cases contain actual non-fill primitive/glyph payloads.

Native queries observe actual texture/renderbuffer attachment storage without
changing bindings. Scene targets are `GL_DEPTH24_STENCIL8` (35056), with eight
stencil bits and unsigned normalized components. Intermediate absent attachments
remain absent. The default drawable reports eight bits; its internal format is
truthfully unavailable (`-1`). No format is inferred from an assumed framebuffer.

The cohort comes from isolated extraction of cached Ubuntu Mesa 25.2.8 packages.
The host driver was not changed. Raw identities show llvmpipe LLVM20.1.2/128-bit
and AMD Radeon 8060S/RADV GFX1151. Software SSE2/128 controls, exact package hashes,
ICD paths, VVL wrapper commands and actual identities are retained. No version
string override substitutes for executing those libraries.

Six earlier text cases failed exact color and final enabled-state comparison.
Pinned `OpenGl_Text.cxx:895` disables stencil after valid glyph rendering. The
Rust event now resets testing after actual glyph draws, preserving comparison,
reference, masks and operations; empty text does not manufacture a reset. Earlier
174-row results and six corrected text reruns remain in the raw archive.

Eleven native lifecycle rows repeat twice/profile. Rust additionally verifies
physical visible/rejected pixels, unchanged state on a named pre-submission
transport failure, repair/reuse, group clear/remove/destruction, and independent
adjacent/replacement views. This fault is not native device-loss recovery.
The original invalid native sequence removed a still-displayed structure, then
removed its view: pinned `Graphic3d_Structure::Remove` nullifies its CStructure
without erasing the view's display entry. Its crash, executable, source and valid
prefix remain attributed. Valid lifecycle proof uses `Erase` before `Remove`;
there is no repaired-success claim for the invalid native state.

Full toolkit, extracted-package and performance acceptance are recorded separately.
The 87-workload harness verifies the same native gates without timings in tests;
its explicit performance entry point measures steady redraw/readback after setup,
with 20 warmup and 100 samples per workload. Setup, native capture, comparison
and destruction are outside each sample; process peak RSS includes them.
No timing run or baseline has been accepted yet. GPU memory is unmeasured.
Existing raster/text/depth and undefined-domain diagnostics remain unchanged.
This work claims no class/API, Windows/macOS or application completion.
