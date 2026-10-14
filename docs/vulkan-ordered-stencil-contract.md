# Martensite ordered stencil-test contract

Pinned upstream revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`Graphic3d_Group.hxx:132` declares the public virtual operation without an
obsolete/deprecated marker. This work replaces renderer behavior and introduces
no additional retired API.

`OpenGl_Group.cxx:216–220` appends an `OpenGl_StencilTest` element for every call.
`OpenGl_StencilTest.cxx:27–40` enables testing and sets NOTEQUAL/reference1/mask255
when true; false only disables testing. Neither path changes operations or the
write mask. Release is empty. `OpenGl_Group::Render` restores aspects alone:
state therefore follows executed element order across groups, structures and
layers. Clearing/deleting retained elements must not invent a GL state reset.

`OpenGl_Workspace::ShouldRender` preserves non-fill commands in the opaque pass,
but rejects them for FillModeOnly and TransparentOnly. Capping's fill-only
traversal consequently does not execute stencil commands. The capping sentry
independently clears stencil, restores depth comparison, sets ALWAYS/reference0/
mask255 and disables stencil at completion; it does not restore an earlier
application stencil function or operations. These boundaries must be preserved
through unordered blending, weighted OIT and dual peeling rather than applying
a bool to all lowered geometry indiscriminately.

The implementation must retain state-only and trailing commands, actual group
clear/delete lifecycle, context/view isolation, empty and filtered geometry,
opaque/transparent ordering, layer boundaries and cap sentry transitions.
Vulkan targets need a combined depth/stencil attachment even without capping
when an executed stencil command requires it. Pipeline variants must preserve
existing capping masks and operations; false must not reset unrelated fields.
Native repeated traces, committed image/depth assets, independent Rust replay,
both-device validation/captures and identical time/peak-memory workloads are
required before integration. No parity or API completion is claimed by this
initial contract audit.

Valid native text is another state boundary: `OpenGl_Text.cxx:895` disables
stencil after glyph rendering, while empty/uninitialized text returns earlier.
The retained timeline therefore carries a text reset after actual glyph draws,
preserving all other context fields. See the final functional proof for original
red captures, unchanged comparison gates and full87-case repeated evidence.
