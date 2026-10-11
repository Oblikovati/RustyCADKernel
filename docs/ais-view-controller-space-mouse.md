# Martensite 3D mouse controller routing

`AisViewController::update_3d_mouse` ports the full native `Update3dMouse`
HID-report path from OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Translation is always processed first. Rotation follows when allowed; buttons
follow both motion stages. Combined packets retain the decoder's shared raw range.

The project-owned `MartensiteSpaceMouseInput` interface supplies listener settings
and fallible keyboard callbacks without requiring window callbacks. Existing
`AspectWindowInputListener` implementations receive a blanket adapter; the AIS
controller supplies its own keyboard target. Both use the same dispatch functions.
This avoids duplicating HID routing or swallowing animation failures to implement
the remaining window-listener interface prematurely.

Translation intentionally updates the key cache directly. Rotation uses the
keyboard callback, skipping disabled axes and reversing configured axes. Buttons
update their native cache before callbacks, including unmapped bits. Held buttons
repeat callbacks; releases dispatch once. Failures propagate immediately, retaining
already-completed cache changes and preventing later callbacks.

## Local verification

The local oracle `tools/oracle/martensite_view_controller_space_mouse.cxx` generated
288 byte-identical snapshots in two runs on this Linux host. Each records report
bytes, raw range, return value, modifier flags, all 32 cached buttons, and every
virtual key's down/free state, held flag, duration and pressure. Scenarios cover
six product IDs, combined and separate packets, idle reports, repeated presses,
release transitions, skipped/reversed axes, policy gates, quadric/linear pressure,
negative/zero/infinite/NaN acceleration and unknown reports.

Rust tests replay committed assets using injected/frozen event time and require
no OCCT installation. Named callback targets additionally verify direct-cache
translation, fail-fast rotation, partial button-cache mutations, repeated presses,
one-time releases and release failures. A window-listener trait object verifies
the shared adapter's virtual keyboard dispatch.

Run `cargo test -p rustycad-tkservice -p rustycad-tkv3d --all-targets --locked`.
Windows/macOS execution is deferred. This platform-neutral HID decoder test does
not require a physical 3D mouse or claim operating-system device ingestion.
Camera/window application, redraw and XR controller paths remain open; the
controller class and constructor remain partial.
