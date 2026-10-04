# TKMesh Delaunay circle selection profile

## Source and change

The behavior reference is OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`, `src/ModelingAlgorithms/TKMesh/BRepMesh/BRepMesh_Delaun.cxx` (SHA-256 `e7382522e4f8d455a87fa95dff3c3d72586f6a2196dd656fee6a1cacf5f8baef`). In `createTrianglesOnNewVertices` around lines 642–702, OCCT selects one `aCirclesList` per inserted vertex, removes the accepted containing triangle from that list, then scans and removes cavity victims from the same list. It does not reselect the spatial cell during each cavity expansion.

At baseline commit `6363dca973e3387db26dc81ab12285f12e3e92fe`, Rust copied that list to a `Vec`, but re-ran `CircleTool::select` when removing each victim and on every cavity expansion. The change in `crates/tkmesh/src/brep_mesh/delaun/batch_insertion.rs` retains the selected IDs once per vertex and performs stable `Vec::remove` operations in the native iteration order. The public `BRepMeshCircleTool::select` API is unchanged. `BRepMeshDelaun::circles()` only exposes an immutable tool reference, so the internal selected-list cache is not externally mutable through the Delaun object.

The resulting Rust source SHA-256 is `a47a48c68ca987aeaf15d9fa90455b697c7dda2b3e6ebcd4d4601232b4342782`.

## Measured profile

The exact workload was `BVH_PairDistanceTest__Bug30655_SphereDistance`: two sphere surface meshes with the pinned `deflection = 0.01` and generated grids of 50,244 and 55,278 UV candidates. On the same optimized test profile, temporary isolated counters measured:

| Measure | Before reuse | After reuse |
| --- | ---: | ---: |
| Instrumented test elapsed | 45.50 s | 13.28 s |
| Cell-filter target scans | 4,969,376,560 | 219,506,324 |
| Cell-filter scan time | 34.36 s | 2.80 s |

The timers were temporary and are not in the committed source. With instrumentation removed, the same optimized test profile passed in 44.60 s before and 12.16 s after the change. The ordinary profile passed in 54.33 s after the change; the owner recorded 252.72 s at baseline. Exact native distance assertions and their existing `1e-2` tolerances passed. These timings are environment-specific and do not imply bitwise equality of the complete triangulation.

## Validation

- `cargo test -p rustycad-tkmesh`: passed all package tests and doc tests.
- `cargo clippy -p rustycad-tkmesh --all-targets -- -D warnings`: passed.
- Ordinary-profile sphere test: `PATH=/home/vmiguel/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/target-rustycad-bvh-normal-diag CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p rustycad-tkmath --test bvh_bug30655 BVH_PairDistanceTest__Bug30655_SphereDistance -- --nocapture` (54.33 s).
- Optimized-profile sphere test: `PATH=/home/vmiguel/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/target-rustycad-bvh-perf-diag2 CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_TEST_OPT_LEVEL=2 cargo test -p rustycad-tkmath --test bvh_bug30655 BVH_PairDistanceTest__Bug30655_SphereDistance -- --nocapture` (12.16 s).
- `python3 tools/verify_tkmesh_delaun_core.py --source /home/vmiguel/git/oblikovati-workspace/OCCT --native-build /home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full`: 2,293 exact native/Rust comparisons passed. The verifier retains 80 previously classified native-crash/Rust-typed-error divergences; `complete_parity` remains false.

The retained numeric report is from the above native build. Its `libTKMesh.so` SHA-256 is `0fbc7887b5a914802d0934d89ed7197ba83d2e38622592383a64aa63e6453f9b`; the generated native probe SHA-256 is `291128ae71821aff03b69394e74a57e98d0f75ebcf3bf5e4fb22271ea3a99b37`. Native and Rust outputs matched byte-for-byte for the 2,293 passing cases (output SHA-256 `1922cc64392fa92da7a1fb60cf1cbf240647dcfcc997635507501fe61eaf8251`).
