# Martensite retained point alpha context

OCCT source pin: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The existing 21 point/sprite alpha goldens remain unchanged. The new generator
`tools/oracle/martensite_vulkan_point_alpha_context.cxx` renders those 21 modes
in their existing order on one real view/context, then checks two outcomes:

- `retained`: opaque plain points with `CAMERA_PERS` inherit enabled
  `GL_ALPHA_TEST`, `GL_GEQUAL`, reference 0.9 from the last mask sprite.
  Their alpha 0.25 is rejected. Both attachments remain exactly cleared.
- `reset`: a real `sprite-alpha-opaque` redraw disables the compatibility alpha
  test through upstream `drawMarkers`; subsequent opaque plain points retain
  this disabled state. Actual camera persistence resets world-view to identity,
  enables depth clamp, and produces 75 red pixels with depth zero.

Both outcomes use exact RGBA and little-endian float depth equality in the
shared offscreen acceptance helper and ordinary Rust toolkit regression.
The state regression also checks retained 0.9 followed by the real sprite reset.
No production alpha bypass, replacement goldens, or tolerance adjustment is used.
Rust tests use package-owned assets and need no OCCT.

The failed application candidate was frozen `bb1e00115`; its binary and original
log remain unmodified in `../Martensite-application-prerequisites`. This fix
starts at local main `43a83e66e`. Proof records distinguish that source from the
failed candidate and the temporary diagnostic source.

`proof/` preserves the original red, diagnostic inherited-alpha suppression
(green), restored-state red, cold-first-native exploratory capture, native
state traces, and source patch. Suppression is causal diagnostic evidence only.
The cold-first native capture is excluded from the accepted warmed comparison;
rendering ordinary points first produced the exact 75-pixel fresh-context
control. The committed final fixtures instead establish state by an actual
opaque sprite operation in the original retained context.

Native final provenance records exact commands, devices, libraries and source
hashes, and two-run byte equality separately for software and AMD. Both-profile
functional gates run tests plus the actual offscreen and lighting examples;
performance timings are outside this correction.
