# Martensite concrete ViewCube appearance evidence

Native OCCT is used only to produce local oracle assets. Rust regression replay
uses the committed CPU snapshots without native linking.

The gzip assets contain complete 320×320 RGBA8 and little-endian f32 depth
buffers for nine cases on native OpenGL, lavapipe and the AMD host. Decompress
a pair with `gzip -dc` and pass the uncompressed prefixes to
`tools/vulkan_capture_compare.py::compare_captures`. The strict identical-RGBA
and depth-delta≤1e-6 policy remains unchanged; `strict-comparisons.json` retains
all failures. R8 masks flag depth differences greater than 0.1 for diagnosis,
not as a replacement acceptance gate.

The source/start/end/quarter/half/end camera logs include Eye, Center, Up,
Direction, scale, FOVy, FOV2d, aspect, projection, near and far planes. View state
records AutoZFit, invalidation and primary/auxiliary bounds before and after
dumping. Endpoints and intermediate camera states match native. Primary bounds
are void and fitting therefore preserves scale; auxiliary bounds agree.
Native dump leaves the view invalidated, while Vulkan clears its rendered state.

The initial end-frame failure buffers remain under `initial-end-diagnostic/`.
Their large maximum depth difference represents covered geometry versus clear
background on narrow one-pixel silhouette columns; no camera-state discrepancy
was found. Pixel-boundary precision work is deferred. No GPU parity or historical
performance baseline acceptance is claimed.

The cyan case deliberately sets the dynamic highlight drawer to TopOSD.
Ordinary cube geometry retains native Topmost and LEFT_LOWER triedron defaults.
Native, lavapipe and AMD each produce 4,149 exact cyan pixels through real pixel
`MoveTo` detection and actual immediate part geometry. The appearance case uses
real geometry/label/style setters followed by context redisplay.

CPU provenance, source hashes, functional results, compressed raw buffers,
strict comparison diagnostics and process/timing records are separate files so
that each document stays below the repository's 500-line limit.

The subsequent function-span audit extracted helpers without changing APIs,
defaults, geometry, order, timer behavior or strict gates. Original captures,
source hashes and aea6 timing evidence remain unchanged. Final source hashes,
991 repeated tests, fresh canonical bindings and relink logs are separate
post-refactor artifacts. Functional captures were not repeated after extraction;
the coordinating agent will run final-main captures on both profiles.
`post-refactor-timing/` retains the independently verified final c9bd executable's
fresh exclusive timings, raw samples and process observations.
