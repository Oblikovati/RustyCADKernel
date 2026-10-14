# Use framebuffer images for Martensite rendering acceptance

Accepted by user decision, 2026-10-14. Rendering and GUI acceptance captures the
completed renderer framebuffer to an image on lavapipe and the native GPU. This
replaces X11 or compositor pixel reads in the GUI capture gate, following
[Oblikovati's capture service](../../../Oblikovati/head/ui/viewport_screenshot.go).
The [bracketed diagnostic](../evidence/native-gui-oracle-2026-10-14/README.md)
demonstrated stale Xwayland client pixels while the displayed GUI was correct.

Capture after model and GUI composition completes, retain the frame's extent and
freshness, and publish the image atomically through a temporary file and rename.
Keep independent expected pixels, existing comparison policies, validation-layer
checks, image artifacts and rendering performance tests. A new GUI packet must
invalidate the preceding capture. Never redraw merely to obtain a different
comparison result.

Framebuffer captures verify rendered content. Window creation, event delivery,
resize, mapping and swapchain lifecycle retain their own functional tests. The
GUI gate does not depend on desktop decorations, occlusion, compositor capture
tools or X11 image-readback behavior. Earlier diagnostic captures remain
historical evidence rather than the current acceptance strategy.
