# Martensite CPU shadow lifecycle

The native CPU oracle previously copied a parent's group nodes into a highlight
shadow. OCCT destroys a structure by clearing those nodes. Destroying an immediate
highlight therefore emptied the parent's point geometry. A full controller-frame
replay exposed this as a changed orbit gravity point after selection/highlighting.

The null drivers now follow the actual `OpenGl_StructureShadow` ownership policy
at OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`: retain the root parent,
keep the shadow's own removable groups empty, and copy the parent's bounds,
infinite flag, transformation and transform persistence. Nested shadows retain
the root parent directly. Clearing or dropping them preserves parent geometry.

`NullCStructure::new(&manager)` now returns `Result<Handle<NullCStructure>, Failure>`
so its retained parent can be a genuine owning handle. The null driver factory
and the named sensitive-display test driver use that result directly. For example:

```rust,ignore
let structure = NullCStructure::new(&manager)?;
let shadow = structure.shadow_link(&manager)?;
drop(shadow); // Parent groups remain intact.
```

The six-row native fixture records parent reference counts, parent group count,
parent bounds and group bounds around two nested shadows and their destruction.
The two original native runs are byte-identical and have the same hash as the
committed asset. Rust replays those exact rows; additional tests verify clearing,
infinite state and genuine transformation/persistence handle identity. The
provenance JSON records the pinned native definitions, local helper sources,
compiler and executable hashes. This correction grants no upstream API credit.

To reproduce the native oracle, create the existing shared camera oracle wrappers
by replacing only `int main(` with a unique unused entry point in these sources:

| Source | Wrapper filename |
| --- | --- |
| `martensite_controller_camera_primitives.cxx` | `martensite_camera_primitives_oracle_shared.cxx` |
| `martensite_navigation_motion.cxx` | `martensite_navigation_motion_oracle_shared.cxx` |
| `martensite_camera_rotation.cxx` | `martensite_camera_rotation_oracle_shared.cxx` |
| `martensite_camera_actions.cxx` | `martensite_camera_actions_oracle_shared.cxx` |

Place those wrappers in `/tmp/martensite-frame-oracle` and compile with:

```sh
g++ -std=c++17 -O0 -I/tmp/martensite-frame-oracle -Itools/oracle \
  -I../RustyCADKernel/target/occt-all-full/include/opencascade \
  tools/oracle/martensite_null_shadow_lifecycle.cxx \
  -L../RustyCADKernel/target/occt-all-full/lin64/gcc/libi \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-all-full/lin64/gcc/libi \
  -lTKV3d -lTKG3d -lTKService -lTKMath -lTKernel \
  -o /tmp/martensite-frame-oracle/null-shadow
/tmp/martensite-frame-oracle/null-shadow
```
