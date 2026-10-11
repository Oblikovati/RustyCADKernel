# Martensite AIS light source binding

Martensite ports `AIS_LightSource` live light ownership, property mutations and
world/local transform binding from OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`AisLightSourceBase` embeds inherited interactive state and native light fields;
`AisLightSourceSource` and `AisLightSourceMembers` provide the operations for
application subclasses. The concrete light presentation remains incomplete.
The base constructor initializes fields only: it does not claim native aspect,
highlight drawer, presentation, selection, owner or drag construction.

Property setters preserve native invalidation distinctions: size, visibility,
zoomability, arc size and arrow count invalidate changed presentations. Marker,
tessellation, dragging and click-policy setters do not. Unsupported arrow counts,
negative tessellation values and non-finite sizes remain unvalidated, matching
native setters. The native arc-size field defaults to 25, despite the upstream
header comment saying 20.

`ais_light_source_update_persistence` chooses world, zoom, triedron, 2D or camera
persistence from light type, headlight state and zoomability. It retains existing
compatible handle identity and updates positional/spot zoom anchors. Ambient
lights retain any existing triedron or 2D persistence, even when its exact mode
differs from the default.

`ais_light_source_update_local_transform` derives translations/orientations from
the live light without sending them back into that light.
`ais_light_source_set_local_transform` propagates an assigned transform into light
position/direction, updates the inherited transform, invokes the subclass's
required aspect refresh, then refreshes persistence. Null transform input becomes
an explicitly retained identity datum. An application implementing a light
presentation can use these operations in its `PrsMgrPresentableObject` overrides;
Martensite does not install incomplete default drawing or aspect bodies.

Local native generation is in `tools/oracle/martensite_light_source.cxx` and
committed assets are `crates/tkv3d/tests/fixtures/light-source-*-native.txt`.
Two identical native runs generated 46 snapshots across all four source types,
zoomability, headlight states, transform assignment and resynchronization.
Rust-only tests replay these assets and exercise retained light/marker identity,
persistence reuse and presentation dirty status. Run:

```sh
cargo test -p rustycad-tkv3d --all-targets --locked
```

Native OCCT remains a local oracle only. Windows/macOS validation is deferred.
This slice changes no renderer submission or shader path. Concrete light aspect
setup, drawings, selection, light-owner click/highlight behavior and directional
dragging remain open; the class is partial and its constructor is unclaimed.
