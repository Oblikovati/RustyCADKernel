# Martensite AIS navigation dependencies

Martensite now ports the complete public APIs in `AIS_MouseGesture.hxx`,
`AIS_NavigationMode.hxx`, `AIS_RotationMode.hxx` and `AIS_WalkDelta.hxx` at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. Camera navigation, rotation pivot,
mouse gestures and walking axes retain their native integer values. The two
anonymous range enums have named Rust counterparts (`AisNavigationModeBounds`
and `AisRotationModeBounds`), preserving both lower/upper values without dropping
the enum declarations from the inventory.

`AisWalkPart` retains value, pressure and duration, defaulting to `(0,1,0)`.
Emptiness uses `abs(value) <= RealSmall()` with the native smallest positive
normal double, rather than an approximate camera tolerance. Positive/negative
zero, subnormals and values exactly at this boundary are empty. Adjacent larger
values, infinities and NaN remain nonempty, matching native comparisons. Pressure
and duration do not affect the predicate.

`AisWalkDelta` owns three translation and three rotation components. Shared and
mutable component methods and typed indexing preserve all four native indexing
overloads. Copies retain independent value state. Motion determines `to_move`,
`to_rotate` and `is_empty`; `is_defined` also considers an explicitly held key.
Jumping, crouching and running flags do not implicitly define motion or held keys.

```rust
use rustycad_tkv3d::ais::{AisWalkDelta, AisWalkTranslation};
let mut delta = AisWalkDelta::new();
delta.set_defined(true);
assert!(delta.is_empty() && delta.is_defined());
delta[AisWalkTranslation::Forward].value = 0.125;
delta.set_defined(false);
assert!(delta.to_move() && delta.is_defined());
```

Committed native fixtures exercise all enum and bound values, defaults, 15
floating-point boundary/nonfinite cases, 16 held-key/stance combinations, six
independently changed/cleared/copied axes and nonfinite delta queries. Rust tests
replay the assets without an OCCT build or runtime library. The local C++ fixture
generator depends only on headers, and compiles without linking the native kernel.
The compiler-bound ledger maps all 58 public captured declarations; native
private layout and parameter records remain under the existing inventory policy.
No denominator change or new exclusion is used.

Run `cargo test -p rustycad-tkv3d --all-targets --locked`. When adding toolkit test
modules, regenerate their canonical grouping with
`python3 tools/toolkit_test_suites.py --write` and verify it before integration.
The local native fixture can be regenerated during porting with:

```sh
c++ -std=c++17 -O2 -Itarget/occt-all-full/include/opencascade \
  tools/oracle/martensite_view_navigation.cxx \
  -o target/martensite-view-navigation-oracle
./target/martensite-view-navigation-oracle > \
  crates/tkv3d/tests/fixtures/view-navigation-native.txt
```

Provenance hashes are recorded beside the committed fixture. This completes the
view-controller navigation value dependencies; gesture dispatch, camera updates
and the AIS view controller itself remain separate porting work.
