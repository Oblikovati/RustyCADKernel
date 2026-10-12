# Martensite ViewCube appearance

This slice implements the remaining 58 declared appearance operations and 18
protected fields of OCCT `AIS_ViewCube` at
`3d097a0328e71b826377d4814ab05ec3c3d23871`. The concrete widget uses real
presentation geometry, 26 sensitive owners, immediate part highlighting and
camera animation. Whole-class coverage remains partial: generated class
API/type aliases remain separate work. The [derived camera transport](ais-view-cube-animation-source.md)
now preserves virtual dispatch, live slot references and native counts. No getter-only façade or GPU pixel-parity credit is claimed.

Geometry setters retain native Confusion guards and only mark presentations
stale. `SetSize` always assigns its value, including a tiny change; adaptation
independently preserves nonpositive facet extension/padding and updates font
height. Selection remains cached through presentation-only recomputation and
changes on normal full redisplay. Style setters synchronize existing group
aspects, preserving the native ownership flags. Labels use actual `DataMap`
storage with the upstream enum hash, including arbitrary valid protected keys,
missing-key empty strings, iterator order and Y-up remapping.

Reference contracts bind actual storage rather than copied conveniences:

- Side/edge/corner style references observe their chosen handle slot. Inherited
  drawer references stay in the parent slot when a child subsequently overrides
  it. Replacing the widget drawer does not redirect an existing reference.
- Color references bind the actual interior/material/text subfield once;
  changing material type does not redirect an existing alias.
- Font references bind the chosen `HAsciiString` payload or static empty string.
  Replacing `TextFont` preserves the old alias, while mutation of the retained
  original string is visible through it.

The views retain no field lock or native reference increment. Explicit
`clone_handle()` creates owning snapshots; callback access releases locks before
user code. Rust observers keep storage safe if callers exceed native lifetime
preconditions. Native fixtures retain original owning handles, excluding native
dangling-reference undefined behavior from the tested domain.

`UnsetAttributes` resets the existing drawer and style handles in place.
`UnsetHilightAttributes` retains the dynamic drawer, replaces its shading with
cyan/unlit defaults and clears ordinary highlighting. `ResetStyles` additionally
restores size 70, extension 10.5, padding 7 and font height 11.2, while preserving
Y-up, custom labels, axis radii, persistence, cameras, animation and callbacks.

The Rust-only replay uses 154 byte-repeatable native CPU records: 105 mutations,
selection/cache/map/range snapshots and 29 retained-alias/reset-ownership states.
The native generators are local development tools; replay does not link OCCT.

```sh
cargo test --locked -p rustycad-tkv3d --all-targets
cargo test --locked -p martensite-vulkan --all-targets
```

The source-bound scoped claims are the focused
`porting/ledger-fragments/view-cube-appearance-{geometry,style,storage}-*.json`
files. Each pins one visible public/protected C++ declaration to an actual fresh
canonical rustdoc declaration, including concrete trait override selectors.
The coordinating agent alone appends them to the main API ledger.

The [actual widget capture fixture](vulkan-view-cube-capture.md) checks the
context display, pixel detection, consumed click, deterministic camera frames,
style mutation and cyan highlighting on both local devices. It retains strict
color/depth comparison failures and performance evidence separately from CPU
behavioral credit.
