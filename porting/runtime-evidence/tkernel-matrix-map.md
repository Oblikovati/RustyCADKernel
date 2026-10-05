# NCollection Mat3/Mat4 `Map` alias behavior

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` defines mutable and
const `Map` overloads in `NCollection_Mat3.hxx` lines 511/517 and
`NCollection_Mat4.hxx` lines 654/660. Both return matrix references over the
provided `Element_t` array. The probe checks the returned matrix address equals
the array address, matrix writes update the array, and array writes are visible
through a fresh const map. Rust now provides safe lifetime-bound borrowed views:

```rust
use rustycad_tkernel::ncollection::{Mat3, Mat4};

let mut values = [0.0; 9];
Mat3::map_mut(&mut values).set_value(1, 2, 4.0);
assert_eq!(values[7], 4.0);
let view = Mat3::map(&values);
assert_eq!(view.get_value(1, 2), 4.0);

let mut values4 = [0.0; 16];
Mat4::map_mut(&mut values4).set_value(1, 2, 5.0);
assert_eq!(values4[9], 5.0);
```

These constructors accept aligned, correctly sized Rust arrays, retain the
column-major alias, and avoid reinterpreting Rust matrix object layout. The
views expose element and slice access rather than the entire matrix method
surface, so the four `Map` declarations remain partial. Rust's borrow checker
also prevents ordinary writes through another reference while a shared view is
live and prevents concurrent mutable aliases; `Cell` tests cover live shared
observation only for explicitly interior-mutable storage. The existing
`from_data` methods remain copying constructors.

## Compiler identities

The pinned compiler inventory SHA-256 is
`4bc75a502ac655a45bacbc426fb5159028d3fb88deb4829135aca2466cd89095`; the
header manifest SHA-256 is
`240a3fce1762bd3ea634922b7f6ee192c1d0c53abf83627aefd74b6b59adbc43`.
`tools/tests/test_tkmath_matrix_selectors.py` requires each selector below to
resolve uniquely to this compiler declaration ID.

| Native selector | Compiler declaration ID | Rust target |
|---|---|---|
| `NCollection_Mat3::Map(Element_t*)` | `5e585163596ae7b4ae99893e33d04cc3f5a3924208df7dcaf46888b0aa143281` | `Mat3::map_mut` |
| `NCollection_Mat3::Map(const Element_t*)` | `d08d3990fe85e9a8ca758ffd72072e2c3f830e605288f88e703da5c8dbc74fd2` | `Mat3::map` |
| `NCollection_Mat4::Map(Element_t*)` | `3460dbd18af0689c543a4636660cddbde649ca7f0a73c48b700637b0af1079b5` | `Mat4::map_mut` |
| `NCollection_Mat4::Map(const Element_t*)` | `03389096a229d7a2e0b9f394b0a8fefbc41c7e2d23e37849aed35c4339cb8092` | `Mat4::map` |

The header hashes are Mat3 `a6e8f11773efab41789dedd45e55d05f556f420c7073f3fd01c96d0e079afa51`
and Mat4 `1b2e16e62d4c520c3939ee6916a9607bd8d128d83850fbca2b52f0b8498e2df0`.

## Native replay

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkernel_matrix_map.cxx -o /tmp/tkernel_matrix_map
/tmp/tkernel_matrix_map > /tmp/tkernel_matrix_map.out
diff -u tools/oracle/tkernel_matrix_map.expected /tmp/tkernel_matrix_map.out
```

The replay passed with g++ 13.3.0. Probe source SHA-256 is
`e6501ee471e7f2665f564f8a8a47bb68c9adfb169820e6d18e3122119a5955cf`.
Expected output SHA-256:
`858a5d95239f1dcede31adf8587f4c96ce9f5e5baae4ac30cafe3ea576b313f1`.
Rust behavior checks are in `crates/tkernel/tests/ncollection_matrix_map.rs`.
